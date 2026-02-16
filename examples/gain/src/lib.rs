use std::fmt::{self, Formatter};
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::raw::c_int;
use serde::{Deserialize, Serialize};

use coupler::format::clap::*;
use coupler::format::vst3::*;
use coupler::params::{ParamId, ParamValue};
use coupler::view::{ParentWindow, RawParent, Size, View};
use coupler::{buffers::*, bus::*, engine::*, events::*, host::*, params::*, plugin::*, view::*};

use flicker::Renderer;

use portlight::{
    Bitmap, Context, Cursor, EventLoop, EventLoopMode, EventLoopOptions, Key, MouseButton, Point,
    RawWindow, Response, Task, TaskHandle, Window, WindowEvent, WindowOptions,
};

#[derive(Params, Serialize, Deserialize, Clone)]
struct GainParams {
    #[param(id = 0, name = "Gain", range = 0.0..1.0, format = "{:.2}")]
    gain: f32,
}

impl Default for GainParams {
    fn default() -> GainParams {
        GainParams { gain: 1.0 }
    }
}

pub struct Gain {
    params: GainParams,
}

impl Plugin for Gain {
    type Engine = GainEngine;
    type View = GainView;

    fn info() -> PluginInfo {
        PluginInfo {
            name: "Gain".to_string(),
            version: "0.1.0".to_string(),
            vendor: "Vendor".to_string(),
            url: "https://example.com".to_string(),
            email: "example@example.com".to_string(),
            buses: vec![BusInfo {
                name: "Main".to_string(),
                dir: BusDir::InOut,
            }],
            layouts: vec![
                Layout {
                    formats: vec![Format::Stereo],
                },
                Layout {
                    formats: vec![Format::Mono],
                },
            ],
            params: GainParams::params(),
            has_view: true,
        }
    }

    fn new(_host: Host) -> Self {
        Gain {
            params: GainParams::default(),
        }
    }

    fn set_param(&mut self, id: ParamId, value: ParamValue) {
        self.params.set_param(id, value);
    }

    fn get_param(&self, id: ParamId) -> ParamValue {
        self.params.get_param(id)
    }

    fn parse_param(&self, id: ParamId, text: &str) -> Option<ParamValue> {
        self.params.parse_param(id, text)
    }

    fn display_param(
        &self,
        id: ParamId,
        value: ParamValue,
        fmt: &mut Formatter,
    ) -> Result<(), fmt::Error> {
        self.params.display_param(id, value, fmt)
    }

    fn save(&self, output: impl Write) -> io::Result<()> {
        serde_json::to_writer(output, &self.params)?;

        Ok(())
    }

    fn load(&mut self, input: impl Read) -> io::Result<()> {
        self.params = serde_json::from_reader(input)?;

        Ok(())
    }

    fn engine(&mut self, _config: &Config) -> Self::Engine {
        GainEngine {
            params: self.params.clone(),
        }
    }

    fn view(&mut self, host: ViewHost, parent: &ParentWindow) -> Self::View {
        GainView::open(host, parent, &self.params).unwrap()
    }
}

impl Vst3Plugin for Gain {
    fn vst3_info() -> Vst3Info {
        Vst3Info {
            class_id: Uuid::from_name("rs.coupler.gain"),
        }
    }
}

impl ClapPlugin for Gain {
    fn clap_info() -> ClapInfo {
        ClapInfo {
            id: "rs.coupler.gain".to_string(),
        }
    }
}

pub struct GainEngine {
    params: GainParams,
}

impl GainEngine {
    fn handle_event(&mut self, event: &Event) {
        if let Data::ParamChange { id, value } = event.data {
            self.params.set_param(id, value);
        }
    }
}

impl Engine for GainEngine {
    fn reset(&mut self) {}

    fn flush(&mut self, events: Events) {
        for event in events {
            self.handle_event(event);
        }
    }

    fn process(&mut self, buffers: Buffers, events: Events) {
        let mut buffers: (BufferMut,) = buffers.try_into().unwrap();
        for (mut buffer, events) in buffers.0.split_at_events(events) {
            for event in events {
                self.handle_event(event);
            }

            for sample in buffer.samples() {
                for channel in sample {
                    *channel *= self.params.gain;
                }
            }
        }
    }
}

struct Gesture {
    start_mouse_pos: Point,
    start_value: f32,
}

struct ViewState {
    host: ViewHost,
    params: GainParams,
    window: Option<Window>,
    renderer: Renderer,
    framebuffer: Vec<u32>,
    mouse_pos: Point,
    gesture: Option<Gesture>,
}

impl ViewState {
    fn new(host: ViewHost, params: &GainParams) -> ViewState {
        ViewState {
            host,
            params: params.clone(),
            window: None,
            renderer: Renderer::new(),
            framebuffer: Vec::new(),
            mouse_pos: Point { x: -1.0, y: -1.0 },
            gesture: None,
        }
    }

    fn update_cursor(&self, window: &Window) {
        let pos = self.mouse_pos;
        if pos.x >= 96.0 && pos.x < 160.0 && pos.y >= 96.0 && pos.y < 160.0 {
            window.set_cursor(Cursor::SizeNs);
        } else {
            window.set_cursor(Cursor::Arrow);
        }
    }
}

