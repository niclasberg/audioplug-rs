use objc2::{Encoding, RefEncode, Encode};

#[repr(C)]
pub struct AudioComponentDescription {

}

unsafe impl Encode for AudioComponentDescription {
    const ENCODING: Encoding = Encoding::Struct("AudioComponentDescription", &[]);
}