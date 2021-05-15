use crate::{Rect, WindowOptions};

use std::cell::Cell;
use std::error::Error;
use std::marker::PhantomData;
use std::rc::Rc;
use std::{ffi, fmt, os, ptr};

use raw_window_handle::{unix::XlibHandle, HasRawWindowHandle, RawWindowHandle};
use xcb_sys as xcb;

unsafe fn intern_atom(
    connection: *mut xcb::xcb_connection_t,
    name: &[u8],
) -> xcb::xcb_intern_atom_cookie_t {
    xcb::xcb_intern_atom(connection, 1, name.len() as u16, name.as_ptr() as *const os::raw::c_char)
}

unsafe fn intern_atom_reply(
    connection: *mut xcb::xcb_connection_t,
    cookie: xcb::xcb_intern_atom_cookie_t,
) -> xcb::xcb_atom_t {
    let reply = xcb::xcb_intern_atom_reply(connection, cookie, ptr::null_mut());
    if reply.is_null() {
        return xcb::XCB_NONE;
    }
    let atom = (*reply).atom;
    libc::free(reply as *mut ffi::c_void);
    atom
}

#[derive(Debug)]
pub enum ApplicationError {
    ConnectionFailed(i32),
    GetEvent(i32),
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
    running: Cell<usize>,
    connection: *mut xcb::xcb_connection_t,
    screen: *mut xcb::xcb_screen_t,
    wm_protocols: xcb::xcb_atom_t,
    wm_delete_window: xcb::xcb_atom_t,
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

            let wm_protocols_cookie = intern_atom(connection, b"WM_PROTOCOLS");
            let wm_delete_window_cookie = intern_atom(connection, b"WM_DELETE_WINDOW");

            let wm_protocols = intern_atom_reply(connection, wm_protocols_cookie);
            let wm_delete_window = intern_atom_reply(connection, wm_delete_window_cookie);
            Ok(Application {
                inner: Rc::new(ApplicationInner {
                    open: Cell::new(true),
                    running: Cell::new(0),
                    connection,
                    screen,
                    wm_protocols,
                    wm_delete_window,
                }),
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
        unsafe {
            if self.inner.open.get() {
                let depth = self.inner.running.get();
                self.inner.running.set(depth + 1);

                while self.inner.open.get() && self.inner.running.get() > depth {
                    let event = xcb::xcb_wait_for_event(self.inner.connection);

                    if event.is_null() {
                        let error = xcb::xcb_connection_has_error(self.inner.connection);
                        return Err(ApplicationError::GetEvent(error));
                    }

                    match ((*event).response_type & !0x80) as u32 {
                        xcb::XCB_CLIENT_MESSAGE => {
                            let event = event as *mut xcb_sys::xcb_client_message_event_t;
                            if (*event).data.data32[0] == self.inner.wm_delete_window {
                                let cookie = xcb::xcb_destroy_window(self.inner.connection, (*event).window);
                                xcb::xcb_request_check(self.inner.connection, cookie);
                            }
                        }
                        _ => {}
                    }

                    libc::free(event as *mut ffi::c_void);
                }
            }

            Ok(())
        }
    }

    pub fn stop(&self) {}
}

#[derive(Debug)]
pub enum WindowError {
    ApplicationClosed,
    WindowCreation(u8),
    MapWindow(u8),
}

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
        unsafe {
            if !application.application.inner.open.get() {
                return Err(WindowError::ApplicationClosed);
            }

            let window = xcb::xcb_generate_id(application.application.inner.connection);
            let cookie = xcb::xcb_create_window_checked(
                application.application.inner.connection,
                xcb::XCB_COPY_FROM_PARENT as u8,
                window,
                (*application.application.inner.screen).root,
                options.rect.x as i16,
                options.rect.y as i16,
                options.rect.w as u16,
                options.rect.h as u16,
                0,
                xcb::XCB_WINDOW_CLASS_INPUT_OUTPUT as u16,
                (*application.application.inner.screen).root_visual,
                0,
                ptr::null(),
                // value_mask,
            );

            let error = xcb::xcb_request_check(application.application.inner.connection, cookie);
            if !error.is_null() {
                let error_code = (*error).error_code;
                libc::free(error as *mut ffi::c_void);
                return Err(WindowError::WindowCreation(error_code));
            }

            let cookie =
                xcb::xcb_map_window_checked(application.application.inner.connection, window);

            let error = xcb::xcb_request_check(application.application.inner.connection, cookie);
            if !error.is_null() {
                let error_code = (*error).error_code;
                libc::free(error as *mut ffi::c_void);
                return Err(WindowError::MapWindow(error_code));
            }

            let atoms = &[application.application.inner.wm_delete_window];
            xcb::xcb_icccm_set_wm_protocols(
                application.application.inner.connection,
                window,
                application.application.inner.wm_protocols,
                atoms.len() as u32,
                atoms.as_ptr() as *mut xcb::xcb_atom_t,
            );

            xcb::xcb_flush(application.application.inner.connection);

            Ok(crate::Window {
                window: Window { state: Rc::new(WindowState { application: application.clone() }) },
                phantom: PhantomData,
            })
        }
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
