use crate::{bus::*, editor::*, param::*, process::*};

pub struct PluginInfo {
    pub name: &'static str,
    pub vendor: &'static str,
    pub url: &'static str,
    pub email: &'static str,
    pub has_editor: bool,
}

pub trait Plugin: Send + Sync + Sized + 'static {
    type Processor: Processor<Plugin = Self>;
    type Editor: Editor<Plugin = Self>;

    const INFO: PluginInfo;
    const INPUTS: &'static [BusInfo];
    const OUTPUTS: &'static [BusInfo];
    const PARAMS: &'static [ParamKey<Self>];

    fn create() -> Self;
    fn supports_layout(inputs: &[BusLayout], outputs: &[BusLayout]) -> bool;
    fn serialize(&self, write: &mut impl std::io::Write) -> Result<(), ()>;
    fn deserialize(&self, read: &mut impl std::io::Read) -> Result<(), ()>;
}
