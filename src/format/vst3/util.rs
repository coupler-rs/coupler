use std::slice;

use vst3::Steinberg::char16;

pub fn copy_wstring(src: &str, dst: &mut [char16]) {
    let mut len = 0;
    for (src, dst) in src.encode_utf16().zip(dst.iter_mut()) {
        *dst = src as char16;
        len += 1;
    }

    if len < dst.len() {
        dst[len] = 0;
    } else if let Some(last) = dst.last_mut() {
        *last = 0;
    }
}

pub unsafe fn utf16_from_ptr<'a>(ptr: *const char16) -> &'a [u16] {
    let mut len = 0;
    while *ptr.offset(len as isize) != 0 {
        len += 1;
    }

    slice::from_raw_parts(ptr as *const u16, len)
}

pub unsafe fn slice_from_raw_parts_checked<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if len > 0 {
        slice::from_raw_parts(ptr, len)
    } else {
        &[]
    }
}
