use std::rc::Rc;

use crate::{atomic::AtomicF32, editor::EditorContext, process::ParamChange};

pub type ParamId = u32;

pub struct ParamList<P> {
    params: Vec<ParamKey<P>>,
}

impl<P> ParamList<P> {
    pub fn new() -> ParamList<P> {
        ParamList { params: Vec::new() }
    }

    pub fn param(mut self, key: ParamKey<P>) -> ParamList<P> {
        self.params.push(key);
        self
    }

    pub fn params(&self) -> &[ParamKey<P>] {
        &self.params
    }
}

pub struct ParamKey<P>(pub fn(&P) -> &dyn Param);

impl<P> Clone for ParamKey<P> {
    fn clone(&self) -> ParamKey<P> {
        ParamKey(self.0)
    }
}

impl<P> Copy for ParamKey<P> {}

impl<P> ParamKey<P> {
    pub fn apply<'p>(&self, plugin: &'p P) -> &'p dyn Param {
        self.0(plugin)
    }
}

pub struct ParamInfo {
    pub name: &'static str,
    pub units: &'static str,
    pub steps: Option<usize>,
}

pub trait TypedParam {
    type Value;

    fn id(&self) -> ParamId;
    fn info(&self) -> ParamInfo;
    fn set(&self, value: Self::Value);
    fn get(&self) -> Self::Value;
    fn to_normalized(&self, value: Self::Value) -> f64;
    fn from_normalized(&self, value: f64) -> Self::Value;
    fn to_plain(&self, value: Self::Value) -> f64;
    fn from_plain(&self, value: f64) -> Self::Value;
    fn to_string(&self, value: Self::Value, write: &mut dyn std::fmt::Write);
    fn from_string(&self, string: &str) -> Result<Self::Value, ()>;

    fn begin_edit(&self, context: &Rc<dyn EditorContext>) {
        context.begin_edit(self.id());
    }

    fn perform_edit(&self, context: &Rc<dyn EditorContext>, value: Self::Value) {
        context.perform_edit(self.id(), self.to_normalized(value));
    }

    fn end_edit(&self, context: &Rc<dyn EditorContext>) {
        context.end_edit(self.id());
    }

    #[inline]
    fn read_change(&self, change: &ParamChange) -> Option<Self::Value> {
        if change.id == self.id() {
            Some(self.from_normalized(change.value))
        } else {
            None
        }
    }
}

pub trait Param {
    fn id(&self) -> ParamId;
    fn info(&self) -> ParamInfo;
    fn set_normalized(&self, value: f64);
    fn get_normalized(&self) -> f64;
    fn normalized_to_plain(&self, value: f64) -> f64;
    fn plain_to_normalized(&self, value: f64) -> f64;
    fn to_string(&self, value: f64, write: &mut dyn std::fmt::Write);
    fn from_string(&self, string: &str) -> Result<f64, ()>;
}

impl<P: TypedParam> Param for P {
    fn id(&self) -> ParamId {
        TypedParam::id(self)
    }

    fn info(&self) -> ParamInfo {
        TypedParam::info(self)
    }

    fn set_normalized(&self, value: f64) {
        TypedParam::set(self, TypedParam::from_normalized(self, value))
    }

    fn get_normalized(&self) -> f64 {
        TypedParam::to_normalized(self, TypedParam::get(self))
    }

    fn normalized_to_plain(&self, value: f64) -> f64 {
        TypedParam::to_plain(self, TypedParam::from_normalized(self, value))
    }

    fn plain_to_normalized(&self, value: f64) -> f64 {
        TypedParam::to_normalized(self, TypedParam::from_plain(self, value))
    }

    fn to_string(&self, value: f64, write: &mut dyn std::fmt::Write) {
        TypedParam::to_string(self, TypedParam::from_normalized(self, value), write);
    }

    fn from_string(&self, string: &str) -> Result<f64, ()> {
        TypedParam::from_string(self, string).map(|value| TypedParam::to_normalized(self, value))
    }
}

pub struct FloatParam {
    id: ParamId,
    name: &'static str,
    min: f32,
    max: f32,
    value: AtomicF32,
}

impl FloatParam {
    pub fn new(id: ParamId, name: &'static str, value: f32) -> FloatParam {
        FloatParam { id, name, min: 0.0, max: 1.0, value: AtomicF32::new(value) }
    }
}

impl TypedParam for FloatParam {
    type Value = f32;

    fn id(&self) -> ParamId {
        self.id
    }

    fn info(&self) -> ParamInfo {
        ParamInfo { name: self.name, units: "", steps: None }
    }

    fn set(&self, value: Self::Value) {
        self.value.store(value);
    }

    fn get(&self) -> Self::Value {
        self.value.load()
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
