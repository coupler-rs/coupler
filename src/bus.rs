pub struct BusInfo {
    pub name: String,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Layout {
    pub inputs: Vec<Format>,
    pub outputs: Vec<Format>,
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
