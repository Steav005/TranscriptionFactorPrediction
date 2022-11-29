use std::collections::HashMap;
use std::path::Path;

use anyhow::anyhow;
use rayon::prelude::*;
use thiserror::Error;

use crate::matrix::{ExtendedTfpMatrix, Float, PwmMatrix, TfpMatrix};
use crate::parser::fasta::Fasta;
use crate::parser::profile::{Css, Mss, Profile};
use crate::parser::transfac::parse_matrices;
use crate::sequence::{MinusStrand, PlusStrand, Sequence};

#[derive(Debug, Default, Clone)]
pub struct TfpCalculator {
    matrices: Vec<PwmMatrix>,
    sequences: Vec<PlusStrand>,
    profiles: HashMap<String, (Css, Mss)>,
    default_css_threshold: Float,
    default_mss_threshold: Float,
}

#[derive(Debug)]
pub struct Tfp {
    pub sequence: String,
    pub matrix: String,
    pub pos: usize,
    pub strand: bool,
    pub css: Float,
    pub mss: Float,
    pub len: usize,
}

#[derive(Debug, Error)]
pub enum TfpError {
    #[error("Matrix to short({0}) for Core(5) calculation")]
    MatrixToShort(usize),
    #[error("File access problem")]
    FileError(std::io::Error),
    #[error("Problem while parsing")]
    ParseError(anyhow::Error),
}

pub type TfpResult<T> = Result<T, TfpError>;

impl TfpCalculator {
    pub fn add_from_profile_file<P: AsRef<Path>>(&mut self, path: P) -> TfpResult<()> {
        let c = std::fs::read_to_string(path).map_err(TfpError::FileError)?;
        let (_, profiles) = Profile::parse_many(&c)
            .map_err(|e| TfpError::ParseError(anyhow!("Profile parse error: {e:?}")))?;
        self.profiles.extend(profiles);
        Ok(())
    }

    pub fn add_from_transfac_file<P: AsRef<Path>>(&mut self, path: P) -> TfpResult<()> {
        let c = std::fs::read_to_string(path).map_err(TfpError::FileError)?;
        let (_, mut matrices) = parse_matrices(&c)
            .map_err(|e| TfpError::ParseError(anyhow!("Transfac parse error: {e:?}")))?;
        let matrices = matrices
            .drain(..)
            .map(PwmMatrix::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TfpError::ParseError(anyhow!("{e:?}")))?;
        self.matrices.extend(matrices);
        Ok(())
    }

    pub fn add_from_fasta_file<P: AsRef<Path>>(&mut self, path: P) -> TfpResult<()> {
        let c = std::fs::read_to_string(path).map_err(TfpError::FileError)?;
        let (_, mut fasta) = Fasta::parse_many(&c)
            .map_err(|e| TfpError::ParseError(anyhow!("Fasta parse error: {e:?}")))?;
        let seq = fasta
            .drain(..)
            .map(PlusStrand::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TfpError::ParseError(anyhow!("{e:?}")))?;
        self.sequences.extend(seq);
        Ok(())
    }

    pub fn add_sequence(&mut self, seq: PlusStrand) {
        self.sequences.push(seq)
    }

    pub fn add_pwm(&mut self, pwm: PwmMatrix) {
        self.matrices.push(pwm)
    }

    pub fn add_profil(&mut self, profile: Profile) {
        let Profile { css, mss, id } = profile;
        self.profiles.insert(id, (css, mss));
    }

    pub fn set_default_css_threshold(&mut self, css: Css) {
        self.default_css_threshold = css;
    }

    pub fn get_default_css_threshold(&self) -> Float {
        self.default_css_threshold
    }

    pub fn set_default_mss_threshold(&mut self, mss: Mss) {
        self.default_mss_threshold = mss;
    }

    pub fn get_default_mss_threshold(&self) -> Float {
        self.default_mss_threshold
    }

    pub fn evaluate(self) -> Vec<Tfp> {
        let TfpCalculator {
            mut matrices,
            sequences,
            profiles,
            default_css_threshold,
            default_mss_threshold,
        } = self;
        let mut plus = sequences;
        let mut minus: Vec<MinusStrand> = plus.iter().map(MinusStrand::from).collect();
        let sequences: Vec<_> = plus
            .drain(..)
            .map(Sequence::Plus)
            .chain(minus.drain(..).map(Sequence::Minus))
            .collect();
        matrices
            .par_drain(..)
            .map(|m| {
                let (css_threshold, mss_threshold) = profiles
                    .get(&m.name)
                    .cloned()
                    .unwrap_or((default_css_threshold, default_mss_threshold));
                TfpMatrix {
                    name: m.name,
                    matrix: m.matrix,
                    css_threshold,
                    mss_threshold,
                }
            })
            .map(ExtendedTfpMatrix::try_from)
            // This flatten ignores errors from the ExtendedTfpMatrix::try_from function
            .flatten()
            .flat_map(|m| {
                sequences
                    .iter()
                    .flat_map(|s| find_significant_bases(&m, s))
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

fn find_significant_bases(tfp: &ExtendedTfpMatrix, seq: &Sequence) -> Vec<Tfp> {
    let seq_name = seq.name();
    let ppm_nrows = tfp.ppm.nrows();

    let significant_bases = seq
        .as_slice()
        .windows(5)
        .enumerate()
        .skip(tfp.core_start)
        .take(seq.len() - tfp.core_start - (ppm_nrows - tfp.core_start - 5))
        .map(|(i, s)| (i, tfp.sig_map[&(s[0], s[1], s[2], s[3], s[4])]))
        .filter(|(_, css)| css >= &tfp.css_threshold);

    significant_bases
        .map(|(sig_index, css)| {
            let current: Float = tfp
                .ppm
                .row_iter()
                .zip(tfp.iv.iter())
                .zip(seq.as_slice().iter().skip(sig_index - tfp.core_start))
                .map(|((r, iv), b)| iv * r[(0, *b as usize)])
                .sum();
            (
                sig_index,
                css,
                (current - tfp.iv_min_sum) / (tfp.iv_max_sum - tfp.iv_min_sum),
            )
        })
        .filter(|(_, _, mss)| mss >= &tfp.mss_threshold)
        .map(|(i, css, mss)| {
            let (strand, pos) = match seq {
                Sequence::Plus(_) => (true, i),
                Sequence::Minus(_) => (false, seq.len() - i - 5),
            };

            Tfp {
                sequence: seq_name.to_string(),
                matrix: tfp.name.to_string(),
                pos,
                strand,
                css,
                mss,
                len: ppm_nrows,
            }
        })
        .collect()
}
