use plugin::{Param, Plugin, PluginInfo};

pub struct TestPlugin;

const GAIN: Param = Param { id: 0, name: "gain", label: "dB" };

impl Plugin for TestPlugin {
    const INFO: PluginInfo =
        PluginInfo { name: "Test Plugin", vendor: "Test Vendor", unique_id: *b"asdf" };

    const PARAMS: &'static [&'static Param] = &[&GAIN];
}
