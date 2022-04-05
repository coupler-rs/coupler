#[derive(Eq, PartialEq, Clone)]
pub enum BusLayout {
    Stereo,
}

impl BusLayout {
    pub fn channels(&self) -> usize {
        match self {
            BusLayout::Stereo => 2,
        }
    }
}

pub struct BusInfo {
    pub name: String,
    pub default_layout: BusLayout,
}

pub struct BusList {
    pub(crate) inputs: Vec<BusInfo>,
    pub(crate) outputs: Vec<BusInfo>,
}

impl BusList {
    pub fn new() -> BusList {
        BusList { inputs: Vec::new(), outputs: Vec::new() }
    }

    pub fn input(mut self, name: &str, default_layout: BusLayout) -> BusList {
        self.inputs.push(BusInfo { name: name.to_string(), default_layout });
        self
    }

    pub fn output(mut self, name: &str, default_layout: BusLayout) -> BusList {
        self.outputs.push(BusInfo { name: name.to_string(), default_layout });
        self
    }
}
