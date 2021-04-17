use graphics::{Canvas, Path, Vec2};

fn main() {
    let mut path = Path::builder();
    path.move_to(Vec2::new(400.0, 300.0))
        .quadratic_to(Vec2::new(500.0, 200.0), Vec2::new(400.0, 100.0))
        .cubic_to(Vec2::new(350.0, 150.0), Vec2::new(100.0, 250.0), Vec2::new(400.0, 300.0));
    let path = path.build();

    let mut canvas = Canvas::with_size(500, 500);

    let time = std::time::Instant::now();
    for _ in 0..1000 {
        canvas.fill(&path);
    }
    dbg!(time.elapsed().div_f64(1000.0));

    use std::fs::File;
    use std::io::BufWriter;
    use png::HasParameters;

    let path = std::path::Path::new(r"out.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, 500, 500);
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    let data = canvas.data();
    let data_u8: &[u8] = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const _, data.len() * 4) };
    writer.write_image_data(data_u8).unwrap();
}