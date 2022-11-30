use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use tfp::parser::fasta::Fasta;

#[pyclass(name = "Fasta")]
#[derive(Debug, Clone)]
pub struct PyFasta {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub seq: String,
}

#[pymethods]
impl PyFasta {
    fn __repr__(&self) -> String {
        format!("Fasta {}, {}", self.name, self.seq)
    }
}

impl From<PyFasta> for Fasta {
    fn from(f: PyFasta) -> Self {
        Self {
            name: f.name,
            seq: f.seq,
        }
    }
}

impl From<Fasta> for PyFasta {
    fn from(f: Fasta) -> Self {
        Self {
            name: f.name,
            seq: f.seq,
        }
    }
}

#[pyfunction]
pub fn parse_fasta(c: &str) -> PyResult<Vec<PyFasta>> {
    let (rest, mut fasta) =
        Fasta::parse_many(c).map_err(|e| PyOSError::new_err(format!("{e:?}")))?;
    if !rest.is_empty() {
        return Err(PyOSError::new_err(format!(
            "Could not parse completly. {rest}"
        )));
    }

    Ok(fasta.drain(..).map(PyFasta::from).collect())
}
