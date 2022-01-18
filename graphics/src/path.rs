use crate::geom::*;

const TOLERANCE: f32 = 0.1;

#[derive(Clone)]
pub struct Path {
    pub(crate) verbs: Vec<Verb>,
    pub(crate) points: Vec<Vec2>,
}

#[derive(Copy, Clone)]
pub enum Verb {
    Move,
    Line,
    Quadratic,
    Cubic,
    Close,
}

impl Path {
    pub fn new() -> Path {
        Path { verbs: Vec::new(), points: Vec::new() }
    }

    pub fn move_to(&mut self, point: Vec2) -> &mut Self {
        self.verbs.push(Verb::Move);
        self.points.push(point);
        self
    }

    pub fn line_to(&mut self, point: Vec2) -> &mut Self {
        self.verbs.push(Verb::Line);
        self.points.push(point);
        self
    }

    pub fn quadratic_to(&mut self, control: Vec2, point: Vec2) -> &mut Self {
        self.verbs.push(Verb::Quadratic);
        self.points.push(control);
        self.points.push(point);
        self
    }

    pub fn cubic_to(&mut self, control1: Vec2, control2: Vec2, point: Vec2) -> &mut Self {
        self.verbs.push(Verb::Cubic);
        self.points.push(control1);
        self.points.push(control2);
        self.points.push(point);
        self
    }

    pub fn arc(&mut self, radius: f32, start_angle: f32, end_angle: f32) -> &mut Self {
        let mut last = self.points.last().cloned().unwrap_or(Vec2::new(0.0, 0.0));
        let mut vector = Vec2::new(start_angle.cos(), start_angle.sin());
        let mut angle = 0.0;

        let center = last - radius * vector;
        let winding = if end_angle < start_angle { -1.0 } else { 1.0 };
        let total_angle = (end_angle - start_angle).abs();

        // approximate quarter-circle arcs with cubics
        let quarter_circle = 0.5 * std::f32::consts::PI;
        let k = (4.0 / 3.0) * (0.25 * quarter_circle).tan();
        while angle + quarter_circle < total_angle {
            let tangent = winding * Vec2::new(-vector.y, vector.x);

            let control1 = last + radius * k * tangent;
            let point = center + radius * tangent;
            let control2 = point + radius * k * vector;
            self.cubic_to(control1, control2, point);

            angle += quarter_circle;
            vector = tangent;
            last = point;
        }

        // approximate the remainder of the arc with a single cubic
        let tangent = winding * Vec2::new(-vector.y, vector.x);
        let angle_size = total_angle - angle;
        let k = (4.0 / 3.0) * (0.25 * angle_size).tan();

        let end_vector = Vec2::new(end_angle.cos(), end_angle.sin());
        let end_tangent = winding * Vec2::new(-end_vector.y, end_vector.x);

        let control1 = last + radius * k * tangent;
        let point = center + radius * end_vector;
        let control2 = point - radius * k * end_tangent;
        self.cubic_to(control1, control2, point);

        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.verbs.push(Verb::Close);
        self
    }

    pub(crate) fn flatten(&self) -> Path {
        let mut path = Path::new();
        let mut last = Vec2::new(0.0, 0.0);
        let mut points = self.points.iter();
        for verb in self.verbs.iter() {
            match *verb {
                Verb::Move => {
                    let point = *points.next().unwrap();
                    path.move_to(point);
                    last = point;
                }
                Verb::Line => {
                    let point = *points.next().unwrap();
                    path.line_to(point);
                    last = point;
                }
                Verb::Quadratic => {
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
                Verb::Cubic => {
                    let control1 = *points.next().unwrap();
                    let control2 = *points.next().unwrap();
                    let point = *points.next().unwrap();
                    let a = -1.0 * last + 3.0 * control1 - 3.0 * control2 + point;
                    let b = 3.0 * (last - 2.0 * control1 + control2);
                    let conc = b.length().max((a + b).length());
                    let dt = ((8.0f32.sqrt() * TOLERANCE) / conc).sqrt();
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
                Verb::Close => {
                    path.close();
                }
            }
        }

        path
    }

    pub(crate) fn stroke(&self, width: f32) -> Path {
        #[inline]
        fn join(path: &mut Path, width: f32, prev_normal: Vec2, next_normal: Vec2, point: Vec2) {
            let offset = 1.0 / (1.0 + prev_normal.dot(next_normal));
            if offset.abs() > 2.0 {
                path.line_to(point + 0.5 * width * prev_normal);
                path.line_to(point + 0.5 * width * next_normal);
            } else {
                path.line_to(point + 0.5 * width * offset * (prev_normal + next_normal));
            }
        }

        #[inline]
        fn offset(path: &mut Path, width: f32, contour: &[Vec2], closed: bool, reverse: bool) {
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

        let mut path = Path::new();

        let flattened = self.flatten();

        let mut contour_start = 0;
        let mut contour_end = 0;
        let mut closed = false;
        let mut verbs = flattened.verbs.iter();
        loop {
            let verb = verbs.next();

            if let Some(Verb::Close) = verb {
                closed = true;
            }

            if let None | Some(Verb::Move) | Some(Verb::Close) = verb {
                if contour_start != contour_end {
                    let contour = &flattened.points[contour_start..contour_end];

                    let base = path.verbs.len();
                    offset(&mut path, width, contour, closed, false);
                    path.verbs[base] = Verb::Move;
                    if closed {
                        path.close();
                    }

                    let base = path.verbs.len();
                    offset(&mut path, width, contour, closed, true);
                    if closed {
                        path.verbs[base] = Verb::Move;
                    }
                    path.close();
                }
            }

            if let Some(verb) = verb {
                match verb {
                    Verb::Move => {
                        contour_start = contour_end;
                        contour_end = contour_start + 1;
                    }
                    Verb::Line => {
                        contour_end += 1;
                    }
                    Verb::Close => {
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

        path
    }
}
