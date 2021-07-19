use crate::geom::*;

const TOLERANCE: f64 = 0.1;

#[derive(Clone)]
pub struct Path {
    pub(crate) commands: Vec<PathCmd>,
    pub(crate) points: Vec<Vec2>,
}

#[derive(Copy, Clone)]
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
        PathBuilder { commands: Vec::new(), points: Vec::new() }
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
        Path { commands: self.commands, points: self.points }
    }
}

impl Path {
    pub(crate) fn flatten(&self) -> Path {
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

    pub fn stroke(&self, width: f64) -> Path {
        #[inline]
        fn join(
            path: &mut PathBuilder,
            width: f64,
            prev_normal: Vec2,
            next_normal: Vec2,
            point: Vec2,
        ) {
            let offset = 1.0 / (1.0 + prev_normal.dot(next_normal));
            if offset.abs() > 2.0 {
                path.line_to(point + 0.5 * width * prev_normal);
                path.line_to(point + 0.5 * width * next_normal);
            } else {
                path.line_to(point + 0.5 * width * offset * (prev_normal + next_normal));
            }
        }

        #[inline]
        fn offset(
            path: &mut PathBuilder,
            width: f64,
            contour: &[Vec2],
            closed: bool,
            reverse: bool,
        ) {
            let first_point = if closed == reverse { contour[0] } else { *contour.last().unwrap() };
            let mut prev_point = first_point;
            let mut prev_normal = Vec2::new(0.0, 0.0);
            let mut i = 0;
            loop {
                let next_point = if i < contour.len() {
                    if reverse {
                        contour[contour.len() - i - 1]
                    } else {
                        contour[i]
                    }
                } else {
                    first_point
                };

                if next_point != prev_point || i == contour.len() {
                    let next_tangent = next_point - prev_point;
                    let next_normal = Vec2::new(-next_tangent.y, next_tangent.x);
                    let next_normal_len = next_normal.length();
                    let next_normal = if next_normal_len == 0.0 {
                        Vec2::new(0.0, 0.0)
                    } else {
                        next_normal * (1.0 / next_normal_len)
                    };

                    join(path, width, prev_normal, next_normal, prev_point);

                    prev_point = next_point;
                    prev_normal = next_normal;
                }

                i += 1;
                if i > contour.len() {
                    break;
                }
            }
        }

        let mut path = Path::builder();

        let flattened = self.flatten();

        let mut contour_start = 0;
        let mut contour_end = 0;
        let mut closed = false;
        let mut commands = flattened.commands.iter();
        loop {
            let command = commands.next();

            if let Some(PathCmd::Close) = command {
                closed = true;
            }

            if let None | Some(PathCmd::Move) | Some(PathCmd::Close) = command {
                if contour_start != contour_end {
                    let contour = &flattened.points[contour_start..contour_end];

                    let base = path.commands.len();
                    offset(&mut path, width, contour, closed, false);
                    path.commands[base] = PathCmd::Move;
                    if closed {
                        path.close();
                    }

                    let base = path.commands.len();
                    offset(&mut path, width, contour, closed, true);
                    if closed {
                        path.commands[base] = PathCmd::Move;
                    }
                    path.close();
                }
            }

            if let Some(command) = command {
                match command {
                    PathCmd::Move => {
                        contour_start = contour_end;
                        contour_end = contour_start + 1;
                    }
                    PathCmd::Line => {
                        contour_end += 1;
                    }
                    PathCmd::Close => {
                        contour_start = contour_end;
                        contour_end = contour_start;
                        closed = true;
                    }
                    _ => {
                        unreachable!();
                    }
                }
            } else {
                break;
            }
        }

        path.build()
    }
}
