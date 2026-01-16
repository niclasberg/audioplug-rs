use atomic_refcell::AtomicRefCell;
use std::cell::Cell;
use std::mem::MaybeUninit;
use std::rc::Rc;
use vst3::Steinberg::Vst::BusInfo_::BusFlags_;
use vst3::Steinberg::Vst::Event_::EventTypes_;
use vst3::Steinberg::Vst::{
    BusDirection, BusDirections_, BusInfo, BusTypes_, IAudioProcessor, IAudioProcessorTrait,
    IComponent, IComponentTrait, IConnectionPoint, IConnectionPointTrait, IEventListTrait,
    IHostApplication, IHostApplicationTrait, IMessage, IParamValueQueueTrait as _,
    IParameterChangesTrait, IoMode, MediaType, MediaTypes_, ProcessData, ProcessModes_,
    ProcessSetup, RoutingInfo, SpeakerArr, SpeakerArrangement, SymbolicSampleSizes_,
};
use vst3::Steinberg::{
    FUnknown, IBStream, IPluginBase, IPluginBaseTrait, TBool, TUID, kInvalidArgument,
    kNotImplemented, kResultFalse, kResultOk, tresult,
};
use vst3::{ComPtr, ComRef, ComWrapper, Interface};

use super::util::strcpyw;
use crate::midi::{Note, NoteEvent};
use crate::param::{AnyParameterMap, NormalizedValue, ParameterId, ParameterMap, Params};
use crate::wrapper::vst3::Factory;
use crate::wrapper::vst3::util::tuid_from_uuid;
use crate::{AudioBuffer, MidiProcessContext, ProcessContext, ProcessInfo, VST3Plugin};

struct SharedState {
    value: u32,
}

pub struct AudioProcessor<P: VST3Plugin> {
    plugin: AtomicRefCell<P>,
    parameters: Rc<ParameterMap<P::Parameters>>,
    host_context: Cell<Option<ComPtr<IHostApplication>>>,
}

impl<P: VST3Plugin> vst3::Class for AudioProcessor<P> {
    type Interfaces = (IComponent, IAudioProcessor, IConnectionPoint, IPluginBase);
}

impl<P: VST3Plugin> AudioProcessor<P> {
    pub fn new() -> ComWrapper<Self> {
        let parameters = ParameterMap::new(P::Parameters::new());
        ComWrapper::new(Self {
            plugin: AtomicRefCell::new(P::new()),
            parameters,
            host_context: Cell::new(None),
        })
    }
}

impl<P: VST3Plugin> IAudioProcessorTrait for AudioProcessor<P> {
    unsafe fn setBusArrangements(
        &self,
        _inputs: *mut SpeakerArrangement,
        _num_ins: i32,
        _outputs: *mut SpeakerArrangement,
        _num_outs: i32,
    ) -> tresult {
        // From the VST3 docs, we can do the following:
        // 1. Accept the provided speaker arrangement, adjust the bus:es and return true
        // 2. Otherwise, we return false. The host will then repeatedly call get_bus_arrangement,
        // allowing us to adjust. The host will then call set_bus_arrangement with the provided
        // adjustments.

        kResultFalse
    }

