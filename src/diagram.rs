use hashbrown::HashSet;
use pyo3::prelude::*;
/// Stores the pairings from a matrix decomposition,
/// as well as those columns which did not appear in a pairing.
#[pyclass]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct PersistenceDiagram {
    #[pyo3(get)]
    pub unpaired: HashSet<usize>,
    #[pyo3(get)]
    pub paired: HashSet<(usize, usize)>,
}

impl std::fmt::Display for PersistenceDiagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Paired: {:?}\nUnpaired: {:?}",
            self.paired, self.unpaired
        )
    }
}

#[pymethods]
impl PersistenceDiagram {
    fn __richcmp__(&self, other: &PyAny, cmp_op: pyo3::basic::CompareOp) -> bool {
        let other: PersistenceDiagram = other
            .extract()
            .expect("Can only compare PersistenceDiagram to another PersistenceDiagram");
        match cmp_op {
            pyo3::pyclass::CompareOp::Eq => *self == other,
            _ => panic!("Only able to check equality between PersistenceDiagram"),
        }
    }

    fn __repr__(&self) -> String {
        self.to_string()
    }
}

/// Able to construct persistence diagram from structs implementing this trait.
pub trait DiagramReadOff {
    fn diagram(&self) -> PersistenceDiagram;
}
