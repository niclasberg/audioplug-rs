pub const fn four_cc(str: &[u8; 4]) -> i32 {
	(
		((str[0] as u32) << 24 & 0xff000000)
		| ((str[1] as u32) << 16 & 0x00ff0000)
		| ((str[2] as u32) << 8 & 0x0000ff00)
		| ((str[3] as u32) & 0x000000ff)
	) as i32
}