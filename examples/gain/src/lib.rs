use std::io::{self, Read, Write};

use coupler::format::vst3::*;
use coupler::{bus::*, param::*, process::*, *};

pub struct Gain {}

impl Plugin for Gain {
    type Processor = GainProcessor;
    type Editor = GainEditor;

    fn info() -> PluginInfo {
        PluginInfo {
            name: "Gain".to_string(),
            vendor: "Vendor".to_string(),
            url: "https://example.com".to_string(),
            email: "example@example.com".to_string(),
            inputs: vec![BusInfo {
                name: "Input".to_string(),
            }],
            outputs: vec![BusInfo {
                name: "Output".to_string(),
            }],
            layouts: vec![
                Layout {
                    inputs: vec![Format::Stereo],
                    outputs: vec![Format::Stereo],
                },
                Layout {
                    inputs: vec![Format::Mono],
                    outputs: vec![Format::Mono],
                },
            ],
            params: vec![ParamInfo {
                id: 0,
                name: "Gain".to_string(),
                default: 1.0,
                range: Range::Continuous { min: 0.0, max: 1.0 },
                display: Box::new(Float),
            }],
        }
    }

    fn create() -> Self {
        Gain {}
    }

    fn set_param(&self, id: ParamId, value: ParamValue) {}

    fn get_param(&self, id: ParamId) -> ParamValue {
        0.0
    }

    fn save(&self, output: &mut impl Write) -> io::Result<()> {
        Ok(())
    }

    fn load(&self, input: &mut impl Read) -> io::Result<()> {
        Ok(())
    }
}

impl Vst3Plugin for Gain {
    fn vst3_info() -> Vst3Info {
        Vst3Info {
            class_id: Uuid(0x84B4DD04, 0x0D964565, 0x97AC3AAA, 0x87C5CCA7),
        }
    }
}

pub struct GainProcessor {}

impl Processor<Gain> for GainProcessor {
    fn create(plugin: &Gain, config: Config) -> Self {
        GainProcessor {}
    }

    fn info(&self) -> ProcessInfo {
        ProcessInfo { latency: 0 }
    }

    fn set_param(&mut self, id: ParamId, value: ParamValue) {}

    fn reset(&mut self) {}

    fn process(&mut self, buffers: Buffers, events: Events) {}
}

pub struct GainEditor {}

impl Editor<Gain> for GainEditor {
    fn create(plugin: &Gain, context: EditorContext, parent: &ParentWindow) -> Self {
        GainEditor {}
    }

    fn size(&self) -> Size {
        Size {}
    }

    fn set_param(&mut self, id: ParamId, value: ParamValue) {}
}
