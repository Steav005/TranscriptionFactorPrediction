use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use tfp::parser::profile::Profile;

#[pyclass(name = "Profile")]
#[derive(Clone)]
pub struct PyProfile {
    #[pyo3(get, set)]
    pub css: f32,
    #[pyo3(get, set)]
    pub mss: f32,
    #[pyo3(get, set)]
    pub id: String,
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
    Profile::parse_many(c)
        .map_err(|e| PyOSError::new_err(format!("{e:?}")))
        .map(|(_, p)| {
            p.iter()
                .map(|(id, (css, mss))| PyProfile {
                    id: id.to_string(),
                    css: **css,
                    mss: **mss,
                })
                .collect()
        })
}
