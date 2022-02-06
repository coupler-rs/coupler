use crate::color::Color;
use crate::geom::Vec2;
use crate::path::{Command, Path};
use crate::raster::{Contents, Rasterizer};
use crate::text::Font;

use simd::*;

pub struct Canvas {
    width: usize,
    height: usize,
    data: Vec<u32>,
    rasterizer: Rasterizer,
}

impl Canvas {
    pub fn with_size(width: usize, height: usize) -> Canvas {
        Canvas {
            width,
            height,
            data: vec![0xFF000000; width * height + 1],
            rasterizer: Rasterizer::with_size(width, height),
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
            *pixel = color.into();
        }
    }

    pub fn fill_path(&mut self, path: &Path, color: Color) {
        if path.points.is_empty() {
            return;
        }

        let mut first = Vec2::new(0.0, 0.0);
        let mut last = Vec2::new(0.0, 0.0);

        path.flatten(|command| match command {
            Command::Move(point) => {
                first = point;
                last = point;
            }
            Command::Line(point) => {
                self.rasterizer.add_line(last, point);
                last = point;
            }
            Command::Close => {
                self.rasterizer.add_line(last, first);
                last = first;
            }
            _ => {
                unreachable!();
            }
        });

        let r = f32x4::splat(color.r() as f32);
        let g = f32x4::splat(color.g() as f32);
        let b = f32x4::splat(color.b() as f32);
        let a = f32x4::splat(color.a() as f32);

        let a_unit = a * f32x4::splat(1.0 / 255.0);

        let width = self.width;
        let data = &mut self.data;
        self.rasterizer.finish(|span| match span.contents {
            Contents::Solid => {
                let start = span.y * width + span.x;
                let end = start + span.width;
                data[start..end].fill(color.into());
            }
            Contents::Mask(mask) => {
                let start = span.y * width + span.x;
                let end = start + span.width;
                for (pixels, coverage) in data[start..end].chunks_mut(4).zip(mask.chunks(4)) {
                    let pxs = u32x4::from_slice(pixels);
                    let cvg = f32x4::from_slice(coverage);

                    let src_a = cvg * a;
                    let src_r = cvg * r;
                    let src_g = cvg * g;
                    let src_b = cvg * b;

                    let dst_a = f32x4::from((pxs >> 24) & u32x4::splat(0xFF));
                    let dst_r = f32x4::from((pxs >> 16) & u32x4::splat(0xFF));
                    let dst_g = f32x4::from((pxs >> 8) & u32x4::splat(0xFF));
                    let dst_b = f32x4::from((pxs >> 0) & u32x4::splat(0xFF));

                    let inv_a = f32x4::splat(1.0) - cvg * a_unit;
                    let out_a = u32x4::from(src_a + inv_a * dst_a);
                    let out_r = u32x4::from(src_r + inv_a * dst_r);
                    let out_g = u32x4::from(src_g + inv_a * dst_g);
                    let out_b = u32x4::from(src_b + inv_a * dst_b);

                    let out = (out_a << 24) | (out_r << 16) | (out_g << 8) | (out_b << 0);
                    out.write_to_slice(pixels);
                }
            }
        });
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
