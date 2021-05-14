use crate::{Rect, WindowOptions};

use std::cell::Cell;
use std::error::Error;
use std::marker::PhantomData;
use std::rc::Rc;
use std::{fmt, ptr};

use raw_window_handle::{unix::XlibHandle, HasRawWindowHandle, RawWindowHandle};
use xcb_sys as xcb;

#[derive(Debug)]
pub enum ApplicationError {
    ConnectionFailed(i32),
}

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

struct ApplicationInner {
    open: Cell<bool>,
    connection: *mut xcb::xcb_connection_t,
    screen: *mut xcb::xcb_screen_t,
}

impl Application {
    pub fn open() -> Result<Application, ApplicationError> {
        unsafe {
            let mut default_screen_index = 0;
            let connection = xcb::xcb_connect(ptr::null(), &mut default_screen_index);

            let error = xcb::xcb_connection_has_error(connection);
            if error != 0 {
                xcb::xcb_disconnect(connection);
                return Err(ApplicationError::ConnectionFailed(error));
            }

            let setup = xcb::xcb_get_setup(connection);
            let mut roots_iter = xcb::xcb_setup_roots_iterator(setup);
            for _ in 0..default_screen_index {
                xcb::xcb_screen_next(&mut roots_iter);
            }
            let screen = roots_iter.data;

            Ok(Application {
                inner: Rc::new(ApplicationInner { open: Cell::new(true), connection, screen }),
            })
        }
    }

    pub fn close(&self) -> Result<(), ApplicationError> {
        unsafe {
            if self.inner.open.get() {
                self.inner.open.set(false);

                xcb::xcb_disconnect(self.inner.connection);
            }

            Ok(())
        }
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
