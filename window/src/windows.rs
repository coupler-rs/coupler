use crate::{DefaultWindowHandler, Parent, Rect, WindowHandler, WindowOptions};

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::error::Error;
use std::marker::PhantomData;
use std::os::windows::ffi::OsStrExt;
use std::rc::Rc;
use std::{ffi, fmt, mem, ptr};

use raw_window_handle::{windows::WindowsHandle, HasRawWindowHandle, RawWindowHandle};
use winapi::{
    shared::minwindef, shared::ntdef, shared::windef, um::errhandlingapi, um::winnt, um::winuser,
};

fn to_wstring(str: &str) -> Vec<ntdef::WCHAR> {
    let mut wstr: Vec<ntdef::WCHAR> = ffi::OsStr::new(str).encode_wide().collect();
    wstr.push(0);
    wstr
}

#[derive(Debug)]
pub enum ApplicationError {
    WindowClassRegistration(u32),
    WindowClassUnregistration(u32),
    GetMessage(u32),
    AlreadyRunning,
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
    running: Cell<bool>,
    class: minwindef::ATOM,
    windows: Cell<HashMap<windef::HWND, Window>>,
}

extern "C" {
    static __ImageBase: winnt::IMAGE_DOS_HEADER;
}

impl Application {
    pub fn open() -> Result<Application, ApplicationError> {
        unsafe {
            let class_name = to_wstring(&format!("window-{}", uuid::Uuid::new_v4().to_simple()));
            let wnd_class = winuser::WNDCLASSW {
                style: winuser::CS_HREDRAW | winuser::CS_VREDRAW | winuser::CS_OWNDC,
                lpfnWndProc: Some(wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: &__ImageBase as *const winnt::IMAGE_DOS_HEADER as minwindef::HINSTANCE,
                hIcon: ptr::null_mut(),
                hCursor: winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_ARROW),
                hbrBackground: ptr::null_mut(),
                lpszMenuName: ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };

            let class = winuser::RegisterClassW(&wnd_class);
            if class == 0 {
                return Err(ApplicationError::WindowClassRegistration(
                    errhandlingapi::GetLastError(),
                ));
            }

            Ok(Application {
                inner: Rc::new(ApplicationInner {
                    open: Cell::new(true),
                    running: Cell::new(false),
                    class,
                    windows: Cell::new(HashMap::new()),
                }),
            })
        }
    }

    pub fn close(&self) -> Result<(), ApplicationError> {
        unsafe {
            if self.inner.open.get() {
                self.stop();
                self.inner.open.set(false);

                for (_, window) in self.inner.windows.take() {
                    window.close();
                }

                let result = winuser::UnregisterClassW(
                    self.inner.class as *const ntdef::WCHAR,
                    &__ImageBase as *const winnt::IMAGE_DOS_HEADER as minwindef::HINSTANCE,
                );

                if result == 0 {
                    return Err(ApplicationError::WindowClassUnregistration(
                        errhandlingapi::GetLastError(),
                    ));
                }
            }

            Ok(())
        }
    }

    pub fn start(&self) -> Result<(), ApplicationError> {
        unsafe {
            if self.inner.open.get() {
                if self.inner.running.get() {
                    return Err(ApplicationError::AlreadyRunning);
                }

                self.inner.running.set(true);

                while self.inner.open.get() && self.inner.running.get() {
                    let mut msg: winuser::MSG = mem::zeroed();

                    let result = winuser::GetMessageW(&mut msg, ptr::null_mut(), 0, 0);
                    if result < 0 {
                        self.inner.running.set(false);
                        return Err(ApplicationError::GetMessage(errhandlingapi::GetLastError()));
                    } else if result == 0 {
                        // ignore WM_QUIT messages
                        continue;
                    }

                    winuser::TranslateMessage(&msg);
                    winuser::DispatchMessageW(&msg);
                }
            }

            Ok(())
        }
    }

    pub fn stop(&self) {
        self.inner.running.set(false);
    }
}

#[derive(Debug)]
pub enum WindowError {
    ApplicationClosed,
    WindowOpen(u32),
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
    hwnd: Cell<windef::HWND>,
    hdc: Cell<Option<windef::HDC>>,
    deferred: Cell<Vec<Box<dyn FnOnce(&Window)>>>,
    application: crate::Application,
    handler: RefCell<Box<dyn WindowHandler>>,
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

            let mut flags = winuser::WS_CLIPCHILDREN | winuser::WS_CLIPSIBLINGS;

            if let Parent::Parent(_) = options.parent {
                flags |= winuser::WS_CHILD;
            } else {
                flags |= winuser::WS_CAPTION
                    | winuser::WS_SIZEBOX
                    | winuser::WS_SYSMENU
                    | winuser::WS_MINIMIZEBOX
                    | winuser::WS_MAXIMIZEBOX;
            }

            let mut rect = windef::RECT {
                left: options.rect.x.round() as i32,
                top: options.rect.y.round() as i32,
                right: (options.rect.x + options.rect.w).round() as i32,
                bottom: (options.rect.y + options.rect.h).round() as i32,
            };

            winuser::AdjustWindowRectEx(&mut rect, flags, minwindef::FALSE, 0);

            let parent = if let Parent::Parent(parent) = options.parent {
                match parent.raw_window_handle() {
                    RawWindowHandle::Windows(handle) => {
                        if handle.hwnd.is_null() {
                            return Err(WindowError::InvalidWindowHandle);
                        }
                        handle.hwnd as windef::HWND
                    }
                    _ => {
                        return Err(WindowError::InvalidWindowHandle);
                    }
                }
            } else {
                ptr::null_mut()
            };

