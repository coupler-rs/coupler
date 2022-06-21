use crate::bus::*;

use std::ffi::CString;
use std::os::raw::c_char;

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

pub fn validate_bus_configs(buses: &BusList, configs: &BusConfigList) {
    let input_count = buses.get_inputs().len();
    let output_count = buses.get_inputs().len();
    for config in configs.get_configs() {
        assert!(
            config.get_inputs().len() == input_count,
            "bus config specifies {} inputs but plugin has {} inputs:\n{:?}",
            config.get_inputs().len(),
            input_count,
            &config
        );

        assert!(
            config.get_outputs().len() == output_count,
            "bus config specifies {} outputs but plugin has {} outputs:\n{:?}",
            config.get_outputs().len(),
            output_count,
            &config
        );
    }

    configs.get_default().expect("must specify at least one bus config");
}
