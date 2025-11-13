const TILE_SIZE: u32 = 16;

const PI = radians(180.0);
const TAU = radians(360.0);

const SIZE_MASK = 0xFFFF;
const FILL_MASK = (1u << 17);

const SHAPE_TYPE_NONE = 0u;
const SHAPE_TYPE_PATH = 1u;
const SHAPE_TYPE_RECT = 2u;
const SHAPE_TYPE_ROUNDED_RECT = 3u;
const SHAPE_TYPE_ELLIPSE = 4u;
const SHAPE_TYPE_MASK = 7u;

const FILL_RULE_EVEN_ODD = 1u << 3;

const FILL_TYPE_SOLID = 1u;
const FILL_TYPE_BLUR = 2u;
const FILL_TYPE_LINEAR_GRADIENT = 3u;
const FILL_TYPE_RADIAL_GRADIENT = 4u;

struct Params {
	width: u32,
	height: u32,
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

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@group(1) @binding(0)
var<storage, read> segments: array<Segment>;

@group(1) @binding(1)
var<storage, read> shapes: array<ShapeData>;

@group(1) @binding(2)
var<storage, read> fills: array<u32>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
	let tile_x = gid.x;
	let tile_y = gid.y;

	if tile_x >= params.width || tile_y >= params.height {
		return;
	}

	let coord = vec2(tile_x, tile_y);
	let pos = vec2f(coord);

	var color = vec4f(0.1, 0.3, 0.1, 1.0);
	var i = 0u;
	loop {
		if i >= arrayLength(&fills) {
			break;
		}

		let shape_type = fills[i] >> 4;
		let fill_type = fills[i] & 0x7;
		let index = fills[i+1];
		i += 2;

		switch (fill_type) {
			case FILL_TYPE_SOLID: {
				let fill_color = vec4f(
					bitcast<f32>(fills[i]),
					bitcast<f32>(fills[i+1]),
					bitcast<f32>(fills[i+2]),
					bitcast<f32>(fills[i+3]),
				);
				i += 4;

				let coverage = compute_coverage(shape_type, index, pos);
				color = blend(color, fill_color, coverage);
			}
			case FILL_TYPE_BLUR: {
				let blur_color = vec4f(
					bitcast<f32>(fills[i]),
					bitcast<f32>(fills[i+1]),
					bitcast<f32>(fills[i+2]),
					bitcast<f32>(fills[i+3]),
				);
				let blur_radius = bitcast<f32>(fills[i+4]);
				i += 5;

				let coverage = compute_blurred_coverage(shape_type, index, pos, blur_radius);
				color = blend(color, blur_color, coverage);
			}
			case FILL_TYPE_LINEAR_GRADIENT: {
				let start = vec2f(bitcast<f32>(fills[i]), bitcast<f32>(fills[i+1]));
				let end = vec2f(bitcast<f32>(fills[i+2]), bitcast<f32>(fills[i+3]));
				i += 4;
				
				let delta = end - start;
				let t = clamp(dot(pos - start, end - start) / dot(delta, delta), 0.0, 1.0);
				let fill_color = vec4f(t, t, t, 1.0);

				let coverage = compute_coverage(shape_type, index, pos);
				color = blend(color, fill_color, coverage);
			}
			case FILL_TYPE_RADIAL_GRADIENT: {
				
			}
			default: {
				
			}
		}
	};

	textureStore(output_texture, coord, color);
}

fn blend(color: vec4f, fill_color: vec4f, coverage: f32) -> vec4f {
	let alpha = fill_color.w * coverage;
	return (1.0 - alpha) * color + alpha * fill_color;
}

