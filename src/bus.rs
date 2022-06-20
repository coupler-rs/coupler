#[non_exhaustive]
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
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
    name: String,
}

impl BusInfo {
    #[inline]
    pub fn new(name: &str) -> BusInfo {
        BusInfo { name: name.to_string() }
    }

    #[inline]
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    #[inline]
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

pub struct BusList {
    inputs: Vec<BusInfo>,
    outputs: Vec<BusInfo>,
}

impl BusList {
    #[inline]
    pub fn new() -> BusList {
        BusList { inputs: Vec::new(), outputs: Vec::new() }
    }

    #[inline]
    pub fn input(mut self, input: BusInfo) -> Self {
        self.inputs.push(input);
        self
    }

    #[inline]
    pub fn output(mut self, output: BusInfo) -> Self {
        self.outputs.push(output);
        self
    }

    #[inline]
    pub fn get_inputs(&self) -> &[BusInfo] {
        &self.inputs
    }

    #[inline]
    pub fn get_outputs(&self) -> &[BusInfo] {
        &self.outputs
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct BusConfig {
    inputs: Vec<BusFormat>,
    outputs: Vec<BusFormat>,
}

impl BusConfig {
    #[inline]
    pub fn new() -> BusConfig {
        BusConfig { inputs: Vec::new(), outputs: Vec::new() }
    }

    #[inline]
    pub fn input(mut self, input: BusFormat) -> Self {
        self.inputs.push(input);
        self
    }

    #[inline]
    pub fn output(mut self, output: BusFormat) -> Self {
        self.outputs.push(output);
        self
    }

    #[inline]
    pub fn get_inputs(&self) -> &[BusFormat] {
        &self.inputs
    }

    #[inline]
    pub fn get_outputs(&self) -> &[BusFormat] {
        &self.outputs
    }
}

pub struct BusConfigList {
    configs: Vec<BusConfig>,
}

impl BusConfigList {
    #[inline]
    pub fn new() -> BusConfigList {
        BusConfigList { configs: Vec::new() }
    }

    #[inline]
    pub fn config(mut self, config: BusConfig) -> Self {
        self.configs.push(config);
        self
    }

    #[inline]
    pub fn default(mut self, config: BusConfig) -> Self {
        self.configs.insert(0, config);
        self
    }

    #[inline]
    pub fn get_configs(&self) -> &[BusConfig] {
        &self.configs
    }

    #[inline]
    pub fn get_default(&self) -> Option<&BusConfig> {
        self.configs.first()
    }
}
