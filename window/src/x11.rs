use crate::{Parent, Rect, WindowHandler, WindowOptions};

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::error::Error;
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::{ffi, fmt, mem, os, ptr, slice};

use raw_window_handle::{unix::XcbHandle, HasRawWindowHandle, RawWindowHandle};
use xcb_sys as xcb;

const FRAME_INTERVAL: Duration = Duration::from_millis(16);

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
    Close(Vec<WindowError>),
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
    shm: bool,
    next_frame: Cell<Instant>,
    windows: RefCell<HashMap<xcb::xcb_window_t, crate::Window>>,
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

            let shm_cookie = xcb::xcb_shm_query_version(connection);
            let shm_version =
                xcb::xcb_shm_query_version_reply(connection, shm_cookie, ptr::null_mut());
            let shm = !shm_version.is_null();
            if shm {
                libc::free(shm_version as *mut ffi::c_void);
            }

            Ok(Application {
                inner: Rc::new(ApplicationInner {
                    open: Cell::new(true),
                    running: Cell::new(0),
                    connection,
                    screen,
                    wm_protocols,
                    wm_delete_window,
                    shm,
                    next_frame: Cell::new(Instant::now() + FRAME_INTERVAL),
                    windows: RefCell::new(HashMap::new()),
                }),
            })
        }
    }

    pub fn close(&self) -> Result<(), ApplicationError> {
        unsafe {
            if self.inner.open.get() {
                self.inner.open.set(false);

                let mut window_errors = Vec::new();
                for (_, window) in self.inner.windows.take() {
                    if let Err(error) = window.window.close() {
                        window_errors.push(error);
                    }
                }

                xcb::xcb_disconnect(self.inner.connection);

                if !window_errors.is_empty() {
                    return Err(ApplicationError::Close(window_errors));
                }
            }

            Ok(())
        }
    }

    pub fn start(&self) -> Result<(), ApplicationError> {
        unsafe {
            if self.inner.open.get() {
                let depth = self.inner.running.get();
                self.inner.running.set(depth + 1);

                let fd = xcb::xcb_get_file_descriptor(self.inner.connection);

                while self.inner.open.get() && self.inner.running.get() > depth {
                    self.frame();

                    while self.inner.open.get() && self.inner.running.get() > depth {
                        let event = xcb::xcb_poll_for_event(self.inner.connection);
                        if event.is_null() {
                            break;
                        }
                        self.handle_event(event);
                    }

                    let to_next_frame =
                        self.inner.next_frame.get().saturating_duration_since(Instant::now());
                    if !to_next_frame.is_zero() {
                        let mut fds = [libc::pollfd { fd, events: libc::POLLIN, revents: 0 }];
                        libc::poll(
                            fds.as_mut_ptr(),
                            fds.len() as u64,
                            to_next_frame.as_millis() as i32,
                        );
                    }
                }
            }

            Ok(())
        }
    }

    pub fn stop(&self) {
        self.inner.running.set(self.inner.running.get().saturating_sub(1));
    }

    pub fn poll(&self) {
        unsafe {
            while self.inner.open.get() {
                self.frame();

                let event = xcb::xcb_poll_for_event(self.inner.connection);
                if event.is_null() {
                    break;
                }
                self.handle_event(event);
            }
        }
    }

    fn frame(&self) {
        let time = Instant::now();
        let mut next_frame = self.inner.next_frame.get();

        if time >= next_frame {
            let windows: Vec<crate::Window> =
                self.inner.windows.borrow().values().cloned().collect();
            for window in windows {
                window.window.state.handler.frame(&window);
            }

            while next_frame < time {
                next_frame += FRAME_INTERVAL;
            }
            self.inner.next_frame.set(next_frame);
        }
    }

    unsafe fn handle_event(&self, event: *mut xcb::xcb_generic_event_t) {
        match ((*event).response_type & !0x80) as u32 {
            xcb::XCB_EXPOSE => {
                let event = &*(event as *mut xcb_sys::xcb_expose_event_t);
                if let Some(window) = self.inner.windows.borrow().get(&event.window) {
                    window.window.state.expose_rects.borrow_mut().push(xcb::xcb_rectangle_t {
                        x: event.x as i16,
                        y: event.y as i16,
                        width: event.width,
                        height: event.height,
                    });

                    if event.count == 0 {
                        let rects = window.window.state.expose_rects.take();
                        xcb::xcb_set_clip_rectangles(
                            self.inner.connection,
                            xcb::XCB_CLIP_ORDERING_UNSORTED as u8,
                            window.window.state.gcontext_id,
                            0,
                            0,
                            rects.len() as u32,
                            rects.as_ptr(),
                        );

                        window.window.state.handler.display(window);

                        xcb::xcb_set_clip_rectangles(
                            self.inner.connection,
                            xcb::XCB_CLIP_ORDERING_UNSORTED as u8,
                            window.window.state.gcontext_id,
                            0,
                            0,
                            0,
                            ptr::null(),
                        );
                    }
                }
            }
            xcb::XCB_CLIENT_MESSAGE => {
                let event = &*(event as *mut xcb_sys::xcb_client_message_event_t);
                if event.data.data32[0] == self.inner.wm_delete_window {
                    let window = self.inner.windows.borrow().get(&event.window).cloned();
                    if let Some(window) = window {
                        window.window.state.handler.request_close(&window);
                    }
                }
            }
            _ => {}
        }

        libc::free(event as *mut ffi::c_void);
    }

    pub fn file_descriptor(&self) -> Option<std::os::raw::c_int> {
        unsafe {
            if self.inner.open.get() {
                Some(xcb::xcb_get_file_descriptor(self.inner.connection))
            } else {
                None
            }
        }
    }
}

