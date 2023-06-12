use std::cell::{Cell, RefCell};

use coupler::atomic::AtomicF64;
use coupler::format::{clap::*, vst3::*};
use coupler::{buffer::*, bus::*, editor::*, param::*, plugin::*, process::*};
use flicker::{Canvas, Color, Path, Vec2};
use portlight::{
    Application, Cursor, MouseButton, Parent, Point, Rect, Window, WindowHandler, WindowOptions,
};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

const GAIN: ParamId = 0;

struct GainAccessor;

impl Access<Gain> for GainAccessor {
    fn get(&self, target: &Gain) -> f64 {
        target.gain.load()
    }

    fn set(&self, target: &Gain, value: f64) {
        target.gain.store(value);
    }
}

pub struct Gain {
    gain: AtomicF64,
}

impl Plugin for Gain {
    type Processor = GainProcessor;
    type Editor = GainEditor;

    fn info() -> PluginInfo {
        PluginInfo::new()
            .name("Gain")
            .vendor("Photophore Systems")
            .url("https://photophore.systems")
            .email("support@photophore.systems")
            .has_editor(true)
    }

    fn buses() -> BusList {
        BusList::new()
            .input(BusInfo::new("Input"))
            .output(BusInfo::new("Output"))
    }

    fn bus_configs() -> BusConfigList {
        BusConfigList::new().default(
            BusConfig::new()
                .input(BusFormat::Stereo)
                .output(BusFormat::Stereo),
        )
    }

    fn create() -> Gain {
        Gain {
            gain: AtomicF64::new(0.0),
        }
    }

    fn params(&self) -> ParamList<Self> {
        ParamList::new().param(ParamInfo::new(GAIN).name("Gain").accessor(GainAccessor))
    }

    fn serialize(&self, write: &mut impl std::io::Write) -> Result<(), ()> {
        write
            .write(&self.gain.load().to_le_bytes())
            .map(|_| ())
            .map_err(|_| ())
    }

    fn deserialize(&self, read: &mut impl std::io::Read) -> Result<(), ()> {
        let mut buf = [0; std::mem::size_of::<u64>()];
        if read.read(&mut buf).is_ok() {
            self.gain.store(f64::from_le_bytes(buf));
            Ok(())
        } else {
            Err(())
        }
    }
}

impl ClapPlugin for Gain {
    fn clap_info() -> ClapInfo {
        ClapInfo::with_id("systems.photophore.gain")
    }
}

impl Vst3Plugin for Gain {
    fn vst3_info() -> Vst3Info {
        Vst3Info::with_class_id(Uid::new(0x84B4DD04, 0x0D964565, 0x97AC3AAA, 0x87C5CCA7))
    }
}

pub struct GainProcessor {
    plugin: PluginHandle<Gain>,
    gain: f32,
    gain_target: f32,
}

impl Processor<Gain> for GainProcessor {
    fn create(plugin: PluginHandle<Gain>, _context: &ProcessContext) -> Self {
        GainProcessor {
            plugin: plugin.clone(),
            gain: plugin.gain.load() as f32,
            gain_target: plugin.gain.load() as f32,
        }
    }

    fn reset(&mut self, _context: &ProcessContext) {
        self.gain = self.plugin.gain.load() as f32;
        self.gain_target = self.gain;
    }

    fn process(&mut self, _context: &ProcessContext, buffers: Buffers, events: &[Event]) {
        for (buffers, events) in buffers.split_at_events(events) {
            for event in events {
                match event.event {
                    EventType::ParamChange(change) => match change.id {
                        GAIN => {
                            self.gain_target = change.value as f32;
                        }
                        _ => {}
                    },
                }
            }

            let samples = buffers.samples();

            let (inputs, mut outputs) = buffers.split();
            let [input] = inputs.all_buses().unwrap();
            let [mut output] = outputs.all_buses().unwrap();

            let [input_l, input_r] = input.all_channels().unwrap();
            let [output_l, output_r] = output.all_channels().unwrap();

            for i in 0..samples {
                self.gain = 0.9995 * self.gain + 0.0005 * self.gain_target as f32;

                output_l[i] = self.gain * input_l[i];
                output_r[i] = self.gain * input_r[i];
            }
        }
    }
}

