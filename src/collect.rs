use crate::bus::{BuildBusConfigs, BuildBuses, BusConfig, BusDir, BusInfo, Layout};
use crate::key::{Key, KeyList};
use crate::params::{BuildParams, ParamInfo};
use crate::plugin::Plugin;

pub struct OwnedBusInfo {
    pub name: String,
    pub dir: BusDir,
}

pub fn collect_buses<P: Plugin>(plugin: &P) -> (Vec<u32>, Vec<OwnedBusInfo>) {
    struct CollectBuses<'a> {
        keys: &'a mut KeyList,
        buses: &'a mut Vec<OwnedBusInfo>,
    }

    impl<'a> BuildBuses for CollectBuses<'a> {
        fn bus<'k>(self, key: impl Into<Key<'k>>, bus: BusInfo) -> Self {
            self.keys.key(key);
            self.buses.push(OwnedBusInfo {
                name: bus.name.to_string(),
                dir: bus.dir,
            });
            self
        }

        fn reserve<'k>(self, key: impl Into<Key<'k>>) -> Self {
            self.keys.reserve(key);
            self
        }
    }

    let mut keys = KeyList::new();
    let mut buses = Vec::new();
    plugin.buses(CollectBuses {
        keys: &mut keys,
        buses: &mut buses,
    });

    (keys.into_ids(), buses)
}

pub struct OwnedBusConfig {
    pub layouts: Vec<Layout>,
}

pub fn collect_bus_configs<P: Plugin>(plugin: &P) -> (Vec<u32>, Vec<OwnedBusConfig>) {
    struct CollectBusConfigs<'a> {
        keys: &'a mut KeyList,
        configs: &'a mut Vec<OwnedBusConfig>,
    }

    impl<'a> BuildBusConfigs for CollectBusConfigs<'a> {
        fn config<'k>(self, key: impl Into<Key<'k>>, config: BusConfig) -> Self {
            self.keys.key(key);
            self.configs.push(OwnedBusConfig {
                layouts: config.layouts.to_vec(),
            });
            self
        }

        fn reserve<'k>(self, key: impl Into<Key<'k>>) -> Self {
            self.keys.reserve(key);
            self
        }
    }

    let mut keys = KeyList::new();
    let mut configs = Vec::new();
    plugin.bus_configs(CollectBusConfigs {
        keys: &mut keys,
        configs: &mut configs,
    });

    (keys.into_ids(), configs)
}

pub struct OwnedParamInfo {
    pub name: String,
    pub default: f64,
    pub steps: Option<u32>,
}

pub fn collect_params<P: Plugin>(plugin: &P) -> (Vec<u32>, Vec<OwnedParamInfo>) {
    struct CollectParams<'a> {
        keys: &'a mut KeyList,
        params: &'a mut Vec<OwnedParamInfo>,
    }

    impl<'a> BuildParams for CollectParams<'a> {
        fn param<'k>(self, key: impl Into<Key<'k>>, param: ParamInfo) -> Self {
            self.keys.key(key);
            self.params.push(OwnedParamInfo {
                name: param.name.to_string(),
                default: param.default,
                steps: param.steps,
            });
            self
        }

        fn reserve<'k>(self, key: impl Into<Key<'k>>) -> Self {
            self.keys.reserve(key);
            self
        }
    }

    let mut keys = KeyList::new();
    let mut params = Vec::new();
    plugin.params(CollectParams {
        keys: &mut keys,
        params: &mut params,
    });

    (keys.into_ids(), params)
}
