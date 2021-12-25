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
    inputs: Vec<BusInfo>,
    outputs: Vec<BusInfo>,
}

impl BusList {
    pub fn new() -> BusList {
        BusList { inputs: Vec::new(), outputs: Vec::new() }
    }

    pub fn add_input(mut self, bus: BusInfo) -> BusList {
        self.inputs.push(bus);
        self
    }

    pub fn add_output(mut self, bus: BusInfo) -> BusList {
        self.outputs.push(bus);
        self
    }

    pub fn inputs(&self) -> &[BusInfo] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[BusInfo] {
        &self.outputs
    }
}
