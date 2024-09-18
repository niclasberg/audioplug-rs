use atomic_refcell::AtomicRefCell;
use vst3_com::vst::{IComponent, SymbolicSampleSizes, MediaTypes, BusDirections, BusTypes, BusType, BusFlags, kEmpty, kMono, kStereo};
use vst3_sys::{VST3, IID};
use vst3_sys::base::*;
use vst3_sys::utils::SharedVstPtr;
use vst3_sys::vst::{BusDirection, BusInfo, EventTypes, IAudioProcessor, IConnectionPoint, IEventList, IMessage, IParamValueQueue, IParameterChanges, IoMode, MediaType, ProcessData, ProcessModes, ProcessSetup, RoutingInfo, SpeakerArrangement};
use std::cell::RefCell;
use std::ffi::c_void;
use std::mem::MaybeUninit;

use vst3_sys as vst3_com;

use crate::midi::NoteEvent;
use crate::midi_buffer::MidiBuffer;
use crate::param::{AnyParameterMap, NormalizedValue, ParameterId, ParameterMap};
use crate::{Plugin, AudioBuffer, ProcessContext};
use super::editcontroller::EditController;
use super::util::strcpyw;

const NOTE_ON_EVENT: u16 = EventTypes::kNoteOnEvent as u16;
const NOTE_OFF_EVENT: u16 = EventTypes::kNoteOffEvent as u16;

#[VST3(implements(IComponent, IAudioProcessor, IConnectionPoint))]
pub struct Vst3Plugin<P: Plugin> {
    plugin: AtomicRefCell<P>,
    parameters: ParameterMap<P::Parameters> ,
	midi_buffer: RefCell<MidiBuffer>
}

impl<P: Plugin> Vst3Plugin<P> {
    pub const CID: IID = IID { data: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15] };

    pub fn new() -> Box<Self> {
		let parameters = ParameterMap::new(P::Parameters::default());
        Self::allocate(AtomicRefCell::new(P::new()), parameters, RefCell::new(MidiBuffer::new(1024)))
    }

    pub fn create_instance() -> *mut c_void {
        Box::into_raw(Self::new()) as *mut c_void
    }
}

impl<P: Plugin> IAudioProcessor for Vst3Plugin<P> {
    unsafe fn set_bus_arrangements(&self, _inputs: *mut SpeakerArrangement, _num_ins: i32, _outputs: *mut SpeakerArrangement, _num_outs: i32) -> tresult {
        // From the VST3 docs, we can do the following:
        // 1. Accept the provided speaker arrangement, adjust the bus:es and return true
        // 2. Otherwise, we return false. The host will then repeatedly call get_bus_arrangement,
        // allowing us to adjust. The host will then call set_bus_arrangement with the provided
        // adjustments.
        
        kResultFalse
    }

    unsafe fn get_bus_arrangement(&self, dir: BusDirection, index: i32, arr: *mut SpeakerArrangement) -> tresult {
        if arr.is_null() || index < 0 {
            return kInvalidArgument;
        }

        let bus_option = P::AUDIO_LAYOUT.get(index as usize).and_then(|audio_layout| {
            if dir == BusDirections::kInput as i32 {
                audio_layout.main_input.as_ref()
            } else {
                audio_layout.main_output.as_ref()
            }});

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
        let setup = &*setup;
        self.plugin.borrow_mut().reset(setup.sample_rate, setup.max_samples_per_block as usize);
        kResultOk
    }

    unsafe fn set_processing(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        if data.is_null() { return kInvalidArgument; }
        let data = &mut *data;

        if let Some(input_param_changes) = data.input_param_changes.upgrade() {
            let parameter_change_count = input_param_changes.get_parameter_count();
            for i in 0..parameter_change_count {
				if let Some(data) = input_param_changes.get_parameter_data(i).upgrade() {
					let param_id = ParameterId::new(data.get_parameter_id());
					let point_count = data.get_point_count();
					if point_count <= 0 {
						continue;
					}

					if let Some(param_ref) = self.parameters.get_by_id(param_id) {
						let mut value = 0.0;
						let mut sample_offset = 0;
						if data.get_point(point_count - 1, &mut sample_offset as *mut _, &mut value as *mut _) == kResultOk {
							param_ref.internal_set_value_normalized(NormalizedValue::from_f64_unchecked(value));
						}
					}
				}
			}
        }

        if data.inputs.is_null() || data.outputs.is_null() {
            return kResultOk;
        }

		let mut midi_buffer = self.midi_buffer.borrow_mut();
		midi_buffer.reset();
		if let Some(input_events) = data.input_events.upgrade() {
			let event_count = input_events.get_event_count();
			let mut event = MaybeUninit::uninit();
			for i in 0..event_count {
				if input_events.get_event(i, event.as_mut_ptr()) != kResultOk {
					continue;
				}
				let event = event.assume_init();

				match event.type_ {
					NOTE_ON_EVENT => {
						let note_on_event = &event.event.note_on;
						midi_buffer.push(NoteEvent::NoteOn { 
							channel: note_on_event.channel, 
							sample_offset: event.sample_offset, 
							pitch: note_on_event.pitch 
						});
					},
					NOTE_OFF_EVENT => {
						let note_off_event = &event.event.note_off;
						midi_buffer.push(NoteEvent::NoteOn { 
							channel: note_off_event.channel, 
							sample_offset: event.sample_offset, 
							pitch: note_off_event.pitch 
						});
					},
					_ => {}
				}
			}
		}

        let input = AudioBuffer::from_ptr((*data.inputs).buffers as *const *mut _, (*data.inputs).num_channels as usize, data.num_samples as usize);
        let mut output = AudioBuffer::from_ptr((*data.outputs).buffers as *const *mut _, (*data.outputs).num_channels as usize, data.num_samples as usize);

        let context = ProcessContext {
            input: &input,
            output: &mut output,
			rendering_offline: data.process_mode == ProcessModes::kOffline as i32,
			midi_input: &midi_buffer
        };

        self.plugin.borrow_mut().process(context, self.parameters.parameters_ref());

        /*if let Some(output_param_changes) = data.output_param_changes.upgrade() {
            output_param_changes.add_parameter_data(id, index)
        }*/

        kResultOk
    }

