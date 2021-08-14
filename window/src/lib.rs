#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as platform;

#[cfg(target_os = "linux")]
mod x11;
#[cfg(target_os = "linux")]
use x11 as platform;

use std::error::Error;
use std::fmt;
use std::marker::PhantomData;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

#[derive(Debug)]
pub struct ApplicationError(platform::ApplicationError);

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for ApplicationError {}

#[derive(Clone)]
pub struct Application {
    application: platform::Application,
    // ensure that Application is !Send on all platforms
    phantom: PhantomData<*mut ()>,
}

impl Application {
    pub fn new() -> Result<Application, ApplicationError> {
        match platform::Application::new() {
            Ok(application) => Ok(Application { application, phantom: PhantomData }),
            Err(error) => Err(ApplicationError(error)),
        }
    }

    pub fn start(&self) -> Result<(), ApplicationError> {
        match self.application.start() {
            Ok(()) => Ok(()),
            Err(error) => Err(ApplicationError(error)),
        }
    }

    pub fn stop(&self) {
        self.application.stop();
    }

    pub fn poll(&self) {
        self.application.poll();
    }

    pub fn file_descriptor(&self) -> Option<std::os::raw::c_int> {
        self.application.file_descriptor()
    }
}

pub enum Parent<'p> {
    None,
    Parent(&'p dyn HasRawWindowHandle),
    Detached,
}

#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Copy, Clone, Debug)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Cursor {
    Arrow,
    Crosshair,
    Hand,
    IBeam,
    No,
    SizeNs,
    SizeWe,
    SizeNesw,
    SizeNwse,
    Wait,
    None,
}

#[allow(unused_variables)]
pub trait WindowHandler {
    fn create(&self, window: &Window) {}
    fn frame(&self, window: &Window) {}
    fn display(&self, window: &Window) {}
    fn mouse_move(&self, window: &Window, position: Point) {}
    fn mouse_down(&self, window: &Window, button: MouseButton) -> bool {
        false
    }
    fn mouse_up(&self, window: &Window, button: MouseButton) -> bool {
        false
    }
    fn scroll(&self, window: &Window, dx: f64, dy: f64) -> bool {
        false
    }
    fn request_close(&self, window: &Window) {}
    fn destroy(&self, window: &Window) {}
}

struct DefaultWindowHandler;

impl WindowHandler for DefaultWindowHandler {}

pub struct WindowOptions<'p> {
    pub title: String,
    pub rect: Rect,
    pub parent: Parent<'p>,
    pub handler: Box<dyn WindowHandler>,
}

impl<'p> Default for WindowOptions<'p> {
    fn default() -> Self {
        WindowOptions {
            title: "".to_string(),
            rect: Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
            parent: Parent::None,
            handler: Box::new(DefaultWindowHandler),
        }
    }
}

#[derive(Debug)]
pub struct WindowError(platform::WindowError);

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for WindowError {}

#[derive(Clone)]
pub struct Window {
    window: platform::Window,
    // ensure that Window is !Send on all platforms
    phantom: PhantomData<*mut ()>,
}

impl Window {
    pub fn open(application: &Application, options: WindowOptions) -> Result<Window, WindowError> {
        match platform::Window::open(&application, options) {
            Ok(window) => Ok(window),
            Err(error) => Err(WindowError(error)),
        }
    }

    pub fn request_display(&self) {
        self.window.request_display();
    }

    pub fn request_display_rect(&self, rect: Rect) {
        self.window.request_display_rect(rect);
    }

    pub fn update_contents(&self, framebuffer: &[u32], width: usize, height: usize) {
        self.window.update_contents(framebuffer, width, height);
    }

    pub fn set_cursor(&self, cursor: Cursor) {
        self.window.set_cursor(cursor);
    }

    pub fn set_mouse_position(&self, position: Point) {
        self.window.set_mouse_position(position);
    }

    pub fn close(&self) -> Result<(), WindowError> {
        match self.window.close() {
            Ok(()) => Ok(()),
            Err(error) => Err(WindowError(error)),
        }
    }

    pub fn application(&self) -> &Application {
        self.window.application()
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }
}
