use window::{Application, Window, WindowHandler, WindowOptions};

struct Handler;

impl WindowHandler for Handler {}

fn main() {
    let app = Application::new().unwrap();

    Window::open(WindowOptions {
        title: "window".to_string(),
        width: 500.0,
        height: 500.0,
        application: Some(&app),
        handler: Some(Box::new(Handler)),
        ..WindowOptions::default()
    })
    .unwrap();

    app.run();
}
