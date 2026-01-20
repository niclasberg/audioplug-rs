use std::ffi::CStr;

pub const SHARED_STATE_MSG_ID: &'static CStr = c"State";
pub const SHARED_STATE_ATTR_ID: &'static CStr = c"State";

/// State shared between the Editor and the AudioProcessor
pub struct SharedState {}
