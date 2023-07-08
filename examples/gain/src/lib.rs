use std::io::{self, Read, Write};

use coupler::format::vst3::*;
use coupler::{buffers::*, bus::*, param::*, *};

const GAIN: ParamId = 0;

pub struct Gain {
    gain: f64,
}

impl Default for Gain {
    fn default() -> Gain {
        Gain { gain: 1.0 }
    }
}

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
                id: GAIN,
                name: "Gain".to_string(),
                default: 1.0,
                range: Range::Continuous { min: 0.0, max: 1.0 },
                display: Box::new(Float),
            }],
        }
    }

    fn set_param(&mut self, id: ParamId, value: ParamValue) {
        match id {
            GAIN => self.gain = value,
            _ => {}
        }
    }

    fn get_param(&self, id: ParamId) -> ParamValue {
        match id {
            GAIN => self.gain,
            _ => 0.0,
        }
    }

    fn save(&self, output: &mut impl Write) -> io::Result<()> {
        output.write(&self.gain.to_le_bytes())?;

        Ok(())
    }

    fn load(&mut self, input: &mut impl Read) -> io::Result<()> {
        let mut buf = [0; std::mem::size_of::<f64>()];
        input.read_exact(&mut buf)?;
        self.gain = f64::from_le_bytes(buf);

        Ok(())
    }

    fn processor(&self, config: Config) -> Self::Processor {
        GainProcessor {
            gain: self.gain as f32,
        }
    }

    fn editor(&self, context: EditorContext, parent: &ParentWindow) -> GainEditor {
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

pub struct GainProcessor {
    gain: f32,
}

impl Processor for GainProcessor {
    fn set_param(&mut self, id: ParamId, value: ParamValue) {
        match id {
            GAIN => self.gain = value as f32,
            _ => {}
        }
    }

    fn reset(&mut self) {}

    fn process(&mut self, mut buffers: Buffers, events: Events) {
        let (inputs, outputs) = buffers.split();
        let input = inputs.get(0).unwrap();
        let mut output = outputs.get(0).unwrap();

        for i in 0..input.channel_count() {
            for (in_sample, out_sample) in input[i].iter().zip(&mut output[i]) {
                *out_sample = self.gain * in_sample;
            }
        }
    }
}

pub struct GainEditor {}

impl Editor for GainEditor {
    fn size(&self) -> Size {
        Size {}
    }

    fn set_param(&mut self, id: ParamId, value: ParamValue) {}
}