impl Task for ViewState {
    fn event(&mut self, _cx: &Context, _key: Key, event: portlight::Event) -> Response {
        use flicker::{Affine, Color, Path, Point};
        use portlight::Event;

        if let (Some(window), Event::Window(event)) = (&self.window, event) {
            match event {
                WindowEvent::Frame => {
                    let scale = window.scale();
                    let size = window.size();
                    let width = (size.width * scale) as usize;
                    let height = (size.height * scale) as usize;
                    self.framebuffer.resize(width * height, 0xFF000000);

                    let mut target = self.renderer.attach(&mut self.framebuffer, width, height);

                    target.clear(Color::rgba(21, 26, 31, 255));

                    let transform = Affine::scale(scale as f32);

                    let value = self.params.gain;

                    let center = Point::new(128.0, 128.0);
                    let radius = 32.0;
                    let angle1 = 0.75 * std::f32::consts::PI;
                    let angle2 = angle1 + value * 1.5 * std::f32::consts::PI;
                    let mut path = Path::new();
                    path.move_to(center + radius * Point::new(angle1.cos(), angle1.sin()));
                    path.arc(radius, angle1, angle2);
                    path.line_to(center + (radius - 4.0) * Point::new(angle2.cos(), angle2.sin()));
                    path.arc(radius - 4.0, angle2, angle1);
                    path.close();
                    target.fill_path(&path, transform, Color::rgba(240, 240, 245, 255));

                    let center = Point::new(128.0, 128.0);
                    let radius = 32.0;
                    let angle = 0.75 * std::f32::consts::PI;
                    let span = 1.5 * std::f32::consts::PI;
                    let mut path = Path::new();
                    path.move_to(center + radius * Point::new(angle.cos(), angle.sin()));
                    path.arc(radius, angle, angle + span);
                    path.line_to(center + (radius - 4.0) * Point::new(-angle.cos(), angle.sin()));
                    path.arc(radius - 4.0, angle + span, angle);
                    path.close();
                    target.stroke_path(&path, 1.0, transform, Color::rgba(240, 240, 245, 255));

                    window.present(Bitmap::new(&self.framebuffer, width, height));
                }
                WindowEvent::MouseMove(pos) => {
                    self.mouse_pos = pos;
                    if let Some(gesture) = &self.gesture {
                        let delta = -0.005 * (pos.y - gesture.start_mouse_pos.y) as f32;
                        let new_value = (gesture.start_value + delta).clamp(0.0, 1.0);
                        self.host.set_param(0, new_value as f64);
                        self.params.gain = new_value;
                    } else {
                        self.update_cursor(window);
                    }
                }
                WindowEvent::MouseDown(button) => {
                    if button == MouseButton::Left {
                        let pos = self.mouse_pos;
                        if pos.x >= 96.0 && pos.x < 160.0 && pos.y >= 96.0 && pos.y < 160.0 {
                            window.set_cursor(Cursor::SizeNs);
                            self.host.begin_gesture(0);
                            let value = self.params.gain;
                            self.host.set_param(0, value as f64);
                            self.params.gain = value;
                            self.gesture = Some(Gesture {
                                start_mouse_pos: pos,
                                start_value: value,
                            });
                            return Response::Capture;
                        }
                    }
                }
                WindowEvent::MouseUp(button) => {
                    if button == MouseButton::Left {
                        if self.gesture.is_some() {
                            self.host.end_gesture(0);
                            self.gesture = None;
                            self.update_cursor(window);
                            return Response::Capture;
                        }
                    }
                }
                _ => {}
            }
        }

        Response::Ignore
    }
}

pub struct GainView {
    #[allow(unused)]
    event_loop: EventLoop,
    task: TaskHandle<ViewState>,
}

impl GainView {
    fn open(
        host: ViewHost,
        parent: &ParentWindow,
        params: &GainParams,
    ) -> portlight::Result<GainView> {
        let event_loop = EventLoopOptions::new().mode(EventLoopMode::Guest).build()?;

        let task = event_loop.spawn(ViewState::new(host, params));

        task.with(|state, cx| {
            let mut options = WindowOptions::new();
            options.size(portlight::Size::new(256.0, 256.0));

            let raw_parent = match parent.as_raw() {
                RawParent::Win32(window) => RawWindow::Win32(window),
                RawParent::Cocoa(view) => RawWindow::AppKit(view),
                RawParent::X11(window) => RawWindow::X11(window),
            };
            unsafe { options.raw_parent(raw_parent) };

            let window = options.open(cx, Key(0))?;
            window.show();

            state.window = Some(window);

            portlight::Result::Ok(())
        })?;

        Ok(GainView { event_loop, task })
    }
}

impl View for GainView {
    fn size(&self) -> Size {
        let size = self.task.with(|state, _| state.window.as_ref().unwrap().size());

        Size {
            width: size.width,
            height: size.height,
        }
    }

    fn param_changed(&mut self, id: ParamId, value: ParamValue) {
        self.task.with(|state, _| {
            state.params.set_param(id, value);
        });
    }

    fn file_descriptor(&self) -> Option<c_int> {
        Some(self.event_loop.as_raw_fd())
    }

    fn poll(&mut self) {
        // todo: is unwrapping really the right choice?
        self.event_loop.poll().unwrap();
    }
}
