use super::{Error, core_audio::AudioDeviceID};

pub struct AudioHost;

impl AudioHost {
	pub fn default_output_device() -> Result<Device, Error> {
		todo!()
	}
}

pub struct Device {
	device_id: AudioDeviceID
}

impl Device {
	pub fn sample_rate(&self) -> Result<f64, Error> {
		todo!()
	}
}