    unsafe fn getBusArrangement(
        &self,
        dir: BusDirection,
        index: i32,
        arr: *mut SpeakerArrangement,
    ) -> tresult {
        if index < 0 {
            return kInvalidArgument;
        }
        let Some(arr) = (unsafe { arr.as_mut() }) else {
            return kInvalidArgument;
        };

        let bus_option = (index == 0)
            .then(|| {
                if dir == BusDirections_::kInput as i32 {
                    P::AUDIO_LAYOUT.main_input.as_ref()
                } else {
                    P::AUDIO_LAYOUT.main_output.as_ref()
                }
            })
            .flatten();

        if let Some(bus) = bus_option {
            *arr = match bus.channel {
                crate::ChannelType::Empty => SpeakerArr::kEmpty,
                crate::ChannelType::Mono => SpeakerArr::kMono,
                crate::ChannelType::Stereo => SpeakerArr::kStereo,
            };
            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn canProcessSampleSize(&self, symbolic_sample_size: i32) -> tresult {
        if symbolic_sample_size == SymbolicSampleSizes_::kSample32 as i32 {
            kResultOk
        } else {
            kResultFalse
        }
    }

    unsafe fn getLatencySamples(&self) -> u32 {
        self.plugin.borrow().latency_samples() as u32
    }

    unsafe fn setupProcessing(&self, setup: *mut ProcessSetup) -> tresult {
        let Some(setup) = (unsafe { setup.as_ref() }) else {
            return kInvalidArgument;
        };
        self.plugin
            .borrow_mut()
            .prepare(setup.sampleRate, setup.maxSamplesPerBlock as usize);
        kResultOk
    }

    // Called with true before processing starts, and false after. Can be called from both UI and
    // realtime thread
    unsafe fn setProcessing(&self, state: TBool) -> tresult {
        if state == 0 {
            self.plugin.borrow_mut().reset();
        }
        kResultOk
    }

    // This method is only called from the audio thread
    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        let Some(data) = (unsafe { data.as_mut() }) else {
            return kInvalidArgument;
        };
        let Some(process_context) = (unsafe { data.processContext.as_mut() }) else {
            return kInvalidArgument;
        };

        if let Some(input_param_changes) = unsafe { ComRef::from_raw(data.inputParameterChanges) } {
            let parameter_change_count = unsafe { input_param_changes.getParameterCount() };
            for i in 0..parameter_change_count {
                let parameter_data_ptr = unsafe { input_param_changes.getParameterData(i) };
                if let Some(data) = unsafe { ComRef::from_raw(parameter_data_ptr) } {
                    let point_count = unsafe { data.getPointCount() };
                    if point_count <= 0 {
                        continue;
                    }
                    let param_id = ParameterId(unsafe { data.getParameterId() });

                    if let Some(param_ref) = self.parameters.get_by_id(param_id) {
                        let mut value = 0.0;
                        let mut sample_offset = 0;
                        if unsafe {
                            data.getPoint(
                                point_count - 1,
                                &mut sample_offset as *mut _,
                                &mut value as *mut _,
                            )
                        } == kResultOk
                        {
                            param_ref
                                .set_value_normalized(NormalizedValue::from_f64_unchecked(value));
                        }
                    }
                }
            }
        }

        // Parameter flush
        if data.numSamples == 0 {
            return kResultOk;
        }

        let input = if data.inputs.is_null() {
            AudioBuffer::empty()
        } else {
            unsafe {
                AudioBuffer::from_ptr(
                    (*data.inputs).__field0.channelBuffers32 as *mut *mut _,
                    (*data.inputs).numChannels as usize,
                    data.numSamples as usize,
                )
            }
        };

        let mut output = if data.outputs.is_null() {
            AudioBuffer::empty()
        } else {
            unsafe {
                AudioBuffer::from_ptr(
                    (*data.outputs).__field0.channelBuffers32 as *mut *mut _,
                    (*data.outputs).numChannels as usize,
                    data.numSamples as usize,
                )
            }
        };

        let info = ProcessInfo {
            rendering_offline: data.processMode == ProcessModes_::kOffline as i32,
            sample_rate: process_context.sampleRate,
        };

        let context = ProcessContext {
            input: &input,
            output: &mut output,
            info,
        };

        let mut plugin = self.plugin.borrow_mut();
        if P::ACCEPTS_MIDI {
            const NOTE_ON_EVENT: u16 = EventTypes_::kNoteOnEvent as _;
            const NOTE_OFF_EVENT: u16 = EventTypes_::kNoteOffEvent as _;

            let mut context = MidiProcessContext { info };
            if let Some(input_events) = unsafe { ComRef::from_raw(data.inputEvents) } {
                let event_count = unsafe { input_events.getEventCount() };
                for i in 0..event_count {
                    let mut event = MaybeUninit::uninit();
                    if unsafe { input_events.getEvent(i, event.as_mut_ptr()) } != kResultOk {
                        continue;
                    }
                    let event = unsafe { event.assume_init() };

                    match event.r#type {
                        NOTE_ON_EVENT => {
                            let note_on_event = &unsafe { event.__field0.noteOn };
                            let ev = NoteEvent::NoteOn {
                                channel: note_on_event.channel,
                                sample_offset: event.sampleOffset,
                                note: Note::from_midi(note_on_event.pitch as _),
                            };

                            plugin.process_midi(&mut context, self.parameters.parameters_ref(), ev);
                        }
                        NOTE_OFF_EVENT => {
                            let note_off_event = &unsafe { event.__field0.noteOff };
                            let ev = NoteEvent::NoteOff {
                                channel: note_off_event.channel,
                                sample_offset: event.sampleOffset,
                                note: Note::from_midi(note_off_event.pitch as _),
                            };
                            plugin.process_midi(&mut context, self.parameters.parameters_ref(), ev);
                        }
                        _ => {}
                    }
                }
            }
        }

        plugin.process(context, self.parameters.parameters_ref());

        /*if let Some(output_param_changes) = data.output_param_changes.upgrade() {
            output_param_changes.add_parameter_data(id, index)
        }*/

        kResultOk
    }

    unsafe fn getTailSamples(&self) -> u32 {
        0
    }
}

