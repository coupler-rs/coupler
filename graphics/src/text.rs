use swash::{Attributes, CacheKey, Charmap, FontRef};

pub struct Font {
    data: Vec<u8>,
    offset: u32,
    key: CacheKey,
}

impl Font {
    pub fn from_bytes(data: &[u8], index: usize) -> Option<Font> {
        let data = data.to_vec();
        let font = FontRef::from_index(&data, index)?;
        let (offset, key) = (font.offset, font.key);
        Some(Self { data, offset, key })
    }

    pub(crate) fn as_ref(&self) -> FontRef {
        FontRef { data: &self.data, offset: self.offset, key: self.key }
    }
}
