use window::{Window, WindowOptions};

fn main() {
    Window::open(WindowOptions {
        title: "window".to_string(),
        width: 500.0,
        height: 500.0,
    }).unwrap();
}
