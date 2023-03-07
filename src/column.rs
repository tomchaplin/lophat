use std::cmp::Ordering;

pub trait Column: Sync + Clone + Send + Default {
    fn pivot(&self) -> Option<usize>;
    fn add_col(&mut self, other: &Self);
    fn add_entry(&mut self, entry: usize);
}

#[derive(Debug, Default, Clone)]
pub struct VecColumn {
    pub internal: Vec<usize>,
}

impl VecColumn {
    // Returns the index where we should try to insert next entry
    fn add_entry_starting_at(&mut self, entry: usize, starting_idx: usize) -> usize {
        let mut working_idx = starting_idx;
        while let Some(value_at_idx) = self.internal.iter().nth(working_idx) {
            match value_at_idx.cmp(&entry) {
                Ordering::Less => {
                    working_idx += 1;
                    continue;
                }
                Ordering::Equal => {
                    self.internal.remove(working_idx);
                    return working_idx;
                }
                Ordering::Greater => {
                    self.internal.insert(working_idx, entry);
                    return working_idx + 1;
                }
            }
        }
        // Bigger than all idxs in col - add to end
        self.internal.push(entry);
        return self.internal.len() - 1;
    }
}

impl Column for VecColumn {
    fn pivot(&self) -> Option<usize> {
        self.internal.iter().last().copied()
    }

    fn add_entry(&mut self, entry: usize) {
        self.add_entry_starting_at(entry, 0);
    }

    fn add_col(&mut self, other: &Self) {
        let mut working_idx = 0;
        for entry in other.internal.iter() {
            working_idx = self.add_entry_starting_at(*entry, working_idx);
        }
    }
}
