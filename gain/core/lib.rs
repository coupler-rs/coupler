use graphics::{Canvas, Color, Path, Vec2};
use plugin::*;
use window::{
    Application, Cursor, MouseButton, Parent, Point, Rect, Window, WindowHandler, WindowOptions,
};

use std::cell::{Cell, RefCell};
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

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
    gain: Arc<AtomicU64>,
}

pub struct GainProcess {
    gain: f32,
}

pub struct GainEditor {
    editor_context: EditorContext,
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

    type ProcessData = GainProcess;
    type EditorData = GainEditor;

    fn create(editor_context: EditorContext) -> (Gain, GainProcess, GainEditor) {
        let plugin = Gain { gain: Arc::new(AtomicU64::new(0.0f64.to_bits())) };
        let processor = GainProcess { gain: 0.0 };
        let editor = GainEditor { editor_context, application: None, window: None };

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

    fn process(
        &self,
        process_data: &mut GainProcess,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        param_changes: &[ParamChange],
    ) {
        let mut gain = f64::from_bits(self.gain.load(Ordering::Relaxed)) as f32;

        for change in param_changes {
            match change.id {
                0 => {
                    gain = change.value as f32;
                }
                _ => {}
            }
        }

        for (input, output) in inputs.iter().zip(outputs.iter_mut()) {
            for (input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
                process_data.gain = 0.9995 * process_data.gain + 0.0005 * gain;
                *output_sample = process_data.gain * *input_sample;
            }
        }

        self.gain.store((gain as f64).to_bits(), Ordering::Relaxed);
    }

    fn editor_size(&self, _editor_data: &GainEditor) -> (f64, f64) {
        (256.0, 256.0)
    }

    fn editor_open(&self, editor_data: &mut GainEditor, parent: Option<&ParentWindow>) {
        let parent =
            if let Some(parent) = parent { Parent::Parent(parent) } else { Parent::Detached };

        let application = Application::new().unwrap();

        let window = Window::open(
            &application,
            WindowOptions {
                rect: Rect { x: 0.0, y: 0.0, width: 512.0, height: 512.0 },
                parent,
                handler: Box::new(GainWindowHandler::new(
                    editor_data.editor_context.clone(),
                    self.gain.clone(),
                )),
                ..WindowOptions::default()
            },
        )
        .unwrap();

        editor_data.application = Some(application);
        editor_data.window = Some(window);
    }

    fn editor_close(&self, editor_data: &mut GainEditor) {
        if let Some(window) = &editor_data.window {
            window.close().unwrap();
        }

        editor_data.window = None;
        editor_data.application = None;
    }

    fn editor_poll(&self, editor_data: &mut GainEditor) {
        if let Some(application) = &editor_data.application {
            application.poll();
        }
    }

    fn raw_window_handle(&self, editor_data: &GainEditor) -> Option<RawWindowHandle> {
        if let Some(window) = &editor_data.window {
            Some(window.raw_window_handle())
        } else {
            None
        }
    }

    fn file_descriptor(&self, editor_data: &GainEditor) -> Option<std::os::raw::c_int> {
        if let Some(application) = &editor_data.application {
            application.file_descriptor()
        } else {
            None
        }
    }
}

struct GainWindowHandler {
    editor_context: EditorContext,
    gain: Arc<AtomicU64>,
    canvas: RefCell<Canvas>,
    mouse: Cell<Point>,
    down: Cell<Option<Point>>,
}

impl GainWindowHandler {
    fn new(editor_context: EditorContext, gain: Arc<AtomicU64>) -> GainWindowHandler {
        GainWindowHandler {
            editor_context,
            gain,
            canvas: RefCell::new(Canvas::with_size(256, 256)),
            mouse: Cell::new(Point { x: -1.0, y: -1.0 }),
            down: Cell::new(None),
        }
    }
}

impl WindowHandler for GainWindowHandler {
    fn frame(&self, window: &Window) {
        window.request_display();
    }

    fn display(&self, window: &Window) {
        let mut canvas = self.canvas.borrow_mut();

        canvas.clear(Color::rgba(21, 26, 31, 255));

        let value = f64::from_bits(self.gain.load(Ordering::Relaxed));

        let center = Vec2::new(128.0, 128.0);
        let radius = 32.0;
        let angle1 = 0.75 * std::f64::consts::PI;
        let angle2 = angle1 + value * 1.5 * std::f64::consts::PI;
        let mut path = Path::builder();
        path.move_to(center + radius * Vec2::new(angle1.cos(), angle1.sin()));
        path.arc(radius, angle1, angle2);
        path.line_to(center + (radius - 4.0) * Vec2::new(angle2.cos(), angle2.sin()));
        path.arc(radius - 4.0, angle2, angle1);
        path.close();
        let path = path.build();
        canvas.fill(&path, Color::rgba(240, 240, 245, 255));

        let center = Vec2::new(128.0, 128.0);
        let radius = 32.0;
        let angle = 0.75 * std::f64::consts::PI;
        let span = 1.5 * std::f64::consts::PI;
        let mut path = Path::builder();
        path.move_to(center + radius * Vec2::new(angle.cos(), angle.sin()));
        path.arc(radius, angle, angle + span);
        path.line_to(center + (radius - 4.0) * Vec2::new(-angle.cos(), angle.sin()));
        path.arc(radius - 4.0, angle + span, angle);
        path.close();
        let path = path.build();
        canvas.stroke(&path, 1.0, Color::rgba(240, 240, 245, 255));

        window.update_contents(canvas.data(), 256, 256);
    }

    fn mouse_move(&self, window: &Window, position: Point) {
        if let Some(start_position) = self.down.get() {
            window.set_mouse_position(start_position);

            let value = f64::from_bits(self.gain.load(Ordering::Relaxed));
            let new_value = (value - 0.005 * (position.y - start_position.y)).max(0.0).min(1.0);
            self.gain.store(new_value.to_bits(), Ordering::Relaxed);
            self.editor_context.perform_edit(GAIN.id, new_value);
        } else {
            self.mouse.set(position);

            if position.x >= 96.0 && position.x < 160.0 && position.y >= 96.0 && position.y < 160.0
            {
                window.set_cursor(Cursor::SizeNs);
            } else {
                window.set_cursor(Cursor::Arrow);
            }
        }
    }

    fn mouse_down(&self, window: &Window, button: MouseButton) -> bool {
        if button == MouseButton::Left {
            let position = self.mouse.get();
            if position.x >= 96.0 && position.x < 160.0 && position.y >= 96.0 && position.y < 160.0
            {
                window.set_cursor(Cursor::None);

                self.editor_context.begin_edit(GAIN.id);
                self.down.set(Some(position));
                return true;
            }
        }

        false
    }

    fn mouse_up(&self, _window: &Window, button: MouseButton) -> bool {
        if button == MouseButton::Left {
            if self.down.get().is_some() {
                self.editor_context.end_edit(GAIN.id);
                self.down.set(None);
                return true;
            }
        }

        false
    }
}
