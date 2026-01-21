use std::ffi::CStr;

pub const SHARED_STATE_MSG_ID: &CStr = c"State";
pub const SHARED_STATE_ATTR_ID: &CStr = c"State";

/// State shared between the Editor and the AudioProcessor
#[derive(Debug)]
pub struct SharedState {}
