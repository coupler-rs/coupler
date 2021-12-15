use graphics::{Canvas, Color, Path, Vec2};
use plugin::*;
use window::{
    Application, Cursor, MouseButton, Parent, Point, Rect, Window, WindowHandler, WindowOptions,
};

use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

const GAIN: u32 = 0;

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
    type Processor = GainProcessor;
    type Editor = GainEditor;

    fn info() -> PluginInfo {
        PluginInfo {
            name: "gain".to_string(),
            vendor: "glowcoil".to_string(),
            url: "https://glowcoil.com".to_string(),
            email: "micah@glowcoil.com".to_string(),
            has_editor: true,
        }
    }

    fn describe_buses(inputs: &mut BusDescs, outputs: &mut BusDescs) {
        inputs.add(BusDesc { name: "Input".to_string(), default_layout: BusLayout::Stereo });
        outputs.add(BusDesc { name: "Output".to_string(), default_layout: BusLayout::Stereo });
    }

    fn create() -> Gain {
        Gain { params: Arc::new(GainParams::new()) }
    }

    fn processor(&self, _context: &ProcessContext) -> GainProcessor {
        GainProcessor { params: self.params.clone(), gain: 0.0 }
    }

    fn editor(&self, context: EditorContext, parent: Option<&ParentWindow>) -> GainEditor {
        GainEditor::open(context, parent, self.params.clone())
    }

    fn describe_params(&self, params: &mut ParamDescs) {
        params.add(ParamDesc {
            id: GAIN,
            name: "gain".to_string(),
            label: "dB".to_string(),
            steps: None,
            default: 1.0,
        });
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

    fn display_param(&self, id: ParamId, value: f64, write: &mut impl std::fmt::Write) {
        match id {
            0 => {
                let _ = write!(write, "{}", value);
            }
            _ => {}
        }
    }

    fn parse_param(&self, id: ParamId, string: &str) -> Result<f64, ()> {
        match id {
            0 => string.parse().map_err(|_| ()),
            _ => Err(()),
        }
    }

    fn normalize_param(&self, id: ParamId, value: f64) -> f64 {
        match id {
            0 => value,
            _ => 0.0,
        }
    }

    fn denormalize_param(&self, id: ParamId, value: f64) -> f64 {
        match id {
            0 => value,
            _ => 0.0,
        }
    }

    fn serialize(&self, write: &mut impl std::io::Write) -> Result<(), ()> {
        let gain = self.params.gain.load(Ordering::Relaxed);
        write.write(&gain.to_le_bytes()).map(|_| ()).map_err(|_| ())
    }

    fn deserialize(&self, read: &mut impl std::io::Read) -> Result<(), ()> {
        let mut buf = [0; std::mem::size_of::<u64>()];
        if read.read(&mut buf).is_ok() {
            self.params.gain.store(u64::from_le_bytes(buf), Ordering::Relaxed);
            Ok(())
        } else {
            Err(())
        }
    }

    fn supports_layout(inputs: &[BusLayout], outputs: &[BusLayout]) -> bool {
        inputs[0] == BusLayout::Stereo && outputs[0] == BusLayout::Stereo
    }
}

pub struct GainProcessor {
    params: Arc<GainParams>,
    gain: f32,
}

impl Processor for GainProcessor {
    fn process(
        &mut self,
        _context: &ProcessContext,
        buffers: &mut AudioBuffers,
        param_changes: &[ParamChange],
    ) {
        let mut gain = f64::from_bits(self.params.gain.load(Ordering::Relaxed)) as f32;

        for change in param_changes {
            match change.id {
                0 => {
                    gain = change.value as f32;
                }
                _ => {}
            }
        }

        for i in 0..buffers.samples() {
            self.gain = 0.9995 * self.gain + 0.0005 * gain;

            for channel in 0..2 {
                let input_sample = buffers.inputs().bus(0).unwrap().channel(channel).unwrap()[i];
                buffers.outputs().bus_mut(0).unwrap().channel_mut(channel).unwrap()[i] =
                    self.gain * input_sample;
            }
        }

        self.params.gain.store((gain as f64).to_bits(), Ordering::Relaxed);
    }

