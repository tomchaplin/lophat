use hashbrown::HashSet;

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Stores the pairings from a matrix decomposition,
/// as well as those columns which did not appear in a pairing.
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct PersistenceDiagram {
    /// The set of unpaired columns indexes.
    pub unpaired: HashSet<usize>,
    /// The set of column pairings.
    pub paired: HashSet<(usize, usize)>,
}

impl PersistenceDiagram {
    /// Re-indexes a persistence diagram, assuming that it was produced from an anti-transposed matrix.
    /// Requires `matrix_size` - the size of the decomposed matrix, assumed to be square.
    pub fn anti_transpose(mut self, matrix_size: usize) -> Self {
        let new_paired = self
            .paired
            .into_iter()
            .map(|(b, d)| (matrix_size - 1 - d, matrix_size - 1 - b))
            .collect();
        let new_unpaired = self
            .unpaired
            .into_iter()
            .map(|idx| matrix_size - 1 - idx)
            .collect();
        self.paired = new_paired;
        self.unpaired = new_unpaired;
        self
    }
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

#[cfg(feature = "python")]
#[pymethods]
impl PersistenceDiagram {
    fn __richcmp__(&self, other: &PyAny, cmp_op: pyo3::basic::CompareOp) -> bool {
        if let Ok(other) = other.extract::<PersistenceDiagram>() {
            match cmp_op {
                pyo3::pyclass::CompareOp::Eq => *self == other,
                _ => panic!("Only able to check equality between PersistenceDiagram"),
            }
        } else {
            false
        }
    }

    fn __repr__(&self) -> String {
        self.to_string()
    }
}
