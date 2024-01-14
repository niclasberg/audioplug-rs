use super::OSStatus;

#[derive(Copy, Clone, Debug)]
pub enum AudioError {
	Unimplemented = -4,
	TooManyFilesOpen = -42,
	FileNotFound = -43,
	Param = -50,
	FilePermission = -54,
	MemFull = -108,
	BadFilePath = 561017960,
    NotRunning           = 1937010544, //'stop'
    Unspecified          = 2003329396, //'what'
    UnknownProperty      = 2003332927, //'who?'
    BadPropertySize      = 561211770, //'!siz'
    IllegalOperation     = 1852797029, //'nope'
    BadObject            = 560947818, //'!obj'
    BadDevice            = 560227702, //'!dev'
    BadStream            = 561214578, //'!str'
    UnsupportedOperation = 1970171760, //'unop',
	NotReady             = 1852990585, //'nrdy'
    UnsupportedDeviceFormat      = 560226676, //'!dat'
    DevicePermissions            = 560492391, //'!hog'
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

