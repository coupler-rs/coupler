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
