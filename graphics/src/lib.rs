mod geom;

pub use geom::*;

struct TileMap {
    width: usize,
    height: usize,
    tiles: Vec<Option<usize>>,
    data: Vec<f32>,
}

impl TileMap {
    fn with_size(width: usize, height: usize) -> TileMap {
        TileMap { width, height, tiles: vec![None; width * height], data: Vec::new() }
    }
}

#[derive(Copy, Clone)]
pub struct Color(u32);

impl Color {
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | ((b as u32) << 0))
    }

    pub fn r(&self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    pub fn g(&self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    pub fn b(&self) -> u8 {
        ((self.0 >> 0) & 0xFF) as u8
    }

    pub fn a(&self) -> u8 {
        ((self.0 >> 24) & 0xFF) as u8
    }
}

pub struct Canvas {
    width: usize,
    height: usize,
    data: Vec<u32>,
}

impl Canvas {
    pub fn with_size(width: usize, height: usize) -> Canvas {
        Canvas { width, height, data: vec![0xFFFFFFFF; width * height] }
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

    pub fn fill(&mut self, path: &Path, color: Color) {
        if path.points.is_empty() {
            return;
        }

        let flattened = path.flatten();

        let mut min = flattened.points[0];
        let mut max = flattened.points[0];
        for point in flattened.points.iter() {
            min = min.min(*point);
            max = max.max(*point);
        }

        let left = (min.x.floor() as isize).max(0).min(self.width as isize) as usize;
        let right = (max.x.floor() as isize + 2).max(0).min(self.width as isize) as usize;
        let top = (min.y.floor() as isize).max(0).min(self.height as isize) as usize;
        let bottom = (max.y.floor() as isize + 1).max(0).min(self.height as isize) as usize;
        let width = right - left;
        let height = bottom - top;

        let mut tiles = TileMap::with_size((width + 8) >> 3, (height + 8) >> 3);

        let mut first = Vec2::new(0.0, 0.0);
        let mut last = Vec2::new(0.0, 0.0);

        let mut commands = flattened.commands.iter();
        let mut points = flattened.points.iter();
        loop {
            let command = commands.next();

            let p1;
            let p2;
            if let Some(command) = command {
                match command {
                    PathCmd::Move => {
                        let point = *points.next().unwrap();
                        p1 = last;
                        p2 = first;
                        first = point;
                        last = point;
                    }
                    PathCmd::Line => {
                        let point = *points.next().unwrap();
                        p1 = last;
                        p2 = point;
                        last = point;
                    }
                    PathCmd::Close => {
                        continue;
                    }
                    _ => {
                        unreachable!();
                    }
                }
            } else {
                p1 = last;
                p2 = first;
            }

            if p1 != p2 {
                let x_dir = (p2.x - p1.x).signum() as isize;
                let y_dir = (p2.y - p1.y).signum() as isize;
                let dtdx = 1.0 / (p2.x - p1.x);
                let dtdy = 1.0 / (p2.y - p1.y);
                let mut x = p1.x.floor() as isize;
                let mut y = p1.y.floor() as isize;
                let mut row_t0: f64 = 0.0;
                let mut col_t0: f64 = 0.0;
                let mut row_t1 = if p1.y == p2.y {
                    std::f64::INFINITY
                } else {
                    let next_y = if p2.y > p1.y { (y + 1) as f64 } else { y as f64 };
                    (dtdy * (next_y - p1.y)).min(1.0)
                };
                let mut col_t1 = if p1.x == p2.x {
                    std::f64::INFINITY
                } else {
                    let next_x = if p2.x > p1.x { (x + 1) as f64 } else { x as f64 };
                    (dtdx * (next_x - p1.x)).min(1.0)
                };
                let x_step = dtdx.abs();
                let y_step = dtdy.abs();

                loop {
                    let t0 = row_t0.max(col_t0);
                    let t1 = row_t1.min(col_t1);
                    let p0 = (1.0 - t0) * p1 + t0 * p2;
                    let p1 = (1.0 - t1) * p1 + t1 * p2;
                    let height = (p1.y - p0.y) as f32;
                    let right_edge = (x + 1) as f64;
                    let area = 0.5 * height * ((right_edge - p0.x) + (right_edge - p1.x)) as f32;

                    if x >= left as isize
                        && x < right as isize
                        && y >= top as isize
                        && y < bottom as isize
                    {
                        #[inline(always)]
                        fn add(tile_map: &mut TileMap, x: usize, y: usize, delta: f32) {
                            let tile_x = x >> 3;
                            let tile_y = y >> 3;
                            let tile = &mut tile_map.tiles[tile_y * tile_map.width + tile_x];
                            let index = if let Some(index) = tile {
                                *index
                            } else {
                                let len = tile_map.data.len();
                                tile_map.data.resize(len + 64, 0.0);
                                *tile = Some(len);
                                len
                            };
                            let tile_data = &mut tile_map.data[index..index + 64];
                            tile_data[((y & 0x7) << 3) | (x & 0x7)] += delta;
                        }

                        let x = x as usize - left;
                        let y = y as usize - top;
                        add(&mut tiles, x, y, area);
                        add(&mut tiles, x + 1, y, height - area);
                    }

                    if row_t1 < col_t1 {
                        row_t0 = row_t1;
                        row_t1 = (row_t1 + y_step).min(1.0);
                        y += y_dir;
                    } else {
                        col_t0 = col_t1;
                        col_t1 = (col_t1 + x_step).min(1.0);
                        x += x_dir;
                    }

                    if row_t0 == 1.0 || col_t0 == 1.0 {
                        break;
                    }
                }
            }

            if command.is_none() {
                break;
            }
        }

        for row in 0..height {
            let mut accum = 0.0;
            let mut coverage = 0;
            for tile_col in 0..tiles.width {
                let columns = if tile_col == tiles.width - 1 { width % 8 } else { 8 };
                if let Some(index) = tiles.tiles[(row >> 3) * tiles.width + tile_col] {
                    let tile_data = &mut tiles.data[index..index + 64];
                    for col in 0..columns {
                        accum += tile_data[((row & 0x7) << 3) | col];

                        let pixel = &mut self.data[(top + row) * self.width + left + (tile_col << 3) + col];

                        coverage = (accum.abs() * 255.0 + 0.5).min(255.0) as u32;

                        let mut r = (coverage * color.r() as u32 + 127) / 255;
                        let mut g = (coverage * color.g() as u32 + 127) / 255;
                        let mut b = (coverage * color.b() as u32 + 127) / 255;
                        let mut a = (coverage * color.a() as u32 + 127) / 255;

                        let inv_a = 255 - a;

                        a += (inv_a * ((*pixel >> 24) & 0xFF) + 127) / 255;
                        r += (inv_a * ((*pixel >> 16) & 0xFF) + 127) / 255;
                        g += (inv_a * ((*pixel >> 8) & 0xFF) + 127) / 255;
                        b += (inv_a * ((*pixel >> 0) & 0xFF) + 127) / 255;

                        self.data[(top + row) * self.width + left + (tile_col << 3) + col] =
                            (a << 24) | (r << 16) | (g << 8) | (b << 0);
                    }
                } else {
                    if coverage == 255 {
                        for col in 0..columns {
                            let r = (coverage * color.r() as u32 + 127) / 255;
                            let g = (coverage * color.g() as u32 + 127) / 255;
                            let b = (coverage * color.b() as u32 + 127) / 255;
                            let a = (coverage * color.a() as u32 + 127) / 255;
                            self.data[(top + row) * self.width + left + (tile_col << 3) + col] =
                                (a << 24) | (r << 16) | (g << 8) | (b << 0);
                        }
                    } else if coverage != 0 {
                        for col in 0..columns {
                            let pixel = &mut self.data[(top + row) * self.width + left + (tile_col << 3) + col];

                            let mut r = (coverage * color.r() as u32 + 127) / 255;
                            let mut g = (coverage * color.g() as u32 + 127) / 255;
                            let mut b = (coverage * color.b() as u32 + 127) / 255;
                            let mut a = (coverage * color.a() as u32 + 127) / 255;

                            let inv_a = 255 - a;

                            a += (inv_a * ((*pixel >> 24) & 0xFF) + 127) / 255;
                            r += (inv_a * ((*pixel >> 16) & 0xFF) + 127) / 255;
                            g += (inv_a * ((*pixel >> 8) & 0xFF) + 127) / 255;
                            b += (inv_a * ((*pixel >> 0) & 0xFF) + 127) / 255;

                            self.data[(top + row) * self.width + left + (tile_col << 3) + col] =
                                (a << 24) | (r << 16) | (g << 8) | (b << 0);
                        }
                    }
                }
            }
        }
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
                        contour_start = contour_end + 1;
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
