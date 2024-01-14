use std::{os::raw::c_void, mem::MaybeUninit};

use crate::platform::Error;

use super::{AudioObjectID, AudioTimeStamp, AudioBufferList, OSStatus, error::AudioError, AudioObjectPropertyAddress, AudioObjectPropertyScope, properties::kAudioHardwarePropertyDevices, audio_object::get_property_values, AudioDeviceID};

const SYSTEM_OBJECT_ID: AudioObjectID = 1;

pub struct AudioDevice {
	id: AudioObjectID
}

#[allow(dead_code)]
impl AudioDevice {
	pub fn enumerate() -> Result<Vec<AudioDevice>, AudioError> {
		let device_ids = get_device_ids()?;
		Ok(device_ids.into_iter().map(|id| AudioDevice { id }).collect())
	}

	pub fn start(&self) {
		//AudioDeviceStart(self.id, inProcID)
	}

	pub fn translate_time(&self, timestamp: &AudioTimeStamp) -> Result<AudioTimeStamp, Error> {
		unsafe { 
			let mut out_time = MaybeUninit::uninit();
			AudioDeviceTranslateTime(self.id, timestamp, out_time.as_mut_ptr()) ;
			Ok(out_time.assume_init())
		}
	}
}

fn get_device_ids() -> Result<Vec<AudioDeviceID>, AudioError>{
	let address = AudioObjectPropertyAddress{ 
		mElement: 0, 
		mScope: AudioObjectPropertyScope::Global, 
		mSelector: unsafe { kAudioHardwarePropertyDevices }
	};
	get_property_values(SYSTEM_OBJECT_ID, &address, None)
}


// typedef OSStatus (*AudioDeviceIOProc)(AudioObjectID inDevice, const AudioTimeStamp *inNow, const AudioBufferList *inInputData, const AudioTimeStamp *inInputTime, AudioBufferList *outOutputData, const AudioTimeStamp *inOutputTime, void *inClientData);
type AudioDeviceIOProc = unsafe extern "C" fn(AudioObjectID, *const AudioTimeStamp, *const AudioBufferList, *const AudioTimeStamp, *mut AudioBufferList, *const AudioTimeStamp, *mut c_void) -> OSStatus;
type AudioDeviceIOProcID = AudioDeviceIOProc;

#[link(name = "CoreAudio", kind = "framework")]
extern "C" {
	fn AudioDeviceCreateIOProcID(inDevice: AudioObjectID, inProc: AudioDeviceIOProc, inClientData: *mut c_void, outIOProcID: *mut AudioDeviceIOProcID) -> OSStatus;
	// OSStatus AudioDeviceCreateIOProcIDWithBlock(AudioDeviceIOProcID  _Nullable *outIOProcID, AudioObjectID inDevice, dispatch_queue_t inDispatchQueue, AudioDeviceIOBlock inIOBlock);
	fn AudioDeviceDestroyIOProcID(inDevice: AudioObjectID, inIOProcID: AudioDeviceIOProcID) -> OSStatus;

	fn AudioDeviceStart(inDevice: AudioObjectID, inProcID: AudioDeviceIOProcID) -> OSStatus;
	fn AudioDeviceStop(inDevice: AudioObjectID, inProcID: AudioDeviceIOProcID) -> OSStatus;
	fn AudioDeviceTranslateTime(inDevice: AudioObjectID, inTime: *const AudioTimeStamp, outTime: *mut AudioTimeStamp) -> OSStatus;
}