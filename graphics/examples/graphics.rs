use std::cell::RefCell;

use graphics::{Canvas, Color, Path, Vec2};
use window::{Application, Rect, Window, WindowHandler, WindowOptions};

struct Handler {
    canvas: RefCell<Canvas>,
}

impl Handler {
    fn new() -> Handler {
        Handler { canvas: RefCell::new(Canvas::with_size(500, 500)) }
    }
}

impl WindowHandler for Handler {
    fn display(&self, window: &Window) {
        let mut path = Path::builder();
        path.move_to(Vec2::new(400.0, 300.0))
            .quadratic_to(Vec2::new(500.0, 200.0), Vec2::new(400.0, 100.0))
            .cubic_to(Vec2::new(350.0, 150.0), Vec2::new(100.0, 250.0), Vec2::new(400.0, 300.0));
        let path = path.build();

        self.canvas.borrow_mut().fill(&path, Color::rgba(255, 0, 255, 255));

        window.update_contents(
            self.canvas.borrow().data(),
            self.canvas.borrow().width(),
            self.canvas.borrow().height(),
        );
    }
}

fn main() {
    let app = Application::open().unwrap();

    Window::open(
        &app,
        WindowOptions {
            title: "window".to_string(),
            rect: Rect { x: 0.0, y: 0.0, w: 500.0, h: 500.0 },
            handler: Some(Box::new(Handler::new())),
            ..WindowOptions::default()
        },
    )
    .unwrap();

    app.start().unwrap();
    app.close().unwrap();
}
