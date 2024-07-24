use block2::Block;
use c_enum::c_enum;
use objc2_foundation::{NSError, NSInteger};
use objc2::{extern_class, extern_methods, mutability, rc::{Allocated, Id, Retained}, runtime::{NSObject, NSObjectProtocol}, ClassType};

use crate::platform::{core_audio::AudioBufferList, mac::core_audio::AudioTimeStamp};
use super::AudioComponentDescription;

pub type AUAudioUnitStatus = i32;
pub type AUAudioFrameCount = u32;

c_enum!(
	#[derive(Copy, Clone, PartialEq, Eq, Hash)]
	pub enum AudioUnitRenderActionFlags: u32 {
		/// Called on a render notification Proc - which is called either before or after the render operation of the audio unit. If this flag is set, the proc is being called before the render operation is performed.
		kAudioUnitRenderAction_PreRender = (1 << 2),
		/// Called on a render notification Proc - which is called either before or after the render operation of the audio unit. If this flag is set, the proc is being called after the render operation is completed.
		kAudioUnitRenderAction_PostRender = (1 << 3),
		/// This flag can be set in a render input callback (or in the audio unit's render operation itself) and is used to indicate that the render buffer contains only silence. It can then be used by the caller as a hint to whether the buffer needs to be processed or not.
		kAudioUnitRenderAction_OutputIsSilence = (1 << 4),
		/// This is used with offline audio units (of type 'auol'). It is used when an offline unit is being preflighted, which is performed prior to the actual offline rendering actions are performed. It is used for those cases where the offline process needs it (for example, with an offline unit that normalizes an audio file, it needs to see all of the audio data first before it can perform its normalization).
		kAudioOfflineUnitRenderAction_Preflight = (1 << 5),
		/// Once an offline unit has been successfully preflighted, it is then put into its render mode. So this flag is set to indicate to the audio unit that it is now in that state and that it should perform its processing on the input data.
		kAudioOfflineUnitRenderAction_Render = (1 << 6),
		/// This flag is set when an offline unit has completed either its preflight or performed render operation.
		kAudioOfflineUnitRenderAction_Complete = (1 << 7),
		/// If this flag is set on the post-render call an error was returned by the audio unit's render operation. In this case, the error can be retrieved through the lastRenderError property and the audio data in ioData handed to the post-render notification will be invalid.
		kAudioUnitRenderAction_PostRenderError = (1 << 8),
		/// If this flag is set, then checks that are done on the arguments provided to render are not performed. This can be useful to use to save computation time in situations where you are sure you are providing the correct arguments and structures to the various render calls.
		kAudioUnitRenderAction_DoNotCheckRenderArgs = (1 << 9)
	}
);

pub struct AURenderEvent {

}

type AURenderPullInputBlock = Block<dyn Fn(*mut AudioUnitRenderActionFlags, *const AudioTimeStamp, AUAudioFrameCount, NSInteger, *mut AudioBufferList) -> AUAudioUnitStatus>;
type AUInternalRenderBlock = Block<dyn Fn(*mut AudioUnitRenderActionFlags, *const AudioTimeStamp, AUAudioFrameCount, NSInteger, *mut AudioBufferList, *const AURenderEvent, AURenderPullInputBlock) -> AUAudioUnitStatus>;

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
		#[method_id(initWithComponentDescription:error:)]
		#[allow(non_snake_case)]
		pub unsafe fn initWithComponentDescription_error(
			this: Allocated<Self>,
			componentDescription: AudioComponentDescription, 
			outError: *mut NSError
		) -> Id<Self>;

		/*#[method_id(internalRenderBlock)]
		#[allow(non_snake_case)]
		pub unsafe fn internalRenderBlock(&self) -> Retained<AUInternalRenderBlock>;*/
	}
);