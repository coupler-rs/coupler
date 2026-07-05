use crate::plugin::Plugin;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum BusDir {
    In,
    Out,
    InOut,
}

pub struct BusInfo<'a> {
    pub name: &'a str,
    pub dir: BusDir,
}

#[derive(Clone, Default, Eq, PartialEq, Hash)]
pub struct BusConfig<'a> {
    pub layouts: &'a [Layout],
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Layout {
    Mono,
    Stereo,
}

impl Layout {
    pub fn channel_count(&self) -> usize {
        match self {
            Layout::Mono => 1,
            Layout::Stereo => 2,
        }
    }
}

pub trait BuildBuses {
    fn bus(self, bus: BusInfo) -> Self;
}

pub(crate) struct OwnedBusInfo {
    pub name: String,
    pub dir: BusDir,
}

pub(crate) fn collect_buses<P: Plugin>(plugin: &P) -> Vec<OwnedBusInfo> {
    struct CollectBuses<'a>(&'a mut Vec<OwnedBusInfo>);

    impl<'a> BuildBuses for CollectBuses<'a> {
        fn bus(self, bus: BusInfo) -> Self {
            self.0.push(OwnedBusInfo {
                name: bus.name.to_string(),
                dir: bus.dir,
            });
            self
        }
    }

    let mut buses = Vec::new();
    plugin.buses(CollectBuses(&mut buses));
    buses
}

pub trait BuildBusConfigs {
    fn config(self, config: BusConfig) -> Self;
}

pub(crate) struct OwnedBusConfig {
    pub layouts: Vec<Layout>,
}

pub(crate) fn collect_bus_configs<P: Plugin>(plugin: &P) -> Vec<OwnedBusConfig> {
    struct CollectBusConfigs<'a>(&'a mut Vec<OwnedBusConfig>);

    impl<'a> BuildBusConfigs for CollectBusConfigs<'a> {
        fn config(self, config: BusConfig) -> Self {
            self.0.push(OwnedBusConfig {
                layouts: config.layouts.to_vec(),
            });
            self
        }
    }

    let mut configs = Vec::new();
    plugin.bus_configs(CollectBusConfigs(&mut configs));
    configs
}
