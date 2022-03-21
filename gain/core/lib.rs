use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use graphics::{Canvas, Color, Path, Vec2};
use plugin::{buffer::*, bus::*, editor::*, param::*, plugin::*, process::*};
use window::{
    Application, Cursor, MouseButton, Parent, Point, Rect, Window, WindowHandler, WindowOptions,
};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub struct Gain {
    gain: FloatParam,
}

impl Plugin for Gain {
    type Processor = GainProcessor;
    type Editor = GainEditor;

    const INFO: PluginInfo = PluginInfo {
        name: "Gain",
        vendor: "Photophore Systems",
        url: "https://photophore.systems",
        email: "support@photophore.systems",
        has_editor: true,
    };

    const PARAMS: &'static [ParamKey<Self>] = &[param!(gain)];

    fn buses() -> BusList {
        BusList::new().input("Input", BusLayout::Stereo).output("Output", BusLayout::Stereo)
    }

    fn supports_layout(inputs: &[BusLayout], outputs: &[BusLayout]) -> bool {
        inputs[0] == BusLayout::Stereo && outputs[0] == BusLayout::Stereo
    }

    fn create() -> Gain {
        Gain { gain: FloatParam::new(0, "gain", 1.0) }
    }

    fn serialize(&self, write: &mut impl std::io::Write) -> Result<(), ()> {
        let gain = self.gain.get();
        write.write(&gain.to_le_bytes()).map(|_| ()).map_err(|_| ())
    }

    fn deserialize(&self, read: &mut impl std::io::Read) -> Result<(), ()> {
        let mut buf = [0; std::mem::size_of::<u32>()];
        if read.read(&mut buf).is_ok() {
            self.gain.set(f32::from_le_bytes(buf));
            Ok(())
        } else {
            Err(())
        }
    }
}

pub struct GainProcessor {
    plugin: Arc<Gain>,
    gain: f32,
}

impl Processor for GainProcessor {
    type Plugin = Gain;

    fn create(plugin: &Arc<Self::Plugin>, _context: &ProcessContext) -> Self {
        GainProcessor { plugin: plugin.clone(), gain: plugin.gain.get() }
    }

    fn reset(&mut self, _context: &ProcessContext) {
        self.gain = self.plugin.gain.get();
    }

    fn process(
        &mut self,
        _context: &ProcessContext,
        buffers: &mut Buffers,
        _param_changes: &[ParamChange],
    ) {
        for i in 0..buffers.samples() {
            for channel in 0..2 {
                self.gain = 0.9995 * self.gain + 0.0005 * self.plugin.gain.get();

                buffers.outputs()[0][channel][i] = self.gain * buffers.inputs()[0][channel][i];
            }
        }
    }
}

pub struct GainEditor {
    #[allow(unused)]
    application: Application,
    window: Window,
}

impl Editor for GainEditor {
    type Plugin = Gain;

    fn open(
        plugin: &Arc<Self::Plugin>,
        context: &Rc<dyn EditorContext>,
        parent: Option<&ParentWindow>,
    ) -> Self {
        let parent =
            if let Some(parent) = parent { Parent::Parent(parent) } else { Parent::Detached };

        let application = Application::new().unwrap();

        let window = Window::open(
            &application,
            WindowOptions {
                rect: Rect { x: 0.0, y: 0.0, width: 256.0, height: 256.0 },
                parent,
                handler: Box::new(GainWindowHandler::new(plugin, context)),
                ..WindowOptions::default()
            },
        )
        .unwrap();

        GainEditor { application, window }
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
    plugin: Arc<Gain>,
    context: Rc<dyn EditorContext>,
    canvas: RefCell<Canvas>,
    mouse: Cell<Point>,
    down: Cell<Option<(Point, f32)>>,
}

impl GainWindowHandler {
    fn new(plugin: &Arc<Gain>, context: &Rc<dyn EditorContext>) -> GainWindowHandler {
        GainWindowHandler {
            plugin: plugin.clone(),
            context: context.clone(),
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

        let value = self.plugin.gain.get();

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
            let new_value =
                (start_value - 0.005 * (position.y - start_position.y) as f32).max(0.0).min(1.0);
            self.plugin.gain.perform_edit(&self.context, new_value);
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
                self.plugin.gain.begin_edit(&self.context);
                let value = self.plugin.gain.get();
                self.plugin.gain.perform_edit(&self.context, self.plugin.gain.get());
                self.down.set(Some((position, value)));
                return true;
            }
        }

        false
    }

    fn mouse_up(&self, window: &Window, button: MouseButton) -> bool {
        if button == MouseButton::Left {
            if self.down.get().is_some() {
                self.plugin.gain.end_edit(&self.context);
                self.down.set(None);
                self.update_cursor(window);
                return true;
            }
        }

        false
    }
}
