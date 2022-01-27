use crate::color::Color;
use crate::geom::Vec2;
use crate::path::{Path, Verb};
use crate::raster::{Contents, Rasterizer};
use crate::text::Font;

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

        let flattened = path.flatten();

        let mut first = Vec2::new(0.0, 0.0);
        let mut last = Vec2::new(0.0, 0.0);

        let mut points = flattened.points.iter();
        for verb in flattened.verbs {
            match verb {
                Verb::Move => {
                    let point = *points.next().unwrap();
                    first = point;
                    last = point;
                }
                Verb::Line => {
                    let point = *points.next().unwrap();
                    self.rasterizer.add_line(last, point);
                    last = point;
                }
                Verb::Close => {
                    self.rasterizer.add_line(last, first);
                    last = first;
                }
                _ => {
                    unreachable!();
                }
            }
        }

        let width = self.width;
        let data = &mut self.data;
        self.rasterizer.finish(|span| match span.contents {
            Contents::Solid => {
                data[span.y * width + span.x..span.y * width + span.x + span.width].fill(color.into());
            }
            Contents::Constant(coverage) => {
                let coverage = (coverage * 255.0) as u32;
                for pixel in data[span.y * width + span.x..span.y * width + span.x + span.width].iter_mut() {
                    let mut r = (coverage * color.r() as u32 + 127) / 255;
                    let mut g = (coverage * color.g() as u32 + 127) / 255;
                    let mut b = (coverage * color.b() as u32 + 127) / 255;
                    let mut a = (coverage * color.a() as u32 + 127) / 255;

                    let inv_a = 255 - a;

                    a += (inv_a * ((*pixel >> 24) & 0xFF) + 127) / 255;
                    r += (inv_a * ((*pixel >> 16) & 0xFF) + 127) / 255;
                    g += (inv_a * ((*pixel >> 8) & 0xFF) + 127) / 255;
                    b += (inv_a * ((*pixel >> 0) & 0xFF) + 127) / 255;

                    *pixel = (a << 24) | (r << 16) | (g << 8) | (b << 0);
                }
            }
            Contents::Mask(mask) => {
                for (pixel, coverage) in data[span.y * width + span.x..span.y * width + span.x + span.width].iter_mut().zip(mask.iter()) {
                    let coverage = (*coverage * 255.0) as u32;

                    let mut r = (coverage * color.r() as u32 + 127) / 255;
                    let mut g = (coverage * color.g() as u32 + 127) / 255;
                    let mut b = (coverage * color.b() as u32 + 127) / 255;
                    let mut a = (coverage * color.a() as u32 + 127) / 255;

                    let inv_a = 255 - a;

                    a += (inv_a * ((*pixel >> 24) & 0xFF) + 127) / 255;
                    r += (inv_a * ((*pixel >> 16) & 0xFF) + 127) / 255;
                    g += (inv_a * ((*pixel >> 8) & 0xFF) + 127) / 255;
                    b += (inv_a * ((*pixel >> 0) & 0xFF) + 127) / 255;

                    *pixel = (a << 24) | (r << 16) | (g << 8) | (b << 0);
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
