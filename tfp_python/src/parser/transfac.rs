use nalgebra::DVector;
use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use tfp::matrix::{Float, PwmMatrix, PwmMatrixInner};
use tfp::parser::transfac::parse_matrices;

#[pyclass(name = "PwmMatrix")]
#[derive(Debug, Clone)]
pub struct PyPwmMatrix {
    #[pyo3(get, set)]
    pub name: String,
    pub matrix: PwmMatrixInner,
}

#[pymethods]
impl PyPwmMatrix {
    #[new]
    fn __new__(name: String, mut m: [Vec<f32>; 4]) -> PyResult<Self> {
        let mut matrix = PwmMatrixInner::zeros(m[0].len());
        let column_0 = DVector::from_vec(
            m[0].drain(..)
                .map(|v| Float::try_from(v).map_err(|e| PyOSError::new_err(format!("{e:?}"))))
                .collect::<Result<Vec<_>, _>>()?,
        );
        matrix.set_column(0, &column_0);
        let column_1 = DVector::from_vec(
            m[1].drain(..)
                .map(|v| Float::try_from(v).map_err(|e| PyOSError::new_err(format!("{e:?}"))))
                .collect::<Result<Vec<_>, _>>()?,
        );
        matrix.set_column(1, &column_1);
        let column_2 = DVector::from_vec(
            m[2].drain(..)
                .map(|v| Float::try_from(v).map_err(|e| PyOSError::new_err(format!("{e:?}"))))
                .collect::<Result<Vec<_>, _>>()?,
        );
        matrix.set_column(2, &column_2);
        let column_3 = DVector::from_vec(
            m[3].drain(..)
                .map(|v| Float::try_from(v).map_err(|e| PyOSError::new_err(format!("{e:?}"))))
                .collect::<Result<Vec<_>, _>>()?,
        );
        matrix.set_column(3, &column_3);

        Ok(Self { name, matrix })
    }

    #[getter]
    fn get_matrix(&self) -> Vec<[f32; 4]> {
        self.matrix
            .row_iter()
            .map(|r| {
                r.iter()
                    .cloned()
                    .map(f32::from)
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("Guaranteed to be convertable")
            })
            .collect::<Vec<_>>()
    }

    pub fn __repr__(&self) -> String {
        format!("PwmMatrix: {} {}", self.name, self.matrix)
    }
}

impl From<PyPwmMatrix> for PwmMatrix {
    fn from(m: PyPwmMatrix) -> Self {
        Self {
            name: m.name,
            matrix: m.matrix,
        }
    }
}

impl From<PwmMatrix> for PyPwmMatrix {
    fn from(m: PwmMatrix) -> Self {
        Self {
            name: m.name,
            matrix: m.matrix,
        }
    }
}

#[pyfunction]
pub fn parse_transfac(c: &str) -> PyResult<Vec<PyPwmMatrix>> {
    let (rest, mut matrices) =
        parse_matrices(c).map_err(|e| PyOSError::new_err(format!("{e:?}")))?;
    if !rest.is_empty() {
        return Err(PyOSError::new_err(format!(
            "Could not parse completly. {rest}"
        )));
    }
    let mut matrices = matrices
        .drain(..)
        .map(PwmMatrix::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| PyOSError::new_err(format!("{e:?}")))?;
    Ok(matrices.drain(..).map(PyPwmMatrix::from).collect())
}
