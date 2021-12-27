use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;

pub type ParamId = u32;

pub struct ParamKey<P> {
    id: ParamId,
    phantom: PhantomData<P>,
}

impl<P> Clone for ParamKey<P> {
    fn clone(&self) -> ParamKey<P> {
        ParamKey { id: self.id, phantom: PhantomData }
    }
}

impl<P> Copy for ParamKey<P> {}

impl<P> ParamKey<P> {
    pub const fn new(id: ParamId) -> ParamKey<P> {
        ParamKey { id, phantom: PhantomData }
    }

    pub const fn id(&self) -> ParamId {
        self.id
    }
}

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

pub(crate) trait ParamDyn: Any {
    fn display_encoded(&self, value: f64, write: &mut dyn std::fmt::Write);
    fn parse_encoded(&self, string: &str) -> Result<f64, ()>;
    fn as_any(&self) -> &dyn Any;
}

impl<P: Param + 'static> ParamDyn for P {
    fn display_encoded(&self, value: f64, write: &mut dyn std::fmt::Write) {
        self.display(self.decode(value), write);
    }

    fn parse_encoded(&self, string: &str) -> Result<f64, ()> {
        self.parse(string).map(|value| self.encode(value))
    }

    fn as_any(&self) -> &dyn Any {
        self
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

    pub fn add<P: Param + 'static>(mut self, key: ParamKey<P>, name: &str, param: P) -> ParamList {
        let index = self.params.len();

        self.params.push(ParamDef {
            id: key.id,
            name: name.to_string(),
            info: param.info(),
            default: param.encode(param.default()),
            param: Box::new(param),
        });

        (self.params.last().unwrap() as &dyn Any).downcast_ref::<u32>();

        self.indices.insert(key.id, index);

        self
    }
}
