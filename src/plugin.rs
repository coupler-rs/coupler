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

pub struct PluginHandle<P> {
    plugin: Arc<P>,
}

impl<P> Clone for PluginHandle<P> {
    fn clone(&self) -> PluginHandle<P> {
        PluginHandle { plugin: self.plugin.clone() }
    }
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
