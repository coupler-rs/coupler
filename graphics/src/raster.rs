use crate::geom::Vec2;

#[derive(Debug)]
pub struct Span<'a> {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub contents: Contents<'a>,
}

#[derive(Debug)]
pub enum Contents<'a> {
    Solid,
    Constant(f32),
    Mask(&'a [f32]),
}

pub struct Rasterizer {
    width: usize,
    height: usize,
    coverage: Vec<f32>,
    tiles_width: usize,
    tiles: Vec<bool>,
    min: Vec2,
    max: Vec2,
}

impl Rasterizer {
    pub fn with_size(width: usize, height: usize) -> Rasterizer {
        let tiles_width = (width + 8) >> 3;
        let tiles_height = height;
        let tiles = vec![false; tiles_width * tiles_height];

        Rasterizer {
            width,
            height,
            coverage: vec![0.0; width * height + 1],
            tiles_width,
            tiles,
            min: Vec2::new(width as f32, height as f32),
            max: Vec2::new(0.0, 0.0),
        }
    }

    pub fn add_line(&mut self, p1: Vec2, p2: Vec2) {
        self.min = self.min.min(p1);
        self.min = self.min.min(p2);
        self.max = self.max.max(p1);
        self.max = self.max.max(p2);

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
                let area = 0.5 * height * ((right_edge - prev.x) + (right_edge - next.x)) as f32;

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
                    self.coverage[(y as usize * self.width) + (x + 1) as usize] += height - area;
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
    }

    pub fn finish(&mut self, mut sink: impl FnMut(Span)) {
        let left = (self.min.x.floor() as isize).max(0).min(self.width as isize) as usize;
        let right = (self.max.x.floor() as isize + 8).max(0).min(self.width as isize) as usize;
        let top = (self.min.y.floor() as isize).max(0).min(self.height as isize) as usize;
        let bottom = (self.max.y.floor() as isize + 1).max(0).min(self.height as isize) as usize;

        for row in top..bottom {
            let mut accum = 0.0;
            let mut coverage = 0.0;

            let tile_row = row;

            let mut tile_span_start = left >> 3;
            while tile_span_start < (right >> 3) + 1 {
                let current = self.tiles[tile_row * self.tiles_width + tile_span_start];
                self.tiles[tile_row * self.tiles_width + tile_span_start] = false;
                let mut tile_span_end = tile_span_start + 1;
                while tile_span_end < (right >> 3) + 1
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
                        coverage = accum.abs().min(1.0);
                        self.coverage[row * self.width + col] = coverage;
                    }

                    sink(Span {
                        x: span_start,
                        y: row,
                        width: span_end - span_start,
                        contents: Contents::Mask(
                            &self.coverage
                                [row * self.width + span_start..row * self.width + span_end],
                        ),
                    });

                    for col in span_start..span_end {
                        self.coverage[row * self.width + col] = 0.0;
                    }
                } else if coverage * 255.0 >= 254.5 {
                    sink(Span {
                        x: span_start,
                        y: row,
                        width: span_end - span_start,
                        contents: Contents::Solid,
                    });
                } else if coverage >= 1.0 / 255.0 {
                    sink(Span {
                        x: span_start,
                        y: row,
                        width: span_end - span_start,
                        contents: Contents::Constant(coverage),
                    });
                }

                tile_span_start = tile_span_end;
            }
        }

        self.min = Vec2::new(self.width as f32, self.height as f32);
        self.max = Vec2::new(0.0, 0.0);
    }
}
