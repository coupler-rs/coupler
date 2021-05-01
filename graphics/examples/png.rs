use graphics::{Canvas, Color, Mat2x2, Path, Transform, Vec2};

fn main() {
    let mut canvas = Canvas::with_size(1920, 1080);

    let path = std::env::args().nth(1).expect("provide an svg file");
    let tree = usvg::Tree::from_file(path, &usvg::Options::default()).unwrap();

    fn render(node: &usvg::Node, canvas: &mut Canvas) {
        use usvg::NodeExt;
        match *node.borrow() {
            usvg::NodeKind::Path(ref p) => {
                let t = node.transform();
                let transform =
                    Transform::new(Mat2x2::new(t.a, t.c, t.b, t.d), Vec2::new(t.e, t.f)).then(Transform::scale(2.0));

                let mut path = Path::builder();
                for segment in p.data.0.iter() {
                    match *segment {
                        usvg::PathSegment::MoveTo { x, y } => {
                            path.move_to(Vec2::new(500.0,0.0) + transform.apply(1.0 * Vec2::new(x, y)));
                        }
                        usvg::PathSegment::LineTo { x, y } => {
                            path.line_to(Vec2::new(500.0,0.0) + transform.apply(1.0 * Vec2::new(x, y)));
                        }
                        usvg::PathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                            path.cubic_to(Vec2::new(500.0,0.0) + transform.apply(1.0 * Vec2::new(x1, y1)), Vec2::new(500.0,0.0) + transform.apply(1.0 * Vec2::new(x2, y2)), Vec2::new(500.0,0.0) + transform.apply(1.0 * Vec2::new(x, y)));
                        }
                        usvg::PathSegment::ClosePath => {
                            path.close();
                        }
                    }
                }
                let path = path.build();

                if let Some(ref fill) = p.fill {
                    if let usvg::Paint::Color(color) = fill.paint {
                        let color = Color::rgba(color.red, color.green, color.blue, fill.opacity.to_u8());
                        canvas.fill(&path, color);
                    }
                }

                if let Some(ref stroke) = p.stroke {
                    if let usvg::Paint::Color(color) = stroke.paint {
                        let color = Color::rgba(color.red, color.green, color.blue, stroke.opacity.to_u8());
                        canvas.fill(&path.stroke(stroke.width.value() * 2.0), color);
                    }
                }
            }
            _ => {}
        }

        for child in node.children() {
            render(&child, canvas);
        }
    }

    let time = std::time::Instant::now();
    render(&tree.root(), &mut canvas);
    dbg!(time.elapsed());

    use png::HasParameters;
    use std::fs::File;
    use std::io::BufWriter;

    let path = std::path::Path::new(r"out.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, 1920, 1080);
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    let mut data = vec![0; 4 * 1920 * 1080];
    let mut i = 0;
    for pixel in canvas.data() {
        data[4 * i] = ((pixel >> 16) & 0xFF) as u8;
        data[4 * i + 1] = ((pixel >> 8) & 0xFF) as u8;
        data[4 * i + 2] = ((pixel >> 0) & 0xFF) as u8;
        data[4 * i + 3] = ((pixel >> 24) & 0xFF) as u8;
        i += 1;
    }

    writer.write_image_data(&data[..]).unwrap();
}
