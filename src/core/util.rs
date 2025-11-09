/// Converts a
pub const fn normalized_f32_to_u32(value: f32) -> u32 {
    (value * u32::MAX as f32) as u32
}
