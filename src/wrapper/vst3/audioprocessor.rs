use atomic_refcell::AtomicRefCell;
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::rc::Rc;
use vst3_com::vst::{
    BusDirections, BusFlags, BusTypes, IComponent, MediaTypes, SymbolicSampleSizes, kEmpty, kMono,
    kStereo,
};
use vst3_sys::base::*;
use vst3_sys::utils::SharedVstPtr;
use vst3_sys::vst::{
    BusDirection, BusInfo, EventTypes, IAudioProcessor, IConnectionPoint, IEventList, IMessage,
    IParamValueQueue, IParameterChanges, IoMode, MediaType, ProcessData, ProcessModes,
    ProcessSetup, RoutingInfo, SpeakerArrangement,
};
use vst3_sys::{IID, VST3};

use vst3_sys as vst3_com;

use super::util::strcpyw;
use crate::midi::{Note, NoteEvent};
use crate::param::{AnyParameterMap, NormalizedValue, ParameterId, ParameterMap, Params};
use crate::{AudioBuffer, MidiProcessContext, ProcessContext, ProcessInfo, VST3Plugin};

const NOTE_ON_EVENT: u16 = EventTypes::kNoteOnEvent as u16;
const NOTE_OFF_EVENT: u16 = EventTypes::kNoteOffEvent as u16;

#[VST3(implements(IComponent, IAudioProcessor, IConnectionPoint))]
pub struct AudioProcessor<P: VST3Plugin> {
    plugin: AtomicRefCell<P>,
    parameters: Rc<ParameterMap<P::Parameters>>,
}

impl<P: VST3Plugin> AudioProcessor<P> {
    pub fn new() -> Box<Self> {
        let parameters = ParameterMap::new(P::Parameters::new());
        Self::allocate(AtomicRefCell::new(P::new()), parameters)
    }

    pub fn create_instance() -> *mut c_void {
        Box::into_raw(Self::new()) as *mut c_void
    }
}

impl<P: VST3Plugin> IAudioProcessor for AudioProcessor<P> {
    unsafe fn set_bus_arrangements(
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

    unsafe fn get_bus_arrangement(
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
                if dir == BusDirections::kInput as i32 {
                    P::AUDIO_LAYOUT.main_input.as_ref()
                } else {
                    P::AUDIO_LAYOUT.main_output.as_ref()
                }
            })
            .flatten();

