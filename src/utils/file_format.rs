use std::{fs::File, path::Path};

use serde::{Deserialize, Serialize};

use ciborium::{de::from_reader, ser::into_writer};

use crate::{
    algorithms::RVDecomposition,
    columns::{Column, VecColumn},
};

#[derive(Serialize, Deserialize)]
pub struct RVDFileFormat {
    r: Vec<VecColumn>,
    v: Option<Vec<VecColumn>>,
}

impl RVDFileFormat {
    pub fn new(r: Vec<VecColumn>, v: Option<Vec<VecColumn>>) -> Self {
        Self { r, v }
    }

    pub fn save_to_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), ciborium::ser::Error<std::io::Error>> {
        let file = File::create(path)?;
        into_writer(&self, file)?;
        Ok(())
    }

    pub fn read_from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<Self, ciborium::de::Error<std::io::Error>> {
        let file = File::open(path)?;
        from_reader(file)
    }
}

impl RVDecomposition<VecColumn> for RVDFileFormat {
    type Options = ();

    fn decompose(
        _matrix: impl Iterator<Item = VecColumn>,
        _options: Option<Self::Options>,
    ) -> Self {
        panic!("This is a file format and should not be used to decompose!")
    }

    type RColRef<'a> = &'a VecColumn
    where
        Self: 'a;

    fn get_r_col<'a>(&'a self, index: usize) -> Self::RColRef<'a> {
        &self.r[index]
    }

    type VColRef<'a> = &'a VecColumn
    where
        Self: 'a;

    fn get_v_col<'a>(&'a self, index: usize) -> Option<Self::VColRef<'a>> {
        Some(&self.v.as_ref()?[index])
    }

    fn n_cols(&self) -> usize {
        self.r.len()
    }
}

pub trait ConvertToFileFormat<C>: RVDecomposition<C>
where
    C: Column,
{
    fn convert_to_file_format(&self) -> RVDFileFormat {
        let r = (0..self.n_cols())
            .map(|idx| {
                let col = self.get_r_col(idx);
                let mut output = VecColumn::new_with_dimension(col.dimension());
                output.add_entries(col.entries());
                output
            })
            .collect();
        let v = if self.get_v_col(0).is_some() {
            Some(
                (0..self.n_cols())
                    .map(|idx| {
                        let col = self.get_v_col(idx).unwrap();
                        let mut output = VecColumn::new_with_dimension(col.dimension());
                        output.add_entries(col.entries());
                        output
                    })
                    .collect(),
            )
        } else {
            None
        };
        RVDFileFormat::new(r, v)
    }
}

impl<C: Column, Algo: RVDecomposition<C>> ConvertToFileFormat<C> for Algo {}
