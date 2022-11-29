use std::collections::HashMap;

use itertools::Itertools;
use nalgebra::{Const, Dynamic, Matrix, Matrix1x4, OVector, VecStorage, Vector};
use ordered_float::NotNan;
use strum::IntoEnumIterator;

use crate::builder::TfpError;
use crate::sequence::Base;

pub type Float = NotNan<f32>;
pub type PwmMatrixInner = Matrix<Float, Dynamic, Const<4>, VecStorage<Float, Dynamic, Const<4>>>;
pub type PpmMatrix = PwmMatrixInner;
pub type IvVector = Vector<Float, Dynamic, VecStorage<Float, Dynamic, Const<1>>>;
pub type MaxVector = IvVector;
pub type SigMap = HashMap<(Base, Base, Base, Base, Base), Float>;

#[derive(Debug, Clone)]
pub struct PwmMatrix {
    pub name: String,
    pub matrix: PwmMatrixInner,
}

#[derive(Debug)]
pub struct TfpMatrix {
    pub name: String,
    pub matrix: PwmMatrixInner,
    pub css_threshold: Float,
    pub mss_threshold: Float,
}

#[derive(Debug)]
pub struct ExtendedTfpMatrix {
    pub(crate) name: String,
    pub(crate) css_threshold: Float,
    pub(crate) mss_threshold: Float,
    pub(crate) ppm: PpmMatrix,
    pub(crate) iv: IvVector,
    pub(crate) iv_max_sum: Float,
    pub(crate) iv_min_sum: Float,
    pub(crate) core_start: usize,
    // pub(crate) core_max: Float,
    // pub(crate) core_min: Float,
    pub(crate) sig_map: SigMap,
}

impl TryFrom<TfpMatrix> for ExtendedTfpMatrix {
    type Error = TfpError;

    fn try_from(pwm: TfpMatrix) -> Result<Self, Self::Error> {
        let TfpMatrix {
            name,
            mut matrix,
            css_threshold,
            mss_threshold,
        } = pwm;
        to_ppm(&mut matrix);
        let ppm = matrix;
        let iv = gen_iv(&ppm);
        let iv_max_sum = iv_max_sum(&iv, &ppm);
        let iv_min_sum = iv_min_sum(&iv, &ppm);
        let max_vector = gen_max_vector(&ppm, &iv);
        // TODO return real error
        let (core_start, core_max) =
            find_core(&max_vector).ok_or_else(|| TfpError::MatrixToShort(ppm.nrows()))?;
        let core_min = min_from_core(core_start, &ppm, &iv);
        let sig_map = gen_sig_map(core_start, core_min, core_max, &ppm, &iv);

        Ok(Self {
            name,
            css_threshold,
            mss_threshold,
            ppm,
            iv,
            iv_max_sum,
            iv_min_sum,
            core_start,
            // core_max,
            // core_min,
            sig_map,
        })
    }
}

fn gen_sig_map(
    core_index: usize,
    core_min: Float,
    core_max: Float,
    ppm: &PpmMatrix,
    iv: &IvVector,
) -> SigMap {
    let core = ppm.rows(core_index, 5);
    let core_iv = iv.rows(core_index, 5);
    Base::iter()
        .cartesian_product(Base::iter())
        .cartesian_product(Base::iter())
        .cartesian_product(Base::iter())
        .cartesian_product(Base::iter())
        .map(|((((a, b), c), d), e)| {
            let current = core[(0, a as usize)] * core_iv[0]
                + core[(1, b as usize)] * core_iv[1]
                + core[(2, c as usize)] * core_iv[2]
                + core[(3, d as usize)] * core_iv[3]
                + core[(4, e as usize)] * core_iv[4];
            (
                (a, b, c, d, e),
                (current - core_min) / (core_max - core_min),
            )
        })
        .collect()
}

fn min_from_core(core_index: usize, ppm: &PpmMatrix, iv: &IvVector) -> Float {
    ppm.rows(core_index, 5)
        .row_iter()
        .zip(iv.rows(core_index, 5).iter())
        .map(|(r, iv)| iv * r.iter().min().expect("Guaranteed to not be 'None'"))
        .sum()
}

fn find_core(max_vec: &MaxVector) -> Option<(usize, Float)> {
    max_vec
        .as_slice()
        .windows(5)
        .map(|a| a.iter().sum::<Float>())
        .enumerate()
        .max_by_key(|(_, k)| *k)
}

fn gen_max_vector(ppm: &PpmMatrix, iv: &IvVector) -> MaxVector {
    let max_vector = ppm
        .row_iter()
        .zip(iv.iter())
        .map(|(r, iv)| iv * r.iter().max().expect("Guaranteed to not be 'None'"));
    MaxVector::from_iterator(max_vector.len(), max_vector)
}

fn iv_min_sum(iv: &IvVector, ppm: &PpmMatrix) -> Float {
    iv.row_iter()
        .zip(
            ppm.row_iter()
                .map(|r| *r.iter().min().expect("Guaranteed to not be 'None'")),
        )
        .map(|(r, min)| r[(0, 0)] * min)
        .sum()
}

fn iv_max_sum(iv: &IvVector, ppm: &PpmMatrix) -> Float {
    iv.row_iter()
        .zip(
            ppm.row_iter()
                .map(|r| *r.iter().max().expect("Guaranteed to not be 'None'")),
        )
        .map(|(r, max)| r[(0, 0)] * max)
        .sum()
}

fn gen_iv(ppm: &PpmMatrix) -> IvVector {
    const FOUR: Float = unsafe { Float::new_unchecked(4.0) };
    ppm.compress_columns(
        OVector::zeros_generic(Dynamic::new(ppm.nrows()), Const::<1>),
        |out, col| {
            *out += col * (FOUR * col[(0, 0)]);
        },
    )
}

fn to_ppm(pwm: &mut PwmMatrixInner) {
    const FLOAT_1: Float = unsafe { Float::new_unchecked(1.0) };
    const FLOAT_100: Float = unsafe { Float::new_unchecked(100.0) };
    const ONE: Matrix1x4<Float> = Matrix1x4::new(FLOAT_1, FLOAT_1, FLOAT_1, FLOAT_1);

    pwm.row_iter_mut()
        .map(|mut r| {
            r *= FLOAT_100;
            r += ONE;
            r /= r.sum();
        })
        .for_each(drop);
}
