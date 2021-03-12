#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

use std::error::Error;
use std::fmt;
use std::marker::PhantomData;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

#[derive(Debug)]
pub struct ApplicationError(platform::ApplicationError);

#[derive(Clone)]
pub struct Application {
    application: platform::Application,
    // ensure that Application is !Send on all platforms
    phantom: PhantomData<*mut ()>,
}

impl Application {
    pub fn new() -> Result<Application, ApplicationError> {
        match platform::Application::new() {
            Ok(application) => Ok(Application {
                application,
                phantom: PhantomData,
            }),
            Err(error) => Err(ApplicationError(error)),
        }
    }

    pub fn run(&self) {
        self.application.run();
    }
}

pub enum Parent<'p> {
    None,
    Parent(&'p dyn HasRawWindowHandle),
    Detached,
}

pub struct WindowOptions<'a, 'p> {
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub application: Option<&'a Application>,
    pub parent: Parent<'p>,
}

impl<'a, 'p> Default for WindowOptions<'a, 'p> {
    fn default() -> Self {
        WindowOptions {
            title: "".to_string(),
            width: 0.0,
            height: 0.0,
            application: None,
            parent: Parent::None,
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
    pub fn open(options: WindowOptions) -> Result<Window, WindowError> {
        match platform::Window::open(options) {
            Ok(window) => Ok(Window {
                window,
                phantom: PhantomData,
            }),
            Err(error) => Err(WindowError(error)),
        }
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }
}
