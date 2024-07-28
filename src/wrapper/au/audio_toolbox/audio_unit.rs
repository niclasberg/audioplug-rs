use bitflags::bitflags;
use block2::Block;
use objc2_foundation::{NSError, NSInteger, NSUInteger};
use objc2::{extern_class, extern_methods, mutability, rc::{Allocated, Id}, runtime::{NSObject, NSObjectProtocol}, ClassType, Encode, Encoding, RefEncode};

use crate::platform::mac::core_audio::{AudioBufferList, AudioTimeStamp};
use super::{AURenderEvent, AudioComponentDescription};

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
			outError: *mut *mut NSError
		) -> Id<Self>;

		#[method(internalRenderBlock)]
		#[allow(non_snake_case)]
		pub unsafe fn internalRenderBlock(&self) -> *mut AUInternalRenderBlock;
	}
);