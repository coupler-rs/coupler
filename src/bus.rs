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

pub trait BuildBusConfigs {
    fn config(self, config: BusConfig) -> Self;
}
