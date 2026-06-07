use std::{fmt, io};

use serde::{Deserialize, Serialize};

use coupler::buffers::{BufferMut, Buffers};
use coupler::bus::{BuildBusConfigs, BuildBuses, BusConfig, BusDir, BusInfo, Layout};
use coupler::editor::NoEditor;
use coupler::events::{Data, Events};
use coupler::format::clap::{BuildClapInfo, ClapInfo, ClapPlugin};
use coupler::format::vst3::{BuildVst3Info, Uuid, Vst3Info, Vst3Plugin};
use coupler::host::Host;
use coupler::params::{BuildParams, Params};
use coupler::plugin::{BuildInfo, Plugin, PluginInfo};
use coupler::process::{Config, Processor};

#[derive(Params, Serialize, Deserialize, Clone)]
struct GainParams {
    #[param(id = 0, name = "Gain", range = 0.0..1.0, default = 1.0)]
    gain: f32,
}

pub struct Gain {
    params: GainParams,
}

impl Plugin for Gain {
    type Processor = GainProcessor;
    type Editor = NoEditor;

    fn info(build: impl BuildInfo) {
        build.info(PluginInfo {
            name: "Gain",
            version: "0.1.0",
            vendor: "Vendor",
            url: "https://example.com",
            email: "example@example.com",
        })
    }

    fn new(_host: Host) -> Self {
        Gain {
            params: GainParams::default(),
        }
    }

    fn buses(&self, build: impl BuildBuses) {
        build.bus(BusInfo {
            name: "Main",
            dir: BusDir::InOut,
        });
    }

    fn bus_configs(&self, build: impl BuildBusConfigs) {
        build
            .config(BusConfig {
                layouts: &[Layout::Stereo],
            })
            .config(BusConfig {
                layouts: &[Layout::Mono],
            });
    }

    fn params(&self, build: impl BuildParams) {
        self.params.params(build)
    }

    fn set_param(&mut self, index: usize, value: f64) {
        self.params.set_param(index, value);
    }

    fn get_param(&self, index: usize) -> f64 {
        self.params.get_param(index)
    }

    fn parse_param(&self, index: usize, text: &str) -> Option<f64> {
        self.params.parse_param(index, text)
    }

    fn display_param(
        &self,
        index: usize,
        value: f64,
        write: impl fmt::Write,
    ) -> Result<(), fmt::Error> {
        self.params.display_param(index, value, write)
    }

    fn save(&self, output: impl io::Write) -> io::Result<()> {
        serde_json::to_writer(output, &self.params)?;

        Ok(())
    }

    fn load(&mut self, input: impl io::Read) -> io::Result<()> {
        self.params = serde_json::from_reader(input)?;

        Ok(())
    }

    fn processor(&mut self, _config: Config) -> Self::Processor {
        GainProcessor {
            params: self.params.clone(),
        }
    }
}

impl Vst3Plugin for Gain {
    fn vst3_info(build: impl BuildVst3Info) {
        build.info(Vst3Info {
            class_id: Uuid::from_name("rs.coupler.gain"),
        })
    }
}

impl ClapPlugin for Gain {
    fn clap_info(build: impl BuildClapInfo) {
        build.info(ClapInfo {
            id: "rs.coupler.gain",
        })
    }
}

pub struct GainProcessor {
    params: GainParams,
}

impl Processor for GainProcessor {
    fn set_param(&mut self, index: usize, value: f64) {
        self.params.set_param(index, value);
    }

    fn process(&mut self, buffers: Buffers, events: Events) {
        let mut buffers: (BufferMut,) = buffers.try_into().unwrap();
        for (mut buffer, events) in buffers.0.split_at_events(events) {
            for event in events {
                if let Data::ParamChange { index, value } = event.data {
                    self.set_param(index, value);
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
