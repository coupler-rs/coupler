use std::ffi::CString;
use std::fmt::{self, Display, Formatter};
use std::os::raw::c_char;
use std::slice;

use crate::params::{ParamId, ParamValue};
use crate::plugin::Plugin;

pub fn copy_cstring(src: &str, dst: &mut [c_char]) {
    let c_string = CString::new(src).unwrap_or_else(|_| CString::default());
    let bytes = c_string.as_bytes_with_nul();

    for (src, dst) in bytes.iter().zip(dst.iter_mut()) {
        *dst = *src as c_char;
    }

    if bytes.len() > dst.len() {
        if let Some(last) = dst.last_mut() {
            *last = 0;
        }
    }
}

// The pointer passed to `slice::from_raw_parts` must be non-null and aligned even for zero-length
// slices. This won't be true in general for a pointer to a zero-length array received from an
// external source. `slice_from_raw_parts_checked` is a convenience function that checks if `len`
// is nonzero before calling `from_raw_parts`.
pub unsafe fn slice_from_raw_parts_checked<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if len > 0 {
        slice::from_raw_parts(ptr, len)
    } else {
        &[]
    }
}

pub struct DisplayParam<'a, P> {
    plugin: &'a P,
    id: ParamId,
    value: ParamValue,
}

impl<'a, P> DisplayParam<'a, P> {
    pub fn new(plugin: &'a P, id: ParamId, value: ParamValue) -> Self {
        DisplayParam { plugin, id, value }
    }
}

impl<'a, P> Display for DisplayParam<'a, P>
where
    P: Plugin,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        self.plugin.display_param(self.id, self.value, f)
    }
}
