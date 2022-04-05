use crate::atomic::{AtomicBitset, AtomicF64};
use crate::{editor::EditorContext, plugin::Plugin, process::ParamChange};

use std::collections::HashMap;
use std::marker::PhantomData;

pub type ParamId = u32;

pub struct ParamList<P> {
    pub(crate) params: Vec<Box<dyn ParamDef<P>>>,
}

impl<P> ParamList<P> {
    pub fn new() -> ParamList<P> {
        ParamList { params: Vec::new() }
    }

    pub fn param<Q: ParamDef<P> + 'static>(mut self, param: Q) -> ParamList<P> {
        self.params.push(Box::new(param));
        self
    }
}

pub(crate) struct ParamStates {
    pub index: HashMap<ParamId, usize>,
    pub info: Vec<ParamInfo>,
    pub dirty_processor: AtomicBitset,
    pub dirty_editor: AtomicBitset,
}

impl ParamStates {
    pub fn new<P>(param_list: &ParamList<P>, plugin: &P) -> ParamStates {
        let mut index = HashMap::with_capacity(param_list.params.len());
        let mut info = Vec::with_capacity(param_list.params.len());

        for (i, param) in param_list.params.iter().enumerate() {
            let param_info = param.info(plugin);

            index.insert(param_info.id, i);
            info.push(param_info);
        }

        let dirty_processor = AtomicBitset::with_len(param_list.params.len());
        let dirty_editor = AtomicBitset::with_len(param_list.params.len());

        ParamStates { index, info, dirty_processor, dirty_editor }
    }
}

pub struct ParamInfo {
    pub id: ParamId,
    pub name: String,
    pub label: String,
    pub steps: Option<usize>,
    pub default_normalized: f64,
}

pub trait ParamDef<P> {
    fn info(&self, plugin: &P) -> ParamInfo;
    fn get_normalized(&self, plugin: &P) -> f64;
    fn set_normalized(&self, plugin: &P, value: f64);
    fn normalized_to_plain(&self, plugin: &P, value: f64) -> f64;
    fn plain_to_normalized(&self, plugin: &P, value: f64) -> f64;
    fn normalized_to_string(&self, plugin: &P, value: f64, write: &mut dyn std::fmt::Write);
    fn string_to_normalized(&self, plugin: &P, string: &str) -> Result<f64, ()>;
}

pub trait Param {
    type Value;

    fn id(&self) -> ParamId;
    fn name(&self) -> String;
    fn label(&self) -> String;
    fn steps(&self) -> Option<usize>;
    fn default(&self) -> Self::Value;
    fn get(&self) -> Self::Value;
    fn set(&self, value: Self::Value);
    fn to_normalized(&self, value: Self::Value) -> f64;
    fn from_normalized(&self, value: f64) -> Self::Value;
    fn to_plain(&self, value: Self::Value) -> f64;
    fn from_plain(&self, value: f64) -> Self::Value;
    fn to_string(&self, value: Self::Value, write: &mut dyn std::fmt::Write);
    fn from_string(&self, string: &str) -> Result<Self::Value, ()>;

    #[inline]
    fn read_change(&self, change: ParamChange) -> Option<Self::Value> {
        if change.id == self.id() {
            Some(self.from_normalized(change.value_normalized))
        } else {
            None
        }
    }

    fn begin_edit(&self, context: &EditorContext) {
        context.begin_edit(self.id());
    }

    fn perform_edit(&self, context: &EditorContext, value: Self::Value) {
        context.perform_edit(self.id(), self.to_normalized(value));
    }

    fn end_edit(&self, context: &EditorContext) {
        context.end_edit(self.id());
    }
}

pub struct ParamAccessor<P: Plugin, Q: Param, F: Fn(&P) -> &Q> {
    f: F,
    phantom: PhantomData<fn(&P) -> &Q>,
}

impl<P: Plugin, Q: Param, F: Fn(&P) -> &Q> ParamAccessor<P, Q, F> {
    pub fn new(f: F) -> ParamAccessor<P, Q, F> {
        ParamAccessor { f, phantom: PhantomData }
    }
}

impl<P: Plugin, Q: Param, F: Fn(&P) -> &Q> ParamDef<P> for ParamAccessor<P, Q, F> {
    fn info(&self, plugin: &P) -> ParamInfo {
        let param = (self.f)(plugin);

        ParamInfo {
            id: param.id(),
            name: param.name(),
            label: param.label(),
            steps: param.steps(),
            default_normalized: param.to_normalized(param.default()),
        }
    }

    fn get_normalized(&self, plugin: &P) -> f64 {
        let param = (self.f)(plugin);
        param.to_normalized(param.get())
    }

    fn set_normalized(&self, plugin: &P, value: f64) {
        let param = (self.f)(plugin);
        param.set(param.from_normalized(value));
    }

    fn normalized_to_plain(&self, plugin: &P, value: f64) -> f64 {
        let param = (self.f)(plugin);
        param.to_plain(param.from_normalized(value))
    }

    fn plain_to_normalized(&self, plugin: &P, value: f64) -> f64 {
        let param = (self.f)(plugin);
        param.to_normalized(param.from_plain(value))
    }

    fn normalized_to_string(&self, plugin: &P, value: f64, write: &mut dyn std::fmt::Write) {
        let param = (self.f)(plugin);
        param.to_string(param.from_normalized(value), write);
    }

    fn string_to_normalized(&self, plugin: &P, string: &str) -> Result<f64, ()> {
        let param = (self.f)(plugin);
        param.from_string(string).map(|value| param.to_normalized(value))
    }
}

pub struct FloatParam {
    id: ParamId,
    name: String,
    min: f64,
    max: f64,
    default: f64,
    value: AtomicF64,
}

impl FloatParam {
    pub fn new(id: ParamId, name: &str, min: f64, max: f64, default: f64) -> FloatParam {
        FloatParam { id, name: name.to_string(), min, max, default, value: AtomicF64::new(default) }
    }
}

impl Param for FloatParam {
    type Value = f64;

    fn id(&self) -> ParamId {
        self.id
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn label(&self) -> String {
        String::new()
    }

    fn steps(&self) -> Option<usize> {
        None
    }

    fn default(&self) -> Self::Value {
        self.default
    }

    fn get(&self) -> Self::Value {
        self.value.load()
    }

    fn set(&self, value: Self::Value) {
        self.value.store(value);
    }

    fn to_normalized(&self, value: Self::Value) -> f64 {
        ((value - self.min) / (self.max - self.min)).max(0.0).min(1.0) as f64
    }

    fn from_normalized(&self, value: f64) -> Self::Value {
        (self.min + value * (self.max - self.min)).max(self.min).min(self.max)
    }

    fn to_plain(&self, value: Self::Value) -> f64 {
        value as f64
    }

    fn from_plain(&self, value: f64) -> Self::Value {
        value
    }

    fn to_string(&self, value: Self::Value, write: &mut dyn std::fmt::Write) {
        let _ = write!(write, "{}", value);
    }

    fn from_string(&self, string: &str) -> Result<Self::Value, ()> {
        string.parse().map_err(|_| ())
    }
}
