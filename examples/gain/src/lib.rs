use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize};

use coupler::format::clap::*;
use coupler::format::vst3::*;
use coupler::{buffers::*, bus::*, engine::*, events::*, host::*, params::*, plugin::*, view::*};

use coupler_reflector::PluginWindow;

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
    type View = PluginWindow;

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

    fn save(&self, output: &mut impl Write) -> io::Result<()> {
        serde_json::to_writer(output, &self.params)?;

        Ok(())
    }

    fn load(&mut self, input: &mut impl Read) -> io::Result<()> {
        self.params = serde_json::from_reader(input)?;

        Ok(())
    }

    fn engine(&mut self, _config: Config) -> Self::Engine {
        GainEngine {
            params: self.params.clone(),
        }
    }

    fn view(&mut self, _host: ViewHost, parent: &ParentWindow) -> Self::View {
        let size = Size {
            width: 512.0,
            height: 512.0,
        };

        PluginWindow::open(parent, size).unwrap()
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
