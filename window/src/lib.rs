#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

use std::error::Error;
use std::fmt;
use std::marker::PhantomData;

pub struct WindowOptions {
    pub title: String,
    pub width: f32,
    pub height: f32,
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
