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
    tiles: Vec<u64>,
    min: Vec2,
    max: Vec2,
}

impl Rasterizer {
    pub fn with_size(width: usize, height: usize) -> Rasterizer {
        let tiles_width = ((width + 8 * 64) >> 3) >> 6;
        let tiles_height = height;
        let tiles = vec![0; tiles_width * tiles_height];

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
                    self.tiles[tile_y * self.tiles_width + (tile_x >> 6)] |= 1 << (63 - (tile_x & 0x3F));
                    self.coverage[(y as usize * self.width) + x as usize] += area;

                    let tile_x = (x + 1) as usize >> 3;
                    let tile_y = y as usize;
                    self.tiles[tile_y * self.tiles_width + (tile_x >> 6)] |= 1 << (63 - (tile_x & 0x3F));
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

            let mut tile_col = left >> 9;
            while tile_col < (right >> 9) + 1 {
                let mut tile = std::mem::replace(&mut self.tiles[tile_row * self.tiles_width + tile_col], 0);

                let mut start_index = 0;
                let mut start_x = tile_col << 9;
                loop {
                    let index = tile.leading_zeros() as usize;
                    let x = (tile_col << 9) + (index << 3);

                    if index > start_index {
                        if coverage * 255.0 >= 254.5 {
                            sink(Span {
                                x: start_x,
                                y: row,
                                width: x - start_x,
                                contents: Contents::Solid,
                            });
                        } else if coverage >= 1.0 / 255.0 {
                            sink(Span {
                                x: start_x,
                                y: row,
                                width: x - start_x,
                                contents: Contents::Constant(coverage),
                            });
                        }
                    }

                    if index == 64 {
                        break;
                    }

                    for col in x..x + 8 {
                        accum += self.coverage[row * self.width + col];
                        coverage = accum.abs().min(1.0);
                        self.coverage[row * self.width + col] = coverage;
                    }

                    sink(Span {
                        x: x,
                        y: row,
                        width: 8,
                        contents: Contents::Mask(
                            &self.coverage
                                [row * self.width + x..row * self.width + x + 8],
                        ),
                    });

                    for col in x..x + 8 {
                        self.coverage[row * self.width + col] = 0.0;
                    }

                    tile &= !(1 << (63 - index));
                    start_index = index + 1;
                    start_x = x + 8;
                }

                tile_col += 1;
            }
        }

        self.min = Vec2::new(self.width as f32, self.height as f32);
        self.max = Vec2::new(0.0, 0.0);
    }
}
