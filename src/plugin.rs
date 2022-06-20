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
    fn create() -> Self;
    fn params(&self) -> ParamList<Self>;
    fn serialize(&self, write: &mut impl std::io::Write) -> Result<(), ()>;
    fn deserialize(&self, read: &mut impl std::io::Read) -> Result<(), ()>;
}

struct PluginState<P> {
    params: ParamList<P>,
    plugin: P,
}

pub struct PluginHandle<P> {
    state: Arc<PluginState<P>>,
}

impl<P> Clone for PluginHandle<P> {
    fn clone(&self) -> PluginHandle<P> {
        PluginHandle { state: self.state.clone() }
    }
}

impl<P: Plugin> PluginHandle<P> {
    pub fn new() -> PluginHandle<P> {
        let plugin = P::create();
        let params = plugin.params();

        PluginHandle { state: Arc::new(PluginState { params, plugin }) }
    }
}

impl<P> PluginHandle<P> {
    pub fn params(&self) -> &ParamList<P> {
        &self.state.params
    }
}

impl<P> Deref for PluginHandle<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.state.plugin
    }
}
