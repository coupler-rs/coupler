#[derive(Copy, Clone)]
pub enum BusDir {
    In,
    Out,
}

pub struct BusInfo {
    pub name: String,
    pub dir: BusDir,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Layout {
    pub formats: Vec<Format>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Format {
    Mono,
    Stereo,
}

impl Format {
    pub fn channel_count(&self) -> usize {
        match self {
            Format::Mono => 1,
            Format::Stereo => 2,
        }
    }
}
