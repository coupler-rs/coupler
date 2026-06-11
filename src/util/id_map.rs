use std::collections::HashMap;
use std::hash::Hash;

pub struct IdMap<T> {
    ids: Vec<T>,
    map: HashMap<T, usize>,
}

impl<T: Hash + Eq + Clone> IdMap<T> {
    pub fn from_ids<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let ids: Vec<T> = iter.into_iter().collect();

        let mut map = HashMap::new();
        for (index, id) in ids.iter().enumerate() {
            map.insert(id.clone(), index);
        }

        IdMap { ids, map }
    }

    pub fn id_from_index(&self, index: usize) -> Option<&T> {
        self.ids.get(index)
    }

    pub fn index_from_id(&self, id: &T) -> Option<usize> {
        self.map.get(id).copied()
    }
}
