use window::{Application, Window, WindowOptions};

fn main() {
    let app = Application::new().unwrap();

    Window::open(WindowOptions {
        title: "window".to_string(),
        width: 500.0,
        height: 500.0,
        application: Some(&app),
    })
    .unwrap();

    app.run();
}