    fn reset(&mut self, _context: &ProcessContext) {}
}

pub struct GainEditor {
    #[allow(unused)]
    application: Application,
    window: Window,
}

impl GainEditor {
    fn open(
        context: EditorContext,
        parent: Option<&ParentWindow>,
        params: Arc<GainParams>,
    ) -> GainEditor {
        let parent =
            if let Some(parent) = parent { Parent::Parent(parent) } else { Parent::Detached };

        let application = Application::new().unwrap();

        let window = Window::open(
            &application,
            WindowOptions {
                rect: Rect { x: 0.0, y: 0.0, width: 512.0, height: 512.0 },
                parent,
                handler: Box::new(GainWindowHandler::new(context, params)),
                ..WindowOptions::default()
            },
        )
        .unwrap();

        GainEditor { application, window }
    }
}

impl Editor for GainEditor {
    fn initial_size() -> (f64, f64) {
        (256.0, 256.0)
    }

    fn close(&mut self) {
        self.window.close().unwrap();
    }

    fn raw_window_handle(&self) -> Option<RawWindowHandle> {
        Some(self.window.raw_window_handle())
    }

    #[cfg(target_os = "linux")]
    fn poll(&mut self) {
        self.application.poll();
    }

    #[cfg(target_os = "linux")]
    fn file_descriptor(&self) -> Option<std::os::raw::c_int> {
        Some(self.application.file_descriptor())
    }
}

struct GainWindowHandler {
    context: EditorContext,
    params: Arc<GainParams>,
    canvas: RefCell<Canvas>,
    mouse: Cell<Point>,
    down: Cell<Option<(Point, f64)>>,
}

impl GainWindowHandler {
    fn new(context: EditorContext, params: Arc<GainParams>) -> GainWindowHandler {
        GainWindowHandler {
            context,
            params,
            canvas: RefCell::new(Canvas::with_size(256, 256)),
            mouse: Cell::new(Point { x: -1.0, y: -1.0 }),
            down: Cell::new(None),
        }
    }

    fn update_cursor(&self, window: &Window) {
        let position = self.mouse.get();
        if position.x >= 96.0 && position.x < 160.0 && position.y >= 96.0 && position.y < 160.0 {
            window.set_cursor(Cursor::SizeNs);
        } else {
            window.set_cursor(Cursor::Arrow);
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

        let value = f64::from_bits(self.params.gain.load(Ordering::Relaxed)) as f32;

        let center = Vec2::new(128.0, 128.0);
        let radius = 32.0;
        let angle1 = 0.75 * std::f32::consts::PI;
        let angle2 = angle1 + value * 1.5 * std::f32::consts::PI;
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
        let angle = 0.75 * std::f32::consts::PI;
        let span = 1.5 * std::f32::consts::PI;
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
        self.mouse.set(position);
        if let Some((start_position, start_value)) = self.down.get() {
            let new_value =
                (start_value - 0.005 * (position.y - start_position.y)).max(0.0).min(1.0);
            self.params.gain.store(new_value.to_bits(), Ordering::Relaxed);
            self.context.perform_edit(GAIN, new_value);
        } else {
            self.update_cursor(window);
        }
    }

    fn mouse_down(&self, window: &Window, button: MouseButton) -> bool {
        if button == MouseButton::Left {
            let position = self.mouse.get();
            if position.x >= 96.0 && position.x < 160.0 && position.y >= 96.0 && position.y < 160.0
            {
                window.set_cursor(Cursor::SizeNs);
                self.context.begin_edit(GAIN);
                let value = f64::from_bits(self.params.gain.load(Ordering::Relaxed));
                self.context.perform_edit(GAIN, value);
                self.down.set(Some((position, value)));
                return true;
            }
        }

        false
    }

    fn mouse_up(&self, window: &Window, button: MouseButton) -> bool {
        if button == MouseButton::Left {
            if self.down.get().is_some() {
                self.context.end_edit(GAIN);
                self.down.set(None);
                self.update_cursor(window);
                return true;
            }
        }

        false
    }
}
