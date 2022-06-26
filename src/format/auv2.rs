use std::ffi::c_void;
use std::mem;
use std::ptr;

use auv2_sys::*;

struct Wrapper {
    #[allow(unused)]
    interface: AudioComponentPlugInInterface,
}

#[allow(non_snake_case)]
impl Wrapper {
    fn create() -> *mut Wrapper {
        Box::into_raw(Box::new(Wrapper {
            interface: AudioComponentPlugInInterface {
                Open: Some(Self::Open),
                Close: Some(Self::Close),
                Lookup: Some(Self::Lookup),
                reserved: ptr::null_mut(),
            },
        }))
    }

    pub unsafe extern "C" fn Open(
        _self_: *mut c_void,
        _mInstance: AudioComponentInstance,
    ) -> OSStatus {
        noErr
    }

    pub unsafe extern "C" fn Close(self_: *mut c_void) -> OSStatus {
        drop(Box::from_raw(self_ as *mut Wrapper));

        noErr
    }

    pub unsafe extern "C" fn Lookup(selector: SInt16) -> Option<AudioComponentMethod> {
        #[allow(non_upper_case_globals)]
        match selector {
            kAudioUnitInitializeSelect => {
                let proc: AudioUnitInitializeProc = Self::Initialize;
                Some(mem::transmute(proc))
            }
            kAudioUnitUninitializeSelect => {
                let proc: AudioUnitUninitializeProc = Self::Uninitialize;
                Some(mem::transmute(proc))
            }
            kAudioUnitGetPropertyInfoSelect => {
                let proc: AudioUnitGetPropertyInfoProc = Self::GetPropertyInfo;
                Some(mem::transmute(proc))
            }
            kAudioUnitGetPropertySelect => {
                let proc: AudioUnitGetPropertyProc = Self::GetProperty;
                Some(mem::transmute(proc))
            }
            kAudioUnitSetPropertySelect => {
                let proc: AudioUnitSetPropertyProc = Self::SetProperty;
                Some(mem::transmute(proc))
            }
            kAudioUnitAddPropertyListenerSelect => {
                let proc: AudioUnitAddPropertyListenerProc = Self::AddPropertyListener;
                Some(mem::transmute(proc))
            }
            kAudioUnitRemovePropertyListenerSelect => {
                let proc: AudioUnitRemovePropertyListenerProc = Self::RemovePropertyListener;
                Some(mem::transmute(proc))
            }
            kAudioUnitRemovePropertyListenerWithUserDataSelect => {
                let proc: AudioUnitRemovePropertyListenerWithUserDataProc =
                    Self::RemovePropertyListenerWithUserData;
                Some(mem::transmute(proc))
            }
            kAudioUnitAddRenderNotifySelect => {
                let proc: AudioUnitAddRenderNotifyProc = Self::AddRenderNotify;
                Some(mem::transmute(proc))
            }
            kAudioUnitRemoveRenderNotifySelect => {
                let proc: AudioUnitRemoveRenderNotifyProc = Self::RemoveRenderNotify;
                Some(mem::transmute(proc))
            }
            kAudioUnitScheduleParametersSelect => {
                let proc: AudioUnitScheduleParametersProc = Self::ScheduleParameters;
                Some(mem::transmute(proc))
            }
            kAudioUnitResetSelect => {
                let proc: AudioUnitResetProc = Self::Reset;
                Some(mem::transmute(proc))
            }
            kAudioUnitComplexRenderSelect => {
                let proc: AudioUnitComplexRenderProc = Self::ComplexRender;
                Some(mem::transmute(proc))
            }
            kAudioUnitProcessSelect => {
                let proc: AudioUnitProcessProc = Self::Process;
                Some(mem::transmute(proc))
            }
            kAudioUnitProcessMultipleSelect => {
                let proc: AudioUnitProcessMultipleProc = Self::ProcessMultiple;
                Some(mem::transmute(proc))
            }
            kAudioUnitGetParameterSelect => {
                let proc: AudioUnitGetParameterProc = Self::GetParameter;
                Some(mem::transmute(proc))
            }
            kAudioUnitSetParameterSelect => {
                let proc: AudioUnitSetParameterProc = Self::SetParameter;
                Some(mem::transmute(proc))
            }
            kAudioUnitRenderSelect => {
                let proc: AudioUnitRenderProc = Self::Render;
                Some(mem::transmute(proc))
            }
            _ => None,
        }
    }

    unsafe extern "C" fn Initialize(_self_: *mut c_void) -> OSStatus {
        noErr
    }

    unsafe extern "C" fn Uninitialize(_self_: *mut c_void) -> OSStatus {
        noErr
    }

