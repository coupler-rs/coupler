use plugin::{Param, Params, Plugin, PluginInfo};

pub struct TestPlugin {
    gain: f32,
}

const GAIN: Param = Param { id: 0, name: "gain", label: "dB" };

impl Plugin for TestPlugin {
    const INFO: PluginInfo =
        PluginInfo { name: "Test Plugin", vendor: "Test Vendor", unique_id: *b"asdf" };

    const PARAMS: &'static [&'static Param] = &[&GAIN];

    fn new() -> Self {
        TestPlugin { gain: 0.0 }
    }

    fn process(&mut self, params: &Params, inputs: &[&[f32]], outputs: &mut [&mut [f32]]) {
        let gain = params.get(&GAIN) as f32;
        for (input, output) in inputs.iter().zip(outputs.iter_mut()) {
            for (input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
                self.gain = 0.9995 * self.gain + 0.0005 * gain;
                *output_sample = self.gain * *input_sample;
            }
        }
    }
}