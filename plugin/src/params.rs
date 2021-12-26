use std::collections::HashMap;

pub type ParamId = u32;

pub struct ParamInfo {
    pub units: String,
    pub steps: Option<u32>,
}

pub trait Param: Send + Sync {
    type Value;

    fn info(&self) -> ParamInfo;
    fn default(&self) -> Self::Value;
    fn display(&self, value: Self::Value, write: &mut dyn std::fmt::Write);
    fn parse(&self, string: &str) -> Result<Self::Value, ()>;
    fn encode(&self, value: Self::Value) -> f64;
    fn decode(&self, value: f64) -> Self::Value;
}

pub trait ParamDyn {
    fn display_encoded(&self, value: f64, write: &mut dyn std::fmt::Write);
    fn parse_encoded(&self, string: &str) -> Result<f64, ()>;
}

impl<P: Param> ParamDyn for P {
    fn display_encoded(&self, value: f64, write: &mut dyn std::fmt::Write) {
        self.display(self.decode(value), write);
    }

    fn parse_encoded(&self, string: &str) -> Result<f64, ()> {
        self.parse(string).map(|value| self.encode(value))
    }
}

pub struct ParamDef {
    pub(crate) id: ParamId,
    pub(crate) name: String,
    pub(crate) info: ParamInfo,
    pub(crate) default: f64,
    pub(crate) param: Box<dyn ParamDyn>,
}

pub struct ParamList {
    pub(crate) params: Vec<ParamDef>,
    pub(crate) indices: HashMap<ParamId, usize>,
}

impl ParamList {
    pub fn new() -> ParamList {
        ParamList { params: Vec::new(), indices: HashMap::new() }
    }

    pub fn add(mut self, id: ParamId, name: &str, param: impl Param + 'static) -> ParamList {
        let index = self.params.len();

        self.params.push(ParamDef {
            id,
            name: name.to_string(),
            info: param.info(),
            default: param.encode(param.default()),
            param: Box::new(param),
        });

        self.indices.insert(id, index);

        self
    }
}
