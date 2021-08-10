use crate::{Cursor, MouseButton, Parent, Point, Rect, WindowHandler, WindowOptions};

use std::cell::Cell;
use std::collections::HashSet;
use std::error::Error;
use std::marker::PhantomData;
use std::os::windows::ffi::OsStrExt;
use std::rc::Rc;
use std::{ffi, fmt, mem, os, ptr};

use raw_window_handle::{windows::WindowsHandle, HasRawWindowHandle, RawWindowHandle};
use winapi::{
    shared::minwindef, shared::ntdef, shared::windef, shared::windowsx, um::errhandlingapi,
    um::wingdi, um::winnt, um::winuser,
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
                    running: Cell::new(0),
                    class,
                    windows: Cell::new(HashSet::new()),
                }),
            })
        }
    }

    pub fn close(&self) -> Result<(), ApplicationError> {
        unsafe {
            if self.inner.open.get() {
                self.inner.running.set(0);
                self.inner.open.set(false);

                let mut window_errors = Vec::new();
                for window in self.inner.windows.take() {
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
                let depth = self.inner.running.get();
                self.inner.running.set(depth + 1);

                while self.inner.open.get() && self.inner.running.get() > depth {
                    let mut msg: winuser::MSG = mem::zeroed();

                    let result = winuser::GetMessageW(&mut msg, ptr::null_mut(), 0, 0);
                    if result < 0 {
                        self.inner.running.set(depth);
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
        self.inner.running.set(self.inner.running.get().saturating_sub(1));
    }

    pub fn poll(&self) {
        unsafe {
            while self.inner.open.get() {
                let mut msg: winuser::MSG = mem::zeroed();

                let result =
                    winuser::PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, winuser::PM_REMOVE);
                if result < 0 {
                    break;
                } else if result == 0 {
                    // ignore WM_QUIT messages
                    continue;
                }

                winuser::TranslateMessage(&msg);
                winuser::DispatchMessageW(&msg);
            }
        }
    }

    pub fn file_descriptor(&self) -> Option<std::os::raw::c_int> {
        None
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
    state: Rc<WindowState>,
}

struct WindowState {
    open: Cell<bool>,
    hwnd: Cell<windef::HWND>,
    hdc: Cell<Option<windef::HDC>>,
    mouse_down_count: Cell<usize>,
    cursor: Cell<Cursor>,
    application: crate::Application,
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
                left: options.rect.x.round() as i32,
                top: options.rect.y.round() as i32,
                right: (options.rect.x + options.rect.width).round() as i32,
                bottom: (options.rect.y + options.rect.height).round() as i32,
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

            let state = Rc::new(WindowState {
                open: Cell::new(false),
                hwnd: Cell::new(ptr::null_mut()),
                hdc: Cell::new(None),
                mouse_down_count: Cell::new(0),
                cursor: Cell::new(Cursor::Arrow),
                application: application.clone(),
                handler: options.handler,
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
        unsafe {
            if self.state.open.get() {
                winuser::InvalidateRect(self.state.hwnd.get(), ptr::null(), minwindef::FALSE);
            }
        }
    }

    pub fn request_display_rect(&self, rect: Rect) {
        unsafe {
            if self.state.open.get() {
                let rect = windef::RECT {
                    left: rect.x.round() as winnt::LONG,
                    top: rect.y.round() as winnt::LONG,
                    right: (rect.x + rect.width).round() as winnt::LONG,
                    bottom: (rect.y + rect.height).round() as winnt::LONG,
                };

                winuser::InvalidateRect(self.state.hwnd.get(), &rect, minwindef::FALSE);
            }
        }
    }

    pub fn update_contents(&self, framebuffer: &[u32], width: usize, height: usize) {
        unsafe {
            if self.state.open.get() {
                let hdc = if let Some(hdc) = self.state.hdc.get() {
                    hdc
                } else {
                    winuser::GetDC(self.state.hwnd.get())
                };

                if !hdc.is_null() {
                    let width = width.min(framebuffer.len());
                    let height = height.min(framebuffer.len() / width);

                    let bitmap_info = wingdi::BITMAPINFO {
                        bmiHeader: wingdi::BITMAPINFOHEADER {
                            biSize: mem::size_of::<wingdi::BITMAPINFOHEADER>() as u32,
                            biWidth: width as i32,
                            biHeight: -(height as i32),
                            biPlanes: 1,
                            biBitCount: 32,
                            biCompression: wingdi::BI_RGB,
                            ..mem::zeroed()
                        },
                        ..mem::zeroed()
                    };

                    wingdi::StretchDIBits(
                        hdc,
                        0,
                        0,
                        width as i32,
                        height as i32,
                        0,
                        0,
                        width as i32,
                        height as i32,
                        framebuffer.as_ptr() as *const ntdef::VOID,
                        &bitmap_info,
                        wingdi::DIB_RGB_COLORS,
                        wingdi::SRCCOPY,
                    );

                    if self.state.hdc.get().is_none() {
                        winuser::ReleaseDC(self.state.hwnd.get(), hdc);
                    }
                }
            }
        }
    }

    pub fn set_cursor(&self, cursor: Cursor) {
        unsafe {
            if self.state.open.get() {
                self.state.cursor.set(cursor);

                let hcursor = match cursor {
                    Cursor::Arrow => winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_ARROW),
                    Cursor::Crosshair => winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_CROSS),
                    Cursor::Hand => winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_HAND),
                    Cursor::IBeam => winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_IBEAM),
                    Cursor::No => winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_NO),
                    Cursor::SizeNs => winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_SIZENS),
                    Cursor::SizeWe => winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_SIZEWE),
                    Cursor::SizeNesw => {
                        winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_SIZENESW)
                    }
                    Cursor::SizeNwse => {
                        winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_SIZENWSE)
                    }
                    Cursor::Wait => winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_WAIT),
                    Cursor::None => ptr::null_mut(),
                };

                winuser::SetCursor(hcursor);
            }
        }
    }

    pub fn set_mouse_position(&self, position: Point) {
        unsafe {
            if self.state.open.get() {
                let mut point = windef::POINT {
                    x: position.x as os::raw::c_int,
                    y: position.y as os::raw::c_int,
                };
                winuser::ClientToScreen(self.state.hwnd.get(), &mut point);
                winuser::SetCursorPos(point.x, point.y);
            }
        }
    }

    pub fn close(&self) -> Result<(), WindowError> {
        unsafe {
            if self.state.open.get() {
                let result = winuser::DestroyWindow(self.state.hwnd.get());
                if result == 0 {
                    return Err(WindowError::WindowClose(errhandlingapi::GetLastError()));
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
        let hwnd = if self.state.open.get() {
            self.state.hwnd.get() as *mut std::ffi::c_void
        } else {
            ptr::null_mut()
        };

        RawWindowHandle::Windows(WindowsHandle { hwnd, ..WindowsHandle::empty() })
    }
}

const TIMER_ID: usize = 1;
const TIMER_INTERVAL: u32 = 16;

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
        application_windows.insert(hwnd);
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
                winuser::SetTimer(hwnd, TIMER_ID, TIMER_INTERVAL, None);

                window.window.state.handler.create(&window);
                return 0;
            }
            winuser::WM_LBUTTONDOWN
            | winuser::WM_LBUTTONUP
            | winuser::WM_MBUTTONDOWN
            | winuser::WM_MBUTTONUP
            | winuser::WM_RBUTTONDOWN
            | winuser::WM_RBUTTONUP
            | winuser::WM_XBUTTONDOWN
            | winuser::WM_XBUTTONUP => {
                let button = match msg {
                    winuser::WM_LBUTTONDOWN | winuser::WM_LBUTTONUP => Some(MouseButton::Left),
                    winuser::WM_MBUTTONDOWN | winuser::WM_MBUTTONUP => Some(MouseButton::Middle),
                    winuser::WM_RBUTTONDOWN | winuser::WM_RBUTTONUP => Some(MouseButton::Right),
                    winuser::WM_XBUTTONDOWN | winuser::WM_XBUTTONUP => {
                        match winuser::GET_XBUTTON_WPARAM(wparam) {
                            winuser::XBUTTON1 => Some(MouseButton::Back),
                            winuser::XBUTTON2 => Some(MouseButton::Forward),
                            _ => None,
                        }
                    }
                    _ => None,
                };

                if let Some(button) = button {
                    let result = match msg {
                        winuser::WM_LBUTTONDOWN
                        | winuser::WM_MBUTTONDOWN
                        | winuser::WM_RBUTTONDOWN
                        | winuser::WM_XBUTTONDOWN => {
                            let mouse_down_count =
                                window.window.state.mouse_down_count.get().saturating_add(1);
                            window.window.state.mouse_down_count.set(mouse_down_count);
                            if mouse_down_count == 1 {
                                winuser::SetCapture(window.window.state.hwnd.get());
                            }

                            window.window.state.handler.mouse_down(&window, button)
                        }
                        winuser::WM_LBUTTONUP
                        | winuser::WM_MBUTTONUP
                        | winuser::WM_RBUTTONUP
                        | winuser::WM_XBUTTONUP => {
                            let mouse_down_count =
                                window.window.state.mouse_down_count.get().saturating_sub(1);
                            window.window.state.mouse_down_count.set(mouse_down_count);
                            if mouse_down_count == 0 {
                                winuser::ReleaseCapture();
                            }

                            window.window.state.handler.mouse_up(&window, button)
                        }
                        _ => false,
                    };

                    if result {
                        return 0;
                    }
                }
            }
            winuser::WM_MOUSEMOVE => {
                let point = Point {
                    x: windowsx::GET_X_LPARAM(lparam) as f64,
                    y: windowsx::GET_Y_LPARAM(lparam) as f64,
                };
                window.window.state.handler.mouse_move(&window, point);

                return 0;
            }
            winuser::WM_MOUSEWHEEL | winuser::WM_MOUSEHWHEEL => {
                let delta = winuser::GET_WHEEL_DELTA_WPARAM(wparam) as f64 / 120.0;
                let (dx, dy) = match msg {
                    winuser::WM_MOUSEWHEEL => (0.0, delta),
                    winuser::WM_MOUSEHWHEEL => (delta, 0.0),
                    _ => unreachable!(),
                };
                let result = window.window.state.handler.scroll(&window, dx, dy);

                if result {
                    return 0;
                }
            }
            winuser::WM_SETCURSOR => {
                if minwindef::LOWORD(lparam as minwindef::DWORD)
                    == winuser::HTCLIENT as minwindef::WORD
                {
                    window.set_cursor(window.window.state.cursor.get());
                    return 0;
                }
            }
            winuser::WM_TIMER => {
                if wparam == TIMER_ID {
                    window.window.state.handler.frame(&window);
                }
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

                window.window.state.handler.display(&window);

                window.window.state.hdc.set(None);
                winuser::EndPaint(hwnd, &paint_struct);
            }
            winuser::WM_CLOSE => {
                window.window.state.handler.request_close(&window);
                return 0;
            }
            winuser::WM_DESTROY => {
                window.window.state.open.set(false);
                let mut application_windows = window.application().application.inner.windows.take();
                application_windows.remove(&hwnd);
                window.application().application.inner.windows.set(application_windows);
                window.window.state.handler.destroy(&window);

                winuser::KillTimer(hwnd, TIMER_ID);

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
