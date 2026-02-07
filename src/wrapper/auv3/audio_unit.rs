use std::ffi::{CStr, CString};
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::{OnceLock, Arc};

use atomic_refcell::AtomicRefCell;
use block2::{Block, RcBlock};
use objc2::runtime::{AnyClass, AnyObject, Bool, ClassBuilder, Sel};
use objc2::{AllocAnyThread, extern_class, extern_methods, msg_send, rc::Retained};
use objc2::{ClassType, Encoding, RefEncode, sel};
use objc2_audio_toolbox::{
    AUAudioFrameCount, AUAudioUnit, AUAudioUnitBusArray, AUAudioUnitBusType, AUAudioUnitStatus,
    AUParameterTree, AURenderEventType, AURenderPullInputBlock, AudioComponentDescription,
    AudioComponentInstantiationOptions, AudioUnitRenderActionFlags,
};
use objc2_avf_audio::AVAudioFormat;
use objc2_core_audio_types::{AudioBufferList, AudioTimeStamp};
use objc2_core_foundation::CGFloat;
use objc2_foundation::{
    NSArray, NSError, NSIndexSet, NSInteger, NSNumber, NSObject, NSTimeInterval,
};
use uuid::Uuid;

use super::buffers::create_buffers;
use super::{buffers::BusBuffer, render_event::AURenderEvent, utils::create_parameter_tree};
use crate::param::{AnyParameterMap, ParameterId, ParameterMap, Params, PlainValue};
use crate::{AudioBuffer, Plugin, ProcessContext, ProcessInfo};

const DEFAULT_SAMPLE_RATE: f64 = 44100.0;

// Missing definitions from objc2
pub type AUInternalRenderBlock = Block<
    dyn Fn(
        NonNull<AudioUnitRenderActionFlags>,
        NonNull<AudioTimeStamp>,
        AUAudioFrameCount,
        NSInteger,
        *mut AudioBufferList,
        *const AURenderEvent,
        AURenderPullInputBlock,
    ) -> AUAudioUnitStatus,
>;
pub type AUInternalRenderRcBlock = RcBlock<
    dyn Fn(
        NonNull<AudioUnitRenderActionFlags>,
        NonNull<AudioTimeStamp>,
        AUAudioFrameCount,
        NSInteger,
        *mut AudioBufferList,
        *const AURenderEvent,
        AURenderPullInputBlock,
    ) -> AUAudioUnitStatus,
>;

extern_class!(
    #[unsafe(super(NSObject))]
    pub struct AUAudioUnitViewConfiguration;
);

impl AUAudioUnitViewConfiguration {
    extern_methods!(
        #[unsafe(method(width))]
        pub fn width(&self) -> CGFloat;

        #[unsafe(method(height))]
        pub fn height(&self) -> CGFloat;
    );
}

struct Inner<P: Plugin> {
    plugin: P,
    parameters: Rc<ParameterMap<P::Parameters>>,
    input_buffer: BusBuffer,
    output_buffer: BusBuffer,
    rendering_offline: bool,
    sample_rate: f64,
    last_sample_time: f64,
}

impl<P: Plugin> Inner<P> {
    fn new(
        plugin: P,
        parameters: Rc<ParameterMap<P::Parameters>>,
        input_buffer: BusBuffer,
        output_buffer: BusBuffer,
    ) -> Self {
        Self {
            plugin,
            parameters,
            input_buffer,
            output_buffer,
            rendering_offline: false,
            sample_rate: DEFAULT_SAMPLE_RATE,
            last_sample_time: f64::MAX,
        }
    }

    fn process_events(&mut self, realtime_event_list_head: *const AURenderEvent) {
        let mut event_list = realtime_event_list_head;
        while !event_list.is_null() {
            let header = unsafe { &(&*event_list).head };
            //let ev_timestamp = header.event_sample_time;
            match header.event_type {
                AURenderEventType::Parameter | AURenderEventType::ParameterRamp => {
                    let parameter_event = unsafe { &(&*event_list).parameter };
                    let param_id = ParameterId(parameter_event.parameter_address as _);
                    if let Some(param_ref) = self.parameters.get_by_id(param_id) {
                        param_ref.set_value_plain(PlainValue::new(parameter_event.value as _));
                    }
                }
                AURenderEventType::MIDI => {
                    //let midi_event = unsafe { &(&*event_list).midi };
                }
                AURenderEventType::MIDIEventList => {
                    //let midi_event_list = unsafe { &(&*event_list).midi_events_list };
                }
                _ => {}
            }
            event_list = header.next;
        }
    }

