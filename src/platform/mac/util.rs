pub const fn four_cc(str: &[u8; 4]) -> i32 {
    (((str[0] as u32) << 24 & 0xff000000)
        | ((str[1] as u32) << 16 & 0x00ff0000)
        | ((str[2] as u32) << 8 & 0x0000ff00)
        | ((str[3] as u32) & 0x000000ff)) as i32
}

macro_rules! cf_enum {
	{
		$( #[$attr:meta] )*
		$vis:vis enum $name:ident : $inner:ty {
			$(
				$( #[ $field_attr:meta ] )*
				$field:ident = $value:expr
			),* $(,)?
		}
	} => {
		c_enum::c_enum!(
			$(#[$attr])*
			$vis enum $name : $inner {
				$(
					$(#[$field_attr])*
					$field = $value,
				)*
			}
		);

		unsafe impl objc2::Encode for $name {
			const ENCODING: objc2::Encoding = <$inner>::ENCODING;
		}

		unsafe impl objc2::RefEncode for $name {
			const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&<Self as objc2::Encode>::ENCODING);
		}
	}
}

pub(super) use cf_enum;
use objc2_core_foundation::{CFIndex, CFRange};
