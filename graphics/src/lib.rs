mod geom;

use geom::Vec2;

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

pub struct Path {
    commands: Vec<PathCmd>,
    points: Vec<Vec2>,
}

pub enum PathCmd {
    Move,
    Line,
    Quadratic,
    Cubic,
    Close,
}

pub struct PathBuilder {
    commands: Vec<PathCmd>,
    points: Vec<Vec2>,
}

impl Path {
    pub fn builder() -> PathBuilder {
        PathBuilder {
            commands: Vec::new(),
            points: Vec::new(),
        }
    }
}

impl PathBuilder {
    pub fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.commands.push(PathCmd::Move);
        self.points.push(Vec2::new(x, y));
        self
    }

    pub fn line_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.commands.push(PathCmd::Line);
        self.points.push(Vec2::new(x, y));
        self
    }

    pub fn quadratic_to(&mut self, x1: f64, y1: f64, x: f64, y: f64) -> &mut Self {
        self.commands.push(PathCmd::Quadratic);
        self.points.push(Vec2::new(x1, y1));
        self.points.push(Vec2::new(x, y));
        self
    }

    pub fn cubic_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) -> &mut Self {
        self.commands.push(PathCmd::Cubic);
        self.points.push(Vec2::new(x1, y1));
        self.points.push(Vec2::new(x2, y2));
        self.points.push(Vec2::new(x, y));
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.commands.push(PathCmd::Close);
        self
    }

    pub fn build(self) -> Path {
        Path {
            commands: self.commands,
            points: self.points,
        }
    }
}
