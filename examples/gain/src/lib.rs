use std::{fmt, io};

use serde::{Deserialize, Serialize};

use coupler::buffers::{BufferMut, Buffers};
use coupler::bus::{BusConfig, BusDir, BusInfo, Layout};
use coupler::editor::{EditorHost, NoEditor, ParentWindow, Size};
use coupler::events::{Data, Events};
use coupler::format::clap::{ClapInfo, ClapPlugin};
use coupler::format::vst3::{Uuid, Vst3Info, Vst3Plugin};
use coupler::host::Host;
use coupler::params::{ParamId, ParamInfo, ParamValue, Params};
use coupler::plugin::{Plugin, PluginInfo};
use coupler::process::{Config, Processor};

#[derive(Params, Serialize, Deserialize, Clone)]
struct GainParams {
    #[param(id = 0, name = "Gain", range = 0.0..1.0)]
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
    type Editor = NoEditor;

    fn info() -> PluginInfo {
        PluginInfo {
            name: "Gain".to_string(),
            version: "0.1.0".to_string(),
            vendor: "Vendor".to_string(),
            url: "https://example.com".to_string(),
            email: "example@example.com".to_string(),
        }
    }

    fn new(_host: Host) -> Self {
        Gain {
            params: GainParams::default(),
        }
    }

    fn buses(&self) -> Vec<BusInfo> {
        vec![BusInfo {
            name: "Main".to_string(),
            dir: BusDir::InOut,
        }]
    }

    fn bus_configs(&self) -> Vec<BusConfig> {
        vec![
            BusConfig {
                layouts: vec![Layout::Stereo],
            },
            BusConfig {
                layouts: vec![Layout::Mono],
            },
        ]
    }

    fn params(&self) -> Vec<ParamInfo> {
        GainParams::params()
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
        write: impl fmt::Write,
    ) -> Result<(), fmt::Error> {
        self.params.display_param(id, value, write)
    }

    fn save(&self, output: impl io::Write) -> io::Result<()> {
        serde_json::to_writer(output, &self.params)?;

        Ok(())
    }

    fn load(&mut self, input: impl io::Read) -> io::Result<()> {
        self.params = serde_json::from_reader(input)?;

        Ok(())
    }

    fn processor(&mut self, _config: &Config) -> Self::Processor {
        GainProcessor {
            params: self.params.clone(),
        }
    }

    fn has_editor(&self) -> bool {
        false
    }

    fn editor_size(&self) -> Size {
        Size {
            width: 0.0,
            height: 0.0,
        }
    }

    fn editor(&mut self, _host: EditorHost, _parent: &ParentWindow) -> Self::Editor {
        NoEditor
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

pub struct GainProcessor {
    params: GainParams,
}

impl Processor for GainProcessor {
    fn reset(&mut self) {}

    fn set_param(&mut self, id: ParamId, value: ParamValue) {
        self.params.set_param(id, value);
    }

    fn process(&mut self, buffers: Buffers, events: Events) {
        let mut buffers: (BufferMut,) = buffers.try_into().unwrap();
        for (mut buffer, events) in buffers.0.split_at_events(events) {
            for event in events {
                if let Data::ParamChange { id, value } = event.data {
                    self.set_param(id, value);
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
