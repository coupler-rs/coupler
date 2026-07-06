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
