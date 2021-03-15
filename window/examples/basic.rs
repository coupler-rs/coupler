use window::{Application, Window, WindowHandler, WindowOptions};

struct Handler;

impl WindowHandler for Handler {}

fn main() {
    let app = Application::open().unwrap();

    Window::open(
        &app,
        WindowOptions {
            title: "window".to_string(),
            width: 500.0,
            height: 500.0,
            handler: Some(Box::new(Handler)),
            ..WindowOptions::default()
        },
    )
    .unwrap();

    app.run().unwrap();
}