fn compute_coverage(shape_type: u32, index: u32, pos: vec2f) -> f32 {
	switch (shape_type & SHAPE_TYPE_MASK) {
		case SHAPE_TYPE_NONE: {
			return 1.0;
		}
		case SHAPE_TYPE_PATH: {
			let size = (shape_type >> 4);
			var winding_number = 0.0f;
			for (var i = 0u; i < size; i++) {
				let seg_id = index + i;
				winding_number += winding_contribution(segments[seg_id].p0, segments[seg_id].p1, pos);
			}
			
			let even_odd_fill = 1.0 - abs(1.0 - 2.0 * fract(0.5 * winding_number));
			let non_zero_fill = clamp(abs(winding_number), 0.0, 1.0);
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
fn winding_contribution(p0: vec2<f32>, p1: vec2<f32>, pos: vec2<f32>) -> f32 {
	let delta = p1 - p0;
	let cross = cross(delta, pos - p0);
	let up_crossing = p0.y <= pos.y && p1.y > pos.y && cross > 0.0;
	let down_crossing = p0.y > pos.y && p1.y <= pos.y && cross < 0.0;
	let direction = select(0.0, 1.0, up_crossing) + select(0.0, -1.0, down_crossing);
    return direction;
}

fn cross(u: vec2<f32>, v: vec2<f32>) -> f32 {
    return u.x * v.y - u.y * v.x;
}

/// Returns 1.0 if pos is inside the rect, 0.0 otherwise
fn is_point_in_rect(top_left: vec2f, bottom_right: vec2f, pos: vec2f) -> f32 {
	let s = step(top_left, pos) - step(bottom_right, pos);
	return s.x * s.y;
}

/// Signed distance to a rounded rect centered at the origin
fn sd_rounded_rect(half_size: vec2f, radius: f32, pos: vec2f) -> f32 {
	let q = abs(pos) - half_size + radius;
    return length(max(q, vec2f(0.0))) - radius;
}

/// Pick the radius of the corner that is closest to pos
fn select_rect_corner(c: vec4f, pos: vec2f) -> f32 {
	return mix(mix(c.x, c.y, step(0, pos.x)), mix(c.w, c.z, step(0, pos.x)), step(0, pos.y));
}

fn compute_blurred_coverage(shape_type: u32, index: u32, pos: vec2f, blur_radius: f32) -> f32 {
	let sigma = blur_radius / 3.0;
	switch (shape_type & SHAPE_TYPE_MASK) {
		case SHAPE_TYPE_NONE: {
			return 1.0;
		}
		case SHAPE_TYPE_PATH: {
			// Not supported, need to think about if it's possible to do without an 
			// intermediate texture
			return 0.0; 
		}
		// Rect and rounded rect blur functions adapted from https://madebyevan.com/shaders/fast-rounded-rectangle-shadows/
		case SHAPE_TYPE_RECT: {
			let shape = shapes[index];
			let top_left = shape.bounds.xy;
			let bottom_right = shape.bounds.zw;
			let query = vec4f(top_left - pos, bottom_right - pos); 
			let integral = 0.5 + 0.5 * erf4(query * (sqrt(0.5) / sigma));
			return (integral.z - integral.x) * (integral.w - integral.y);
		}
		case SHAPE_TYPE_ROUNDED_RECT: {
			let shape = shapes[index];
			let top_left = shape.bounds.xy;
			let bottom_right = shape.bounds.zw;
			
			// Center everything to make the math easier
			let center = (top_left + bottom_right) * 0.5;
			let half_size = (bottom_right - top_left) * 0.5;
			let p = pos - top_left - half_size;

			// The signal is only non-zero in a limited range, so don't waste samples
			let low = p.y - half_size.y;
			let high = p.y + half_size.y;
			let start = clamp(-blur_radius, low, high);
			let end = clamp(blur_radius, low, high);

			// Accumulate samples (we can get away with surprisingly few samples)
			let step = (end - start) / 4.0;
			var y = start + step * 0.5;
			var value = 0.0;
			let corner_radius = select_rect_corner(shape.corner_radii, p);
			for (var i = 0; i < 4; i++) {
				value += rounded_box_shadow_x(p.x, p.y - y, sigma, corner_radius, half_size) * gaussian(y, sigma) * step;
				y += step;
			}
			return value;
		}
		default: {
			return 0.0;
		}
	}
}

fn erf4(x: vec4f) -> vec4f {
	let s = sign(x);
	let a = abs(x);
	let y = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
	return s - s / (y * y * y * y);
}

fn rounded_box_shadow_x(x: f32, y: f32, sigma: f32, corner: f32, half_size: vec2f) -> f32{
	let delta = min(half_size.y - corner - abs(y), 0.0);
	let curved = half_size.x - corner + sqrt(max(0.0, corner * corner - delta * delta));
	let integral = 0.5 + 0.5 * erf2((x + vec2(-curved, curved)) * (sqrt(0.5) / sigma));
	return integral.y - integral.x;
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