#[derive(Debug)]
pub enum WindowError {
    ApplicationClosed,
    WindowClosed,
    WindowCreation(u8),
    MapWindow(u8),
    InvalidWindowHandle,
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
    open: Cell<bool>,
    window_id: xcb::xcb_window_t,
    gcontext_id: xcb::xcb_gcontext_t,
    shm_state: RefCell<Option<ShmState>>,
    expose_rects: RefCell<Vec<xcb::xcb_rectangle_t>>,
    application: crate::Application,
    handler: Box<dyn WindowHandler>,
}

struct ShmState {
    shm_id: os::raw::c_int,
    shm_seg_id: xcb::xcb_shm_seg_t,
    shm_ptr: *mut ffi::c_void,
    width: usize,
    height: usize,
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

            let parent_id = if let Parent::Parent(parent) = options.parent {
                let parent_id = match parent.raw_window_handle() {
                    RawWindowHandle::Xcb(handle) => handle.window,
                    RawWindowHandle::Xlib(handle) => handle.window as u32,
                    _ => {
                        return Err(WindowError::InvalidWindowHandle);
                    }
                };

                if parent_id == 0 {
                    return Err(WindowError::InvalidWindowHandle);
                }

                parent_id
            } else {
                (*application.application.inner.screen).root
            };

            let window_id = xcb::xcb_generate_id(application.application.inner.connection);
            let value_mask = xcb::XCB_CW_EVENT_MASK;
            let value_list = &[xcb::XCB_EVENT_MASK_EXPOSURE];
            let cookie = xcb::xcb_create_window_checked(
                application.application.inner.connection,
                xcb::XCB_COPY_FROM_PARENT as u8,
                window_id,
                parent_id,
                options.rect.x as i16,
                options.rect.y as i16,
                options.rect.width as u16,
                options.rect.height as u16,
                0,
                xcb::XCB_WINDOW_CLASS_INPUT_OUTPUT as u16,
                xcb::XCB_COPY_FROM_PARENT,
                value_mask,
                value_list.as_ptr() as *const ffi::c_void,
            );

            let error = xcb::xcb_request_check(application.application.inner.connection, cookie);
            if !error.is_null() {
                let error_code = (*error).error_code;
                libc::free(error as *mut ffi::c_void);
                return Err(WindowError::WindowCreation(error_code));
            }

            let gcontext_id = xcb::xcb_generate_id(application.application.inner.connection);
            xcb::xcb_create_gc_checked(
                application.application.inner.connection,
                gcontext_id,
                window_id,
                0,
                ptr::null(),
            );

            let shm_state = Self::init_shm(
                application,
                options.rect.width as usize,
                options.rect.height as usize,
            );

            let atoms = &[application.application.inner.wm_delete_window];
            xcb::xcb_icccm_set_wm_protocols(
                application.application.inner.connection,
                window_id,
                application.application.inner.wm_protocols,
                atoms.len() as u32,
                atoms.as_ptr() as *mut xcb::xcb_atom_t,
            );

            let title =
                ffi::CString::new(options.title).unwrap_or_else(|_| ffi::CString::default());
            xcb::xcb_change_property(
                application.application.inner.connection,
                xcb::XCB_PROP_MODE_REPLACE as u8,
                window_id,
                xcb::XCB_ATOM_WM_NAME,
                xcb::XCB_ATOM_STRING,
                8,
                title.as_bytes().len() as u32,
                title.as_ptr() as *const ffi::c_void,
            );

            let window = crate::Window {
                window: Window {
                    state: Rc::new(WindowState {
                        open: Cell::new(true),
                        window_id,
                        gcontext_id,
                        shm_state: RefCell::new(shm_state),
                        expose_rects: RefCell::new(Vec::new()),
                        application: application.clone(),
                        handler: options.handler,
                    }),
                },
                phantom: PhantomData,
            };

            application.application.inner.windows.borrow_mut().insert(window_id, window.clone());

            window.window.state.handler.create(&window);

            let cookie =
                xcb::xcb_map_window_checked(application.application.inner.connection, window_id);

            let error = xcb::xcb_request_check(application.application.inner.connection, cookie);
            if !error.is_null() {
                let error_code = (*error).error_code;
                libc::free(error as *mut ffi::c_void);
                return Err(WindowError::MapWindow(error_code));
            }

            xcb::xcb_flush(application.application.inner.connection);

