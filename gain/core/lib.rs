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

pub struct GainParams {
    gain: AtomicU64,
}

impl GainParams {
    fn new() -> GainParams {
        GainParams { gain: AtomicU64::new(0.0f64.to_bits()) }
    }
}

pub struct Gain {
    params: Arc<GainParams>,
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

    fn create() -> Gain {
        Gain { params: Arc::new(GainParams::new()) }
    }

    fn processor(&self) -> GainProcessor {
        GainProcessor { params: self.params.clone(), gain: 0.0 }
    }

    fn editor(&self, editor_context: EditorContext) -> GainEditor {
        GainEditor { editor_context, params: self.params.clone(), application: None, window: None }
    }

    fn get_param(&self, id: ParamId) -> f64 {
        match id {
            0 => f64::from_bits(self.params.gain.load(Ordering::Relaxed)),
            _ => 0.0,
        }
    }

    fn set_param(&self, id: ParamId, value: f64) {
        match id {
            0 => {
                self.params.gain.store(value.to_bits(), Ordering::Relaxed);
            }
            _ => {}
        }
    }
}

pub struct GainProcessor {
    params: Arc<GainParams>,
    gain: f32,
}

impl Processor for GainProcessor {
    fn process(&mut self, audio_buses: &mut AudioBuses, param_changes: &[ParamChange]) {
        let mut gain = f64::from_bits(self.params.gain.load(Ordering::Relaxed)) as f32;

        for change in param_changes {
            match change.id {
                0 => {
                    gain = change.value as f32;
                }
                _ => {}
            }
        }

        for i in 0..audio_buses.frames() {
            self.gain = 0.9995 * self.gain + 0.0005 * gain;

            for channel in 0..2 {
                let input_sample = audio_buses.input(0).unwrap().channel(channel).unwrap()[i];
                audio_buses.output(0).unwrap().channel_mut(channel).unwrap()[i] = self.gain * input_sample;
            }
        }

        self.params.gain.store((gain as f64).to_bits(), Ordering::Relaxed);
    }
}

pub struct GainEditor {
    editor_context: EditorContext,
    params: Arc<GainParams>,
    application: Option<Application>,
    window: Option<Window>,
}

impl Editor for GainEditor {
    fn size(&self) -> (f64, f64) {
        (256.0, 256.0)
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
                handler: Box::new(GainWindowHandler::new(
                    self.editor_context.clone(),
                    self.params.clone(),
                )),
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

struct GainWindowHandler {
    editor_context: EditorContext,
    params: Arc<GainParams>,
    canvas: RefCell<Canvas>,
    mouse: Cell<Point>,
    down: Cell<Option<Point>>,
}

impl GainWindowHandler {
    fn new(editor_context: EditorContext, params: Arc<GainParams>) -> GainWindowHandler {
        GainWindowHandler {
            editor_context,
            params,
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

        let value = f64::from_bits(self.params.gain.load(Ordering::Relaxed));

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
        canvas.fill_path(&path, Color::rgba(240, 240, 245, 255));

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
        canvas.stroke_path(&path, 1.0, Color::rgba(240, 240, 245, 255));

        window.update_contents(canvas.data(), 256, 256);
    }

    fn mouse_move(&self, window: &Window, position: Point) {
        if let Some(start_position) = self.down.get() {
            window.set_mouse_position(start_position);

            let value = f64::from_bits(self.params.gain.load(Ordering::Relaxed));
            let new_value = (value - 0.005 * (position.y - start_position.y)).max(0.0).min(1.0);
            self.params.gain.store(new_value.to_bits(), Ordering::Relaxed);
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
