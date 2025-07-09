use std::{cell::Cell, ops::Deref};

use serde::{ser::SerializeStruct, Deserialize, Serialize};

use crate::{
    algorithms::{Decomposition, NoVMatrixError},
    columns::{Column, VecColumn},
};

#[macro_export]
/// Implements [`Serialize`](serde::Serialize) on the provided algorithm, for any column representation.
///
/// The struct must be generic over the column type and implement [`Decomposition`](crate::algorithms::Decomposition).
/// As a fallback, you may wish to use [`serialize_algo`] to implement [`Serialize`](serde::Serialize) yourself.
///
/// **Note:** We intentionally *do not* implement [`Deserialize`](serde::Deserialize).
/// Instead, you should deserialize to [`DecompositionFileFormat`].
///
/// # Example usage
///
/// ```ignore
/// use lophat::impl_rvd_serialize;
/// use lophat::columns::Column;
/// use lophat::algorithms::Decomposition;
///
/// struct MyAlgo<C: Column> { ... }
///
/// impl<C:Column> Decomposition<C> for MyAlgo<C> { ... }
///
/// impl_rvd_serialize!(MyAlgo);
/// ````
macro_rules! impl_rvd_serialize {
    ($struct:ident) => {
        use serde::Serialize;
        use $crate::utils::serialize_algo;
        impl<C: Column> Serialize for $struct<C> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serialize_algo(self, serializer)
            }
        }
    };
}

/// A struct which represents the output of an R=DV decomposition in a standardised format.
/// The struct can be serialized and deserialized, and hence written to file.
///
/// Internally, R and V are stored as a vector of [`VecColumn`].
/// Usually, this is constructed via serializing another [`Decomposition`] (e.g. using [`impl_rvd_serialize`]) and then deserializing
/// into a [`DecompositionFileFormat`].
///
/// # Example
/// ```
/// use lophat::{
///     algorithms::{LockFreeAlgorithm, DecompositionAlgo},
///     columns::VecColumn,
///     utils::DecompositionFileFormat
/// };
/// // Use CBOR file format to store serialization
/// use ciborium::{de::from_reader, ser::into_writer};
///
/// let matrix = vec![
///     (0, vec![]),
///     (0, vec![]),
///     (0, vec![]),
///     (1, vec![0, 1]),
///     (1, vec![0, 2]),
///     (1, vec![1, 2]),
///     (2, vec![3, 4, 5]),
/// ]
/// .into_iter()
/// .map(VecColumn::from);
///
/// let decomp = LockFreeAlgorithm::init(None).add_cols(matrix).decompose();
/// // Serialize into bytes (could write to file here instead)
/// let mut bytes: Vec<u8> = vec![];
/// into_writer(&decomp, &mut bytes).ok();
/// // Deserialize to file format
/// let rvdff: DecompositionFileFormat = from_reader(bytes.as_slice()).ok().unwrap();
/// ```
#[derive(Deserialize, PartialEq, Debug)]
pub struct DecompositionFileFormat {
    r: Vec<VecColumn>,
    v: Option<Vec<VecColumn>>,
}

impl DecompositionFileFormat {
    /// Construct the [`DecompositionFileFormat`] using the provided matrices.
    pub fn new(r: Vec<VecColumn>, v: Option<Vec<VecColumn>>) -> Self {
        Self { r, v }
    }
}

impl Decomposition<VecColumn> for DecompositionFileFormat {
    type RColRef<'a> = &'a VecColumn
    where
        Self: 'a;

    fn get_r_col<'a>(&'a self, index: usize) -> Self::RColRef<'a> {
        &self.r[index]
    }

    type VColRef<'a> = &'a VecColumn
    where
        Self: 'a;

    fn get_v_col<'a>(&'a self, index: usize) -> Result<Self::VColRef<'a>, NoVMatrixError> {
        Ok(&self.v.as_ref().ok_or(NoVMatrixError)?[index])
    }

    fn n_cols(&self) -> usize {
        self.r.len()
    }
}

/// Clones the column, converting it to [`VecColumn`] format.
/// Under the hood, calls [`col.entries()`] to populate the output.
pub fn clone_to_veccolumn<C: Column>(col: &C) -> VecColumn {
    let mut output = VecColumn::new_with_dimension(col.dimension());
    output.add_entries(col.entries());
    output
}

/// After serializing your decomposition, you should deserialize to [`DecompositionFileFormat`].
pub fn serialize_algo<C, Algo, S>(algo: &Algo, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    C: Column,
    Algo: Decomposition<C>,
{
    // Taken from https://users.rust-lang.org/t/how-to-serialize-an-iterator-to-json/59272
    // We wrap the iterator in a cell so that we can implement Serialize on it
    // We also wrap it in an option because in order to call Cell::take() and accept ownership of the iterator
    // we must leave behind a default value.
    struct IteratorWrapper<T>(Cell<Option<T>>);

    impl<T> IteratorWrapper<T> {
        fn new(value: T) -> Self {
            IteratorWrapper(Cell::new(Some(value)))
        }
    }
    impl<I, J> Serialize for IteratorWrapper<I>
    where
        I: IntoIterator<Item = J>,
        J: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.collect_seq(self.0.take().unwrap())
        }
    }

    // Set up struct
    let mut rvdff = serializer.serialize_struct("DecompositionFileFormat", 2)?;

    // Serialize R
    let r_col_iter = (0..algo.n_cols()).map(|idx| {
        let col = algo.get_r_col(idx);
        clone_to_veccolumn(col.deref())
    });
    let r_col_iter = IteratorWrapper::new(r_col_iter);
    rvdff.serialize_field("r", &r_col_iter)?;

    // Serialize V
    let has_v = algo.get_v_col(0).is_ok();
    let v_col_iter_opt = if has_v {
        let v_col_iter = (0..algo.n_cols()).map(|idx| {
            // Can safely unwrap everything because V was maintained
            let col = algo.get_v_col(idx).unwrap();
            clone_to_veccolumn(col.deref())
        });
        Some(IteratorWrapper::new(v_col_iter))
    } else {
        None
    };
    rvdff.serialize_field("v", &v_col_iter_opt)?;
    rvdff.end()
}

