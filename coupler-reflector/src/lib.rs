use coupler::editor::{Editor, ParentWindow, RawParent, Size};
use coupler::params::{ParamId, ParamValue};
use reflector::platform::{
    App, AppMode, AppOptions, Bitmap, Event, RawWindow, Response, Result, Window, WindowContext,
    WindowOptions,
};

struct State {
    framebuffer: Vec<u32>,
}

impl State {
    fn new() -> State {
        State {
            framebuffer: Vec::new(),
        }
    }

    fn handle_event(&mut self, cx: &WindowContext, event: Event) -> Response {
        match event {
            Event::Frame => {
                let scale = cx.window().scale();
                let size = cx.window().size();
                let width = (size.width * scale) as usize;
                let height = (size.height * scale) as usize;
                self.framebuffer.resize(width * height, 0xFF000000);

                cx.window().present(Bitmap::new(&self.framebuffer, width, height));
            }
            _ => {}
        }

        Response::Ignore
    }
}

pub struct PluginWindow {
    #[allow(unused)]
    app: App,
    window: Window,
}

impl PluginWindow {
    pub fn open(parent: &ParentWindow, size: Size) -> Result<PluginWindow> {
        let app = AppOptions::new().mode(AppMode::Guest).build()?;

        let mut options = WindowOptions::new();
        options.size(reflector::platform::Size::new(size.width, size.height));

        let raw_parent = match parent.as_raw() {
            RawParent::Win32(window) => RawWindow::Win32(window),
            RawParent::Cocoa(view) => RawWindow::Cocoa(view),
            RawParent::X11(window) => RawWindow::X11(window),
        };
        unsafe { options.raw_parent(raw_parent) };

        let mut state = State::new();
        let window = options.open(app.handle(), move |cx, event| state.handle_event(cx, event))?;

        window.show();

        Ok(PluginWindow { app, window })
    }
}

impl Editor for PluginWindow {
    fn size(&self) -> Size {
        let size = self.window.size();

        Size {
            width: size.width,
            height: size.height,
        }
    }

    fn param_changed(&mut self, _id: ParamId, _value: ParamValue) {}
}
