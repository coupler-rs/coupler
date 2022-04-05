use crate::internal::param_states::*;
use crate::{bus::*, editor::*, param::*, process::*};

use std::ops::Deref;
use std::sync::Arc;

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

#[derive(Clone)]
pub struct PluginHandle<P> {
    plugin: Arc<P>,
}

impl<P> PluginHandle<P> {
    pub fn new(plugin: Arc<P>) -> PluginHandle<P> {
        PluginHandle { plugin }
    }
}

impl<P> Deref for PluginHandle<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.plugin
    }
}

pub struct SerializeContext<'a> {
    param_states: &'a ParamStates,
}

impl<'a> SerializeContext<'a> {
    pub fn new(param_states: &'a ParamStates) -> SerializeContext<'a> {
        SerializeContext { param_states }
    }

    pub fn param_list(&self) -> &ParamList {
        &self.param_states.list
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

    pub fn param_list(&self) -> &ParamList {
        &self.param_states.list
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
