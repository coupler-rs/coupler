use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize};

use coupler::buffers::bind::*;
use coupler::format::clap::*;
use coupler::format::vst3::*;
use coupler::{bus::*, editor::*, events::*, params::smooth::*, params::*, plugin::*, process::*};

#[derive(Params, Smooth, Serialize, Deserialize, Clone)]
struct GainParams {
    #[param(id = 0, name = "Gain", range = 0.0..1.0, format = "{:.2}")]
    #[smooth(Exp, ms = 10.0)]
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

    fn processor(&self, config: Config) -> Self::Processor {
        GainProcessor {
            params: self.params.smoothed(config.sample_rate),
        }
    }

    fn editor(&self, _parent: Parent) -> GainEditor {
        GainEditor {}
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
    params: Smoothed<GainParams>,
}

impl Processor for GainProcessor {
    fn set_param(&mut self, id: ParamId, value: ParamValue) {
        self.params.set_param(id, value);
    }

    fn reset(&mut self) {}

    fn process(&mut self, block: Block) {
        let mut split = block.split_at_events();
        while let Some(mut block) = split.next() {
            for event in block.events {
                match event.data {
                    Data::ParamChange { id, value } => {
                        self.params.set_param(id, value);
                    }
                    _ => {}
                }
            }

            let InOut(mut main) = block.buffers.bind().unwrap();

            for i in 0..main.len() {
                let gain = self.params.gain.next();
                for channel in 0..main.channel_count() {
                    main[channel][i] *= gain;
                }
            }
        }
    }
}

pub struct GainEditor {}

impl Editor for GainEditor {
    fn size(&self) -> Size {
        Size {
            width: 320.0,
            height: 240.0,
        }
    }

    fn set_param(&mut self, _id: ParamId, _value: ParamValue) {}
}
