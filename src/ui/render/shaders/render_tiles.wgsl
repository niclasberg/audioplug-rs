const TILE_SIZE: u32 = 16;

const SIZE_MASK = 0xFFFF;
const FILL_MASK = (1u << 17);

const SHAPE_TYPE_RECT = 0u;
const SHAPE_TYPE_ROUNDED_RECT = 1u;
const SHAPE_TYPE_ELLIPSE = 2u;
const SHAPE_TYPE_PATH = 3u;

struct Params {
	width: u32,
	height: u32,
}

struct Segment {
	p0: vec2f,
	p1: vec2f,
}

struct Fill {
	segment_offset: u32,
	size: u32,
	color: vec4f,
}


// Shoot ray in positive x direction
fn winding_contribution(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> i32 {
    if (a.y <= p.y) {
        if (b.y > p.y) {
            // Upward crossing
            let cross = cross(b - a, p - a); // 2D cross product (b-a) × (p-a)
            if (cross > 0.0) {
                return 1;
            }
        }
    } else {
        if (b.y <= p.y) {
            // Downward crossing
            let cross = cross(b - a, p - a);
            if (cross < 0.0) {
                return -1;
            }
        }
    }
    return 0;
}

fn cross(u: vec2<f32>, v: vec2<f32>) -> f32 {
    return u.x * v.y - u.y * v.x;
}

const segments = array(
	Segment(vec2f(100.0, 100.0), vec2f(100.0, 800.0)),
	Segment(vec2f(100.0, 800.0), vec2f(800.0, 800.0)),
	Segment(vec2f(800.0, 800.0), vec2f(700.0, 400.0)),
	Segment(vec2f(700.0, 400.0), vec2f(100.0, 100.0)),
	Segment(vec2f(100.0, 800.0), vec2f(800.0, 800.0)),
	Segment(vec2f(800.0, 800.0), vec2f(900.0, 500.0)),
	Segment(vec2f(900.0, 500.0), vec2f(100.0, 800.0)),
);

const fills = array(
	Fill(0, 4, vec4f(1.0, 0.0, 0.0, 1.0)),
	Fill(4, 3, vec4f(0.0, 1.0, 0.0, 1.0)),
);

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

	var color = vec4(0.1, 0.1, 0.1, 1.0);
	for(var fill_id = 0; fill_id < 2; fill_id++) {
		let fill = fills[fill_id];
		var winding_number = 0;
		let segment_end = fill.segment_offset + fill.size;	
		for (var seg_id = fill.segment_offset; seg_id < segment_end; seg_id++) {
			winding_number += winding_contribution(pos, segments[seg_id].p0, segments[seg_id].p1);
		}

		if winding_number != 0 {
			color = fill.color;
		}
	}

	textureStore(output_texture, coord, color);
}