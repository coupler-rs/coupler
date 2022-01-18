#[derive(Copy, Clone)]
pub struct Color(u32);

impl Color {
    #[inline]
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | ((b as u32) << 0))
    }

    #[inline]
    pub fn r(&self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    #[inline]
    pub fn g(&self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    #[inline]
    pub fn b(&self) -> u8 {
        ((self.0 >> 0) & 0xFF) as u8
    }

    #[inline]
    pub fn a(&self) -> u8 {
        ((self.0 >> 24) & 0xFF) as u8
    }
}

impl From<u32> for Color {
    fn from(value: u32) -> Color {
        Color(value)
    }
}

impl From<Color> for u32 {
    fn from(color: Color) -> u32 {
        color.0
    }
}
