use std::marker::PhantomData;
use std::sync::Arc;

use vst3_bindgen::{Class, Steinberg::Vst::*, Steinberg::*};

use crate::{Plugin, PluginInfo};

pub struct Component<P: Plugin> {
    info: Arc<PluginInfo>,
    _marker: PhantomData<P>,
}

impl<P: Plugin> Component<P> {
    pub fn new(info: &Arc<PluginInfo>) -> Component<P> {
        Component {
            info: info.clone(),
            _marker: PhantomData,
        }
    }
}

impl<P: Plugin> Class for Component<P> {
    type Interfaces = (
        IComponent,
        IAudioProcessor,
        IProcessContextRequirements,
        IEditController,
    );
}

impl<P: Plugin> IPluginBaseTrait for Component<P> {
    unsafe fn initialize(&self, _context: *mut FUnknown) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl<P: Plugin> IComponentTrait for Component<P> {
    unsafe fn getControllerClassId(&self, classId: *mut TUID) -> tresult {
        unimplemented!()
    }

    unsafe fn setIoMode(&self, mode: IoMode) -> tresult {
        unimplemented!()
    }

    unsafe fn getBusCount(&self, type_: MediaType, dir: BusDirection) -> int32 {
        unimplemented!()
    }

    unsafe fn getBusInfo(
        &self,
        type_: MediaType,
        dir: BusDirection,
        index: int32,
        bus: *mut BusInfo,
    ) -> tresult {
        unimplemented!()
    }

    unsafe fn getRoutingInfo(
        &self,
        inInfo: *mut RoutingInfo,
        outInfo: *mut RoutingInfo,
    ) -> tresult {
        unimplemented!()
    }

    unsafe fn activateBus(
        &self,
        type_: MediaType,
        dir: BusDirection,
        index: int32,
        state: TBool,
    ) -> tresult {
        unimplemented!()
    }

    unsafe fn setActive(&self, state: TBool) -> tresult {
        unimplemented!()
    }

    unsafe fn setState(&self, state: *mut IBStream) -> tresult {
        unimplemented!()
    }

    unsafe fn getState(&self, state: *mut IBStream) -> tresult {
        unimplemented!()
    }
}

impl<P: Plugin> IAudioProcessorTrait for Component<P> {
    unsafe fn setBusArrangements(
        &self,
        inputs: *mut SpeakerArrangement,
        numIns: int32,
        outputs: *mut SpeakerArrangement,
        numOuts: int32,
    ) -> tresult {
        unimplemented!()
    }

    unsafe fn getBusArrangement(
        &self,
        dir: BusDirection,
        index: int32,
        arr: *mut SpeakerArrangement,
    ) -> tresult {
        unimplemented!()
    }

    unsafe fn canProcessSampleSize(&self, symbolicSampleSize: int32) -> tresult {
        unimplemented!()
    }

    unsafe fn getLatencySamples(&self) -> uint32 {
        unimplemented!()
    }

    unsafe fn setupProcessing(&self, setup: *mut ProcessSetup) -> tresult {
        unimplemented!()
    }

    unsafe fn setProcessing(&self, state: TBool) -> tresult {
        unimplemented!()
    }

    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        unimplemented!()
    }

    unsafe fn getTailSamples(&self) -> uint32 {
        unimplemented!()
    }
}

impl<P: Plugin> IProcessContextRequirementsTrait for Component<P> {
    unsafe fn getProcessContextRequirements(&self) -> uint32 {
        unimplemented!()
    }
}

impl<P: Plugin> IEditControllerTrait for Component<P> {
    unsafe fn setComponentState(&self, state: *mut IBStream) -> tresult {
        unimplemented!()
    }

    unsafe fn setState(&self, state: *mut IBStream) -> tresult {
        unimplemented!()
    }

    unsafe fn getState(&self, state: *mut IBStream) -> tresult {
        unimplemented!()
    }

    unsafe fn getParameterCount(&self) -> int32 {
        unimplemented!()
    }

    unsafe fn getParameterInfo(&self, paramIndex: int32, info: *mut ParameterInfo) -> tresult {
        unimplemented!()
    }

    unsafe fn getParamStringByValue(
        &self,
        id: ParamID,
        valueNormalized: ParamValue,
        string: *mut String128,
    ) -> tresult {
        unimplemented!()
    }

    unsafe fn getParamValueByString(
        &self,
        id: ParamID,
        string: *mut TChar,
        valueNormalized: *mut ParamValue,
    ) -> tresult {
        unimplemented!()
    }

    unsafe fn normalizedParamToPlain(
        &self,
        id: ParamID,
        valueNormalized: ParamValue,
    ) -> ParamValue {
        unimplemented!()
    }

    unsafe fn plainParamToNormalized(&self, id: ParamID, plainValue: ParamValue) -> ParamValue {
        unimplemented!()
    }

    unsafe fn getParamNormalized(&self, id: ParamID) -> ParamValue {
        unimplemented!()
    }

    unsafe fn setParamNormalized(&self, id: ParamID, value: ParamValue) -> tresult {
        unimplemented!()
    }

    unsafe fn setComponentHandler(&self, handler: *mut IComponentHandler) -> tresult {
        unimplemented!()
    }

    unsafe fn createView(&self, name: FIDString) -> *mut IPlugView {
        unimplemented!()
    }
}
