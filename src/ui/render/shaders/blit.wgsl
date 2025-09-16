struct VSOut {
	@builtin(position) pos: vec4<f32>,
	@location(0) uv: vec2<f32>,
}

@group(0) @binding(0)
var tex: texture_2d<f32>;

@group(0) @binding(1)
var tex_sampler: sampler;

/// Generate a triangle with vertices: (-1, 1), (1, 1), (-1, -1)
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
	let u = 2.0 * f32((vertex_index << 1) & 2) - 1.0;
	let v = -2.0 * f32(vertex_index & 2) + 1.0;
	return vec4(u, v, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
	let uv = pos.xy / vec2<f32>(textureDimensions(tex));
	return textureSample(tex, tex_sampler, uv);
}