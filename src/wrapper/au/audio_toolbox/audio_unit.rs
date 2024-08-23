use bitflags::bitflags;
use block2::{Block, RcBlock};
use objc2_foundation::{NSArray, NSError, NSInteger, NSNumber, NSString, NSTimeInterval, NSUInteger};
use objc2::{extern_class, extern_methods, mutability, rc::{Allocated, Id, Retained}, runtime::{AnyClass, Bool, NSObject}, ClassType, Encode, Encoding, RefEncode};

use crate::platform::mac::core_audio::{AudioBufferList, AudioTimeStamp};
use super::{AUAudioUnitBusArray, AUParameterTree, AURenderEvent, AUViewControllerBase, AudioComponentDescription};

pub type AUAudioUnitStatus = i32;
pub type AUAudioFrameCount = u32;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AudioUnitRenderActionFlags(pub NSUInteger);
bitflags! {
	impl AudioUnitRenderActionFlags: NSUInteger {
		/// Called on a render notification Proc - which is called either before or after the render operation of the audio unit. If this flag is set, the proc is being called before the render operation is performed.
		const kAudioUnitRenderAction_PreRender = (1 << 2);
		/// Called on a render notification Proc - which is called either before or after the render operation of the audio unit. If this flag is set, the proc is being called after the render operation is completed.
		const kAudioUnitRenderAction_PostRender = (1 << 3);
		/// This flag can be set in a render input callback (or in the audio unit's render operation itself) and is used to indicate that the render buffer contains only silence. It can then be used by the caller as a hint to whether the buffer needs to be processed or not.
		const kAudioUnitRenderAction_OutputIsSilence = (1 << 4);
		/// This is used with offline audio units (of type 'auol'). It is used when an offline unit is being preflighted, which is performed prior to the actual offline rendering actions are performed. It is used for those cases where the offline process needs it (for example, with an offline unit that normalizes an audio file, it needs to see all of the audio data first before it can perform its normalization).
		const kAudioOfflineUnitRenderAction_Preflight = (1 << 5);
		/// Once an offline unit has been successfully preflighted, it is then put into its render mode. So this flag is set to indicate to the audio unit that it is now in that state and that it should perform its processing on the input data.
		const kAudioOfflineUnitRenderAction_Render = (1 << 6);
		/// This flag is set when an offline unit has completed either its preflight or performed render operation.
		const kAudioOfflineUnitRenderAction_Complete = (1 << 7);
		/// If this flag is set on the post-render call an error was returned by the audio unit's render operation. In this case, the error can be retrieved through the lastRenderError property and the audio data in ioData handed to the post-render notification will be invalid.
		const kAudioUnitRenderAction_PostRenderError = (1 << 8);
		/// If this flag is set, then checks that are done on the arguments provided to render are not performed. This can be useful to use to save computation time in situations where you are sure you are providing the correct arguments and structures to the various render calls.
		const kAudioUnitRenderAction_DoNotCheckRenderArgs = (1 << 9);
	}
}

unsafe impl Encode for AudioUnitRenderActionFlags {
    const ENCODING: Encoding = NSUInteger::ENCODING;
}

