use std::collections::HashMap;
use std::ffi::{CStr, c_char};
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::Arc;

use clap_sys::ext::{gui::*, params::*};
use clap_sys::{host::*, plugin::*};

use super::instance::Instance;
use crate::editor::{Editor, EditorHost, EditorHostInner, ParentWindow, RawParent};
use crate::params::{ParamId, ParamValue};
use crate::plugin::Plugin;
use crate::sync::thread_cell::ThreadCell;
use crate::sync::param_gestures::ParamGestures;

struct ClapEditorHost {
    host: *const clap_host,
    host_params: Option<NonNull<clap_host_params>>,
    param_map: Arc<HashMap<ParamId, usize>>,
    param_gestures: Arc<ParamGestures>,
}

impl EditorHostInner for ClapEditorHost {
    fn begin_gesture(&self, id: ParamId) {
        self.param_gestures.begin_gesture(self.param_map[&id]);

        if let Some(host_params) = self.host_params {
            unsafe { host_params.as_ref().request_flush.unwrap()(self.host) };
        }
    }

    fn end_gesture(&self, id: ParamId) {
        self.param_gestures.end_gesture(self.param_map[&id]);

        if let Some(host_params) = self.host_params {
            unsafe { host_params.as_ref().request_flush.unwrap()(self.host) };
        }
    }

    fn set_param(&self, id: ParamId, value: ParamValue) {
        self.param_gestures.set_value(self.param_map[&id], value);

        if let Some(host_params) = self.host_params {
            unsafe { host_params.as_ref().request_flush.unwrap()(self.host) };
        }
    }
}

impl<P: Plugin> Instance<P> {
    pub(super) const GUI: clap_plugin_gui = clap_plugin_gui {
        is_api_supported: Some(Self::gui_is_api_supported),
        get_preferred_api: Some(Self::gui_get_preferred_api),
        create: Some(Self::gui_create),
        destroy: Some(Self::gui_destroy),
        set_scale: Some(Self::gui_set_scale),
        get_size: Some(Self::gui_get_size),
        can_resize: Some(Self::gui_can_resize),
        get_resize_hints: Some(Self::gui_get_resize_hints),
        adjust_size: Some(Self::gui_adjust_size),
        set_size: Some(Self::gui_set_size),
        set_parent: Some(Self::gui_set_parent),
        set_transient: Some(Self::gui_set_transient),
        suggest_title: Some(Self::gui_suggest_title),
        show: Some(Self::gui_show),
        hide: Some(Self::gui_hide),
    };

    #[cfg(target_os = "windows")]
    const API: &'static CStr = CLAP_WINDOW_API_WIN32;

    #[cfg(target_os = "macos")]
    const API: &'static CStr = CLAP_WINDOW_API_COCOA;

    #[cfg(target_os = "linux")]
    const API: &'static CStr = CLAP_WINDOW_API_X11;

    unsafe extern "C" fn gui_is_api_supported(
        _plugin: *const clap_plugin,
        api: *const c_char,
        is_floating: bool,
    ) -> bool {
        if is_floating {
            return false;
        }

        let api = unsafe { CStr::from_ptr(api) };
        api == Self::API
    }

    unsafe extern "C" fn gui_get_preferred_api(
        _plugin: *const clap_plugin,
        api: *mut *const c_char,
        is_floating: *mut bool,
    ) -> bool {
        let is_floating = unsafe { &mut *is_floating };
        *is_floating = false;

        let api = unsafe { &mut *api };
        *api = Self::API.as_ptr();

        true
    }

    unsafe extern "C" fn gui_create(
        plugin: *const clap_plugin,
        api: *const c_char,
        is_floating: bool,
    ) -> bool {
        if !unsafe { Self::gui_is_api_supported(plugin, api, is_floating) } {
            return false;
        }

        true
    }

    unsafe extern "C" fn gui_destroy(plugin: *const clap_plugin) {
        let instance = unsafe { &*(plugin as *const Self) };
        let mut main_thread_state = instance.main_thread_state.borrow();

        main_thread_state.editor = None;
    }

    unsafe extern "C" fn gui_set_scale(_plugin: *const clap_plugin, _scale: f64) -> bool {
        false
    }

    unsafe extern "C" fn gui_get_size(
        plugin: *const clap_plugin,
        width: *mut u32,
        height: *mut u32,
    ) -> bool {
        let instance = unsafe { &*(plugin as *const Self) };
        let main_thread_state = instance.main_thread_state.borrow();

        let size = if let Some(editor) = &main_thread_state.editor {
            editor.size()
        } else {
            main_thread_state.plugin.editor_size()
        };

        let width = unsafe { &mut *width };
        *width = size.width.round() as u32;

        let height = unsafe { &mut *height };
        *height = size.height.round() as u32;

        true
    }

    unsafe extern "C" fn gui_can_resize(_plugin: *const clap_plugin) -> bool {
        false
    }

    unsafe extern "C" fn gui_get_resize_hints(
        _plugin: *const clap_plugin,
        _hints: *mut clap_gui_resize_hints,
    ) -> bool {
        false
    }

    unsafe extern "C" fn gui_adjust_size(
        _plugin: *const clap_plugin,
        _width: *mut u32,
        _height: *mut u32,
    ) -> bool {
        false
    }

    unsafe extern "C" fn gui_set_size(
        _plugin: *const clap_plugin,
        _width: u32,
        _height: u32,
    ) -> bool {
        false
    }

    unsafe extern "C" fn gui_set_parent(
        plugin: *const clap_plugin,
        window: *const clap_window,
    ) -> bool {
        let window = unsafe { &*window };

        if unsafe { CStr::from_ptr(window.api) } != Self::API {
            return false;
        }

        #[cfg(target_os = "windows")]
        let raw_parent = { RawParent::Win32(unsafe { window.specific.win32 }) };

        #[cfg(target_os = "macos")]
        let raw_parent = { RawParent::Cocoa(unsafe { window.specific.cocoa }) };

        #[cfg(target_os = "linux")]
        let raw_parent = { RawParent::X11(unsafe { window.specific.x11 }) };

        let instance = unsafe { &*(plugin as *const Self) };
        let mut main_thread_state = instance.main_thread_state.borrow();

        let host = EditorHost::from_inner(Rc::new(ClapEditorHost {
            host: instance.host,
            host_params: main_thread_state.host_params,
            param_map: Arc::clone(&instance.param_map),
            param_gestures: Arc::clone(&instance.param_gestures),
        }));
        let parent = unsafe { ParentWindow::from_raw(raw_parent) };
        let editor = main_thread_state.plugin.editor(host, &parent);
        main_thread_state.editor = Some(ThreadCell::new(editor));

        true
    }

    unsafe extern "C" fn gui_set_transient(
        _plugin: *const clap_plugin,
        _window: *const clap_window,
    ) -> bool {
        false
    }

    unsafe extern "C" fn gui_suggest_title(_plugin: *const clap_plugin, _title: *const c_char) {}

    unsafe extern "C" fn gui_show(_plugin: *const clap_plugin) -> bool {
        false
    }

    unsafe extern "C" fn gui_hide(_plugin: *const clap_plugin) -> bool {
        false
    }
}