    fn render(
        &mut self,
        _action_flags: NonNull<AudioUnitRenderActionFlags>,
        timestamp: NonNull<AudioTimeStamp>,
        _frame_count: AUAudioFrameCount,
        _output_bus_number: NSInteger,
        _output_data: *mut AudioBufferList,
        realtime_event_list_head: *const AURenderEvent,
        _pull_input_block: AURenderPullInputBlock,
    ) -> AUAudioUnitStatus {
        // This method gets called once for each bus, we just want to render all busses at the same time,
        // so check the sample time.
        let sample_time = unsafe { timestamp.as_ref() }.mSampleTime;
        if (sample_time - self.last_sample_time).abs() > 1.0e-6 {
            self.process_events(realtime_event_list_head);

            let input = AudioBuffer::empty();
            let mut output = AudioBuffer::empty();

            //self.input_buffer.pull_inputs(action_flags, timestamp, frame_count, input_bus_number, pull_input_block)

            let info = ProcessInfo {
                rendering_offline: self.rendering_offline,
                sample_rate: DEFAULT_SAMPLE_RATE,
            };

            let context = ProcessContext {
                input: &input,
                output: &mut output,
                info,
            };

            self.plugin
                .process(context, self.parameters.parameters_ref());
        }
        0
    }

    fn allocate_render_resources(&mut self, max_frames_to_render: usize) {
        self.sample_rate = self
            .input_buffer
            .sample_rate()
            .or_else(|| self.output_buffer.sample_rate())
            .unwrap_or(DEFAULT_SAMPLE_RATE);

        self.input_buffer.allocate(max_frames_to_render);
        self.output_buffer.allocate(max_frames_to_render);

        self.plugin.prepare(self.sample_rate, max_frames_to_render);
    }

    fn deallocate_render_resources(&mut self) {
        self.input_buffer.deallocate();
        self.output_buffer.deallocate();
    }
}

#[repr(C)]
pub struct MyAudioUnit<P: Plugin> {
    inner: Arc<AtomicRefCell<Inner<P>>>,
    internal_render_block: AUInternalRenderRcBlock,
    inputs: Retained<AUAudioUnitBusArray>,
    outputs: Retained<AUAudioUnitBusArray>,
    channel_capabilities: Retained<NSArray<NSNumber>>,
    parameter_tree: Retained<AUParameterTree>,
}

unsafe impl<P: Plugin> RefEncode for MyAudioUnit<P> {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Encoding::Struct("?", &[]));
}

const AUDIOUNIT_VAR_NAME: &CStr = c"_impl";
static OBJC_CLASS: OnceLock<&'static AnyClass> = OnceLock::new();

impl<P: Plugin> MyAudioUnit<P> {
	pub fn new_with_component_descriptor_error(component_description: AudioComponentDescription,
        out_error: *mut *mut NSError,
    ) -> Option<Retained<AUAudioUnit>> {
		unsafe {
			msg_send![
				msg_send![Self::class(), alloc], 
				initWithComponentDescription: component_description, 
				error: out_error]
		}
	}

