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
const FILL_TYPE_DROP_SHADOW = 2u;
const FILL_TYPE_INNER_SHADOW = 3u;
const FILL_TYPE_LINEAR_GRADIENT = 4u;
const FILL_TYPE_RADIAL_GRADIENT = 5u;

struct Params {
	width: u32,
	height: u32,
}

struct LinearGradient {
	p0: vec2f,
	p1: vec2f
}

struct RadialGradient {
	center: vec2f,
	radius: f32,
}

struct LineSegment {
	p0: vec2f,
	p1: vec2f,
}

struct Rect {
	top_left: vec2f,
	bottom_right: vec2f,
}

struct RoundedRect {
	top_left: vec2f,
	bottom_right: vec2f,
	corner_radii: vec4f, 
}

struct Ellipse {
	center: vec2f,
	radii: vec2f,
}

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@group(1) @binding(0)
var<storage, read> shape_data: array<f32>;

@group(1) @binding(1)
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
		let fill_type = fills[i] & 0xF;
		let index = fills[i+1];
		i += 2;

		if (fill_type == FILL_TYPE_SOLID) {
			color = fill_solid(&i, shape_type, index, color, pos);
		} else if (fill_type == FILL_TYPE_DROP_SHADOW || fill_type == FILL_TYPE_INNER_SHADOW) {
			color = fill_shadow(&i, shape_type, index, color, fill_type, pos);
		} else if (fill_type == FILL_TYPE_LINEAR_GRADIENT) {
			color = fill_linear_gradient(&i, shape_type, index, color, pos);
		} else if (fill_type == FILL_TYPE_RADIAL_GRADIENT) {
			color = fill_radial_gradient(&i, shape_type, index, color, pos);
		}
	};

	textureStore(output_texture, coord, color);
}

fn fill_solid(i: ptr<function, u32>, shape_type: u32, shape_index: u32, color: vec4f, pos: vec2f) -> vec4f {
	let fill_color = vec4f(
		bitcast<f32>(fills[*i]),
		bitcast<f32>(fills[*i+1]),
		bitcast<f32>(fills[*i+2]),
		bitcast<f32>(fills[*i+3]),
	);
	*i += 4;

	let coverage = compute_coverage(shape_type, shape_index, pos);
	return blend(color, fill_color, coverage);
}

fn fill_shadow(i: ptr<function, u32>, shape_type: u32, shape_index: u32, color: vec4f, shadow_type: u32, pos: vec2f) -> vec4f {
	let blur_color = vec4f(
		bitcast<f32>(fills[*i]),
		bitcast<f32>(fills[*i+1]),
		bitcast<f32>(fills[*i+2]),
		bitcast<f32>(fills[*i+3]),
	);
	let offset = vec2f(bitcast<f32>(fills[*i+4]), bitcast<f32>(fills[*i+5]));
	let blur_radius = bitcast<f32>(fills[*i+6]);
	*i += 7;

	let blur_mask = compute_blurred_coverage(shape_type, shape_index, pos - offset, blur_radius);
	let shape_mask = compute_coverage(shape_type, shape_index, pos);
	let drop_shadow_coverage = blur_mask * (1.0 - shape_mask);
	let inner_shadow_coverage = (1.0 - blur_mask) * shape_mask;
	let coverage = select(inner_shadow_coverage, drop_shadow_coverage, shadow_type == FILL_TYPE_DROP_SHADOW);
	return blend(color, blur_color, coverage);
}

fn fill_linear_gradient(i: ptr<function, u32>, shape_type: u32, shape_index: u32, color: vec4f, pos: vec2f) -> vec4f {
	let start = vec2f(bitcast<f32>(fills[*i]), bitcast<f32>(fills[*i+1]));
	let end = vec2f(bitcast<f32>(fills[*i+2]), bitcast<f32>(fills[*i+3]));
	*i += 4;
	
	let delta = end - start;
	let t = clamp(dot(pos - start, delta) / dot(delta, delta), 0.0, 1.0);
	let fill_color = vec4f(t, t, t, 1.0);

	let coverage = compute_coverage(shape_type, shape_index, pos);
	return blend(color, fill_color, coverage);
}

fn fill_radial_gradient(i: ptr<function, u32>, shape_type: u32, shape_index: u32, color: vec4f, pos: vec2f) -> vec4f {
	let center = vec2f(bitcast<f32>(fills[*i]), bitcast<f32>(fills[*i+1]));
	let radius = max(bitcast<f32>(fills[*i+2]), 1.0e-6);
	*i += 3;

	let t = clamp(length(pos - center) / radius, 0.0, 1.0);
	let fill_color = vec4f(t, t, t, 1.0);

	let coverage = compute_coverage(shape_type, shape_index, pos);
	return blend(color, fill_color, coverage);
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
				let segment = read_line_segment(index + 4*i);
				winding_number += winding_contribution(segment.p0, segment.p1, pos);
			}
			
			let even_odd_fill = 1.0 - abs(1.0 - 2.0 * fract(0.5 * winding_number));
			let non_zero_fill = clamp(abs(winding_number), 0.0, 1.0);
			return select(non_zero_fill, even_odd_fill, (shape_type & FILL_RULE_EVEN_ODD) != 0);
		}
		case SHAPE_TYPE_RECT: {
			let rect = read_rect(index);

			// TODO: anti-aliasing: Compute area of intersection between a unit rectangle centered at pos and the shape's rectangle

			let s = step(rect.top_left, pos) - step(rect.bottom_right, pos);
			return s.x * s.y;
		}
		case SHAPE_TYPE_ROUNDED_RECT: {
			let rect = read_rounded_rect(index);

			let half_size = 0.5 * (rect.bottom_right - rect.top_left);
			let p = pos - rect.top_left - half_size;
			let corner_radius = select_rect_corner(rect.corner_radii, p);
			let dist = sd_rounded_rect(half_size, corner_radius, p);
			
			return smoothstep(-0.5, 0.5, -dist);
		}
		case SHAPE_TYPE_ELLIPSE: {
			let ellipse = read_ellipse(index);

			let dist = sd_ellipse(ellipse.radii, pos - ellipse.center);
			return smoothstep(-0.5, 0.5, -dist);
		}
		default: {
			return 0.0;
		}
	}
}

