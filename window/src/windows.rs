use crate::{DefaultWindowHandler, Parent, WindowHandler, WindowOptions};

use std::cell::Cell;
use std::collections::HashSet;
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
    Close(Vec<WindowError>),
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
    windows: Cell<HashSet<windef::HWND>>,
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
                    windows: Cell::new(HashSet::new()),
                }),
            })
        }
    }

    pub fn close(&self) -> Result<(), ApplicationError> {
        unsafe {
            if self.inner.open.get() {
                self.stop();
                self.inner.open.set(false);

                let windows = self.inner.windows.take();
                let mut window_errors = Vec::new();
                for window in windows {
                    let result = winuser::DestroyWindow(window);
                    if result == 0 {
                        window_errors
                            .push(WindowError::WindowClose(errhandlingapi::GetLastError()));
                    }
                }
                if !window_errors.is_empty() {
                    return Err(ApplicationError::Close(window_errors));
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
        self.inner.open.set(false);
    }
}

#[derive(Debug)]
pub enum WindowError {
    ApplicationClosed,
    WindowOpen(u32),
    WindowClose(u32),
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
    inner: Rc<WindowInner>,
}

struct WindowInner {
    open: Cell<bool>,
    hwnd: Cell<windef::HWND>,
    application: crate::Application,
}

struct WindowState {
    window: crate::Window,
    handler: Box<dyn WindowHandler>,
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
                left: 0,
                top: 0,
                right: options.width.round() as i32,
                bottom: options.height.round() as i32,
            };

            winuser::AdjustWindowRectEx(&mut rect, flags, minwindef::FALSE, 0);

            let parent = if let Parent::Parent(parent) = options.parent {
                match parent.raw_window_handle() {
                    RawWindowHandle::Windows(handle) => handle.hwnd as windef::HWND,
                    _ => {
                        return Err(WindowError::InvalidWindowHandle);
                    }
                }
            } else {
                ptr::null_mut()
            };

            let window = crate::Window {
                window: Window {
                    inner: Rc::new(WindowInner {
                        open: Cell::new(false),
                        hwnd: Cell::new(ptr::null_mut()),
                        application: application.clone(),
                    }),
                },
                phantom: PhantomData,
            };

            let handler = options.handler.unwrap_or_else(|| Box::new(DefaultWindowHandler));

            let state_ptr = Rc::into_raw(Rc::new(WindowState {
                window: window.clone(),
                handler,
            }));

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
                state_ptr as minwindef::LPVOID,
            );
            if hwnd.is_null() {
                drop(Rc::from_raw(state_ptr));
                return Err(WindowError::WindowOpen(errhandlingapi::GetLastError()));
            }

            if window.window.inner.open.get() {
                winuser::ShowWindow(hwnd, winuser::SW_SHOWNORMAL);
            }

            Ok(window)
        }
    }

    pub fn close(&self) -> Result<(), WindowError> {
        unsafe {
            if self.inner.open.get() {
                let result = winuser::DestroyWindow(self.inner.hwnd.get());
                if result == 0 {
                    return Err(WindowError::WindowClose(errhandlingapi::GetLastError()));
                }
            }

            Ok(())
        }
    }

    pub fn application(&self) -> &crate::Application {
        &self.inner.application
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let hwnd = if self.inner.open.get() {
            self.inner.hwnd.get() as *mut std::ffi::c_void
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

        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, state_ptr as isize);

        let state = &*state_ptr;
        state.window.window.inner.open.set(true);
        state.window.window.inner.hwnd.set(hwnd);

        let mut application_windows = state.window.application().application.inner.windows.take();
        application_windows.insert(hwnd.clone());
        state.window.application().application.inner.windows.set(application_windows);

        return 1;
    }

    let state_ptr = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA) as *mut WindowState;
    if !state_ptr.is_null() {
        let state = Rc::from_raw(state_ptr);
        let _ = Rc::into_raw(state.clone());

        match msg {
            winuser::WM_CREATE => {
                state.handler.open(&state.window);
                return 0;
            }
            winuser::WM_CLOSE => {
                state.handler.should_close(&state.window);
                return 0;
            }
            winuser::WM_DESTROY => {
                state.window.window.inner.open.set(false);
                let mut application_windows =
                    state.window.application().application.inner.windows.take();
                application_windows.remove(&hwnd);
                state.window.application().application.inner.windows.set(application_windows);
                state.handler.close(&state.window);
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
