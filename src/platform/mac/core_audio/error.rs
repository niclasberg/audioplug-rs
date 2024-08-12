use crate::platform::four_cc;

use super::OSStatus;

#[derive(Copy, Clone, Debug)]
#[repr(i32)]
pub enum AudioError {
	Unimplemented = -4,
	TooManyFilesOpen = -42,
	FileNotFound = -43,
	Param = -50,
	FilePermission = -54,
	MemFull = -108,
	BadFilePath = 561017960,
    NotRunning           = four_cc(b"stop"),
    Unspecified          = four_cc(b"what"), 
    UnknownProperty      = four_cc(b"who?"), 
    BadPropertySize      = four_cc(b"!siz"), 
    IllegalOperation     = four_cc(b"nope"), 
    BadObject            = four_cc(b"!obj"),
    BadDevice            = four_cc(b"!dev"), 
    BadStream            = four_cc(b"!str"),
    UnsupportedOperation = four_cc(b"unop"),
	NotReady             = four_cc(b"nrdy"),
    UnsupportedDeviceFormat      = four_cc(b"!dat"),
    DevicePermissions            = four_cc(b"!hog"),
	Unknown,
}

impl AudioError {
	pub fn from_osstatus(status: OSStatus) -> Result<(), Self> {
		match status {
			0 => Ok(()),
			-4 => Err(Self::Unimplemented),
			-43 => Err(Self::FileNotFound),
			-54 => Err(Self::FilePermission),
			-42 => Err(Self::TooManyFilesOpen),
			561017960 => Err(Self::BadFilePath),
			-50 => Err(Self::Param),
			-108 => Err(Self::MemFull),
			_ => Err(Self::Unknown),
		}
	}
}

impl std::error::Error for AudioError {}

impl ::std::fmt::Display for AudioError {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
		let description = match *self {
			Self::Unimplemented => "Unimplemented",
			Self::FileNotFound => "File not found",
			Self::FilePermission => "File permission",
			Self::TooManyFilesOpen => "Too many files open",
			Self::BadFilePath => "Bad file path",
			Self::Param => "Parameter error",
			Self::MemFull => "Memory full",
			Self::Unknown | _ => "An unknown error occurred",
		};
		write!(f, "{}", description)
	}
}