fn read_line_segment(index: u32) -> LineSegment {
	return LineSegment(
		vec2f(shape_data[index], shape_data[index+1]), 
		vec2f(shape_data[index+2], shape_data[index+3])
	);
}

fn read_rect(index: u32) -> Rect {
	return Rect(
		vec2f(shape_data[index], shape_data[index+1]), 
		vec2f(shape_data[index+2], shape_data[index+3])
	);
}

fn read_rounded_rect(index: u32) -> RoundedRect {
	let top_left = vec2f(shape_data[index], shape_data[index+1]);
	let bottom_right = vec2f(shape_data[index+2], shape_data[index+3]);
	let corner_radii = vec4f(shape_data[index+4], shape_data[index+5], shape_data[index+6], shape_data[index+7]);
	return RoundedRect(top_left, bottom_right, corner_radii);
}

fn read_ellipse(index: u32) -> Ellipse {
	return Ellipse(
		vec2f(shape_data[index], shape_data[index+1]), 
		vec2f(shape_data[index+2], shape_data[index+3])
	);
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

fn sd_ellipse(radii: vec2f, pos: vec2f) -> f32 {
    // symmetry
	let p = abs(pos);

    // find root with Newton solver
    let q = radii*(p-radii);
	var w = select(0.0, PI / 2.0, q.x<q.y);
    for (var i=0; i < 5; i++ ) {
        let cs = vec2(cos(w),sin(w));
        let u = radii * vec2f( cs.x,cs.y);
        let v = radii * vec2f(-cs.y,cs.x);
        w = w + dot(p-u,v)/(dot(p-u,u)+dot(v,v));
    }
    
    // compute final point and distance
    let d = length(p - radii * vec2f(cos(w),sin(w)));
    
    // return signed distance
    return select(-d, d, dot(p/radii,p/radii) > 1.0);
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
			let rect = read_rect(index);
			let query = vec4f(rect.top_left - pos, rect.bottom_right - pos); 
			let integral = 0.5 + 0.5 * erf4(query * (sqrt(0.5) / sigma));
			return (integral.z - integral.x) * (integral.w - integral.y);
		}
		case SHAPE_TYPE_ROUNDED_RECT: {
			let rect = read_rounded_rect(index);
			
			// Center everything to make the math easier
			let half_size = (rect.bottom_right - rect.top_left) * 0.5;
			let p = pos - rect.top_left - half_size;

			// The signal is only non-zero in a limited range, so don't waste samples
			let low = p.y - half_size.y;
			let high = p.y + half_size.y;
			let start = clamp(-blur_radius, low, high);
			let end = clamp(blur_radius, low, high);

			// Accumulate samples (we can get away with surprisingly few samples)
			let step = (end - start) / 4.0;
			var y = start + step * 0.5;
			var value = 0.0;
			let corner_radius = select_rect_corner(rect.corner_radii, p);
			for (var i = 0; i < 4; i++) {
				value += blurred_rounded_box_x(p.x, p.y - y, sigma, corner_radius, half_size) * gaussian(y, sigma) * step;
				y += step;
			}
			return value;
		}
		case SHAPE_TYPE_ELLIPSE: {
			let ellipse = read_ellipse(index);
			let p = pos - ellipse.center;

			let low = p.y - ellipse.radii.y;
			let high = p.y + ellipse.radii.y;
			let start = clamp(-blur_radius, low, high);
			let end = clamp(blur_radius, low, high);

			let step = (end - start) / 4.0;
			var y = start + step * 0.5;
			var value = 0.0;
			for (var i = 0; i < 4; i++) {
				value += blurred_ellipse_x(p.x, p.y - y, sigma, ellipse.radii) * gaussian(y, sigma) * step;
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

/// Returns the integral of a Gaussian centered at (x, y) along the width of a rounded rectangle
/// The (x, y) position is relative to the center of the rectangle
fn blurred_rounded_box_x(x: f32, y: f32, sigma: f32, corner: f32, half_size: vec2f) -> f32 {
	let delta = min(half_size.y - corner - abs(y), 0.0);
	let curved = half_size.x - corner + sqrt(max(0.0, corner * corner - delta * delta));
	let integral = 0.5 + 0.5 * erf2((x + vec2(-curved, curved)) * (sqrt(0.5) / sigma));
	return integral.y - integral.x;
}

fn blurred_ellipse_x(x: f32, y: f32, sigma: f32, radii: vec2f) -> f32 { 
	let y_rel = clamp(y / radii.y, -1.0, 1.0);
	let half_width = radii.x * sqrt(1.0 - y_rel * y_rel);
	let integral = 0.5 + 0.5 * erf2((x + vec2(-half_width, half_width)) * (sqrt(0.5) / sigma));
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