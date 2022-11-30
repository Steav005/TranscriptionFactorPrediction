use ::tfp::builder::{Tfp, TfpCalculator};
use ::tfp::matrix::PwmMatrix;
use ::tfp::parser::fasta::Fasta;
use parser::fasta::PyFasta;
use parser::profile::{parse_profile, PyProfile};
use parser::transfac::PyPwmMatrix;
use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use rayon::prelude::*;

use crate::parser::fasta::parse_fasta;
use crate::parser::transfac::parse_transfac;

pub(crate) mod parser;

#[pyclass(name = "Tfp")]
#[derive(Debug)]
pub struct PyTfp {
    #[pyo3(get, set)]
    sequence: String,
    #[pyo3(get, set)]
    matrix: String,
    #[pyo3(get, set)]
    pos: usize,
    #[pyo3(get, set)]
    strand: bool,
    #[pyo3(get, set)]
    css: f32,
    #[pyo3(get, set)]
    mss: f32,
    #[pyo3(get, set)]
    len: usize,
}

#[pymethods]
impl PyTfp {
    fn __repr__(&self) -> String {
        format!(
            "Tfp (sequence: {}, matrix: {}, pos: {}, strand: {}, css: {}, mss: {}, len: {})",
            self.sequence,
            self.matrix,
            self.pos,
            if self.strand { "+" } else { "-" },
            self.css,
            self.mss,
            self.len
        )
    }
}

impl From<Tfp> for PyTfp {
    fn from(t: Tfp) -> Self {
        PyTfp {
            sequence: t.sequence,
            matrix: t.matrix,
            pos: t.pos,
            strand: t.strand,
            css: *t.css,
            mss: *t.mss,
            len: t.len,
        }
    }
}

#[pyclass(name = "TfpCalculator")]
#[derive(Debug, Default)]
pub struct PyTfpCalculator {
    calculator: TfpCalculator,
}

#[pymethods]
impl PyTfpCalculator {
    #[new]
    fn __new__() -> Self {
        Default::default()
    }

    fn __repr__(&self) -> String {
        let matrices = self
            .calculator
            .matrices
            .iter()
            .cloned()
            .map(PyPwmMatrix::from)
            .map(|m| m.__repr__())
            .collect::<String>();
        let sequences = self
            .calculator
            .sequences
            .iter()
            .map(|s| {
                format!(
                    "Fasta {} {}\n",
                    s.name,
                    s.seq.iter().map(|b| b.to_string()).collect::<String>()
                )
            })
            .collect::<String>();
        let profiles = self
            .calculator
            .profiles
            .iter()
            .map(|(id, (css, mss))| format!("Profile(id: {id}, css: {css}, mss: {mss})\n"))
            .collect::<String>();

        format!("TfpCalculator: \n{matrices} {sequences} {profiles} default_css_threshold: {}, default_mss_threshold: {}", self.calculator.default_css_threshold, self.calculator.default_mss_threshold)
    }

    #[setter]
    fn set_default_css_threshold(&mut self, value: f32) -> PyResult<()> {
        self.calculator.set_default_css_threshold(
            value
                .try_into()
                .map_err(|e| PyOSError::new_err(format!("{e:?}")))?,
        );
        Ok(())
    }

    #[getter]
    fn get_default_css_threshold(&self) -> f32 {
        *self.calculator.get_default_css_threshold()
    }

    #[setter]
    fn set_default_mss_threshold(&mut self, value: f32) -> PyResult<()> {
        self.calculator.set_default_mss_threshold(
            value
                .try_into()
                .map_err(|e| PyOSError::new_err(format!("{e:?}")))?,
        );
        Ok(())
    }

    #[getter]
    fn get_default_mss_threshold(&self) -> f32 {
        *self.calculator.get_default_mss_threshold()
    }

    fn add_from_profile_file(&mut self, path: &str) -> PyResult<()> {
        self.calculator
            .add_from_profile_file(path)
            .map_err(|e| PyOSError::new_err(format!("{e:?}")))
    }

    fn add_from_transfac_file(&mut self, path: &str) -> PyResult<()> {
        self.calculator
            .add_from_transfac_file(path)
            .map_err(|e| PyOSError::new_err(format!("{e:?}")))
    }

    fn add_from_fasta_file(&mut self, path: &str) -> PyResult<()> {
        self.calculator
            .add_from_fasta_file(path)
            .map_err(|e| PyOSError::new_err(format!("{e:?}")))
    }

    fn add_profile(&mut self, profile: PyProfile) -> PyResult<()> {
        self.calculator.add_profil(profile.try_into()?);
        Ok(())
    }

    fn add_fasta(&mut self, fasta: PyFasta) -> PyResult<()> {
        self.calculator.add_sequence(
            Fasta::from(fasta)
                .try_into()
                .map_err(|e| PyOSError::new_err(format!("{e:?}")))?,
        );
        Ok(())
    }

    fn add_pwm(&mut self, pwm: PyPwmMatrix) {
        self.calculator.add_pwm(PwmMatrix::from(pwm));
    }

    fn evaluate(&self) -> Vec<PyTfp> {
        self.calculator
            .clone()
            .evaluate()
            .par_drain(..)
            .map(PyTfp::from)
            .collect()
    }
}

#[pymodule]
fn tfp(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTfpCalculator>()?;

    m.add_function(wrap_pyfunction!(parse_profile, m)?)?;
    m.add_class::<PyProfile>()?;

    m.add_function(wrap_pyfunction!(parse_fasta, m)?)?;
    m.add_class::<PyFasta>()?;

    m.add_function(wrap_pyfunction!(parse_transfac, m)?)?;
    m.add_class::<PyPwmMatrix>()?;

    // m.add_submodule(parser_module)?;
    Ok(())
}
