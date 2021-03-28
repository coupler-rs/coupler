pub struct Image {
    width: usize,
    height: usize,
    data: Vec<u32>,
}

impl Image {
    pub fn with_size(width: usize, height: usize) -> Image {
        Image { width, height, data: vec![0; width * height] }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn data(&self) -> &[u32] {
        &self.data
    }
}

pub struct Canvas<'i> {
    target: &'i mut Image,
    scale_factor: f32,
}

impl<'i> Canvas<'i> {
    pub fn with_target(target: &'i mut Image, scale_factor: f32) -> Canvas<'i> {
        Canvas { target, scale_factor }
    }

    pub fn push(&'i mut self) -> Canvas<'i> {
        Canvas { target: self.target, scale_factor: self.scale_factor }
    }
}
