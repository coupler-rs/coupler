use crate::geom::Vec2;

pub const TILE_SIZE: usize = 4;
pub const TILE_SIZE_BITS: usize = 2;

const BITMASK_SIZE: usize = u64::BITS as usize;
const BITMASK_SIZE_BITS: usize = 6;

#[derive(Debug)]
pub struct Span<'a> {
    pub tile_x: usize,
    pub tile_y: usize,
    pub width: usize,
    pub contents: Contents<'a>,
}

#[derive(Debug)]
pub enum Contents<'a> {
    Solid,
    Mask(&'a [f32]),
}

pub struct Rasterizer {
    width: usize,
    height: usize,
    coverage: Vec<f32>,
    bitmasks_width: usize,
    bitmasks: Vec<u64>,
    min_tile_x: usize,
    min_tile_y: usize,
    max_tile_x: usize,
    max_tile_y: usize,
}

impl Rasterizer {
    pub fn with_size(width: usize, height: usize) -> Rasterizer {
        // round up to next multiple of tile size
        let width_rounded = (width + TILE_SIZE - 1) & !(TILE_SIZE - 1);
        let height_rounded = (height + TILE_SIZE - 1) & !(TILE_SIZE - 1);

        // round up to next multiple of bitmask size
        let bitmasks_width = (width_rounded + (TILE_SIZE * BITMASK_SIZE) - 1)
            >> (TILE_SIZE_BITS + BITMASK_SIZE_BITS);
        let bitmasks_height = height_rounded >> TILE_SIZE_BITS;
        let bitmasks = vec![0; bitmasks_width * bitmasks_height];

        Rasterizer {
            width: width_rounded,
            height: height_rounded,
            coverage: vec![0.0; width_rounded * height_rounded],
            bitmasks_width,
            bitmasks,
            min_tile_x: width_rounded >> TILE_SIZE_BITS,
            min_tile_y: height_rounded >> TILE_SIZE_BITS,
            max_tile_x: 0,
            max_tile_y: 0,
        }
    }

    pub fn add_line(&mut self, p1: Vec2, p2: Vec2) {
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
                    let tile_x = x as usize >> TILE_SIZE_BITS;
                    let tile_y = y as usize >> TILE_SIZE_BITS;
                    let tile_bit = 1 << (BITMASK_SIZE - 1 - (tile_x & (BITMASK_SIZE - 1)));
                    self.bitmasks[tile_y * self.bitmasks_width + (tile_x >> BITMASK_SIZE_BITS)] |=
                        tile_bit;
                    self.coverage[(y as usize * self.width) + x as usize] += area;

                    self.min_tile_x = self.min_tile_x.min(tile_x);
                    self.min_tile_y = self.min_tile_y.min(tile_y);

                    let tile_x = (x + 1) as usize >> TILE_SIZE_BITS;
                    let tile_y = y as usize >> TILE_SIZE_BITS;
                    let tile_bit = 1 << (BITMASK_SIZE - 1 - (tile_x & (BITMASK_SIZE - 1)));
                    self.bitmasks[tile_y * self.bitmasks_width + (tile_x >> BITMASK_SIZE_BITS)] |=
                        tile_bit;
                    self.coverage[(y as usize * self.width) + (x + 1) as usize] += height - area;

                    self.max_tile_x = self.max_tile_x.max(tile_x);
                    self.max_tile_y = self.max_tile_y.max(tile_y);
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
        for tile_y in self.min_tile_y..=self.max_tile_y {
            let mut accum = [0.0; TILE_SIZE];
            let mut coverages = [0.0; TILE_SIZE];

            let mut tile_x = self.min_tile_x;

            let bitmask_start = self.min_tile_x >> BITMASK_SIZE_BITS;
            let bitmask_end = (self.max_tile_x + BITMASK_SIZE - 1) >> BITMASK_SIZE_BITS;
            for bitmask_x in bitmask_start..=bitmask_end {
                let bitmask_tile_x = bitmask_x << BITMASK_SIZE_BITS;
                let mut tile = self.bitmasks[tile_y * self.bitmasks_width + bitmask_x];
                self.bitmasks[tile_y * self.bitmasks_width + bitmask_x] = 0;

                loop {
                    let index = tile.leading_zeros() as usize;
                    let next_tile_x = bitmask_tile_x + index;

                    if next_tile_x > tile_x {
                        let mut solid = true;
                        for coverage in coverages {
                            if coverage < 254.5 / 255.0 {
                                solid = false;
                            }
                        }

                        if solid {
                            sink(Span {
                                tile_x,
                                tile_y,
                                width: next_tile_x - tile_x,
                                contents: Contents::Solid,
                            });
                        } else {
                            let mut empty = true;
                            for coverage in coverages {
                                if coverage >= 0.5 / 255.0 {
                                    empty = false;
                                }
                            }

                            if !empty {
                                let mut mask = [0.0; TILE_SIZE * TILE_SIZE];
                                for y in 0..TILE_SIZE {
                                    for x in 0..TILE_SIZE {
                                        mask[(y << TILE_SIZE_BITS) + x] = coverages[y];
                                    }
                                }

                                sink(Span {
                                    tile_x,
                                    tile_y,
                                    width: next_tile_x - tile_x,
                                    contents: Contents::Mask(&mask),
                                });
                            }
                        }
                    }

                    if index == BITMASK_SIZE {
                        break;
                    }

                    tile_x = next_tile_x;

                    let mut mask = [0.0; TILE_SIZE * TILE_SIZE];

                    let x_offset = tile_x << TILE_SIZE_BITS;
                    let y_offset = tile_y << TILE_SIZE_BITS;
                    for y in 0..TILE_SIZE {
                        for x in 0..TILE_SIZE {
                            let coverage =
                                &mut self.coverage[(y_offset + y) * self.width + x_offset + x];
                            accum[y] += std::mem::replace(coverage, 0.0);
                            coverages[y] = accum[y].abs().min(1.0);
                            mask[(y << TILE_SIZE_BITS) + x] = coverages[y];
                        }
                    }

                    sink(Span { tile_x, tile_y, width: 1, contents: Contents::Mask(&mask) });

                    tile &= !(1 << (BITMASK_SIZE - 1 - index));
                    tile_x += 1;
                }
            }
        }

        self.min_tile_x = self.width >> TILE_SIZE_BITS;
        self.min_tile_y = self.height >> TILE_SIZE_BITS;
        self.max_tile_x = 0;
        self.max_tile_y = 0;
    }
}