            Ok(window)
        }
    }

    fn init_shm(application: &crate::Application, width: usize, height: usize) -> Option<ShmState> {
        unsafe {
            if !application.application.inner.shm {
                return None;
            }

            let shm_id =
                libc::shmget(libc::IPC_PRIVATE, width * height * 4, libc::IPC_CREAT | 0o600);
            if shm_id == -1 {
                return None;
            }

            let shm_ptr = libc::shmat(shm_id, ptr::null(), 0);
            if shm_ptr == usize::MAX as *mut ffi::c_void {
                libc::shmctl(shm_id, libc::IPC_RMID, ptr::null_mut());
                return None;
            }

            let shm_seg_id = xcb::xcb_generate_id(application.application.inner.connection);
            let cookie = xcb::xcb_shm_attach_checked(
                application.application.inner.connection,
                shm_seg_id,
                shm_id as u32,
                0,
            );
            let error = xcb::xcb_request_check(application.application.inner.connection, cookie);
            if !error.is_null() {
                libc::free(error as *mut ffi::c_void);
                libc::shmctl(shm_id, libc::IPC_RMID, ptr::null_mut());
                return None;
            }

            Some(ShmState { shm_id, shm_seg_id, shm_ptr, width, height })
        }
    }

    fn deinit_shm(application: &crate::Application, shm_state: ShmState) {
        unsafe {
            xcb::xcb_shm_detach(application.application.inner.connection, shm_state.shm_seg_id);
            libc::shmdt(shm_state.shm_ptr);
            libc::shmctl(shm_state.shm_id, libc::IPC_RMID, ptr::null_mut());
        }
    }

    pub fn request_display(&self) {}

    pub fn request_display_rect(&self, _rect: Rect) {}

    pub fn update_contents(&self, framebuffer: &[u32], width: usize, height: usize) {
        unsafe {
            let width = width.min(framebuffer.len());
            let height = height.min(framebuffer.len() / width);

            if let Some(ref shm_state) = *self.state.shm_state.borrow() {
                // this is safe because shm_ptr is page-aligned and thus u32-aligned
                let data = slice::from_raw_parts_mut(
                    shm_state.shm_ptr as *mut u32,
                    shm_state.width * shm_state.height * std::mem::size_of::<u32>(),
                );

                let copy_width = width.min(shm_state.width);
                let copy_height = height.min(shm_state.height);
                for row in 0..copy_height {
                    let src = &framebuffer[row * width..row * width + copy_width];
                    let dst = &mut data[row * shm_state.width..row * shm_state.width + copy_width];
                    dst.copy_from_slice(src);
                }

                let cookie = xcb::xcb_shm_put_image(
                    self.state.application.application.inner.connection,
                    self.state.window_id,
                    self.state.gcontext_id,
                    shm_state.width as u16,
                    shm_state.height as u16,
                    0,
                    0,
                    shm_state.width as u16,
                    shm_state.height as u16,
                    0,
                    0,
                    24,
                    xcb::XCB_IMAGE_FORMAT_Z_PIXMAP as u8,
                    0,
                    shm_state.shm_seg_id,
                    0,
                );

                xcb::xcb_request_check(self.state.application.application.inner.connection, cookie);
            } else {
                xcb::xcb_put_image(
                    self.state.application.application.inner.connection,
                    xcb::XCB_IMAGE_FORMAT_Z_PIXMAP as u8,
                    self.state.window_id,
                    self.state.gcontext_id,
                    width as u16,
                    height as u16,
                    0,
                    0,
                    0,
                    24,
                    (width * height * mem::size_of::<u32>()) as u32,
                    framebuffer.as_ptr() as *const u8,
                );
            }

            xcb::xcb_flush(self.state.application.application.inner.connection);
        }
    }

    pub fn close(&self) -> Result<(), WindowError> {
        unsafe {
            if self.state.open.get() {
                self.state.open.set(false);

                let application = &self.state.application;
                application.application.inner.windows.borrow_mut().remove(&self.state.window_id);

                let window = crate::Window { window: self.clone(), phantom: PhantomData };
                window.window.state.handler.destroy(&window);

                if let Some(shm_state) = self.state.shm_state.take() {
                    Self::deinit_shm(&self.state.application, shm_state);
                }

                xcb::xcb_free_gc(
                    self.state.application.application.inner.connection,
                    self.state.gcontext_id,
                );

                let cookie = xcb::xcb_destroy_window_checked(
                    self.state.application.application.inner.connection,
                    self.state.window_id,
                );
                let error = xcb::xcb_request_check(
                    self.state.application.application.inner.connection,
                    cookie,
                );

                if !error.is_null() {
                    libc::free(error as *mut ffi::c_void);
                    return Err(WindowError::WindowClosed);
                }
            }

            Ok(())
        }
    }

    pub fn application(&self) -> &crate::Application {
        &self.state.application
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        if self.state.open.get() {
            RawWindowHandle::Xcb(XcbHandle {
                window: self.state.window_id,
                connection: self.state.application.application.inner.connection as *mut ffi::c_void,
                ..XcbHandle::empty()
            })
        } else {
            RawWindowHandle::Xcb(XcbHandle::empty())
        }
    }
}
