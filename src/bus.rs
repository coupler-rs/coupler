pub struct BusInfo {
    pub name: String,
}

pub struct Layout {
    pub inputs: Vec<Format>,
    pub outputs: Vec<Format>,
}

pub enum Format {
    Mono,
    Stereo,
}
