use crate::geom::Vec2;

const CELL_SIZE: usize = 4;
const CELL_SIZE_BITS: usize = 2;

const BITMASK_SIZE: usize = u64::BITS as usize;
const BITMASK_SIZE_BITS: usize = 6;

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
    Mask(&'a [f32]),
}

pub struct Rasterizer {
    width: usize,
    height: usize,
    coverage: Vec<f32>,
    bitmasks_width: usize,
    bitmasks: Vec<u64>,
    min_x: usize,
    min_y: usize,
    max_x: usize,
    max_y: usize,
}

impl Rasterizer {
    pub fn with_size(width: usize, height: usize) -> Rasterizer {
        // round width up to next multiple of tile size
        let width_rounded = (width + CELL_SIZE - 1) & !(CELL_SIZE - 1);

        // round up to next multiple of bitmask size
        let bitmasks_width = (width_rounded + (CELL_SIZE * BITMASK_SIZE) - 1)
            >> (CELL_SIZE_BITS + BITMASK_SIZE_BITS);
        let bitmasks_height = height;
        let bitmasks = vec![0; bitmasks_width * bitmasks_height];

        Rasterizer {
            width: width_rounded,
            height,
            coverage: vec![0.0; width_rounded * height],
            bitmasks_width,
            bitmasks,
            min_x: width_rounded >> CELL_SIZE_BITS,
            min_y: height,
            max_x: 0,
            max_y: 0,
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
                    && x < self.width as isize - 1
                    && y >= 0 as isize
                    && y < self.height as isize
                {
                    let cell_x = x as usize >> CELL_SIZE_BITS;
                    let cell_bit = 1 << (BITMASK_SIZE - 1 - (cell_x & (BITMASK_SIZE - 1)));
                    self.bitmasks
                        [y as usize * self.bitmasks_width + (cell_x >> BITMASK_SIZE_BITS)] |=
                        cell_bit;
                    self.coverage[(y as usize * self.width) + x as usize] += area;

                    self.min_x = self.min_x.min(x as usize);
                    self.min_y = self.min_y.min(y as usize);

                    let cell_x = (x + 1) as usize >> CELL_SIZE_BITS;
                    let y = y;
                    let cell_bit = 1 << (BITMASK_SIZE - 1 - (cell_x & (BITMASK_SIZE - 1)));
                    self.bitmasks
                        [y as usize * self.bitmasks_width + (cell_x >> BITMASK_SIZE_BITS)] |=
                        cell_bit;
                    self.coverage[(y as usize * self.width) + (x + 1) as usize] += height - area;

                    self.max_x = self.max_x.max((x + 1) as usize);
                    self.max_y = self.max_y.max(y as usize);
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
        self.min_x = self.min_x.max(0);
        self.min_y = self.min_y.max(0);
        self.max_x = self.max_x.min(self.width - 1);
        self.max_y = self.max_y.min(self.height - 1);

        for y in self.min_y..=self.max_y {
            let mut accum = 0.0;
            let mut coverage = 0.0;

            let mut x = self.min_x;

            let bitmask_start = self.min_x >> (BITMASK_SIZE_BITS + CELL_SIZE_BITS);
            let bitmask_end = self.max_x >> (BITMASK_SIZE_BITS + CELL_SIZE_BITS);
            for bitmask_x in bitmask_start..=bitmask_end {
                let bitmask_cell_x = bitmask_x << BITMASK_SIZE_BITS;
                let mut tile = self.bitmasks[y * self.bitmasks_width + bitmask_x];
                self.bitmasks[y * self.bitmasks_width + bitmask_x] = 0;

                while x <= self.max_x {
                    let index = tile.leading_zeros() as usize;
                    let next_x = ((bitmask_cell_x + index) << CELL_SIZE_BITS)
                        .min((self.max_x >> CELL_SIZE_BITS) << CELL_SIZE_BITS);

                    if next_x > x {
                        if coverage >= 254.5 / 255.0 {
                            sink(Span { x, y, width: next_x - x, contents: Contents::Solid });
                        } else {
                            if coverage > 0.5 / 255.0 {
                                let mask = [coverage; CELL_SIZE];
                                sink(Span {
                                    x,
                                    y,
                                    width: next_x - x,
                                    contents: Contents::Mask(&mask),
                                });
                            }
                        }
                    }

                    if index == BITMASK_SIZE {
                        break;
                    }

                    x = next_x;

                    let mask_start = y * self.width + x;
                    let mask_end = mask_start + CELL_SIZE;
                    let mask = &mut self.coverage[mask_start..mask_end];
                    for delta in mask.iter_mut() {
                        accum += *delta;
                        coverage = accum.abs().min(1.0);
                        *delta = coverage;
                    }

                    sink(Span { x, y, width: CELL_SIZE, contents: Contents::Mask(mask) });

                    mask.fill(0.0);

                    tile &= !(1 << (BITMASK_SIZE - 1 - index));
                    x += CELL_SIZE;
                }
            }
        }

        self.min_x = self.width >> CELL_SIZE_BITS;
        self.min_y = self.height;
        self.max_x = 0;
        self.max_y = 0;
    }
}