// We do not derive directly because we want all algorithms to use the same serialize function.
impl Serialize for DecompositionFileFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serialize_algo(self, serializer)
    }
}

/// Converts the provided algorithm output to [`DecompositionFileFormat`], making a copy in memory.
///
/// Typically, it is more useful to directly serialize `algo`, e.g. using the [`impl_rvd_serialize`] macro.
/// This avoids making an extra copy of `algo` in memory, before writing to file.
/// The resulting serialization can then be deserialized into a [`DecompositionFileFormat`].
pub fn clone_to_file_format<C: Column, Algo: Decomposition<C>>(
    algo: &Algo,
) -> DecompositionFileFormat {
    let r = (0..algo.n_cols())
        .map(|idx| {
            let col = algo.get_r_col(idx);
            clone_to_veccolumn(col.deref())
        })
        .collect();
    let v = algo.get_v_col(0).ok().map(|_| (0..algo.n_cols())
                .map(|idx| {
                    let col = algo.get_v_col(idx).unwrap();
                    clone_to_veccolumn(col.deref())
                })
                .collect());
    DecompositionFileFormat::new(r, v)
}

#[cfg(test)]
mod tests {
    use crate::{
        algorithms::{DecompositionAlgo, LockFreeAlgorithm},
        columns::VecColumn,
        options::LoPhatOptions,
    };
    use ciborium::{de::from_reader, ser::into_writer};

    use super::DecompositionFileFormat;

    fn get_matrix() -> impl Iterator<Item = VecColumn> {
        vec![
            (0, vec![]),
            (0, vec![]),
            (0, vec![]),
            (1, vec![0, 1]),
            (1, vec![0, 2]),
            (1, vec![1, 2]),
            (2, vec![3, 4, 5]),
        ]
        .into_iter()
        .map(VecColumn::from)
    }

    fn get_rvdff(with_g: bool) -> DecompositionFileFormat {
        let correct_r = vec![
            (0, vec![]),
            (0, vec![]),
            (0, vec![]),
            (1, vec![0, 1]),
            (1, vec![0, 2]),
            (1, vec![]),
            (2, vec![3, 4, 5]),
        ]
        .into_iter()
        .map(VecColumn::from);
        let correct_v = vec![
            (0, vec![0]),
            (0, vec![1]),
            (0, vec![2]),
            (1, vec![3]),
            (1, vec![4]),
            (1, vec![3, 4, 5]),
            (2, vec![6]),
        ]
        .into_iter()
        .map(VecColumn::from);
        if with_g {
            DecompositionFileFormat::new(correct_r.collect(), Some(correct_v.collect()))
        } else {
            DecompositionFileFormat::new(correct_r.collect(), None)
        }
    }

    #[test]
    fn serialize_fileformat_and_back() {
        // Serialize and back again with V
        let rvdff_1 = get_rvdff(true);
        let mut bytes: Vec<u8> = vec![];
        into_writer(&rvdff_1, &mut bytes).ok();
        let rvdff_2: DecompositionFileFormat = from_reader(bytes.as_slice()).ok().unwrap();
        assert_eq!(rvdff_1, rvdff_2);
        // Serialize and back again without V
        let rvdff_1 = get_rvdff(false);
        let mut bytes: Vec<u8> = vec![];
        into_writer(&rvdff_1, &mut bytes).ok();
        let rvdff_2: DecompositionFileFormat = from_reader(bytes.as_slice()).ok().unwrap();
        assert_eq!(rvdff_1, rvdff_2);
    }

    #[test]
    fn serialize_lfa_and_back() {
        let matrix = get_matrix();
        let correct_rvdff = get_rvdff(true);
        // Decompose via LFA
        let options = LoPhatOptions {
            maintain_v: true,
            clearing: false, // Just do normal left-to-right reduction in decreasing dimensions
            num_threads: 1,  // So we can predict the output
            ..Default::default()
        };
        let decomp = LockFreeAlgorithm::init(Some(options))
            .add_cols(matrix)
            .decompose();
        // Serialize into bytes
        let mut bytes: Vec<u8> = vec![];
        into_writer(&decomp, &mut bytes).ok();
        // Deserialize to file format
        let rvdff: DecompositionFileFormat = from_reader(bytes.as_slice()).ok().unwrap();
        // Check all columns are correct
        assert_eq!(rvdff, correct_rvdff)
    }

    #[test]
    fn serialize_lfa_without_v() {
        let matrix = get_matrix();
        let correct_rvdff = get_rvdff(false); // Decompose via LFA
        let options = LoPhatOptions {
            maintain_v: false, // Just do normal left-to-right reduction in decreasing dimensions
            clearing: false,
            num_threads: 1, // So we can predict the output
            ..Default::default()
        };
        let decomp = LockFreeAlgorithm::init(Some(options))
            .add_cols(matrix)
            .decompose();
        // Serialize into bytes
        let mut bytes: Vec<u8> = vec![];
        into_writer(&decomp, &mut bytes).ok();
        // Deserialize to file format
        let rvdff: DecompositionFileFormat = from_reader(bytes.as_slice()).ok().unwrap();
        // Check all columns are correct and V is none
        assert_eq!(rvdff, correct_rvdff)
    }
}
