use window::{Application, Window, WindowHandler, WindowOptions};

struct Handler;

impl WindowHandler for Handler {
    fn open(&mut self, _window: &Window) {
        println!("open");
    }

    fn close(&mut self, window: &Window) {
        println!("close");
        window.application().stop();
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        println!("drop");
    }
}

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

    app.start().unwrap();
    app.close().unwrap();
}
