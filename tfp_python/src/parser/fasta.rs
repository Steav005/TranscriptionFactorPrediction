use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use tfp::parser::fasta::Fasta;

#[pyclass(name = "Fasta")]
#[derive(Clone)]
pub struct PyFasta {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub seq: String,
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
    Fasta::parse_many(c)
        .map_err(|e| PyOSError::new_err(format!("{e:?}")))
        .map(|(_, mut p)| p.drain(..).map(PyFasta::from).collect())
}
