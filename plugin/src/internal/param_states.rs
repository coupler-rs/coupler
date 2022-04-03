use crate::atomic::AtomicF64;
use crate::param::*;

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
