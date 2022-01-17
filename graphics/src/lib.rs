mod geom;
mod path;
mod text;

pub use geom::*;
pub use path::*;
pub use text::*;

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
    coverage: Vec<f32>,
    tiles_width: usize,
    tiles: Vec<bool>,
}

impl Canvas {
    pub fn with_size(width: usize, height: usize) -> Canvas {
        let tiles_width = (width + 8) >> 3;
        let tiles_height = height;
        let tiles = vec![false; tiles_width * tiles_height];

        Canvas {
            width,
            height,
            data: vec![0xFF000000; width * height + 1],
            coverage: vec![0.0; width * height + 1],
            tiles_width,
            tiles,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn data(&self) -> &[u32] {
        &self.data[0..self.width * self.height]
    }

    pub fn clear(&mut self, color: Color) {
        for pixel in self.data.iter_mut() {
            *pixel = color.0;
        }
    }

    pub fn fill_path(&mut self, path: &Path, color: Color) {
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
        let right = (max.x.floor() as isize + 8).max(0).min(self.width as isize) as usize;
        let top = (min.y.floor() as isize).max(0).min(self.height as isize) as usize;
        let bottom = (max.y.floor() as isize + 1).max(0).min(self.height as isize) as usize;

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

            if p1.y != p2.y {
                let x_dir = (p2.x - p1.x).signum() as isize;
                let y_dir = (p2.y - p1.y).signum() as isize;
                let dtdx = 1.0 / (p2.x - p1.x);
                let dtdy = 1.0 / (p2.y - p1.y);
                let mut x = p1.x.floor() as isize;
                let mut y = p1.y.floor() as isize;
                let mut row_t0: f32 = 0.0;
                let mut col_t0: f32 = 0.0;
                let mut row_t1 = if p1.y == p2.y {
                    std::f32::INFINITY
                } else {
                    let next_y = if p2.y > p1.y { (y + 1) as f32 } else { y as f32 };
                    (dtdy * (next_y - p1.y)).min(1.0)
                };
                let mut col_t1 = if p1.x == p2.x {
                    std::f32::INFINITY
                } else {
                    let next_x = if p2.x > p1.x { (x + 1) as f32 } else { x as f32 };
                    (dtdx * (next_x - p1.x)).min(1.0)
                };
                let x_step = dtdx.abs();
                let y_step = dtdy.abs();

                let mut prev = p1;

                loop {
                    let t1 = row_t1.min(col_t1);
                    let next = (1.0 - t1) * p1 + t1 * p2;
                    let height = (next.y - prev.y) as f32;
                    let right_edge = (x + 1) as f32;
                    let area =
                        0.5 * height * ((right_edge - prev.x) + (right_edge - next.x)) as f32;

                    if x >= 0 as isize
                        && x < self.width as isize
                        && y >= 0 as isize
                        && y < self.height as isize
                    {
                        let tile_x = x as usize >> 3;
                        let tile_y = y as usize;
                        self.tiles[tile_y * self.tiles_width + tile_x] = true;
                        self.coverage[(y as usize * self.width) + x as usize] += area;

                        let tile_x = (x + 1) as usize >> 3;
                        let tile_y = y as usize;
                        self.tiles[tile_y * self.tiles_width + tile_x] = true;
                        self.coverage[(y as usize * self.width) + (x + 1) as usize] +=
                            height - area;
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

                    prev = next;
                }
            }

            if command.is_none() {
                break;
            }
        }

        for row in top..bottom {
            let mut accum = 0.0;
            let mut coverage = 0;

            let tile_row = row;

            let mut tile_span_start = left >> 3;
            while tile_span_start < (right >> 3) + 1 {
                let current = self.tiles[tile_row * self.tiles_width + tile_span_start];
                self.tiles[tile_row * self.tiles_width + tile_span_start] = false;
                let mut tile_span_end = tile_span_start + 1;
                while tile_span_end < self.tiles_width
                    && self.tiles[tile_row * self.tiles_width + tile_span_end] == current
                {
                    self.tiles[tile_row * self.tiles_width + tile_span_end] = false;
                    tile_span_end += 1;
                }

                let span_start = tile_span_start << 3;
                let span_end = (tile_span_end << 3).min(self.width);

                if current {
                    for col in span_start..span_end {
                        accum += self.coverage[row * self.width + col];
                        self.coverage[row * self.width + col] = 0.0;

                        let pixel = self.data[row * self.width + col];

                        coverage = (accum.abs() * 255.0 + 0.5).min(255.0) as u32;

                        let mut r = (coverage * color.r() as u32 + 127) / 255;
                        let mut g = (coverage * color.g() as u32 + 127) / 255;
                        let mut b = (coverage * color.b() as u32 + 127) / 255;
                        let mut a = (coverage * color.a() as u32 + 127) / 255;

                        let inv_a = 255 - a;

                        a += (inv_a * ((pixel >> 24) & 0xFF) + 127) / 255;
                        r += (inv_a * ((pixel >> 16) & 0xFF) + 127) / 255;
                        g += (inv_a * ((pixel >> 8) & 0xFF) + 127) / 255;
                        b += (inv_a * ((pixel >> 0) & 0xFF) + 127) / 255;

                        self.data[row * self.width + col] =
                            (a << 24) | (r << 16) | (g << 8) | (b << 0);
                    }
                } else if coverage == 255 {
                    for col in span_start..span_end {
                        let r = (coverage * color.r() as u32 + 127) / 255;
                        let g = (coverage * color.g() as u32 + 127) / 255;
                        let b = (coverage * color.b() as u32 + 127) / 255;
                        let a = (coverage * color.a() as u32 + 127) / 255;
                        self.data[row * self.width + col] =
                            (a << 24) | (r << 16) | (g << 8) | (b << 0);
                    }
                } else if coverage != 0 {
                    for col in span_start..span_end {
                        let pixel = self.data[row * self.width + col];

                        let mut r = (coverage * color.r() as u32 + 127) / 255;
                        let mut g = (coverage * color.g() as u32 + 127) / 255;
                        let mut b = (coverage * color.b() as u32 + 127) / 255;
                        let mut a = (coverage * color.a() as u32 + 127) / 255;

                        let inv_a = 255 - a;

                        a += (inv_a * ((pixel >> 24) & 0xFF) + 127) / 255;
                        r += (inv_a * ((pixel >> 16) & 0xFF) + 127) / 255;
                        g += (inv_a * ((pixel >> 8) & 0xFF) + 127) / 255;
                        b += (inv_a * ((pixel >> 0) & 0xFF) + 127) / 255;

                        self.data[row * self.width + col] =
                            (a << 24) | (r << 16) | (g << 8) | (b << 0);
                    }
                }

                tile_span_start = tile_span_end;
            }
        }
    }

    pub fn stroke_path(&mut self, path: &Path, width: f32, color: Color) {
        self.fill_path(&path.stroke(width), color);
    }

    pub fn fill_text(&mut self, text: &str, font: &Font, size: f32, color: Color) {
        use swash::scale::*;
        use swash::shape::*;
        use zeno::*;

        let mut shape_context = ShapeContext::new();
        let mut shaper = shape_context.builder(font.as_ref()).size(size).build();

        let mut scale_context = ScaleContext::new();
        let mut scaler = scale_context.builder(font.as_ref()).size(size).build();

        let mut offset = 1.0;
        shaper.add_str(text);
        shaper.shape_with(|cluster| {
            for glyph in cluster.glyphs {
                if let Some(outline) = scaler.scale_outline(glyph.id) {
                    let mut path = Path::new();

                    let mut points = outline.points().iter();
                    for verb in outline.verbs() {
                        match verb {
                            Verb::MoveTo => {
                                let point = points.next().unwrap();
                                path.move_to(Vec2::new(point.x + offset, -point.y + size));
                            }
                            Verb::LineTo => {
                                let point = points.next().unwrap();
                                path.line_to(Vec2::new(point.x + offset, -point.y + size));
                            }
                            Verb::CurveTo => {
                                let control1 = points.next().unwrap();
                                let control2 = points.next().unwrap();
                                let point = points.next().unwrap();
                                path.cubic_to(
                                    Vec2::new(control1.x + offset, -control1.y + size),
                                    Vec2::new(control2.x + offset, -control2.y + size),
                                    Vec2::new(point.x + offset, -point.y + size),
                                );
                            }
                            Verb::QuadTo => {
                                let control = points.next().unwrap();
                                let point = points.next().unwrap();
                                path.quadratic_to(
                                    Vec2::new(control.x + offset, -control.y + size),
                                    Vec2::new(point.x + offset, -point.y + size),
                                );
                            }
                            Verb::Close => {
                                path.close();
                            }
                        }
                    }

                    self.fill_path(&path, color);

                    offset += glyph.advance;
                }
            }
        });
    }
}
