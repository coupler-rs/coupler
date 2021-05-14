use crate::{Rect, WindowOptions};

use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use raw_window_handle::{unix::XlibHandle, HasRawWindowHandle, RawWindowHandle};

#[derive(Debug)]
pub enum ApplicationError {}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ApplicationError {}

#[derive(Clone)]
pub struct Application {
    inner: Rc<ApplicationInner>,
}

struct ApplicationInner {}

impl Application {
    pub fn open() -> Result<Application, ApplicationError> {
        Ok(Application { inner: Rc::new(ApplicationInner {}) })
    }

    pub fn close(&self) -> Result<(), ApplicationError> {
        Ok(())
    }

    pub fn start(&self) -> Result<(), ApplicationError> {
        Ok(())
    }

    pub fn stop(&self) {}
}

#[derive(Debug)]
pub enum WindowError {}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for WindowError {}

#[derive(Clone)]
pub struct Window {
    state: Rc<WindowState>,
}

struct WindowState {
    application: crate::Application,
}

impl Window {
    pub fn open(
        application: &crate::Application,
        options: WindowOptions,
    ) -> Result<crate::Window, WindowError> {
        Ok(crate::Window {
            window: Window { state: Rc::new(WindowState { application: application.clone() }) },
            phantom: PhantomData,
        })
    }

    pub fn request_display(&self) {}

    pub fn request_display_rect(&self, rect: Rect) {}

    pub fn update_contents(&self, framebuffer: &[u32], width: usize, height: usize) {}

    pub fn close(&self) -> Result<(), WindowError> {
        Ok(())
    }

    pub fn application(&self) -> &crate::Application {
        &self.state.application
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        RawWindowHandle::Xlib(XlibHandle { ..XlibHandle::empty() })
    }
}