    unsafe extern "C" fn GetPropertyInfo(
        _self_: *mut c_void,
        _prop: AudioUnitPropertyID,
        _scope: AudioUnitScope,
        _elem: AudioUnitElement,
        _outDataSize: *mut UInt32,
        _outWritable: *mut Boolean,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn GetProperty(
        _self_: *mut c_void,
        _inID: AudioUnitPropertyID,
        _inScope: AudioUnitScope,
        _inElement: AudioUnitElement,
        _outData: *mut c_void,
        _ioDataSize: *mut UInt32,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn SetProperty(
        _self_: *mut c_void,
        _inID: AudioUnitPropertyID,
        _inScope: AudioUnitScope,
        _inElement: AudioUnitElement,
        _inData: *const c_void,
        _inDataSize: UInt32,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn AddPropertyListener(
        _self_: *mut c_void,
        _prop: AudioUnitPropertyID,
        _proc: Option<AudioUnitPropertyListenerProc>,
        _userData: *mut c_void,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn RemovePropertyListener(
        _self_: *mut c_void,
        _prop: AudioUnitPropertyID,
        _proc: Option<AudioUnitPropertyListenerProc>,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn RemovePropertyListenerWithUserData(
        _self_: *mut c_void,
        _prop: AudioUnitPropertyID,
        _proc: Option<AudioUnitPropertyListenerProc>,
        _userData: *mut c_void,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn AddRenderNotify(
        _self_: *mut c_void,
        _proc: Option<AURenderCallback>,
        _userData: *mut c_void,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn RemoveRenderNotify(
        _self_: *mut c_void,
        _proc: Option<AURenderCallback>,
        _userData: *mut c_void,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn ScheduleParameters(
        _self_: *mut c_void,
        _events: *const AudioUnitParameterEvent,
        _numEvents: UInt32,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn Reset(
        _self_: *mut c_void,
        _inScope: AudioUnitScope,
        _inElement: AudioUnitElement,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn ComplexRender(
        _self_: *mut c_void,
        _ioActionFlags: *const AudioUnitRenderActionFlags,
        _nTimeStamp: *const AudioTimeStamp,
        _inOutputBusNumber: UInt32,
        _inNumberOfPackets: UInt32,
        _outNumberOfPackets: *mut UInt32,
        _outPacketDescriptions: *mut AudioStreamPacketDescription,
        _ioData: *mut AudioBufferList,
        _outMetadata: *mut c_void,
        _outMetadataByteSize: *mut UInt32,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn Process(
        _self_: *mut c_void,
        _ioActionFlags: *const AudioUnitRenderActionFlags,
        _nTimeStamp: *const AudioTimeStamp,
        _inNumberFrames: UInt32,
        _ioData: *mut AudioBufferList,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn ProcessMultiple(
        _self_: *mut c_void,
        _ioActionFlags: *const AudioUnitRenderActionFlags,
        _nTimeStamp: *const AudioTimeStamp,
        _inNumberFrames: UInt32,
        _inNumberInputBufferLists: UInt32,
        _inInputBufferLists: *const *const AudioBufferList,
        _inNumberOutputBufferLists: UInt32,
        _ioOutputBufferLists: *mut *mut AudioBufferList,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn GetParameter(
        _inComponentStorage: *mut c_void,
        _inID: AudioUnitParameterID,
        _inScope: AudioUnitScope,
        _inElement: AudioUnitElement,
        _outValue: *mut AudioUnitParameterValue,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn SetParameter(
        _inComponentStorage: *mut c_void,
        _inID: AudioUnitParameterID,
        _inScope: AudioUnitScope,
        _inElement: AudioUnitElement,
        _inValue: AudioUnitParameterValue,
        _inBufferOffsetInFrames: UInt32,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }

    unsafe extern "C" fn Render(
        _inComponentStorage: *mut c_void,
        _ioActionFlags: *const AudioUnitRenderActionFlags,
        _inTimeStamp: *const AudioTimeStamp,
        _inOutputBusNumber: UInt32,
        _inNumberFrames: UInt32,
        _ioData: *mut AudioBufferList,
    ) -> OSStatus {
        kAudio_UnimplementedError
    }
}

#[doc(hidden)]
pub unsafe fn auv2_factory(_in_desc: *mut c_void) -> *mut c_void {
    Wrapper::create() as *mut c_void
}

#[macro_export]
macro_rules! auv2 {
    ($plugin:ty) => {
        #[no_mangle]
        unsafe extern "C" fn AUFactory(inDesc: *mut ::std::ffi::c_void) -> *mut ::std::ffi::c_void {
            ::coupler::format::auv2::auv2_factory(inDesc)
        }
    };
}