	pub fn class() -> &'static AnyClass {
		OBJC_CLASS.get_or_init(|| {
			let name = format!("AP_AudioInit{}", Uuid::new_v4().as_simple());
			let name = CString::new(name).unwrap();
			let mut builder =
				ClassBuilder::new(&name, AUAudioUnit::class()).expect("Class already registered");

			builder.add_ivar::<*mut MyAudioUnit<P>>(AUDIOUNIT_VAR_NAME);

			unsafe {
				// Class methods
				builder.add_class_method(
					sel!(providesUserInterface),
					Self::provides_user_interface as unsafe extern "C-unwind" fn(_, _) -> _,
				);

                // Instance methods
				builder.add_method(
					sel!(initWithComponentDescription:options:error:),
					Self::init_with_component_description_options_error
						as unsafe extern "C-unwind" fn(_, _, _, _, _) -> _,
				);
				builder.add_method(
					sel!(dealloc),
					Self::dealloc as unsafe extern "C-unwind" fn(_, _) -> _,
				);
				builder.add_method(
					sel!(inputBusses),
					Self::input_busses as unsafe extern "C-unwind" fn(_, _) -> _,
				);
				builder.add_method(
					sel!(outputBusses),
					Self::output_busses as unsafe extern "C-unwind" fn(_, _) -> _,
				);
				builder.add_method(
					sel!(channelCapabilities),
					Self::channel_capabilities as unsafe extern "C-unwind" fn(_, _) -> _,
				);
				builder.add_method(
					sel!(parameterTree),
					Self::parameter_tree as unsafe extern "C-unwind" fn(_, _) -> _,
				);
				builder.add_method(
					sel!(supportedViewConfiguations:),
					Self::supported_view_configurations as unsafe extern "C-unwind" fn(_, _, _) -> _,
				);
				builder.add_method(
					sel!(internalRenderBlock),
					Self::internal_render_block as unsafe extern "C-unwind" fn(_, _) -> _,
				);
				builder.add_method(
					sel!(allocateRenderResourcesAndReturnError:),
					Self::allocate_render_resources_and_return_error as unsafe extern "C-unwind" fn(_, _, _) -> _,
				);
				builder.add_method(
					sel!(deallocateRenderResources),
					Self::deallocate_render_resources as unsafe extern "C-unwind" fn(_, _) -> _,
				);
				builder.add_method(
					sel!(latency),
					Self::latency as unsafe extern "C-unwind" fn(_, _) -> _,
				);
				builder.add_method(
					sel!(tailTime),
					Self::tail_time as unsafe extern "C-unwind" fn(_, _) -> _,
				);
			}

			builder.register()
		})
	}

	unsafe extern "C-unwind" fn init_with_component_description_options_error(
        this: &mut AnyObject,
        _cmd: Sel,
        component_description: AudioComponentDescription,
        options: AudioComponentInstantiationOptions,
        out_error: *mut *mut NSError,
    ) -> Option<&mut AnyObject> {
		println!("INIT");
        let this: Option<&mut AnyObject> = unsafe {
            msg_send![super(this, AUAudioUnit::class()), 
				initWithComponentDescription: component_description, 
				options: options, 
				error: out_error]
        };
		this.map(|this| {
			let audio_unit = this.downcast_ref().unwrap();
            let ivar = this.class().instance_variable(AUDIOUNIT_VAR_NAME).unwrap();
            let wrapper = Box::new(Self::new(&audio_unit));
			*unsafe { ivar.load_mut::<*mut Self>(this) } = Box::into_raw(wrapper);
			this
		})
    }

    fn new(audio_unit: &AUAudioUnit) -> Self {
        let format = unsafe {
            AVAudioFormat::initStandardFormatWithSampleRate_channels(
                AVAudioFormat::alloc(),
                DEFAULT_SAMPLE_RATE,
                2,
            )
        }
        .unwrap();

        let parameters = ParameterMap::new(P::Parameters::new());
        let parameter_tree = create_parameter_tree(parameters.clone());
        let (input_buffer, output_buffer) = create_buffers(&format, &P::AUDIO_LAYOUT);

        let input_bus_array = NSArray::from_retained_slice(input_buffer.buses());
        let output_bus_array = NSArray::from_retained_slice(output_buffer.buses());
        let inputs = unsafe {
            AUAudioUnitBusArray::initWithAudioUnit_busType_busses(
                AUAudioUnitBusArray::alloc(),
                &audio_unit,
                AUAudioUnitBusType::Input,
                &input_bus_array,
            )
        };
        let outputs = unsafe {
            AUAudioUnitBusArray::initWithAudioUnit_busType_busses(
                AUAudioUnitBusArray::alloc(),
                &audio_unit,
                AUAudioUnitBusType::Output,
                &output_bus_array,
            )
        };

        let channel_capabilities =
            NSArray::from_retained_slice(&[NSNumber::new_i16(2), NSNumber::new_i16(2)]);

        // TODO: Use NSExtensionContext.hostBundleIdentifier to get the bundle name
        let plugin = P::new(crate::HostInfo {
            name: "AU Host".to_string(),
        });

        let inner = Inner::new(plugin, parameters, input_buffer, output_buffer);
        let inner = Arc::new(AtomicRefCell::new(inner));
        let internal_render_block = {
            let inner = inner.clone();
            AUInternalRenderRcBlock::new(
                move |flags,
                      timestamp,
                      frame_count,
                      channels,
                      buffers,
                      events,
                      pull_input_block|
                      -> AUAudioUnitStatus {
                    inner.borrow_mut().render(
                        flags,
                        timestamp,
                        frame_count,
                        channels,
                        buffers,
                        events,
                        pull_input_block,
                    )
                },
            )
        };

        Self {
            inner,
            internal_render_block,
            inputs,
            outputs,
            channel_capabilities,
            parameter_tree,
        }
    }

	unsafe extern "C-unwind" fn dealloc(this: &AUAudioUnit, _cmd: Sel) {
		let ivar = this.class().instance_variable(AUDIOUNIT_VAR_NAME).unwrap();
		let wrapper: &*mut MyAudioUnit<P> = unsafe { ivar.load(this) };
		drop(unsafe { Box::from_raw(*wrapper) });
		unsafe { msg_send![super(this, AUAudioUnit::class()), dealloc] }
	}

	unsafe fn get_self(this: &AnyObject) -> &Self {
		let ivar = this.class().instance_variable(AUDIOUNIT_VAR_NAME).unwrap();
		let wrapper: &*const MyAudioUnit<P> = unsafe { ivar.load(this) };
		unsafe { wrapper.as_ref() }.unwrap()
	}

    unsafe extern "C-unwind" fn input_busses(
        this: &AUAudioUnit,
        _cmd: Sel,
    ) -> &AUAudioUnitBusArray {
        &unsafe { Self::get_self(this) }.inputs
    }

    unsafe extern "C-unwind" fn output_busses(
        this: &AUAudioUnit,
        _cmd: Sel,
    ) -> &AUAudioUnitBusArray {
        &unsafe { Self::get_self(this) }.outputs
    }

	unsafe extern "C-unwind" fn channel_capabilities(this: &AUAudioUnit, _cmd: Sel) -> *mut NSArray<NSNumber> {
        Retained::into_raw(unsafe { Self::get_self(this) }.channel_capabilities.clone())
    }

	unsafe extern "C-unwind" fn provides_user_interface(_cls: &AnyClass, _cmd: Sel) -> Bool {
        Bool::YES
    }

	unsafe extern "C-unwind" fn parameter_tree(this: &AUAudioUnit, _cmd: Sel) -> *mut AUParameterTree {
        Retained::into_raw(unsafe { Self::get_self(this) }.parameter_tree.clone())
    }

    unsafe extern "C-unwind" fn supported_view_configurations(
        _this: &AUAudioUnit, _cmd: Sel,
        available_view_configurations: &NSArray<AUAudioUnitViewConfiguration>,
    ) -> *mut NSIndexSet {
        Retained::into_raw(NSIndexSet::indexSetWithIndexesInRange(
            (0..available_view_configurations.count()).into(),
        ))
    }

    unsafe extern "C-unwind" fn internal_render_block(this: &AUAudioUnit, _cmd: Sel) -> *mut AUInternalRenderBlock {
        RcBlock::into_raw(unsafe { Self::get_self(this) }.internal_render_block.clone())
    }

    unsafe extern "C-unwind" fn allocate_render_resources_and_return_error(this: &AUAudioUnit, _cmd: Sel, error: *mut *mut NSError) -> Bool {
        let max_frames = unsafe { this.maximumFramesToRender() };
        unsafe { Self::get_self(this) }
            .inner
            .borrow_mut()
            .allocate_render_resources(max_frames as _);
        unsafe { msg_send![super(this, AUAudioUnit::class()), allocateRenderResourcesAndReturnError: error] }
    }

    unsafe extern "C-unwind" fn deallocate_render_resources(this: &AUAudioUnit, _cmd: Sel) {
        unsafe { Self::get_self(this) }
            .inner
            .borrow_mut()
            .deallocate_render_resources();
        unsafe { msg_send![super(this, AUAudioUnit::class()), deallocateRenderResources] }
    }

	unsafe extern "C-unwind" fn latency(this: &AUAudioUnit, _cmd: Sel) -> NSTimeInterval {
        let inner = unsafe { Self::get_self(this) }.inner.borrow();
        inner.plugin.latency_samples() as f64 / inner.sample_rate
    }

    unsafe extern "C-unwind" fn tail_time(this: &AUAudioUnit, _cmd: Sel) -> NSTimeInterval {
        unsafe { Self::get_self(this) }.inner.borrow().plugin.tail_time().as_secs_f64()
    }
}

