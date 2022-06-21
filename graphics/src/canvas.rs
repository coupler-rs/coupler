use crate::color::Color;
use crate::geom::Vec2;
use crate::path::{Command, Path};
use crate::raster::Rasterizer;
use crate::text::Font;

const BAND_HEIGHT: usize = 64;
const BAND_HEIGHT_BITS: usize = 6;

pub struct Canvas {
    width: usize,
    height: usize,
    data: Vec<u32>,
    rasterizer: Rasterizer,
    bands: Vec<Vec<Segment>>,
}

struct Segment {
    p1: Vec2,
    p2: Vec2,
}

impl Canvas {
    pub fn with_size(width: usize, height: usize) -> Canvas {
        let band_count = (height + BAND_HEIGHT - 1) >> BAND_HEIGHT_BITS;
        let mut bands = Vec::new();
        for _ in 0..band_count {
            bands.push(Vec::new());
        }

        Canvas {
            width,
            height,
            data: vec![0xFF000000; width * height],
            rasterizer: Rasterizer::with_size(width, BAND_HEIGHT),
            bands,
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

    fn add_line(&mut self, p1: Vec2, p2: Vec2) {
        if (p1.x > self.width as f32 && p2.x > self.width as f32)
            || (p1.y > self.height as f32 && p2.y > self.height as f32)
            || (p1.y < 0.0 && p2.y < 0.0)
        {
            return;
        }

        let band_y1 = p1.y as usize >> BAND_HEIGHT_BITS;
        let band_y2 = p2.y as usize >> BAND_HEIGHT_BITS;

        let band_y_start = band_y1.min(band_y2).max(0);
        let band_y_end = (band_y1.max(band_y2) + 1).min(self.bands.len());
        for band_y in band_y_start..band_y_end {
            self.bands[band_y].push(Segment { p1, p2 });
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
                self.add_line(last, point);
                last = point;
            }
            Command::Close => {
                self.add_line(last, first);
                last = first;
            }
            _ => {
                unreachable!();
            }
        });
        if last != first {
            self.add_line(last, first);
        }

        for (index, band) in self.bands.iter().enumerate() {
            if band.is_empty() {
                continue;
            }

            let offset = Vec2::new(0.0, (index << BAND_HEIGHT_BITS) as f32);
            for segment in band.iter() {
                self.rasterizer
                    .add_line(segment.p1 - offset, segment.p2 - offset);
            }

            let data_start = (index << BAND_HEIGHT_BITS) * self.width;
            let data_end = data_start + BAND_HEIGHT * self.width;
            self.rasterizer
                .finish(color, &mut self.data[data_start..data_end], self.width);
        }

        for band in self.bands.iter_mut() {
            band.clear();
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
