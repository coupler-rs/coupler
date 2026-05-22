use std::slice;

use vst3::Steinberg::char16;

use super::{BuildVst3Info, Vst3Info, Vst3Plugin};

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
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }

    unsafe { slice::from_raw_parts(ptr, len) }
}

pub fn with_vst3_info<P, F>(f: F)
where
    P: Vst3Plugin,
    F: FnOnce(Vst3Info),
{
    struct BuildVst3InfoFn<F>(F);

    impl<F> BuildVst3Info for BuildVst3InfoFn<F>
    where
        F: FnOnce(Vst3Info),
    {
        fn info(self, info: Vst3Info) {
            self.0(info)
        }
    }

    P::vst3_info(BuildVst3InfoFn(f))
}
