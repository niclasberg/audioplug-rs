use super::Error;
use objc2_core_audio::AudioDeviceID;

pub struct AudioHost;

impl AudioHost {
    pub fn devices() -> Result<Vec<Device>, Error> {
        todo!()
    }

    pub fn default_output_device() -> Result<Device, Error> {
        todo!()
    }
}

pub struct Device {
    _device_id: AudioDeviceID,
}

impl Device {
    pub fn name(&self) -> Result<String, Error> {
        todo!()
    }

    pub fn sample_rate(&self) -> Result<f64, Error> {
        todo!()
    }
}
