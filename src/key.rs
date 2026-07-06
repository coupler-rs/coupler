#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Key<'a> {
    pub generation: usize,
    pub str: &'a str,
}

impl<'a> Key<'a> {
    pub fn new(generation: usize, str: &'a str) -> Key<'a> {
        Key { generation, str }
    }
}

impl<'a> From<&'a str> for Key<'a> {
    fn from(value: &'a str) -> Key<'a> {
        Key::new(0, value)
    }
}

struct Entry {
    generation: usize,
    str: String,
    index: Option<usize>,
}

impl Entry {
    fn key(&self) -> Key<'_> {
        Key {
            generation: self.generation,
            str: &self.str,
        }
    }
}

pub(crate) struct KeyList {
    count: usize,
    entries: Vec<Entry>,
}

impl KeyList {
    pub fn new() -> Self {
        KeyList {
            count: 0,
            entries: Vec::new(),
        }
    }

    pub fn key<'k>(&mut self, key: impl Into<Key<'k>>) {
        let key = key.into();
        self.entries.push(Entry {
            generation: key.generation,
            str: key.str.to_string(),
            index: Some(self.count),
        });
        self.count += 1;
    }

    pub fn reserve<'k>(&mut self, key: impl Into<Key<'k>>) {
        let key = key.into();
        self.entries.push(Entry {
            generation: key.generation,
            str: key.str.to_string(),
            index: None,
        });
    }

    pub fn into_ids(self) -> Vec<u32> {
        let mut entries = self.entries;
        entries.sort_by(|a, b| a.key().cmp(&b.key()));

        let mut ids = vec![0; self.count];
        for (id, entry) in entries.iter().enumerate() {
            if let Some(index) = entry.index {
                ids[index] = id.try_into().unwrap();
            }
        }

        ids
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter;

    use super::{Key, KeyList};

    struct KeyMap {
        keys: Vec<String>,
        list: KeyList,
    }

    impl KeyMap {
        fn new() -> KeyMap {
            KeyMap {
                keys: Vec::new(),
                list: KeyList::new(),
            }
        }

        pub fn key<'k>(mut self, key: impl Into<Key<'k>>) -> Self {
            let key = key.into();
            self.keys.push(key.str.to_string());
            self.list.key(key);

            self
        }

        pub fn reserve<'k>(mut self, key: impl Into<Key<'k>>) -> Self {
            self.list.reserve(key);
            self
        }

        fn build(self) -> HashMap<String, u32> {
            let ids = self.list.into_ids();
            iter::zip(self.keys, ids).collect()
        }
    }

    #[test]
    fn reorder_keys() {
        let map1 = KeyMap::new().key("a").key("b").key("c").build();
        let map2 = KeyMap::new().key("c").key("b").key("a").build();

        // Reordering the list of keys should have no effect on the resulting IDs
        assert_eq!(map1, map2);
    }

    #[test]
    fn add_key() {
        let map1 = KeyMap::new().key("a").key("b").key("c").build();
        let map2 = KeyMap::new().key("a").key("b").key("c").key(Key::new(1, "aa")).build();

        // Adding a new key should have no effect on the IDs for keys with lower generation numbers
        for (key, id) in &map1 {
            assert_eq!(id, &map2[key]);
        }
    }

    #[test]
    fn remove_key() {
        let map1 = KeyMap::new().key("a").key("b").key("c").build();
        let map2 = KeyMap::new().key("a").reserve("b").key("c").build();

        // Removing and reserving a key should have no effect on the IDs for the remaining keys
        for (key, id) in &map2 {
            assert_eq!(id, &map1[key]);
        }
    }
}
