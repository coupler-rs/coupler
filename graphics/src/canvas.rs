use crate::color::Color;
use crate::geom::Vec2;
use crate::path::{Command, Path};
use crate::raster::Rasterizer;
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
        if last != first {
            self.rasterizer.add_line(last, first);
        }

        self.rasterizer.finish(color, &mut self.data, self.width);
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
