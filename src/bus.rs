#[derive(Eq, PartialEq, Clone)]
pub enum BusLayout {
    Stereo,
}

impl BusLayout {
    #[inline]
    pub fn channels(&self) -> usize {
        match self {
            BusLayout::Stereo => 2,
        }
    }
}

pub struct BusState {
    layout: BusLayout,
    enabled: bool,
}

impl BusState {
    #[inline]
    pub fn new(layout: BusLayout, enabled: bool) -> BusState {
        BusState { layout, enabled }
    }

    #[inline]
    pub fn layout(&self) -> &BusLayout {
        &self.layout
    }

    #[inline]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    #[inline]
    pub fn set_layout(&mut self, layout: BusLayout) {
        self.layout = layout;
    }

    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
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
