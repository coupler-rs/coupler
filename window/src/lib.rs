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
    pub fn open() -> Result<Application, ApplicationError> {
        match platform::Application::open() {
            Ok(application) => Ok(Application {
                application,
                phantom: PhantomData,
            }),
            Err(error) => Err(ApplicationError(error)),
        }
    }

    pub fn close(&self) -> Result<(), ApplicationError> {
        match self.application.close() {
            Ok(()) => Ok(()),
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

pub trait WindowHandler {}

struct DefaultWindowHandler;

impl WindowHandler for DefaultWindowHandler {}

pub struct WindowOptions<'p> {
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub parent: Parent<'p>,
    pub handler: Option<Box<dyn WindowHandler>>,
}

impl<'p> Default for WindowOptions<'p> {
    fn default() -> Self {
        WindowOptions {
            title: "".to_string(),
            width: 0.0,
            height: 0.0,
            parent: Parent::None,
            handler: None,
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
        match platform::Window::open(&application.application, options) {
            Ok(window) => Ok(Window {
                window,
                phantom: PhantomData,
            }),
            Err(error) => Err(WindowError(error)),
        }
    }

    pub fn close(&self) -> Result<(), WindowError> {
        match self.window.close() {
            Ok(()) => Ok(()),
            Err(error) => Err(WindowError(error)),
        }
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }
}