pub struct GainEditor {
    #[allow(unused)]
    application: Application,
    window: Window,
}

impl Editor<Gain> for GainEditor {
    fn open(
        plugin: PluginHandle<Gain>,
        context: EditorContext<Gain>,
        parent: Option<&ParentWindow>,
    ) -> Self {
        let parent = if let Some(parent) = parent {
            Parent::Parent(parent)
        } else {
            Parent::Detached
        };

        let application = Application::new().unwrap();

        let window = Window::open(
            &application,
            WindowOptions {
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 256.0,
                    height: 256.0,
                },
                parent,
                handler: Box::new(GainWindowHandler::new(plugin, context)),
                ..WindowOptions::default()
            },
        )
        .unwrap();

        GainEditor {
            application,
            window,
        }
    }

    fn close(&mut self) {}

    fn size() -> (f64, f64) {
        (256.0, 256.0)
    }

    fn raw_window_handle(&self) -> Option<RawWindowHandle> {
        Some(self.window.raw_window_handle())
    }

    #[cfg(target_os = "linux")]
    fn file_descriptor(&self) -> Option<std::os::raw::c_int> {
        Some(self.application.file_descriptor())
    }

    #[cfg(target_os = "linux")]
    fn poll(&mut self) {
        self.application.poll();
    }
}

struct GainWindowHandler {
    plugin: PluginHandle<Gain>,
    context: EditorContext<Gain>,
    canvas: RefCell<Canvas>,
    mouse: Cell<Point>,
    down: Cell<Option<(Point, f32)>>,
}

impl GainWindowHandler {
    fn new(plugin: PluginHandle<Gain>, context: EditorContext<Gain>) -> GainWindowHandler {
        GainWindowHandler {
            plugin,
            context,
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

        let value = self.plugin.gain.load() as f32;

        let center = Vec2::new(128.0, 128.0);
        let radius = 32.0;
        let angle1 = 0.75 * std::f32::consts::PI;
        let angle2 = angle1 + value * 1.5 * std::f32::consts::PI;
        let mut path = Path::new();
        path.move_to(center + radius * Vec2::new(angle1.cos(), angle1.sin()));
        path.arc(radius, angle1, angle2);
        path.line_to(center + (radius - 4.0) * Vec2::new(angle2.cos(), angle2.sin()));
        path.arc(radius - 4.0, angle2, angle1);
        path.close();
        canvas.fill_path(&path, Color::rgba(240, 240, 245, 255));

        let center = Vec2::new(128.0, 128.0);
        let radius = 32.0;
        let angle = 0.75 * std::f32::consts::PI;
        let span = 1.5 * std::f32::consts::PI;
        let mut path = Path::new();
        path.move_to(center + radius * Vec2::new(angle.cos(), angle.sin()));
        path.arc(radius, angle, angle + span);
        path.line_to(center + (radius - 4.0) * Vec2::new(-angle.cos(), angle.sin()));
        path.arc(radius - 4.0, angle + span, angle);
        path.close();
        canvas.stroke_path(&path, 1.0, Color::rgba(240, 240, 245, 255));

        window.update_contents(canvas.data(), 256, 256);
    }

    fn mouse_move(&self, window: &Window, position: Point) {
        self.mouse.set(position);
        if let Some((start_position, start_value)) = self.down.get() {
            let new_value = (start_value - 0.005 * (position.y - start_position.y) as f32)
                .max(0.0)
                .min(1.0);
            self.context.perform_edit(GAIN, new_value as f64);
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
                let value = self.plugin.gain.load() as f32;
                self.context.perform_edit(GAIN, value as f64);
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
