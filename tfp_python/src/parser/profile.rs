use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use tfp::parser::profile::Profile;

#[pyclass(name = "Profile")]
#[derive(Debug, Clone)]
pub struct PyProfile {
    #[pyo3(get, set)]
    pub css: f32,
    #[pyo3(get, set)]
    pub mss: f32,
    #[pyo3(get, set)]
    pub id: String,
}

#[pymethods]
impl PyProfile {
    fn __repr__(&self) -> String {
        format!(
            "Profile(id: {}, css: {}, mss: {})",
            self.id, self.css, self.mss
        )
    }
}

impl TryFrom<PyProfile> for Profile {
    type Error = PyErr;

    fn try_from(value: PyProfile) -> Result<Self, Self::Error> {
        Ok(Self {
            css: value
                .css
                .try_into()
                .map_err(|e| PyOSError::new_err(format!("{e:?}")))?,
            mss: value
                .mss
                .try_into()
                .map_err(|e| PyOSError::new_err(format!("{e:?}")))?,
            id: value.id,
        })
    }
}

#[pyfunction]
pub fn parse_profile(c: &str) -> PyResult<Vec<PyProfile>> {
    let (rest, mut profiles) =
        Profile::parse_many(c).map_err(|e| PyOSError::new_err(format!("{e:?}")))?;
    if !rest.is_empty() {
        return Err(PyOSError::new_err(format!(
            "Could not parse completly. {rest}"
        )));
    }
    Ok(profiles
        .drain()
        .map(|(id, (css, mss))| PyProfile {
            id,
            css: *css,
            mss: *mss,
        })
        .collect())
}