        if let Some(bus) = bus_option {
            *arr = match bus.channel {
                crate::ChannelType::Empty => kEmpty,
                crate::ChannelType::Mono => kMono,
                crate::ChannelType::Stereo => kStereo,
            };
            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn can_process_sample_size(&self, symbolic_sample_size: i32) -> tresult {
        if symbolic_sample_size == SymbolicSampleSizes::kSample32 as i32 {
            kResultOk
        } else {
            kResultFalse
        }
    }

    unsafe fn get_latency_samples(&self) -> u32 {
        self.plugin.borrow().latency_samples() as u32
    }

    unsafe fn setup_processing(&self, setup: *const ProcessSetup) -> tresult {
        let Some(setup) = (unsafe { setup.as_ref() }) else {
            return kInvalidArgument;
        };
        self.plugin
            .borrow_mut()
            .prepare(setup.sample_rate, setup.max_samples_per_block as usize);
        kResultOk
    }

    // Called with true before processing starts, and false after. Can be called from both UI and
    // realtime thread
    unsafe fn set_processing(&self, state: TBool) -> tresult {
        if state == 0 {
            self.plugin.borrow_mut().reset();
        }
        kResultOk
    }

    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        let Some(data) = (unsafe { data.as_mut() }) else {
            return kInvalidArgument;
        };
        let Some(process_context) = (unsafe { data.context.as_mut() }) else {
            return kInvalidArgument;
        };

        if let Some(input_param_changes) = data.input_param_changes.upgrade() {
            let parameter_change_count = unsafe { input_param_changes.get_parameter_count() };
            for i in 0..parameter_change_count {
                let parameter_data = unsafe { input_param_changes.get_parameter_data(i) };
                if let Some(data) = parameter_data.upgrade() {
                    let param_id = ParameterId(unsafe { data.get_parameter_id() });
                    let point_count = unsafe { data.get_point_count() };
                    if point_count <= 0 {
                        continue;
                    }

                    if let Some(param_ref) = self.parameters.get_by_id(param_id) {
                        let mut value = 0.0;
                        let mut sample_offset = 0;
                        if unsafe {
                            data.get_point(
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
        if data.num_samples == 0 {
            return kResultOk;
        }

        let input = if data.inputs.is_null() {
            AudioBuffer::empty()
        } else {
            unsafe {
                AudioBuffer::from_ptr(
                    (*data.inputs).buffers as *mut *mut _,
                    (*data.inputs).num_channels as usize,
                    data.num_samples as usize,
                )
            }
        };

        let mut output = if data.outputs.is_null() {
            AudioBuffer::empty()
        } else {
            unsafe {
                AudioBuffer::from_ptr(
                    (*data.outputs).buffers as *mut *mut _,
                    (*data.outputs).num_channels as usize,
                    data.num_samples as usize,
                )
            }
        };

        let info = ProcessInfo {
            rendering_offline: data.process_mode == ProcessModes::kOffline as i32,
            sample_rate: process_context.sample_rate,
        };

        let context = ProcessContext {
            input: &input,
            output: &mut output,
            info,
        };

        let mut plugin = self.plugin.borrow_mut();
        if P::ACCEPTS_MIDI {
            let mut context = MidiProcessContext { info };
            if let Some(input_events) = data.input_events.upgrade() {
                let event_count = unsafe { input_events.get_event_count() };
                for i in 0..event_count {
                    let mut event = MaybeUninit::uninit();
                    if unsafe { input_events.get_event(i, event.as_mut_ptr()) } != kResultOk {
                        continue;
                    }
                    let event = unsafe { event.assume_init() };

                    match event.type_ {
                        NOTE_ON_EVENT => {
                            let note_on_event = &unsafe { event.event.note_on };
                            let ev = NoteEvent::NoteOn {
                                channel: note_on_event.channel,
                                sample_offset: event.sample_offset,
                                note: Note::from_midi(note_on_event.pitch as _),
                            };

                            plugin.process_midi(&mut context, self.parameters.parameters_ref(), ev);
                        }
                        NOTE_OFF_EVENT => {
                            let note_off_event = &unsafe { event.event.note_off };
                            let ev = NoteEvent::NoteOff {
                                channel: note_off_event.channel,
                                sample_offset: event.sample_offset,
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

    unsafe fn get_tail_samples(&self) -> u32 {
        0
    }
}

impl<P: VST3Plugin> IPluginBase for AudioProcessor<P> {
    unsafe fn initialize(&self, _context: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl<P: VST3Plugin> IComponent for AudioProcessor<P> {
    unsafe fn get_controller_class_id(&self, tuid: *mut IID) -> tresult {
        if let Some(tuid) = unsafe { tuid.as_mut() } {
            *tuid = IID {
                data: P::EDITOR_UUID,
            };
            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn set_io_mode(&self, _mode: IoMode) -> tresult {
        kNotImplemented
    }

    unsafe fn get_bus_count(&self, type_: MediaType, dir: BusDirection) -> i32 {
        if type_ == MediaTypes::kAudio as MediaType {
            if dir == BusDirections::kInput as BusDirection {
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
        } else if type_ == MediaTypes::kEvent as MediaType {
            if (dir == BusDirections::kInput as BusDirection && P::ACCEPTS_MIDI)
                || (dir == BusDirections::kOutput as BusDirection && P::PRODUCES_MIDI)
            {
                1
            } else {
                0
            }
        } else {
            0
        }
    }

    unsafe fn get_bus_info(
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

        const AUDIO: MediaType = MediaTypes::kAudio as MediaType;
        const EVENT: MediaType = MediaTypes::kEvent as MediaType;
        const INPUT: BusDirection = BusDirections::kInput as BusDirection;
        const OUTPUT: BusDirection = BusDirections::kOutput as BusDirection;

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
                    info.channel_count = bus.channel.size() as i32;
                    info.direction = dir;
                    info.media_type = type_;
                    strcpyw(bus.name, &mut info.name);
                    info.bus_type = BusTypes::kMain as vst3_sys::vst::BusType;
                    info.flags = BusFlags::kDefaultActive as u32;
                    kResultOk
                } else {
                    kInvalidArgument
                }
            }
            EVENT => {
                if dir == INPUT && P::ACCEPTS_MIDI {
                    info.channel_count = 16;
                    info.direction = INPUT;
                    info.media_type = EVENT;
                    strcpyw("MIDI in", &mut info.name);
                    info.bus_type = BusTypes::kMain as vst3_sys::vst::BusType;
                    info.flags = BusFlags::kDefaultActive as u32;
                    kResultOk
                } else if dir == OUTPUT && P::PRODUCES_MIDI {
                    info.channel_count = 16;
                    info.direction = OUTPUT;
                    info.media_type = EVENT;
                    strcpyw("MIDI out", &mut info.name);
                    info.bus_type = BusTypes::kMain as vst3_sys::vst::BusType;
                    info.flags = BusFlags::kDefaultActive as u32;

                    kResultOk
                } else {
                    kInvalidArgument
                }
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn get_routing_info(
        &self,
        _in_info: *mut RoutingInfo,
        _out_info: *mut RoutingInfo,
    ) -> tresult {
        kNotImplemented
    }

    unsafe fn activate_bus(
        &self,
        _type_: MediaType,
        _dir: BusDirection,
        _index: i32,
        _state: TBool,
    ) -> tresult {
        kResultOk
    }

    unsafe fn set_active(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn set_state(&self, state: SharedVstPtr<dyn IBStream>) -> tresult {
        // TODO: Deserialize the state
        if let Some(_state) = state.upgrade() {
            kResultOk
        } else {
            kResultFalse
        }
    }

    unsafe fn get_state(&self, state: SharedVstPtr<dyn IBStream>) -> tresult {
        // TODO: Serialize the state
        if let Some(_state) = state.upgrade() {
            kResultOk
        } else {
            kResultFalse
        }
    }
}

impl<P: VST3Plugin> IConnectionPoint for AudioProcessor<P> {
    unsafe fn connect(&self, _other: SharedVstPtr<dyn IConnectionPoint>) -> tresult {
        // TODO: We will need some way to share messages between the plugin and the editor.
        // For instance, FFT data. Says in the VST3 docs that this API should not be called from the
        // process function (think it allocates). We could do like JUCE, and send a pointer to this
        // object to the editor (and vice versa) using the IConnectionPoint API and then implement our own system.
        kResultOk
    }

    unsafe fn disconnect(&self, _other: SharedVstPtr<dyn IConnectionPoint>) -> tresult {
        kResultOk
    }

    unsafe fn notify(&self, _message: SharedVstPtr<dyn IMessage>) -> tresult {
        kResultOk
    }
}
