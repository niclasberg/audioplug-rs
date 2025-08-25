const TILE_SIZE: u32 = 16;

struct Params {
	width: u32,
	height: u32,
}

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
	let tile_x = gid.x;
	let tile_y = gid.y;

	if tile_x >= params.width || tile_y >= params.height {
		return;
	}

	let coord = vec2(tile_x, tile_y);
	let pos = vec2f(coord);
	let size = vec2f(f32(params.width), f32(params.height));
	let min_dim = min(params.width, params.height);
	let p2 = pos - 0.5 * size;
	let radius = 0.25 * f32(min_dim);

	if dot(p2, p2) < radius*radius {
		textureStore(output_texture, coord, vec4(1.0, 0.2, 1.0, 1.0));
	} else {
		textureStore(output_texture, coord, vec4(0.1, 0.1, 0.1, 1.0));
	}
}