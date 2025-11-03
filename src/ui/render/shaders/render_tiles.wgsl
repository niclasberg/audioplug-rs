const TILE_SIZE: u32 = 16;

const PI = radians(180.0);
const TAU = radians(360.0);

const SIZE_MASK = 0xFFFF;
const FILL_MASK = (1u << 17);

struct Params {
	width: u32,
	height: u32,
}

const SHAPE_TYPE_PATH = 1u;
const SHAPE_TYPE_RECT = 2u;
const SHAPE_TYPE_ROUNDED_RECT = 3u;
const SHAPE_TYPE_ELLIPSE = 4u;
const SHAPE_TYPE_MASK = 7u;

const FILL_RULE_EVEN_ODD = 1u << 3;

struct FillOp {
	color: vec4f,
	/// bits 0-2: Shape type
	/// bit 3: Fill rule (path only): 0 -> even-odd, 1 -> non-zero
	/// bits 4-31: Number of segments (path only)
	shape_type: u32,
	/// ShapeData index or offset to first line segment
	index: u32,
}

struct Segment {
	p0: vec2f,
	p1: vec2f,
}

struct ShapeData {
	bounds: vec4f, // [left, top, right, bottom]
	corner_radii: vec4f, // only for rounded rect [upper-left, upper-right, bottom-]
}

struct LinearGradient {
	p0: vec2f,
	p1: vec2f
}

struct RadialGradient {
	center: vec2f,
	radius: f32,
}

/*const segments = array(
	Segment(vec2f(100.0, 100.0), vec2f(100.0, 800.0)),
	Segment(vec2f(100.0, 800.0), vec2f(800.0, 800.0)),
	Segment(vec2f(800.0, 800.0), vec2f(700.0, 400.0)),
	Segment(vec2f(700.0, 400.0), vec2f(100.0, 100.0)),
	Segment(vec2f(100.0, 800.0), vec2f(800.0, 800.0)),
	Segment(vec2f(800.0, 800.0), vec2f(900.0, 500.0)),
	Segment(vec2f(900.0, 500.0), vec2f(100.0, 800.0)),
);

const shapes = array(
	ShapeData(vec4f(10.0, 10.0, 150.0, 200.0), vec4f(10.0, 20.0, 30.0, 40.0)),
	ShapeData(vec4f(650.0, 390.0, 720.0, 800.0), vec4f(0.0))
);

const fills = array(
	FillOp(SHAPE_TYPE_PATH | (4 << 4), 0, vec4f(1.0, 0.0, 0.0, 1.0)),
	FillOp(SHAPE_TYPE_PATH | (3 << 4), 4, vec4f(0.0, 1.0, 0.0, 1.0)),
	FillOp(SHAPE_TYPE_RECT, 1, vec4f(0.3, 0.1, 0.7, 1.0)),
	FillOp(SHAPE_TYPE_ROUNDED_RECT, 0, vec4f(0.5, 0.5, 0.0, 1.0)*0.4),
);
const N_FILLS = 4;*/

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@group(1) @binding(0)
var<storage, read> segments: array<Segment>;

@group(1) @binding(1)
var<storage, read> shapes: array<ShapeData>;

@group(1) @binding(2)
var<storage, read> fills: array<FillOp>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
	let tile_x = gid.x;
	let tile_y = gid.y;

	if tile_x >= params.width || tile_y >= params.height {
		return;
	}

	let coord = vec2(tile_x, tile_y);
	let pos = vec2f(coord);

	var color = vec4(0.1, 0.1, 0.1, 1.0);
	for(var fill_id = 0u; fill_id < arrayLength(&fills); fill_id++) {
		let fill = fills[fill_id];
		let coverage = compute_coverage(fill.shape_type, fill.index, pos);
		let alpha = fill.color.w * coverage;
		color = (1.0 - alpha) * color + alpha * fill.color;
	}

	textureStore(output_texture, coord, color);
}

fn compute_coverage(shape_type: u32, index: u32, pos: vec2f) -> f32 {
	switch (shape_type & SHAPE_TYPE_MASK) {
		case SHAPE_TYPE_PATH: {
			let fill_rule = (shape_type >> 3) & 1u;
			let size = (shape_type >> 4);
			var winding_number = 0;
			for (var i = 0u; i < size; i++) {
				let seg_id = index + i;
				winding_number += winding_contribution(pos, segments[seg_id].p0, segments[seg_id].p1);
			}
			
			let even_odd_fill = select(0.0, 1.0, winding_number != 0);
			let non_zero_fill = f32(winding_number & 1);
			return select(non_zero_fill, even_odd_fill, (shape_type & FILL_RULE_EVEN_ODD) != 0);
		}
		case SHAPE_TYPE_RECT: {
			let shape = shapes[index];
			let top_left = shape.bounds.xy;
			let bottom_right = shape.bounds.zw;

			// Compute area of intersection between a unit rectangle centered at pos and the shape's rectangle

			let s = step(top_left, pos) - step(bottom_right, pos);
			return s.x * s.y;
		}
		case SHAPE_TYPE_ROUNDED_RECT: {
			let shape = shapes[index];
			let top_left = shape.bounds.xy;
			let bottom_right = shape.bounds.zw;

			let half_size = 0.5 * (bottom_right - top_left);
			let p = pos - top_left - half_size;
			let corner_radius = select_rect_corner(shape.corner_radii, p);
			let dist = sd_rounded_rect(half_size, corner_radius, p);
			
			return smoothstep(-0.5, 0.5, -dist);
		}
		default: {
			return 0.0;
		}
	}
}

// Shoot ray in positive x direction, returns the number of path crossings
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

/// Returns 1.0 if pos is inside the rect, 0.0 otherwise
fn is_point_in_rect(top_left: vec2f, bottom_right: vec2f, pos: vec2f) -> f32 {
	let s = step(top_left, pos) - step(bottom_right, pos);
	return s.x * s.y;
}

/// Returns the mask for a blurred rectangle. 
fn blurred_rect_mask(top_left: vec2f, bottom_right: vec2f, sigma: f32, pos: vec2f) -> f32 {
	let query = vec4f(top_left - pos, bottom_right - pos); 
	let integral = 0.5 + 0.5 * erf4(query * (sqrt(0.5) / sigma));
  	return (integral.z - integral.x) * (integral.w - integral.y);
}

fn erf4(x: vec4f) -> vec4f {
	let s = sign(x);
	let a = abs(x);
	let y = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
	return s - s / (y * y * y * y);
}

/// Signed distance to a rounded rect centered at the origin
fn sd_rounded_rect(half_size: vec2f, radius: f32, pos: vec2f) -> f32 {
	let q = abs(pos) - half_size + radius;
    return length(max(q, vec2f(0.0))) - radius;
}

fn select_rect_corner(c: vec4f, pos: vec2f) -> f32 {
	return mix(mix(c.x, c.y, step(0, pos.x)), mix(c.w, c.z, step(0, pos.x)), step(0, pos.y));
}

fn blurred_rounded_rect_mask(top_left: vec2f, bottom_right: vec2f, corner_radius: f32, sigma: f32, pos: vec2f) -> f32 {
	return 0.0;
}

fn erf2(x: vec2f) -> vec2f {
	let s = sign(x);
	let a = abs(x);
	let y = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
	return s - s / (y * y * y * y);
}

fn gaussian(x: f32, sigma: f32) -> f32 {
  	return exp(-(x * x) / (2.0 * sigma * sigma)) / (sqrt(2.0 * PI) * sigma);
}