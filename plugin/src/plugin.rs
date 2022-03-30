use crate::{atomic::AtomicF64, bus::*, editor::*, param::*, process::*};

#[derive(Clone)]
pub struct PluginInfo {
    pub name: String,
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub has_editor: bool,
}

pub trait Plugin: Send + Sync + Sized + 'static {
    type Processor: Processor<Plugin = Self>;
    type Editor: Editor<Plugin = Self>;

    fn info() -> PluginInfo;
    fn buses() -> BusList;
    fn supports_layout(inputs: &[BusLayout], outputs: &[BusLayout]) -> bool;
    fn params() -> ParamList;
    fn create() -> Self;
    fn serialize(
        &self,
        context: &SerializeContext,
        write: &mut impl std::io::Write,
    ) -> Result<(), ()>;
    fn deserialize(
        &self,
        context: &DeserializeContext,
        read: &mut impl std::io::Read,
    ) -> Result<(), ()>;
}

pub struct SerializeContext<'a> {
    param_list: &'a ParamList,
    param_values: &'a [AtomicF64],
}

impl<'a> SerializeContext<'a> {
    pub fn new(param_list: &'a ParamList, param_values: &'a [AtomicF64]) -> SerializeContext<'a> {
        SerializeContext { param_list, param_values }
    }

    #[inline]
    pub fn get_param<P: Param + 'static>(&self, key: ParamKey<P>) -> P::Value {
        let index = self.param_list.get_param_index(key.id).expect("Invalid parameter key");
        let param_info = self.param_list.get_param(key.id).unwrap();
        let param = param_info.param.downcast_ref::<P>().expect("Incorrect parameter type");
        param.from_normalized(self.param_values[index].load())
    }
}

pub struct DeserializeContext<'a> {
    param_list: &'a ParamList,
    param_values: &'a [AtomicF64],
}

impl<'a> DeserializeContext<'a> {
    pub fn new(param_list: &'a ParamList, param_values: &'a [AtomicF64]) -> DeserializeContext<'a> {
        DeserializeContext { param_list, param_values }
    }

    #[inline]
    pub fn get_param<P: Param + 'static>(&self, key: ParamKey<P>) -> P::Value {
        let index = self.param_list.get_param_index(key.id).expect("Invalid parameter key");
        let param_info = self.param_list.get_param(key.id).unwrap();
        let param = param_info.param.downcast_ref::<P>().expect("Incorrect parameter type");
        param.from_normalized(self.param_values[index].load())
    }

    #[inline]
    pub fn set_param<P: Param + 'static>(&self, key: ParamKey<P>, value: P::Value) {
        let index = self.param_list.get_param_index(key.id).expect("Invalid parameter key");
        let param_info = self.param_list.get_param(key.id).unwrap();
        let param = param_info.param.downcast_ref::<P>().expect("Incorrect parameter type");
        self.param_values[index].store(param.to_normalized(value))
    }
}
