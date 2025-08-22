struct VSOut {
	@builtin(position) pos: vec4<f32>,
	@location(0) uv: vec2<f32>,
}

/// Generate a triangle with vertices: (-1, 1), (1, 1), (-1, -1)
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VSOut {
	let u = 2.0 * f32((vertex_index << 1) & 2) - 1.0;
	let v = -2.0 * f32(vertex_index & 2) + 1.0;
	let out = VSOut(vec4(u, v, 0.0, 1.0), vec2(u, v));
	return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
	let radius_sqr = in.uv.x*in.uv.x + in.uv.y*in.uv.y;
	if radius_sqr < 0.25 {
		return vec4(1.0, 0.2, 1.0, 1.0);
	} else {
		return vec4(0.1, 0.1, 0.1, 1.0);
	}
}