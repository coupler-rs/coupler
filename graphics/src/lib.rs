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

const TOLERANCE: f64 = 0.1;

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
    pub fn move_to(&mut self, point: Vec2) -> &mut Self {
        self.commands.push(PathCmd::Move);
        self.points.push(point);
        self
    }

    pub fn line_to(&mut self, point: Vec2) -> &mut Self {
        self.commands.push(PathCmd::Line);
        self.points.push(point);
        self
    }

    pub fn quadratic_to(&mut self, control: Vec2, point: Vec2) -> &mut Self {
        self.commands.push(PathCmd::Quadratic);
        self.points.push(control);
        self.points.push(point);
        self
    }

    pub fn cubic_to(&mut self, control1: Vec2, control2: Vec2, point: Vec2) -> &mut Self {
        self.commands.push(PathCmd::Cubic);
        self.points.push(control1);
        self.points.push(control2);
        self.points.push(point);
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

impl Path {
    fn flatten(&self) -> Path {
        let mut path = Path::builder();
        let mut last = Vec2::new(0.0, 0.0);
        let mut points = self.points.iter();
        for command in self.commands.iter() {
            match *command {
                PathCmd::Move => {
                    let point = *points.next().unwrap();
                    path.move_to(point);
                    last = point;
                }
                PathCmd::Line => {
                    let point = *points.next().unwrap();
                    path.line_to(point);
                    last = point;
                }
                PathCmd::Quadratic => {
                    let control = *points.next().unwrap();
                    let point = *points.next().unwrap();
                    let dt = ((4.0 * TOLERANCE) / (last - 2.0 * control + point).length()).sqrt();
                    let mut t = 0.0;
                    while t < 1.0 {
                        t = (t + dt).min(1.0);
                        let p01 = Vec2::lerp(t, last, control);
                        let p12 = Vec2::lerp(t, control, point);
                        path.line_to(Vec2::lerp(t, p01, p12));
                    }
                    last = point;
                }
                PathCmd::Cubic => {
                    let control1 = *points.next().unwrap();
                    let control2 = *points.next().unwrap();
                    let point = *points.next().unwrap();
                    let a = -1.0 * last + 3.0 * control1 - 3.0 * control2 + point;
                    let b = 3.0 * (last - 2.0 * control1 + control2);
                    let conc = b.length().max((a + b).length());
                    let dt = ((8.0f64.sqrt() * TOLERANCE) / conc).sqrt();
                    let mut t = 0.0;
                    while t < 1.0 {
                        t = (t + dt).min(1.0);
                        let p01 = Vec2::lerp(t, last, control1);
                        let p12 = Vec2::lerp(t, control1, control2);
                        let p23 = Vec2::lerp(t, control2, point);
                        let p012 = Vec2::lerp(t, p01, p12);
                        let p123 = Vec2::lerp(t, p12, p23);
                        path.line_to(Vec2::lerp(t, p012, p123));
                    }
                    last = point;
                }
                PathCmd::Close => {
                    path.close();
                }
            }
        }
        path.build()
    }
}