impl<P: VST3Plugin> IPluginBaseTrait for AudioProcessor<P> {
    unsafe fn initialize(&self, context: *mut FUnknown) -> tresult {
        let old_host_context = self.host_context.take();
        if old_host_context.is_some() {
            self.host_context.set(old_host_context);
            return kResultFalse;
        }

        let host_context =
            unsafe { ComRef::from_raw(context) }.and_then(|cx| cx.cast::<IHostApplication>());
        if let Some(context) = host_context {
            self.host_context.set(Some(context));
            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn terminate(&self) -> tresult {
        self.host_context.replace(None);
        kResultOk
    }
}

impl<P: VST3Plugin> IComponentTrait for AudioProcessor<P> {
    unsafe fn getControllerClassId(&self, tuid: *mut TUID) -> tresult {
        if let Some(tuid) = unsafe { tuid.as_mut() } {
            *tuid = Factory::<P>::editor_cid_as_tuid();
            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn setIoMode(&self, _mode: IoMode) -> tresult {
        kNotImplemented
    }

    unsafe fn getBusCount(&self, type_: MediaType, dir: BusDirection) -> i32 {
        if type_ == MediaTypes_::kAudio as MediaType {
            if dir == BusDirections_::kInput as BusDirection {
                if P::AUDIO_LAYOUT.main_input.is_some() {
                    1
                } else {
                    0
                }
            } else if P::AUDIO_LAYOUT.main_output.is_some() {
                1
            } else {
                0
            }
        } else if type_ == MediaTypes_::kEvent as MediaType {
            if (dir == BusDirections_::kInput as BusDirection && P::ACCEPTS_MIDI)
                || (dir == BusDirections_::kOutput as BusDirection && P::PRODUCES_MIDI)
            {
                1
            } else {
                0
            }
        } else {
            0
        }
    }

    unsafe fn getBusInfo(
        &self,
        type_: MediaType,
        dir: BusDirection,
        index: i32,
        info: *mut BusInfo,
    ) -> tresult {
        if index < 0 {
            return kInvalidArgument;
        }
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };

        const AUDIO: MediaType = MediaTypes_::kAudio as MediaType;
        const EVENT: MediaType = MediaTypes_::kEvent as MediaType;
        const INPUT: BusDirection = BusDirections_::kInput as BusDirection;
        const OUTPUT: BusDirection = BusDirections_::kOutput as BusDirection;

        match type_ {
            AUDIO => {
                let matched_bus = (index == 0)
                    .then(|| match dir {
                        INPUT => P::AUDIO_LAYOUT.main_input.as_ref(),
                        OUTPUT => P::AUDIO_LAYOUT.main_output.as_ref(),
                        _ => None,
                    })
                    .flatten();

                if let Some(bus) = matched_bus {
                    info.channelCount = bus.channel.size() as i32;
                    info.direction = dir;
                    info.mediaType = type_;
                    strcpyw(bus.name, &mut info.name);
                    info.busType = BusTypes_::kMain as _;
                    info.flags = BusFlags_::kDefaultActive;
                    kResultOk
                } else {
                    kInvalidArgument
                }
            }
            EVENT => {
                if dir == INPUT && P::ACCEPTS_MIDI {
                    info.channelCount = 16;
                    info.direction = INPUT;
                    info.mediaType = EVENT;
                    strcpyw("MIDI in", &mut info.name);
                    info.busType = BusTypes_::kMain as _;
                    info.flags = BusFlags_::kDefaultActive;
                    kResultOk
                } else if dir == OUTPUT && P::PRODUCES_MIDI {
                    info.channelCount = 16;
                    info.direction = OUTPUT;
                    info.mediaType = EVENT;
                    strcpyw("MIDI out", &mut info.name);
                    info.busType = BusTypes_::kMain as _;
                    info.flags = BusFlags_::kDefaultActive;

                    kResultOk
                } else {
                    kInvalidArgument
                }
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn getRoutingInfo(
        &self,
        _in_info: *mut RoutingInfo,
        _out_info: *mut RoutingInfo,
    ) -> tresult {
        kNotImplemented
    }

    unsafe fn activateBus(
        &self,
        _type_: MediaType,
        _dir: BusDirection,
        _index: i32,
        _state: TBool,
    ) -> tresult {
        kResultOk
    }

    unsafe fn setActive(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn setState(&self, state: *mut IBStream) -> tresult {
        let Some(_state) = (unsafe { state.as_mut() }) else {
            return kInvalidArgument;
        };

        // TODO: Deserialize the state
        kResultOk
    }

    unsafe fn getState(&self, state: *mut IBStream) -> tresult {
        let Some(_state) = (unsafe { state.as_mut() }) else {
            return kInvalidArgument;
        };

        // TODO: Serialize the state
        kResultOk
    }
}

impl<P: VST3Plugin> IConnectionPointTrait for AudioProcessor<P> {
    unsafe fn connect(&self, other: *mut IConnectionPoint) -> tresult {
        let Some(other) = (unsafe { ComRef::from_raw(other) }) else {
            return kResultFalse;
        };

        let Some(host_context) = self.host_context.take() else {
            return kResultFalse;
        };

        // TODO: We will need some way to share messages between the plugin and the editor.
        // For instance, FFT data. Says in the VST3 docs that this API should not be called from the
        // process function (think it allocates). We could do like JUCE, and send a pointer to this
        // object to the editor (and vice versa) using the IConnectionPoint API and then implement our own system.

        let mut message_tuid = tuid_from_uuid(&IMessage::IID);
        let mut message = std::ptr::null_mut();
        if unsafe {
            host_context.createInstance(&mut message_tuid, &mut message_tuid, &mut message)
        } == kResultOk
        {}
        //;

        self.host_context.set(Some(host_context));
        kResultOk
    }

    unsafe fn disconnect(&self, _other: *mut IConnectionPoint) -> tresult {
        kResultOk
    }

    unsafe fn notify(&self, _message: *mut IMessage) -> tresult {
        kResultOk
    }
}
