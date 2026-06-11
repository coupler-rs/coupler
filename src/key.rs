pub struct Key<'a> {
    version: usize,
    str: &'a str,
}

impl<'a> Key<'a> {
    pub fn new(str: &'a str, version: usize) -> Key<'a> {
        Key { version, str }
    }
}

impl<'a> From<&'a str> for Key<'a> {
    fn from(value: &'a str) -> Key<'a> {
        Key::new(value, 0)
    }
}
