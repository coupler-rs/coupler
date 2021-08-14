use plugin::{Editor, EditorContext, ParamInfo, ParamValues, ParentWindow, Plugin, PluginInfo};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use window::{Application, Parent, Rect, Window, WindowOptions};

use std::rc::Rc;
use std::str::FromStr;

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
    gain: f32,
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

    type Editor = GainEditor;

    fn create(_editor_context: Rc<dyn EditorContext>) -> (Gain, GainEditor) {
        let gain = Gain { gain: 0.0 };
        let gain_editor = GainEditor { application: Application::new().unwrap(), window: None };

        (gain, gain_editor)
    }

    fn process(&mut self, params: &ParamValues, inputs: &[&[f32]], outputs: &mut [&mut [f32]]) {
        let gain = params.get(&GAIN) as f32;
        for (input, output) in inputs.iter().zip(outputs.iter_mut()) {
            for (input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
                self.gain = 0.9995 * self.gain + 0.0005 * gain;
                *output_sample = self.gain * *input_sample;
            }
        }
    }
}

pub struct GainEditor {
    application: Application,
    window: Option<Window>,
}

impl Editor for GainEditor {
    fn size(&self) -> (f64, f64) {
        (512.0, 512.0)
    }

    fn open(&mut self, parent: Option<&ParentWindow>) {
        let parent =
            if let Some(parent) = parent { Parent::Parent(parent) } else { Parent::Detached };

        let window = Window::open(
            &self.application,
            WindowOptions {
                rect: Rect { x: 0.0, y: 0.0, width: 512.0, height: 512.0 },
                parent,
                ..WindowOptions::default()
            },
        )
        .unwrap();

        self.window = Some(window);
    }

    fn close(&mut self) {
        if let Some(window) = self.window.take() {
            let _ = window.close();
        }
    }

    fn poll(&mut self) {
        if self.window.is_some() {
            self.application.poll();
        }
    }

    fn raw_window_handle(&self) -> Option<RawWindowHandle> {
        if let Some(ref window) = self.window {
            Some(window.raw_window_handle())
        } else {
            None
        }
    }

    fn file_descriptor(&self) -> Option<std::os::raw::c_int> {
        self.application.file_descriptor()
    }
}
