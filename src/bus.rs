#[derive(Copy, Clone, Eq, PartialEq)]
pub enum BusDir {
    In,
    Out,
    InOut,
}

pub struct BusInfo {
    pub name: String,
    pub dir: BusDir,
}

#[derive(Clone, Default, Eq, PartialEq, Hash)]
pub struct BusConfig {
    pub layouts: Vec<Layout>,
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
