use std::ffi::CString;
use std::os::raw::c_char;
use std::slice;

use crate::bus::{BuildBuses, BusDir, BusInfo};
use crate::plugin::{BuildInfo, Plugin, PluginInfo};

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
        unsafe { slice::from_raw_parts(ptr, len) }
    } else {
        &[]
    }
}

#[allow(unused)]
pub trait RequireSendSync: Send + Sync {}

pub fn with_info<P, F>(f: F)
where
    P: Plugin,
    F: FnOnce(PluginInfo),
{
    struct BuildInfoFn<F>(F);

    impl<F> BuildInfo for BuildInfoFn<F>
    where
        F: FnOnce(PluginInfo),
    {
        fn info(self, info: PluginInfo) {
            self.0(info)
        }
    }

    P::info(BuildInfoFn(f))
}

pub struct OwnedBusInfo {
    pub name: String,
    pub dir: BusDir,
}

pub fn collect_buses<P: Plugin>(plugin: &P) -> Vec<OwnedBusInfo> {
    struct CollectBuses<'a>(&'a mut Vec<OwnedBusInfo>);

    impl<'a> BuildBuses for CollectBuses<'a> {
        fn bus(self, bus: BusInfo) -> Self {
            self.0.push(OwnedBusInfo {
                name: bus.name.to_string(),
                dir: bus.dir,
            });
            self
        }
    }

    let mut buses = Vec::new();
    plugin.buses(CollectBuses(&mut buses));
    buses
}
