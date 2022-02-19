use crate::{geom::Vec2, Color};

use simd::*;

const CELL_SIZE: usize = 4;
const CELL_SIZE_BITS: usize = 2;

const BITMASK_SIZE: usize = u64::BITS as usize;
const BITMASK_SIZE_BITS: usize = 6;

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
        let mut x = p1.x as isize;
        let mut y = p1.y as isize;

        let x_end = p2.x as isize;
        let y_end = p2.y as isize;

        self.min_x = self.min_x.min(x as usize).min(x_end as usize);
        self.min_y = self.min_y.min(y as usize).min(y_end as usize);
        self.max_x = self.max_x.max(x as usize + 1).max(x_end as usize + 1);
        self.max_y = self.max_y.max(y as usize).max(y_end as usize);

        let x_inc;
        let mut x_offset;
        let x_offset_end;
        let dx;
        let area_offset;
        let area_sign;
        if p2.x > p1.x {
            x_inc = 1;
            x_offset = p1.x - x as f32;
            x_offset_end = p2.x - x_end as f32;
            dx = p2.x - p1.x;
            area_offset = 2.0;
            area_sign = -1.0;
        } else {
            x_inc = -1;
            x_offset = 1.0 - (p1.x - x as f32);
            x_offset_end = 1.0 - (p2.x - x_end as f32);
            dx = p1.x - p2.x;
            area_offset = 0.0;
            area_sign = 1.0;
        }

        let y_inc;
        let mut y_offset;
        let y_offset_end;
        let dy;
        let sign;
        if p2.y > p1.y {
            y_inc = 1;
            y_offset = p1.y - y as f32;
            y_offset_end = p2.y - y_end as f32;
            dy = p2.y - p1.y;
            sign = 1.0;
        } else {
            y_inc = -1;
            y_offset = 1.0 - (p1.y - y as f32);
            y_offset_end = 1.0 - (p2.y - y_end as f32);
            dy = p1.y - p2.y;
            sign = -1.0;
        }

        let dxdy = dx / dy;
        let dydx = dy / dx;

        let mut y_offset_for_prev_x = y_offset - dydx * x_offset;
        let mut x_offset_for_prev_y = x_offset - dxdy * y_offset;

        while x != x_end || y != y_end {
            let col = x;
            let row = y;

            let x1 = x_offset;
            let y1 = y_offset;

            let x2;
            let y2;
            if y != y_end && (x == x_end || x_offset_for_prev_y + dxdy < 1.0) {
                y_offset = 0.0;
                x_offset = x_offset_for_prev_y + dxdy;
                x_offset_for_prev_y = x_offset;
                y_offset_for_prev_x -= 1.0;
                y += y_inc;

                x2 = x_offset;
                y2 = 1.0;
            } else {
                x_offset = 0.0;
                y_offset = y_offset_for_prev_x + dydx;
                x_offset_for_prev_y -= 1.0;
                y_offset_for_prev_x = y_offset;
                x += x_inc;

                x2 = 1.0;
                y2 = y_offset;
            }

            let height = sign * (y2 - y1);
            let area = 0.5 * height * (area_offset + area_sign * (x1 + x2));

            self.add_delta(col, row, height, area);
        }

        let height = sign * (y_offset_end - y_offset);
        let area = 0.5 * height * (area_offset + area_sign * (x_offset + x_offset_end));

        self.add_delta(x, y, height, area);
    }

    #[inline(always)]
    fn add_delta(&mut self, x: isize, y: isize, height: f32, area: f32) {
        if x >= 0 as isize
            && x < self.width as isize - 1
            && y >= 0 as isize
            && y < self.height as isize
        {
            let coverage_index = y as usize * self.width + x as usize;
            self.coverage[coverage_index] += area;
            self.coverage[coverage_index + 1] += height - area;

            let bitmask_row = y as usize * self.bitmasks_width;

            let cell_x = x as usize >> CELL_SIZE_BITS;
            let cell_bit = 1 << (BITMASK_SIZE - 1 - (cell_x & (BITMASK_SIZE - 1)));
            self.bitmasks[bitmask_row + (cell_x >> BITMASK_SIZE_BITS)] |= cell_bit;

            let cell_x = (x + 1) as usize >> CELL_SIZE_BITS;
            let cell_bit = 1 << (BITMASK_SIZE - 1 - (cell_x & (BITMASK_SIZE - 1)));
            self.bitmasks[bitmask_row + (cell_x >> BITMASK_SIZE_BITS)] |= cell_bit;
        }
    }

    pub fn finish(&mut self, color: Color, data: &mut [u32], width: usize) {
        self.min_x = self.min_x.max(0);
        self.min_y = self.min_y.max(0);
        self.max_x = self.max_x.min(self.width - 1);
        self.max_y = self.max_y.min(self.height - 1);

        let a = f32x4::splat(color.a() as f32);
        let a_unit = a * f32x4::splat(1.0 / 255.0);
        let r = a_unit * f32x4::splat(color.r() as f32);
        let g = a_unit * f32x4::splat(color.g() as f32);
        let b = a_unit * f32x4::splat(color.b() as f32);

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
                        if coverage > 254.5 / 255.0 && color.a() == 255 {
                            let start = y * width + x;
                            let end = y * width + next_x;
                            data[start..end].fill(color.into());
                        } else if coverage > 0.5 / 255.0 {
                            let start = y * width + x;
                            let end = y * width + next_x;
                            for pixels_slice in data[start..end].chunks_mut(4) {
                                let mask = f32x4::splat(coverage);
                                let pixels = u32x4::from_slice(pixels_slice);

                                let dst_a = f32x4::from((pixels >> 24) & u32x4::splat(0xFF));
                                let dst_r = f32x4::from((pixels >> 16) & u32x4::splat(0xFF));
                                let dst_g = f32x4::from((pixels >> 8) & u32x4::splat(0xFF));
                                let dst_b = f32x4::from((pixels >> 0) & u32x4::splat(0xFF));

                                let inv_a = f32x4::splat(1.0) - mask * a_unit;
                                let out_a = u32x4::from(mask * a + inv_a * dst_a);
                                let out_r = u32x4::from(mask * r + inv_a * dst_r);
                                let out_g = u32x4::from(mask * g + inv_a * dst_g);
                                let out_b = u32x4::from(mask * b + inv_a * dst_b);

                                let out =
                                    (out_a << 24) | (out_r << 16) | (out_g << 8) | (out_b << 0);
                                out.write_to_slice(pixels_slice);
                            }
                        }
                    }

                    if index == BITMASK_SIZE {
                        break;
                    }

                    x = next_x;

                    let coverage_start = y * self.width + x;
                    let coverage_end = coverage_start + CELL_SIZE;
                    let coverage_slice = &mut self.coverage[coverage_start..coverage_end];

                    let deltas = f32x4::from_slice(coverage_slice);
                    let accums = f32x4::splat(accum) + deltas.scan();
                    accum = accums.get::<3>();
                    let mask = accums.abs().min(f32x4::splat(1.0));
                    coverage = mask.get::<3>();

                    coverage_slice.fill(0.0);

                    let pixels_start = y * width + x;
                    let pixels_end = pixels_start + CELL_SIZE;
                    let pixels_slice = &mut data[pixels_start..pixels_end];
                    let pixels = u32x4::from_slice(pixels_slice);

                    let dst_a = f32x4::from((pixels >> 24) & u32x4::splat(0xFF));
                    let dst_r = f32x4::from((pixels >> 16) & u32x4::splat(0xFF));
                    let dst_g = f32x4::from((pixels >> 8) & u32x4::splat(0xFF));
                    let dst_b = f32x4::from((pixels >> 0) & u32x4::splat(0xFF));

                    let inv_a = f32x4::splat(1.0) - mask * a_unit;
                    let out_a = u32x4::from(mask * a + inv_a * dst_a);
                    let out_r = u32x4::from(mask * r + inv_a * dst_r);
                    let out_g = u32x4::from(mask * g + inv_a * dst_g);
                    let out_b = u32x4::from(mask * b + inv_a * dst_b);

                    let out = (out_a << 24) | (out_r << 16) | (out_g << 8) | (out_b << 0);
                    out.write_to_slice(pixels_slice);

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
