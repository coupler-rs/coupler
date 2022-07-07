use std::cell::RefCell;

use graphics::{Canvas, Color, Font, Path, Vec2};
use portlight::{Application, Rect, Window, WindowHandler, WindowOptions};

struct Handler {
    canvas: RefCell<Canvas>,
    font: Font,
}

impl Handler {
    fn new() -> Handler {
        Handler {
            canvas: RefCell::new(Canvas::with_size(500, 500)),
            font: Font::from_bytes(include_bytes!("res/SourceSansPro-Regular.otf"), 0).unwrap(),
        }
    }
}

impl WindowHandler for Handler {
    fn display(&self, window: &Window) {
        self.canvas.borrow_mut().clear(Color::rgba(0, 0, 0, 255));

        let mut path = Path::new();
        path.move_to(Vec2::new(200.0, 300.0))
            .quadratic_to(Vec2::new(300.0, 200.0), Vec2::new(200.0, 100.0))
            .cubic_to(
                Vec2::new(150.0, 150.0),
                Vec2::new(-100.0, 250.0),
                Vec2::new(200.0, 300.0),
            );

        self.canvas
            .borrow_mut()
            .fill_path(&path, Color::rgba(255, 255, 255, 255));

        let time = std::time::Instant::now();
        self.canvas.borrow_mut().fill_text(
            "the quick brown fox jumps over the lazy dog.",
            &self.font,
            72.0,
            Color::rgba(255, 255, 255, 255),
        );
        dbg!(time.elapsed());

        window.update_contents(
            self.canvas.borrow().data(),
            self.canvas.borrow().width(),
            self.canvas.borrow().height(),
        );
    }

    fn request_close(&self, window: &Window) {
        window.close();
        window.application().stop();
    }
}

fn main() {
    let app = Application::new().unwrap();

    Window::open(
        &app,
        WindowOptions {
            title: "window".to_string(),
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 500.0,
                height: 500.0,
            },
            handler: Box::new(Handler::new()),
            ..WindowOptions::default()
        },
    )
    .unwrap();

    app.start().unwrap();
}
