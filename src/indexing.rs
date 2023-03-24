pub trait IndexMap<'a>: Clone {
    type InverseMap: IndexMap<'a>;
    fn map_idx(&self, index: usize) -> Option<usize>;
    fn inverse(&self) -> Self::InverseMap;
}

#[derive(Clone)]
pub struct VecIndexMap {
    mapping: Vec<usize>,
}

#[derive(Clone)]
pub struct InverseVecIndexMap<'a> {
    underlying: &'a VecIndexMap,
}

impl<'a> IndexMap<'a> for &'a VecIndexMap {
    type InverseMap = InverseVecIndexMap<'a>;
    fn map_idx(&self, index: usize) -> Option<usize> {
        self.mapping.get(index).cloned()
    }

    fn inverse(&self) -> Self::InverseMap {
        InverseVecIndexMap { underlying: &self }
    }
}

impl<'a> IndexMap<'a> for InverseVecIndexMap<'a> {
    type InverseMap = &'a VecIndexMap;

    fn map_idx(&self, index: usize) -> Option<usize> {
        self.underlying
            .mapping
            .iter()
            .position(|&elem| elem == index)
    }

    fn inverse(&self) -> Self::InverseMap {
        self.underlying
    }
}

pub struct ReverseIndexMap {
    index_len: usize,
}

impl<'a> IndexMap<'a> for &'a ReverseIndexMap {
    type InverseMap = &'a ReverseIndexMap;

    fn map_idx(&self, index: usize) -> Option<usize> {
        if index >= self.index_len {
            None
        } else {
            Some(self.index_len - 1 - index)
        }
    }

    fn inverse(&self) -> Self::InverseMap {
        self
    }
}
