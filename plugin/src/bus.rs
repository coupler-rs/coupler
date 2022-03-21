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
    name: String,
    default_layout: BusLayout,
}

impl BusInfo {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn default_layout(&self) -> &BusLayout {
        &self.default_layout
    }
}

pub struct BusList {
    inputs: Vec<BusInfo>,
    outputs: Vec<BusInfo>,
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

    pub fn inputs(&self) -> &[BusInfo] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[BusInfo] {
        &self.outputs
    }
}
