use crate::WindowOptions;

use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

use winapi::{
    shared::minwindef, shared::windef, um::errhandlingapi, um::libloaderapi, um::winuser,
};

fn to_wstring(str: &str) -> Vec<u16> {
    let mut wstr: Vec<u16> = OsStr::new(str).encode_wide().collect();
    wstr.push(0);
    wstr
}

#[derive(Debug)]
pub enum WindowError {
    ClassCreation(u32),
    WindowCreation(u32),
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for WindowError {}

#[derive(Clone)]
pub struct Window {
    hwnd: windef::HWND,
}

impl Window {
    pub fn open(options: WindowOptions) -> Result<Window, WindowError> {
        unsafe {
            let class_name = to_wstring("plugin-window");
            let class = winuser::WNDCLASSW {
                style: winuser::CS_HREDRAW | winuser::CS_VREDRAW | winuser::CS_OWNDC,
                lpfnWndProc: Some(wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: libloaderapi::GetModuleHandleA(ptr::null()),
                hIcon: ptr::null_mut(),
                hCursor: winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_ARROW),
                hbrBackground: ptr::null_mut(),
                lpszMenuName: ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };

            if winuser::RegisterClassW(&class) == 0 {
                // ignore the "class already exists" error
                if errhandlingapi::GetLastError() as u32 != 1410 {
                    return Err(WindowError::ClassCreation(errhandlingapi::GetLastError()));
                }
            }

            let flags = winuser::WS_POPUPWINDOW
                | winuser::WS_CAPTION
                | winuser::WS_VISIBLE
                | winuser::WS_SIZEBOX
                | winuser::WS_MINIMIZEBOX
                | winuser::WS_MAXIMIZEBOX
                | winuser::WS_CLIPSIBLINGS;

            let mut rect = windef::RECT {
                left: 0,
                top: 0,
                right: options.width.round() as i32,
                bottom: options.height.round() as i32,
            };

            winuser::AdjustWindowRectEx(&mut rect, flags, minwindef::FALSE, 0);

            let window_name = to_wstring(&options.title);
            let hwnd = winuser::CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_name.as_ptr(),
                flags,
                winuser::CW_USEDEFAULT,
                winuser::CW_USEDEFAULT,
                rect.right - rect.left,
                rect.bottom - rect.top,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
            if hwnd.is_null() {
                return Err(WindowError::WindowCreation(errhandlingapi::GetLastError()));
            }

            winuser::ShowWindow(hwnd, winuser::SW_NORMAL);

            Ok(Window { hwnd })
        }
    }
}

unsafe extern "system" fn wnd_proc(
    window: windef::HWND,
    msg: minwindef::UINT,
    wparam: minwindef::WPARAM,
    lparam: minwindef::LPARAM,
) -> minwindef::LRESULT {
    winuser::DefWindowProcW(window, msg, wparam, lparam)
}
