use plugin::*;
use window::{Application, Parent, Rect, Window, WindowOptions};

use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

const GAIN: ParamInfo = ParamInfo {
    id: 0,
    name: "gain",
    label: "dB",
    steps: None,
    default: 1.0,
    to_normal: |x| x,
    from_normal: |x| x,
    to_string: |x| x.to_string(),
    from_string: |s| f64::from_str(s).unwrap_or(0.0),
};

pub struct Gain {
    gain: AtomicU64,
}

pub struct GainProcessor {
    gain: f32,
}

pub struct GainEditor {
    application: Option<Application>,
    window: Option<Window>,
}

impl Plugin for Gain {
    const INFO: PluginInfo = PluginInfo {
        name: "gain",
        vendor: "glowcoil",
        url: "https://glowcoil.com",
        email: "micah@glowcoil.com",
        unique_id: *b"asdf",
        uid: [0x84B4DD04, 0x0D964565, 0x97AC3AAA, 0x87C5CCA7],
        has_editor: true,
    };

    const PARAMS: &'static [ParamInfo] = &[GAIN];

    type Processor = GainProcessor;
    type Editor = GainEditor;

    fn create(_editor_context: EditorContext) -> (Gain, GainProcessor, GainEditor) {
        let plugin = Gain { gain: AtomicU64::new(0.0f64.to_bits()) };
        let processor = GainProcessor { gain: 0.0 };
        let editor = GainEditor { application: None, window: None };

        (plugin, processor, editor)
    }

    fn get_param(&self, id: ParamId) -> f64 {
        match id {
            0 => f64::from_bits(self.gain.load(Ordering::Relaxed)),
            _ => 0.0,
        }
    }

    fn set_param(&self, id: ParamId, value: f64) {
        match id {
            0 => {
                self.gain.store(value.to_bits(), Ordering::Relaxed);
            }
            _ => {}
        }
    }
}

impl Processor for GainProcessor {
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        param_changes: &[ParamChange],
    ) {
        for change in param_changes {
            match change.id {
                0 => {
                    self.gain = change.value as f32;
                }
                _ => {}
            }
        }

        for (input, output) in inputs.iter().zip(outputs.iter_mut()) {
            for (input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
                *output_sample = self.gain * *input_sample;
            }
        }
    }
}

impl Editor for GainEditor {
    fn size(&self) -> (f64, f64) {
        (0.0, 0.0)
    }

    fn open(&mut self, parent: Option<&ParentWindow>) {
        let parent =
            if let Some(parent) = parent { Parent::Parent(parent) } else { Parent::Detached };

        let application = Application::new().unwrap();

        let window = Window::open(
            &application,
            WindowOptions {
                rect: Rect { x: 0.0, y: 0.0, width: 512.0, height: 512.0 },
                parent,
                ..WindowOptions::default()
            },
        )
        .unwrap();

        self.application = Some(application);
        self.window = Some(window);
    }

    fn close(&mut self) {
        if let Some(window) = &self.window {
            window.close().unwrap();
        }

        self.window = None;
        self.application = None;
    }

    fn poll(&mut self) {
        if let Some(application) = &self.application {
            application.poll();
        }
    }

    fn raw_window_handle(&self) -> Option<RawWindowHandle> {
        if let Some(window) = &self.window {
            Some(window.raw_window_handle())
        } else {
            None
        }
    }

    fn file_descriptor(&self) -> Option<std::os::raw::c_int> {
        if let Some(application) = &self.application {
            application.file_descriptor()
        } else {
            None
        }
    }
}
