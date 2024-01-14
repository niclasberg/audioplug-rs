use std::{mem::MaybeUninit, os::raw::c_void};

use super::{AudioObjectID, AudioObjectPropertyAddress, AudioObjectGetPropertyDataSize, AudioObjectGetPropertyData, error::AudioError};

struct AudioObject {
	id: AudioObjectID
}

impl AudioObject {
	pub fn id(&self) -> AudioObjectID {
		self.id
	}
}

#[derive(Debug, Clone, Copy)]
pub struct QualifierData {
	qualifier_size: u32,
	qualifier_data: *const c_void
}

impl Default for QualifierData {
    fn default() -> Self {
        Self { qualifier_size: 0, qualifier_data: std::ptr::null() }
    }
}

pub fn get_property_value<T: Sized>(id: AudioObjectID, address: &AudioObjectPropertyAddress, qualifier_data: Option<QualifierData>) -> Result<T, AudioError> {
	unsafe {
		let QualifierData { qualifier_size, qualifier_data } = qualifier_data.unwrap_or_default();
		let mut data_size = std::mem::size_of::<T>() as u32;
		let mut data = MaybeUninit::<T>::uninit();
		let status = AudioObjectGetPropertyData(id, address, qualifier_size, qualifier_data, &mut data_size, data.as_mut_ptr() as *mut _);
		AudioError::from_osstatus(status)
			.map(|_| data.assume_init())
	}
}

pub fn get_property_values<T: Sized>(id: AudioObjectID, address: &AudioObjectPropertyAddress, qualifier_data: Option<QualifierData>) -> Result<Vec<T>, AudioError> {
	let mut data_size_in_bytes = get_property_data_size(id, address, qualifier_data)?;
	let data_size = data_size_in_bytes as usize/ std::mem::size_of::<T>();
	
	unsafe {
		let QualifierData { qualifier_size, qualifier_data } = qualifier_data.unwrap_or_default();
		let mut data = Vec::with_capacity(data_size);

		AudioError::from_osstatus(AudioObjectGetPropertyData(id, address, qualifier_size, qualifier_data, &mut data_size_in_bytes, data.as_mut_ptr() as *mut _))?;
		// Re-evaluate data size, just to make sure
		let data_size = data_size_in_bytes as usize/ std::mem::size_of::<T>();
		data.set_len(data_size);
		Ok(data)
	}
}

/// Returns the size of a property (in bytes)
pub fn get_property_data_size(id: AudioObjectID, address: &AudioObjectPropertyAddress, qualifier_data: Option<QualifierData>) -> Result<u32, AudioError> {
	unsafe {
		let QualifierData { qualifier_size, qualifier_data } = qualifier_data.unwrap_or_default();

		let mut data_size = MaybeUninit::uninit();
		let status = AudioObjectGetPropertyDataSize(id, address, qualifier_size, qualifier_data, data_size.as_mut_ptr());
		AudioError::from_osstatus(status)
			.map(|_| data_size.assume_init())
	}
}
