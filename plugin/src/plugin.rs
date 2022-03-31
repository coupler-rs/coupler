use crate::{bus::*, editor::*, param::*, process::*};

#[derive(Clone)]
pub struct PluginInfo {
    pub name: String,
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub has_editor: bool,
}

pub trait Plugin: Send + Sync + Sized + 'static {
    type Processor: Processor<Plugin = Self>;
    type Editor: Editor<Plugin = Self>;

    fn info() -> PluginInfo;
    fn buses() -> BusList;
    fn supports_layout(inputs: &[BusLayout], outputs: &[BusLayout]) -> bool;
    fn params() -> ParamList;
    fn create() -> Self;
    fn serialize(
        &self,
        context: &SerializeContext,
        write: &mut impl std::io::Write,
    ) -> Result<(), ()>;
    fn deserialize(
        &self,
        context: &DeserializeContext,
        read: &mut impl std::io::Read,
    ) -> Result<(), ()>;
}

pub struct SerializeContext<'a> {
    param_states: &'a ParamStates,
}

impl<'a> SerializeContext<'a> {
    pub fn new(param_states: &'a ParamStates) -> SerializeContext<'a> {
        SerializeContext { param_states }
    }

    #[inline]
    pub fn get_param<P: Param + 'static>(&self, key: ParamKey<P>) -> P::Value {
        self.param_states.get_param(key)
    }
}

pub struct DeserializeContext<'a> {
    param_states: &'a ParamStates,
}

impl<'a> DeserializeContext<'a> {
    pub fn new(param_states: &'a ParamStates) -> DeserializeContext<'a> {
        DeserializeContext { param_states }
    }

    #[inline]
    pub fn get_param<P: Param + 'static>(&self, key: ParamKey<P>) -> P::Value {
        self.param_states.get_param(key)
    }

    #[inline]
    pub fn set_param<P: Param + 'static>(&self, key: ParamKey<P>, value: P::Value) {
        self.param_states.set_param(key, value);
    }
}
