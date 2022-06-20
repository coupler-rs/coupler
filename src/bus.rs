#[derive(Eq, PartialEq, Clone)]
pub enum BusFormat {
    Stereo,
}

impl BusFormat {
    #[inline]
    pub fn channels(&self) -> usize {
        match self {
            BusFormat::Stereo => 2,
        }
    }
}

pub struct BusState {
    format: BusFormat,
    enabled: bool,
}

impl BusState {
    #[inline]
    pub fn new(format: BusFormat, enabled: bool) -> BusState {
        BusState { format, enabled }
    }

    #[inline]
    pub fn format(&self) -> &BusFormat {
        &self.format
    }

    #[inline]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    #[inline]
    pub fn set_format(&mut self, format: BusFormat) {
        self.format = format;
    }

    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

pub struct BusInfo {
    pub name: String,
    pub default_format: BusFormat,
}

pub struct BusList {
    pub(crate) inputs: Vec<BusInfo>,
    pub(crate) outputs: Vec<BusInfo>,
}

impl BusList {
    pub fn new() -> BusList {
        BusList { inputs: Vec::new(), outputs: Vec::new() }
    }

    pub fn input(mut self, name: &str, default_format: BusFormat) -> BusList {
        self.inputs.push(BusInfo { name: name.to_string(), default_format });
        self
    }

    pub fn output(mut self, name: &str, default_format: BusFormat) -> BusList {
        self.outputs.push(BusInfo { name: name.to_string(), default_format });
        self
    }
}
