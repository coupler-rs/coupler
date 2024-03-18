use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize};

use portlight::{App, AppMode, AppOptions, Response, Window, WindowOptions};

use coupler::format::clap::*;
use coupler::format::vst3::*;
use coupler::{buffers::*, bus::*, editor::*, events::*, params::*, plugin::*, process::*};

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
    type Processor = GainProcessor;
    type Editor = GainEditor;

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
            has_editor: true,
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

    fn save(&self, output: &mut impl Write) -> io::Result<()> {
        serde_json::to_writer(output, &self.params)?;

        Ok(())
    }

    fn load(&mut self, input: &mut impl Read) -> io::Result<()> {
        self.params = serde_json::from_reader(input)?;

        Ok(())
    }

    fn processor(&mut self, _config: Config) -> Self::Processor {
        GainProcessor {
            params: self.params.clone(),
        }
    }

    fn editor(&mut self, parent: Parent) -> Self::Editor {
        GainEditor::open(parent).unwrap()
    }
}

impl Vst3Plugin for Gain {
    fn vst3_info() -> Vst3Info {
        Vst3Info {
            class_id: Uuid(0x84B4DD04, 0x0D964565, 0x97AC3AAA, 0x87C5CCA7),
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

pub struct GainProcessor {
    params: GainParams,
}

impl Processor for GainProcessor {
    fn reset(&mut self) {}

    fn process(&mut self, buffers: Buffers, events: Events) {
        let mut buffer = buffers.collect::<BufferMut>().unwrap();
        for (mut buffer, events) in buffer.split_at_events(events) {
            for event in events {
                match event.data {
                    Data::ParamChange { id, value } => {
                        self.params.set_param(id, value);
                    }
                    _ => {}
                }
            }

            for sample in buffer.samples() {
                for channel in sample {
                    *channel *= self.params.gain;
                }
            }
        }
    }
}

pub struct GainEditor {
    _app: App,
    window: Window,
}

impl GainEditor {
    fn open(parent: Parent) -> portlight::Result<GainEditor> {
        let app = AppOptions::new().mode(AppMode::Guest).build()?;

        let mut options = WindowOptions::new();
        options.size(portlight::Size::new(512.0, 512.0));

        let raw_parent = match parent.as_raw() {
            RawParent::Win32(window) => portlight::RawParent::Win32(window),
            RawParent::Cocoa(view) => portlight::RawParent::Cocoa(view),
            RawParent::X11(window) => portlight::RawParent::X11(window),
        };
        unsafe { options.raw_parent(raw_parent) };

        let window = options.open(app.handle(), move |_cx, _event| Response::Ignore)?;

        window.show();

        Ok(GainEditor { _app: app, window })
    }
}

impl Editor for GainEditor {
    fn size(&self) -> Size {
        let size = self.window.size();

        Size {
            width: size.width,
            height: size.height,
        }
    }

    fn param_changed(&mut self, _id: ParamId, _value: ParamValue) {}
}