#[cfg(test)]
mod test {
	use crate::{AudioLayout, GenericEditor, HostInfo, platform::four_cc};
	use super::*;

	struct TestPlugin;
	impl Plugin for TestPlugin {
		const NAME: &'static str = "test";
		const VENDOR: &'static str = "test";
		const URL: &'static str = "www.test.com";
		const EMAIL: &'static str = "test@test.com";
		const AUDIO_LAYOUT: AudioLayout = AudioLayout::EMPTY;
		type Editor = GenericEditor<()>;
		type Parameters = ();

		fn new(_: HostInfo) -> Self {
			Self {}
		}

		fn prepare(&mut self, _sample_rate: f64, _max_samples_per_frame: usize) {}

		fn process(&mut self, _context: ProcessContext, _parameters: &()) {}
	}

	#[test]
	fn test_create_audio_unit() {
		let desc: AudioComponentDescription = AudioComponentDescription {
			componentType: four_cc(b"aufx"),
			componentSubType: four_cc(b"demo"),
			componentManufacturer: four_cc(b"Nibe"),
			componentFlags: 0,
			componentFlagsMask: 0,
		};
		let mut error: *mut NSError = std::ptr::null_mut();
		let audio_unit: Option<Retained<AUAudioUnit>> = MyAudioUnit::<TestPlugin>::new_with_component_descriptor_error(desc, &mut error);
		assert!(audio_unit.is_some());
	}
}
