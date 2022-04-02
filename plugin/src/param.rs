use crate::atomic::AtomicF64;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

pub type ParamId = u32;

pub struct ParamKey<P> {
    pub id: ParamId,
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
}

pub struct ParamChange {
    pub id: ParamId,
    pub value: f64,
}

pub struct ParamInfo {
    pub id: ParamId,
    pub name: String,
    pub param: Box<dyn DynParam>,
}

pub struct ParamList {
    params: Vec<ParamInfo>,
    index: HashMap<ParamId, usize>,
}

impl ParamList {
    pub fn new() -> ParamList {
        ParamList { params: Vec::new(), index: HashMap::new() }
    }

    pub fn param<P: Param + 'static>(
        mut self,
        key: ParamKey<P>,
        name: &str,
        param: P,
    ) -> ParamList {
        if self.index.contains_key(&key.id) {
            panic!("ParamList already contains parameter with id {}", key.id);
        }

        self.index.insert(key.id, self.params.len());
        self.params.push(ParamInfo { id: key.id, name: name.to_string(), param: Box::new(param) });

        self
    }

    pub fn params(&self) -> &[ParamInfo] {
        &self.params
    }

    #[inline]
    pub fn get_param(&self, id: ParamId) -> Option<&ParamInfo> {
        self.get_param_index(id).map(|i| &self.params[i])
    }

    #[inline]
    pub fn get_param_index(&self, id: ParamId) -> Option<usize> {
        self.index.get(&id).cloned()
    }
}

pub struct ParamStates {
    pub list: ParamList,
    pub values: Vec<AtomicF64>,
}

impl ParamStates {
    pub fn new(list: ParamList) -> ParamStates {
        let mut values = Vec::with_capacity(list.params().len());
        for param_info in list.params() {
            values.push(AtomicF64::new(param_info.param.default_normalized()));
        }

        ParamStates { list, values }
    }

    #[inline]
    pub fn get_param<P: Param + 'static>(&self, key: ParamKey<P>) -> P::Value {
        let index = self.list.get_param_index(key.id).expect("Invalid parameter key");
        let param_info = &self.list.params()[index];
        let param = param_info.param.downcast_ref::<P>().expect("Incorrect parameter type");
        param.from_normalized(self.values[index].load())
    }

    #[inline]
    pub fn set_param<P: Param + 'static>(&self, key: ParamKey<P>, value: P::Value) {
        let index = self.list.get_param_index(key.id).expect("Invalid parameter key");
        let param_info = &self.list.params()[index];
        let param = param_info.param.downcast_ref::<P>().expect("Incorrect parameter type");
        self.values[index].store(param.to_normalized(value))
    }
}

pub trait Param {
    type Value;

    fn steps(&self) -> Option<usize>;
    fn label(&self) -> String;
    fn default(&self) -> Self::Value;
    fn to_normalized(&self, value: Self::Value) -> f64;
    fn from_normalized(&self, value: f64) -> Self::Value;
    fn to_plain(&self, value: Self::Value) -> f64;
    fn from_plain(&self, value: f64) -> Self::Value;
    fn to_string(&self, value: Self::Value, write: &mut dyn std::fmt::Write);
    fn from_string(&self, string: &str) -> Result<Self::Value, ()>;
}

pub trait DynParam {
    fn steps(&self) -> Option<usize>;
    fn label(&self) -> String;
    fn default_normalized(&self) -> f64;
    fn normalized_to_plain(&self, value: f64) -> f64;
    fn plain_to_normalized(&self, value: f64) -> f64;
    fn normalized_to_string(&self, value: f64, write: &mut dyn std::fmt::Write);
    fn string_to_normalized(&self, string: &str) -> Result<f64, ()>;
    fn type_id(&self) -> TypeId;
}

impl<P: Param + 'static> DynParam for P {
    fn default_normalized(&self) -> f64 {
        Param::to_normalized(self, Param::default(self))
    }

    fn steps(&self) -> Option<usize> {
        Param::steps(self)
    }

    fn label(&self) -> String {
        Param::label(self)
    }

    fn normalized_to_plain(&self, value: f64) -> f64 {
        Param::to_plain(self, Param::from_normalized(self, value))
    }

    fn plain_to_normalized(&self, value: f64) -> f64 {
        Param::to_normalized(self, Param::from_plain(self, value))
    }

    fn normalized_to_string(&self, value: f64, write: &mut dyn std::fmt::Write) {
        Param::to_string(self, Param::from_normalized(self, value), write);
    }

    fn string_to_normalized(&self, string: &str) -> Result<f64, ()> {
        Param::from_string(self, string).map(|value| Param::to_normalized(self, value))
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl dyn DynParam {
    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.type_id() == TypeId::of::<T>() {
            Some(unsafe { &*(self as *const dyn DynParam as *const T) })
        } else {
            None
        }
    }
}

pub struct FloatParam {
    min: f32,
    max: f32,
    default: f32,
}

impl FloatParam {
    pub fn new(min: f32, max: f32, default: f32) -> FloatParam {
        FloatParam { min, max, default }
    }
}

impl Param for FloatParam {
    type Value = f32;

    fn default(&self) -> f32 {
        self.default
    }

    fn steps(&self) -> Option<usize> {
        None
    }

    fn label(&self) -> String {
        String::new()
    }

    fn to_normalized(&self, value: Self::Value) -> f64 {
        ((value - self.min) / (self.max - self.min)).max(0.0).min(1.0) as f64
    }

    fn from_normalized(&self, value: f64) -> Self::Value {
        (self.min + value as f32 * (self.max - self.min)).max(self.min).min(self.max)
    }

    fn to_plain(&self, value: Self::Value) -> f64 {
        value as f64
    }

    fn from_plain(&self, value: f64) -> Self::Value {
        value as f32
    }

    fn to_string(&self, value: Self::Value, write: &mut dyn std::fmt::Write) {
        let _ = write!(write, "{}", value);
    }

    fn from_string(&self, string: &str) -> Result<Self::Value, ()> {
        string.parse().map_err(|_| ())
    }
}
