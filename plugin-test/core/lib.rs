use plugin::{Plugin, PluginInfo};

pub struct TestPlugin;

impl Plugin for TestPlugin {
    const INFO: PluginInfo = PluginInfo {
        name: "Test Plugin",
        vendor: "Test Vendor",
        unique_id: *b"asdf",
    };
}
