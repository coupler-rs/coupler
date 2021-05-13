use crate::{Rect, WindowOptions};

use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use cocoa::{appkit, base, foundation};
use objc::{class, msg_send, sel, sel_impl};
use objc::{declare, runtime};
use raw_window_handle::{macos::MacOSHandle, HasRawWindowHandle, RawWindowHandle};

#[derive(Debug)]
pub enum ApplicationError {
    ViewClassRegistration,
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
    class: *mut runtime::Class,
}

impl Application {
    pub fn open() -> Result<Application, ApplicationError> {
        unsafe {
            let class_name = format!("window-{}", uuid::Uuid::new_v4().to_simple());

            let mut class_decl =
                if let Some(class_decl) = declare::ClassDecl::new(&class_name, class!(NSView)) {
                    class_decl
                } else {
                    return Err(ApplicationError::ViewClassRegistration);
                };

            let class = class_decl.register();

            Ok(Application {
                inner: Rc::new(ApplicationInner {
                    open: Cell::new(true),
                    class: class as *const runtime::Class as *mut runtime::Class,
                }),
            })
        }
    }

    pub fn close(&self) -> Result<(), ApplicationError> {
        unsafe {
            if self.inner.open.get() {
                self.inner.open.set(false);

                runtime::objc_disposeClassPair(self.inner.class);
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
        RawWindowHandle::MacOS(MacOSHandle { ..MacOSHandle::empty() })
    }
}