unsafe impl RefEncode for AudioUnitRenderActionFlags {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AudioComponentInstantiationOptions(pub u32);
bitflags!{
	impl AudioComponentInstantiationOptions: u32 {
		const kAudioComponentInstantiation_LoadOutOfProcess = 1;
		const kAudioComponentInstantiation_LoadInProcess = 2;
		const kAudioComponentInstantiation_LoadedRemotely = 1 << 31;
	}
}

unsafe impl Encode for AudioComponentInstantiationOptions {
    const ENCODING: Encoding = u32::ENCODING;
}

unsafe impl RefEncode for AudioComponentInstantiationOptions {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

pub type AURenderPullInputBlock = Block<dyn Fn(*mut AudioUnitRenderActionFlags, *const AudioTimeStamp, AUAudioFrameCount, NSInteger, *mut AudioBufferList) -> AUAudioUnitStatus>;
pub type AUInternalRenderBlock = Block<dyn Fn(*mut AudioUnitRenderActionFlags, *const AudioTimeStamp, AUAudioFrameCount, NSInteger, *mut AudioBufferList, *const AURenderEvent, *mut AURenderPullInputBlock) -> AUAudioUnitStatus>;
pub type AUInternalRenderRcBlock = RcBlock<dyn Fn(*mut AudioUnitRenderActionFlags, *const AudioTimeStamp, AUAudioFrameCount, NSInteger, *mut AudioBufferList, *const AURenderEvent, *mut AURenderPullInputBlock) -> AUAudioUnitStatus>;
pub type AURenderBlock = AUInternalRenderBlock;

extern_class!(
	#[derive(PartialEq, Eq, Hash)]
	pub struct AUAudioUnit;

	unsafe impl ClassType for AUAudioUnit {
		type Super = NSObject;
		type Mutability = mutability::InteriorMutable;
	}
);

extern_methods!(
	unsafe impl AUAudioUnit {
		#[method(registerSubclass:asComponentDescription:name:version:)]
		#[allow(non_snake_case)]
		pub fn registerSubclass(
			cls: &AnyClass, 
			componentDescription: AudioComponentDescription,
			name: &NSString,
			version: u32
		);

		// Initialization
		#[method_id(initWithComponentDescription:error:_)]
		#[allow(non_snake_case)]
		pub fn initWithComponentDescription(
			this: Allocated<Self>,
			componentDescription: AudioComponentDescription
		) -> Result<Retained<Self>, Retained<NSError>>;

		#[method_id(initWithComponentDescription:options:error:)]
		#[allow(non_snake_case)]
		pub fn initWithComponentDescription_options_error(
			this: Allocated<Self>,
			componentDescription: AudioComponentDescription, 
			options: AudioComponentInstantiationOptions,
			outError: *mut *mut NSError
		) -> Id<Self>;

		// Querying Parameters
		#[method_id(parameterTree)]
		#[allow(non_snake_case)]
		pub fn parameterTree(&self) -> Option<Retained<AUParameterTree>>;

		#[method(allParameterValues)]
		#[allow(non_snake_case)]
		pub fn allParameterValues(&self) -> Bool;

		#[method_id(parametersForOverviewWithCount:)]
		#[allow(non_snake_case)]
		pub fn parametersForOverviewWithCount(count: NSInteger) -> Option<Retained<NSArray<NSNumber>>>;

		// Render cycle methods
		#[method(allocateRenderResourcesAndReturnError:_)]
		#[allow(non_snake_case)]
		pub fn allocate_render_resources(&self) -> Result<(), Retained<NSError>>;

		#[method(deallocateRenderResources)]
		#[allow(non_snake_case)]
		pub fn deallocateRenderResources(&self);

		#[method(reset)]
		pub unsafe fn reset(&self);

		#[method(renderResourcesAllocated)]
		#[allow(non_snake_case)]
		pub unsafe fn renderResourcesAllocated(&self) -> bool;

		#[method(renderBlock)]
		#[allow(non_snake_case)]
		pub fn render_block(&self) -> Option<&AURenderBlock>;

		#[method(maximumFramesToRender)]
		#[allow(non_snake_case)]
		pub fn maximumFramesToRender(&self) -> AUAudioFrameCount;

		#[method(setMaximumFramesToRender:)]
		#[allow(non_snake_case)]
		pub fn setMaximumFramesToRender(&self, maximumFramesToRender: AUAudioFrameCount);

		#[method(internalRenderBlock)]
		#[allow(non_snake_case)]
		pub fn internalRenderBlock(&self) -> Option<&AUInternalRenderBlock>;

		#[method_id(inputBusses)]
		#[allow(non_snake_case)]
		pub fn input_busses(&self) -> Option<Retained<AUAudioUnitBusArray>>;

		#[method_id(outputBusses)]
		#[allow(non_snake_case)]
		pub fn output_busses(&self) -> Option<Retained<AUAudioUnitBusArray>>;

		// Optimizing performance
		#[method(latency)]
		pub fn latency(&self) -> NSTimeInterval;

		#[method(tailTime)]
		#[allow(non_snake_case)]
		pub fn tailTime(&self) -> NSTimeInterval;

		#[method(renderQuality)]
		#[allow(non_snake_case)]
		pub fn renderQuality(&self) -> NSInteger;

		#[method(setRenderQuality:)]
		#[allow(non_snake_case)]
		pub fn setRenderQuality(&self, value: NSInteger);

		#[method(shouldBypassEffect)]
		#[allow(non_snake_case)]
		pub fn shouldBypassEffect(&self) -> Bool;

		#[method(setShouldBypassEffect:)]
		#[allow(non_snake_case)]
		pub fn setShouldBypassEffect(&self, value: Bool);

		#[method(canProcessInPlace)]
		#[allow(non_snake_case)]
		pub fn canProcessInPlace(&self) -> Bool;

		#[method(isRenderingOffline)]
		#[allow(non_snake_case)]
		pub fn isRenderingOffline(&self) -> Bool;

		#[method(setRenderingOffline:)]
		#[allow(non_snake_case)]
		pub fn setRenderingOffline(&self, value: Bool);


		// Configuring channel capabilities
		#[method_id(channelCapabilities)]
		#[allow(non_snake_case)]
		pub fn channelCapabilities(&self) -> Option<Retained<NSArray<NSNumber>>>;
		


		#[method(providesUserInterface)]
		#[allow(non_snake_case)]
		pub unsafe fn providesUserInterface(&self) -> bool;

		#[method(requestViewControllerWithCompletionHandler:)]
		#[allow(non_snake_case)]
		pub unsafe fn requestViewControllerWithCompletionHandler(&self, completion_handler: &Block<dyn Fn(*mut AUViewControllerBase)>);
	}
);