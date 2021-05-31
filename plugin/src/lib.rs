pub mod vst2;

pub struct PluginInfo {
    pub name: &'static str,
    pub vendor: &'static str,
    pub unique_id: [u8; 4],
}

pub struct Param {
    pub id: usize,
    pub name: &'static str,
    pub label: &'static str,
}

pub trait Plugin: Send + Sync {
    const INFO: PluginInfo;
    const PARAMS: &'static [&'static Param];
}
