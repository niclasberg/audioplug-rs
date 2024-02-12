use std::ffi::c_int;

use super::{AudioObjectID, AudioDeviceID, error::AudioError, audio_object::get_property_values, AudioObjectPropertyAddress, properties::kAudioHardwarePropertyDevices, AudioObjectPropertyScope};



pub struct AudioSystemObject;

impl AudioSystemObject {
	pub fn device_ids() -> Result<Vec<AudioDeviceID>, AudioError> {
		todo!()
	}


}