            let handler = options.handler.unwrap_or_else(|| Box::new(DefaultWindowHandler));

            let state = Rc::new(WindowState {
                open: Cell::new(false),
                hwnd: Cell::new(ptr::null_mut()),
                hdc: Cell::new(None),
                deferred: Cell::new(Vec::new()),
                application: application.clone(),
                handler: RefCell::new(handler),
            });

            let window_name = to_wstring(&options.title);
            let hwnd = winuser::CreateWindowExW(
                0,
                application.application.inner.class as *const ntdef::WCHAR,
                window_name.as_ptr(),
                flags,
                winuser::CW_USEDEFAULT,
                winuser::CW_USEDEFAULT,
                rect.right - rect.left,
                rect.bottom - rect.top,
                parent,
                ptr::null_mut(),
                ptr::null_mut(),
                Rc::into_raw(state.clone()) as minwindef::LPVOID,
            );
            if hwnd.is_null() {
                return Err(WindowError::WindowOpen(errhandlingapi::GetLastError()));
            }

            if state.open.get() {
                winuser::ShowWindow(hwnd, winuser::SW_SHOWNORMAL);
                winuser::UpdateWindow(hwnd);
            }

            Ok(crate::Window { window: Window { state }, phantom: PhantomData })
        }
    }

    pub fn request_display(&self) {
        self.defer(|window| unsafe {
            if window.state.open.get() {
                winuser::InvalidateRect(window.state.hwnd.get(), ptr::null(), minwindef::FALSE);
            }
        });
    }

    pub fn request_display_rect(&self, rect: Rect) {
        self.defer(move |window| unsafe {
            if window.state.open.get() {
                let rect = windef::RECT {
                    left: rect.x.round() as winnt::LONG,
                    top: rect.y.round() as winnt::LONG,
                    right: (rect.x + rect.w).round() as winnt::LONG,
                    bottom: (rect.y + rect.h).round() as winnt::LONG,
                };

                winuser::InvalidateRect(window.state.hwnd.get(), &rect, minwindef::FALSE);
            }
        });
    }

    pub fn close(&self) {
        self.defer(|window| unsafe {
            if window.state.open.get() {
                winuser::DestroyWindow(window.state.hwnd.get());
            }
        });
    }

    pub fn application(&self) -> &crate::Application {
        &self.state.application
    }

    fn defer(&self, action: impl FnOnce(&Window) + 'static) {
        if self.state.handler.try_borrow().is_ok() {
            action(self);
        } else {
            let mut deferred = self.state.deferred.take();
            deferred.push(Box::new(action));
            self.state.deferred.set(deferred);
        }
    }

    fn process_deferred(&self) {
        for action in self.state.deferred.take() {
            action(self);
        }
    }

    fn with_handler(&self, f: impl FnOnce(&mut Box<dyn WindowHandler>)) {
        if let Ok(mut handler) = self.state.handler.try_borrow_mut() {
            f(&mut *handler);
        }
        self.process_deferred();
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let hwnd = if self.state.open.get() {
            self.state.hwnd.get() as *mut std::ffi::c_void
        } else {
            ptr::null_mut()
        };

        RawWindowHandle::Windows(WindowsHandle { hwnd, ..WindowsHandle::empty() })
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: windef::HWND,
    msg: minwindef::UINT,
    wparam: minwindef::WPARAM,
    lparam: minwindef::LPARAM,
) -> minwindef::LRESULT {
    if msg == winuser::WM_NCCREATE {
        let create_struct = &*(lparam as *const winuser::CREATESTRUCTW);
        let state_ptr = create_struct.lpCreateParams as *mut WindowState;
        let state = Rc::from_raw(state_ptr);

        state.open.set(true);
        state.hwnd.set(hwnd);

        let mut application_windows = state.application.application.inner.windows.take();
        application_windows.insert(hwnd, Window { state: state.clone() });
        state.application.application.inner.windows.set(application_windows);

        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, Rc::into_raw(state) as isize);

        return 1;
    }

    let state_ptr = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA) as *mut WindowState;
    if !state_ptr.is_null() {
        let state = Rc::from_raw(state_ptr);
        let _ = Rc::into_raw(state.clone());
        let window = crate::Window { window: Window { state }, phantom: PhantomData };

        match msg {
            winuser::WM_CREATE => {
                window.window.with_handler(|handler| handler.open(&window));
                return 0;
            }
            winuser::WM_ERASEBKGND => {
                return 1;
            }
            winuser::WM_PAINT => {
                let mut paint_struct: winuser::PAINTSTRUCT = mem::zeroed();
                let hdc = winuser::BeginPaint(hwnd, &mut paint_struct);
                if !hdc.is_null() {
                    window.window.state.hdc.set(Some(hdc));
                }

                window.window.with_handler(|handler| handler.display(&window));

                window.window.state.hdc.set(None);
                winuser::EndPaint(hwnd, &paint_struct);
            }
            winuser::WM_CLOSE => {
                window.window.with_handler(|handler| handler.request_close(&window));
                return 0;
            }
            winuser::WM_DESTROY => {
                window.window.state.open.set(false);
                let mut application_windows = window.application().application.inner.windows.take();
                application_windows.remove(&hwnd);
                window.application().application.inner.windows.set(application_windows);
                window.window.with_handler(|handler| handler.close(&window));
                return 0;
            }
            winuser::WM_NCDESTROY => {
                drop(Rc::from_raw(state_ptr));
                winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, 0);
                return 0;
            }
            _ => {}
        }
    }

    winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
}