    unsafe fn get_tail_samples(&self) -> u32 {
        0
    }
}

impl<P: Plugin> IPluginBase for Vst3Plugin<P> {
    unsafe fn initialize(&self, _context: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl<P: Plugin> IComponent for Vst3Plugin<P> {
    unsafe fn get_controller_class_id(&self, tuid: *mut IID) -> tresult {
        *tuid = EditController::<P::Editor>::CID;
        kResultOk
    }

    unsafe fn set_io_mode(&self, _mode: IoMode) -> tresult {
        kNotImplemented
    }

    unsafe fn get_bus_count(&self, type_: MediaType, dir: BusDirection) -> i32 {
        if type_ == MediaTypes::kAudio as MediaType {
            if dir == BusDirections::kInput as BusDirection {
                1
            } else {
                1
            }
        } else if type_ == MediaTypes::kEvent as MediaType {
            if (dir == BusDirections::kInput as BusDirection && P::ACCEPTS_MIDI) ||  
                (dir == BusDirections::kOutput as BusDirection && P::PRODUCES_MIDI) {
                1
            } else {
                0
            }
        } else {
            0
        }
    }

    unsafe fn get_bus_info(&self, type_: MediaType, dir: BusDirection, index: i32, info: *mut BusInfo) -> tresult {
        if info.is_null() || index < 0{
            return kInvalidArgument;
        }

        let info = &mut *info;
		const AUDIO: MediaType = MediaTypes::kAudio as MediaType;
		const EVENT: MediaType = MediaTypes::kEvent as MediaType;
		const INPUT: BusDirection = BusDirections::kInput as BusDirection;	
		const OUTPUT: BusDirection = BusDirections::kOutput as BusDirection;

		match type_ {
			AUDIO => {
				let matched_bus = P::AUDIO_LAYOUT.get(index as usize)
					.and_then(|layout| match dir {
						INPUT => layout.main_input.as_ref(),
						OUTPUT => layout.main_output.as_ref(),
						_ => None,
					});

				if let Some(bus) = matched_bus {
					info.channel_count = bus.channel.size() as i32;
					info.direction = dir;
					info.media_type = type_;
					strcpyw(bus.name, &mut info.name);
					info.bus_type = BusTypes::kMain as BusType;
					info.flags = BusFlags::kDefaultActive as u32;
					kResultOk
				} else {
					kInvalidArgument
				}
			},
			EVENT => {
				if dir == INPUT && P::ACCEPTS_MIDI {
					info.channel_count = 16;
					info.direction = INPUT;
					info.media_type = EVENT;
					strcpyw("MIDI in", &mut info.name);
					info.bus_type = BusTypes::kMain as BusType;
					info.flags = BusFlags::kDefaultActive as u32;
					kResultOk
				} else if dir == OUTPUT && P::PRODUCES_MIDI {
					info.channel_count = 16;
					info.direction = OUTPUT;
					info.media_type = EVENT;
					strcpyw("MIDI out", &mut info.name);
					info.bus_type = BusTypes::kMain as BusType;
					info.flags = BusFlags::kDefaultActive as u32;

					kResultOk
				} else {
					kInvalidArgument	
				}
			},
			_ => kInvalidArgument
		}
    }

    unsafe fn get_routing_info(&self, _in_info: *mut RoutingInfo, _out_info: *mut RoutingInfo) -> tresult {
        kNotImplemented
    }

    unsafe fn activate_bus(&self, _type_: MediaType, _dir: BusDirection, _index: i32, _state: TBool) -> tresult {
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

impl<P: Plugin> IConnectionPoint for Vst3Plugin<P> {
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