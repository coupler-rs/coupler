use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::atomic::AtomicF64;

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

pub(crate) struct ParamDef {
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

pub struct ParamValues<'a> {
    pub(crate) param_list: &'a ParamList,
    pub(crate) values: &'a [AtomicF64],
}

impl<'a> ParamValues<'a> {
    pub fn get_param<P: Param + 'static>(&self, key: ParamKey<P>) -> P::Value {
        let index = self.param_list.indices[&key.id()];
        let param = self.param_list.params[index].param.as_any().downcast_ref::<P>().unwrap();
        param.decode(self.values[index].load())
    }

    pub fn set_param<P: Param + 'static>(&mut self, key: ParamKey<P>, value: P::Value) {
        let index = self.param_list.indices[&key.id()];
        let param = self.param_list.params[index].param.as_any().downcast_ref::<P>().unwrap();
        self.values[index].store(param.encode(value));
    }
}

pub struct BoolParam {
    default: bool,
}

impl BoolParam {
    pub fn new(default: bool) -> BoolParam {
        BoolParam { default }
    }
}

impl Param for BoolParam {
    type Value = bool;

    fn info(&self) -> ParamInfo {
        ParamInfo { units: "".to_string(), steps: Some(1) }
    }

    fn default(&self) -> bool {
        self.default
    }

    fn display(&self, value: Self::Value, write: &mut dyn std::fmt::Write) {
        let _ = write!(write, "{}", value);
    }

    fn parse(&self, string: &str) -> Result<Self::Value, ()> {
        string.parse().map_err(|_| ())
    }

    fn encode(&self, value: Self::Value) -> f64 {
        if value { 1.0 } else { 0.0 }
    }

    fn decode(&self, value: f64) -> Self::Value {
        value >= 0.5
    }
}
