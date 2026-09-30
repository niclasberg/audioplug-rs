// Ideas:
// 1. Use a BVH for paths, should speed up SDF computation
// 2. Also use this for winding number evaluation. Or keep a separate y-binned data structure

const TILE_SIZE: u32 = 16;

const PI = radians(180.0);
const TAU = radians(360.0);

const SHAPE_TYPE_NONE = 0u;
const SHAPE_TYPE_PATH = 1u;
const SHAPE_TYPE_RECT = 2u;
const SHAPE_TYPE_ROUNDED_RECT = 3u;
const SHAPE_TYPE_ELLIPSE = 4u;
const SHAPE_TYPE_MASK = 7u;

const SEGMENT_KIND_LINE = 0u;
const SEGMENT_KIND_QUAD_BEZ = 1u;
const SEGMENT_KIND_CUBIC_BEZ = 2u;

const FILL_RULE_EVEN_ODD = 1u << 3;

const FILL_FLAG = 1u;
const STROKE_FLAG = 2u;
const SHADOW_FLAG = 4u;

const PAINT_KIND_SOLID = 1u;
const PAINT_KIND_LINEAR_GRADIENT = 2u;
const PAINT_KIND_RADIAL_GRADIENT = 3u;
const PAINT_KIND_IMAGE = 4u;

const SHADOW_KIND_OUTER = 1u;
const SHADOW_KIND_INNER = 2u;

struct Params {
	width: u32,
	height: u32,
	draw_command_count: u32
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

struct PathSegment {
    start: vec2f,
    end: vec2f,
    control1: vec2f,
    control2: vec2f,
    kind: u32,
    /// Padding needed to fullfill wgsl alignment requirements
    _padding: u32,
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

struct Paint {
	kind: u32,
	data: array<u32, 7>
}

struct Shadow {
    kind: u32,
    radius: f32,
    offset: vec2f,
    color: vec4f,
}

struct Appearance {
	fill: Paint,
	stroke: Paint,
	shadow: Shadow,
	stroke_width: f32, 
	flags: u32,
	margin: f32,
	_padding: u32,
}

struct DrawCommand {
	shape_type: u32,
    shape_index: u32,
    appearance_index: u32,
	scale: f32,
	translation: vec2f,
	bounds: Rect
}

@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@group(1) @binding(0)
var<uniform> params: Params;

@group(1) @binding(1)
var<storage, read> shape_data: array<f32>;

@group(1) @binding(2)
var<storage, read> segments: array<PathSegment>;

@group(1) @binding(3)
var<storage, read> segment_bounds: array<Rect>;

@group(1) @binding(4)
var<storage, read> appearances: array<Appearance>;

@group(1) @binding(5)
var<storage, read> draw_commands: array<DrawCommand>;

const MAX_TILE_DRAW_COMMANDS = 256u;
var<workgroup> tile_draw_commands: array<u32, MAX_TILE_DRAW_COMMANDS>;
var<workgroup> tile_command_count: atomic<u32>;

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
	
	for (var i = 0u; i < params.draw_command_count; i++) {
		let command = draw_commands[i];

		if !is_point_in_rect(command.bounds, pos) {
			continue;
		}

		let appearance = appearances[command.appearance_index];
		let shape_pos = (pos - command.translation) / command.scale;

		let signed_dist = command.scale * sd_shape(command.shape_type, command.shape_index, shape_pos, appearance.margin / command.scale);

		if ((appearance.flags & SHADOW_FLAG) != 0 && appearance.shadow.kind == SHADOW_KIND_OUTER) {
			let pt = pos - appearance.shadow.offset;
			let blur_mask = compute_blurred_coverage(command.shape_type, command.shape_index, pt, appearance.shadow.radius);
			let shape_mask = distance_to_coverage(signed_dist);
			let coverage = blur_mask * (1.0 - shape_mask);
			color = blend(color, appearance.shadow.color, coverage);
		}

		if ((appearance.flags & FILL_FLAG) != 0) {
			let fill_color = eval_paint(appearance.fill, pos);
			let coverage = distance_to_coverage(signed_dist);
			color = blend(color, fill_color, coverage);
		}

		if ((appearance.flags & STROKE_FLAG) != 0) {
			let stroke_color = eval_paint(appearance.stroke, pos);
			let signed_dist_stroked = abs(signed_dist) - appearance.stroke_width * 0.5;
			let coverage = distance_to_coverage(signed_dist_stroked);
			color = blend(color, stroke_color, coverage);
		}

		if ((appearance.flags & SHADOW_FLAG) != 0 && appearance.shadow.kind == SHADOW_KIND_INNER) {
			let pt = pos - appearance.shadow.offset;
			let blur_mask = compute_blurred_coverage(command.shape_type, command.shape_index, pt, appearance.shadow.radius);
			let shape_mask = distance_to_coverage(signed_dist);
			let coverage = (1.0 - blur_mask) * shape_mask;
			color = blend(color, appearance.shadow.color, coverage);
		}
	}

	textureStore(output_texture, coord, color);
}

fn eval_paint(paint: Paint, pos: vec2f) -> vec4f {
	switch (paint.kind) {
		case PAINT_KIND_SOLID: {
			return vec4f(
				bitcast<f32>(paint.data[0]),
				bitcast<f32>(paint.data[1]),
				bitcast<f32>(paint.data[2]),
				bitcast<f32>(paint.data[3]),
			);
		}
		case PAINT_KIND_LINEAR_GRADIENT: {
			let start = vec2f(bitcast<f32>(paint.data[0]), bitcast<f32>(paint.data[1]));
			let end = vec2f(bitcast<f32>(paint.data[2]), bitcast<f32>(paint.data[3]));
			let delta = end - start;
			let t = clamp(dot(pos - start, delta) / dot(delta, delta), 0.0, 1.0);
			// TODO: Sample from colormap LUT
			return vec4f(t, t, t, 1.0);
		}
		case PAINT_KIND_RADIAL_GRADIENT: {
			let center = vec2f(bitcast<f32>(paint.data[0]), bitcast<f32>(paint.data[1]));
			let radius = max(bitcast<f32>(paint.data[2]), 1.0e-6);
			let t = clamp(length(pos - center) / radius, 0.0, 1.0);
			return vec4f(t, t, t, 1.0);
		}
		case PAINT_KIND_IMAGE: {
			return vec4f(0.0f);
		}
		default: {
			return vec4f(0.0f);
		}
	}
}

fn blend(color: vec4f, fill_color: vec4f, coverage: f32) -> vec4f {
	let alpha = fill_color.w * coverage;
	return (1.0 - alpha) * color + alpha * fill_color;
}

fn distance_to_coverage(dist: f32) -> f32 {
	return clamp(0.5 - dist, 0.0, 1.0);
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

fn sd_shape(shape_type: u32, index: u32, pos: vec2f, max_dist: f32) -> f32 {
	switch (shape_type & SHAPE_TYPE_MASK) {
		case SHAPE_TYPE_NONE: {
			return -1.0;
		}
		case SHAPE_TYPE_PATH: {
			let size = (shape_type >> 4);
			var winding_number = 0.0f;
			var dist = max_dist;
			for (var i = 0u; i < size; i++) {
				let bounds = segment_bounds[index + i];
				let segment = segments[index + i];
				if rect_dist_less_than(bounds, pos, dist) {
					switch (segment.kind) {
						case SEGMENT_KIND_LINE: {
							dist = min(dist, sd_line(segment.start, segment.end, pos));
						}
						case SEGMENT_KIND_QUAD_BEZ: {
							dist = min(dist, sd_quad_bezier(segment.start, segment.control1, segment.end, pos));
						}
						default: {

						}
					}
				}

				winding_number += winding_contribution(segment.start, segment.end, pos);
			}
			
			let is_inside = select(
				abs(winding_number) > 0.0, 
				(u32(abs(winding_number)) & 1u) != 0u, 
				(shape_type & FILL_RULE_EVEN_ODD) != 0u);
			return select(-dist, dist, is_inside);
		}
		case SHAPE_TYPE_RECT: {
			let rect = read_rect(index);
			let half_size = 0.5 * (rect.bottom_right - rect.top_left);
			let p = pos - rect.top_left - half_size;
			return sd_rect(half_size, p);
		}
		case SHAPE_TYPE_ROUNDED_RECT: {
			let rect = read_rounded_rect(index);
			let half_size = 0.5 * (rect.bottom_right - rect.top_left);
			let p = pos - rect.top_left - half_size;
			let corner_radius = select_rect_corner(rect.corner_radii, p);
			return sd_rounded_rect(half_size, corner_radius, p);
		}
		case SHAPE_TYPE_ELLIPSE: {
			let ellipse = read_ellipse(index);
			return sd_ellipse(ellipse.radii, pos - ellipse.center);
		}
		default: {
			return 0.0;
		}
	}
}

/// Returns 1.0 if pos is inside the rect, 0.0 otherwise
fn is_point_in_rect(r: Rect, pos: vec2f) -> bool {
	return all(pos >= r.top_left) && all(pos <= r.bottom_right);
}

fn rect_dist_less_than(r: Rect, pos: vec2f, dist: f32) -> bool {
    let d = max(max(r.top_left - pos, pos - r.bottom_right), vec2f(0.0));
    return dot(d, d) < dist * dist;
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

fn dot2(u: vec2<f32>) -> f32 {
    return dot(u, u);
}

/// Signed distance to a rect centered at the origin (adapted 
/// from https://iquilezles.org/articles/distfunctions2d/)
fn sd_rect(half_size: vec2f, pos: vec2f) -> f32 {
	let d = abs(pos) - half_size;
	return length(max(d, vec2f(0.0))) + min(max(d.x, d.y), 0.0);
}

/// Signed distance to a rounded rect centered at the origin (adapted 
/// from https://iquilezles.org/articles/distfunctions2d/)
fn sd_rounded_rect(half_size: vec2f, radius: f32, pos: vec2f) -> f32 {
	let q = abs(pos) - half_size + radius;
    return length(max(q, vec2f(0.0))) - radius;
}

/// Signed distance to an ellipse centered at the origin (adapted 
/// from https://iquilezles.org/articles/ellipsedist/)
fn sd_ellipse(radii: vec2f, pos: vec2f) -> f32 {
    // symmetry
	let p = abs(pos);

    // Find the angle, w, of the point on the ellipse that is closes to pos, using Newton-Raphson
    let q = radii * (p - radii);

	// Maybe we can use a better initial condition?
	var w = select(0.0, PI / 2.0, q.x < q.y);
    for (var i=0; i < 5; i++ ) {
        let cs = vec2(cos(w), sin(w));
        let u = radii * vec2f( cs.x, cs.y);
        let v = radii * vec2f(-cs.y, cs.x);
        w = w + dot(p - u, v) / (dot(p - u, u) + dot(v, v));
    }
    
    let d = length(p - radii * vec2f(cos(w), sin(w)));
    return select(-d, d, dot(p / radii, p / radii) > 1.0);
}

fn sd_line(a: vec2f, b: vec2f, pos: vec2f) -> f32 {
	let pa = pos - a;
	let ba = b - a;
	let t = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0 );
    return length(pa - ba * t);
}

fn sd_quad_bezier(p0: vec2f, p1: vec2f, p2: vec2f, pos: vec2f) -> f32 {
	// Want to solve:
	// 	0 = 2|a|^2 t^3 + 3dot(a, b) t^2 + (2dot(c - pos, a) + |b|^2) + dot(c-pos, b)
	// Where a = p0 - 2*p1 + p2, b = -2*p0 + 2*p1, c = p0
	let a = p0 - 2.0 * p1 + p2;
	let b = -2.0 * p0 + 2.0 * p1;
	let c = p0 - pos;

	let k0 = 2.0 * dot(a, a);
	let k1 = 3.0 * dot(a, b);
	let k2 = (2.0 * dot(c, a) + dot(b, b));
	let k3 = dot(c, b);

	let roots = solve_cubic(k0, k1, k2, k3);
	// The maxima always occurs at one of the two first roots
	let t1 = clamp(roots.roots[0], 0.0, 1.0);
	let dist1 = dot2(c + (b + a * t1) * t1);
	let t2 = clamp(roots.roots[1], 0.0, 1.0);
	let dist2 = dot2(c + (b + a * t2) * t2);
	if dist1 < dist2 {
		return sqrt(dist1);
	} else {
		return sqrt(dist2);
	}
}

struct CubicRoots {
	roots: array<f32, 3>,
	count: u32
}

/// Solve a polynomial x^3 + ax^2 + bx + c = 0
fn solve_normalized_cubic(a: f32, b: f32, c: f32) -> CubicRoots {
	// Apply transformation x = y - a/3, turning the equation 
	// into a Depressed cubic on the form:
	//   y^3 + py + q = 0
	let p = b - a * a / 3.0;
	let q = a * (2.0 * a * a / 27.0 - b / 3.0) + c;

	var result = CubicRoots(array<f32, 3>(), 0);
	// discriminant is (q/2)^2 + (p/3)^3
	let disc = (q * q) / 4.0 + (p * p * p) / 27.0;
	let shift = a / 3.0;
	if (disc >= 0.0) {
		// Single root, solution is u + v, where 
		//  u = (-q/2 + sqrt(disc))^(1/3)
		//  v = (-q/2 - sqrt(disc))^(1/3)
		// However, u*v = -p/3, so we can eliminate one cube root
		let u_cube = -0.5 * q + sqrt(disc);
		let u = sign(u_cube) * pow(abs(u_cube), 1.0 / 3.0); 
		let v = -p / (3.0 * u);
		let x = u + v - shift;
		result.roots[0] = x;
		result.count = 1;
	} else {
		let r = 2.0 * sqrt(-p / 3.0);
		let c0 = cos_acos_3((3.0 * q) / (2.0 * p) * sqrt(-3.0 / p));
		let s0 = sqrt(1.0 - c0 * c0);

		const SQRT3_HALF: f32 = 0.8660254037844386;
		// No need to sort the roots, they are in increasing order
		result.roots[0] = r * c0 - shift;
		result.roots[1] = r * (-0.5 * c0 - SQRT3_HALF * s0) - shift;
		result.roots[2] = r * (-0.5 * c0 + SQRT3_HALF * s0) - shift;
		result.count = 3;
	}
	return result;
}

// Evaluate cos(arccos(theta) / 3)
// https://www.shadertoy.com/view/WltSD7
fn cos_acos_3(theta: f32) -> f32 { 
	let x = sqrt(0.5 + 0.5 * theta); 
	return x * (x * (x * (x * -0.008972 + 0.039071) - 0.107074) + 0.576975) + 0.5;
} 

/// Solve a polynomial ax^3 + bx^2 + cx + d = 0
/// in the interval 0 <= x <= 1
fn solve_cubic(a: f32, b: f32, c: f32, d: f32) -> CubicRoots {
	return solve_normalized_cubic(b / a, c / a, d / a);
}


/*struct QuarticRoots {
	roots: array<f32, 4>,
	count: u32
}

/// Solve a polynomial ax^4 + bx^3 + cx^2 + dx + e = 0
/// in the interval 0 <= x <= 1
fn solve_quartic(a: f32, b: f32, c: f32, d: f32, e: f32) -> QuarticRoots {
	let extremas = solve_cubic(4.0 * a, 3.0 * b, 2.0 * c, d);
	
	var interval_pts = array<vec2f, 5>();
	interval_pts[0] = vec2f(0.0, eval_quartic(0.0, a, b, c, d, e));
	var interval_count = 1;
	for (var i = 0; i < extremas.count; i++) {
		// Solve cubic returns all extremas, need to ensure that it is in (0, 1)
		if (extremas.roots[i] > 0.0 && extremas.roots[i] < 1.0) {
			interval_pts[interval_count] = vec2f(extremas.roots[i], eval_quartic(extremas.roots[i], a, b, c, d, e));
			interval_count = interval_count + 1;
		}
	}
	interval_pts[interval_count] = vec2f(1.0, eval_quartic(1.0, a, b, c, d, e));

	var result = QuarticRoots(vec4f(0.0), 0);
	for (var i = 0; i < interval_count; i++) {
		let start = interval_pts[i];
		let end = interval_pts[i+1];
		if (start.y * end.y < 0.0) {
			result[result.count] = bisect_quartic(start.x, end.x, a, b, c, d, e);
			result.count = result.count + 1;
		}
	}
	return result;
}

fn eval_quartic(x: f32, a: f32, b: f32, c: f32, d: f32, e: f32) -> f32 {
	return x * (x * (x * (a * x + b) + c) + d) + e;
}

fn bisect_quartic(x_min: f32, x_max: f32, a: f32, b: f32, c: f32, d: f32, e: f32) -> f32 {
	return 0.0;
}

struct QuinticRoots {
	roots: array<f32, 5>,
	count: u32
}

/// Solve a polynomial ax^5 + bx^4 + cx^3 + dx^2 + ex + f = 0
/// in the interval 0 <= x <= 1
fn solve_quintic(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> QuinticRoots {
	let extremas = solve_quartic(5.0 * a, 4.0 * b, 3.0 * c, 2.0 * d, e);
	
	let interval_count = extremas.count + 1;
	var interval_pts = array<vec2f, 6>();
	interval_pts[0] = vec2f(0.0, eval_quintic(0.0, a, b, c, d, e, f));
	for (var i = 0; i < extremas.count; i++) {
		interval_pts[i + 1] = vec2f(extremas.roots[i], eval_quintic(extremas.roots[i], a, b, c, d, e, f));
	}
	interval_pts[interval_count] = vec2f(1.0, eval_quintic(1.0, a, b, c, d, e, f));

	var result = QuinticRoots(array<vec2f, 4>(), 0);
	for (var i = 0; i < interval_count; i++) {
		let start = interval_pts[i];
		let end = interval_pts[i+1];
		if (start.y * end.y < 0.0) {
			result[result.count] = bisect_quintic(start.x, end.x, a, b, c, d, e, f);
			result.count = result.count + 1;
		}
	}
	return result;
}

fn eval_quintic(x: f32, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> f32 {
	return x * (x * (x * (x * (a * x + b) + c) + d) + e) + f;
}

fn bisect_quintic(x_min: f32, x_max: f32, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> f32 {
	return 0.0;
}*/

/// Pick the radius of the corner of a rounded rectangle that is closest to pos
fn select_rect_corner(c: vec4f, pos: vec2f) -> f32 {
	return mix(mix(c.x, c.y, step(0, pos.x)), mix(c.w, c.z, step(0, pos.x)), step(0, pos.y));
}

fn compute_blurred_coverage(shape_type: u32, index: u32, pos: vec2f, blur_radius: f32) -> f32 {
	let sigma = blur_radius / 3.0;
	switch (shape_type & SHAPE_TYPE_MASK) {
		case SHAPE_TYPE_NONE: {
			return 0.0;
		}
		case SHAPE_TYPE_PATH: {
			// Not supported, need to think about how to do it.
			// We want to evaluate the integral: 
			//.   \int_A exp(-((x-x0)^2 + (y-y0)^2)/(2 sigma^2)) / (2 pi sigma^2) dA
			//.   Using Green's theorem this can be turned into a path integral \int_P F.n dl = \int div(F)
			//.   We can use: F = (exp(-y^2/(2sigma^2)/(2 sqrt(2pi)sigma) * (1-erf(x/(sqrt(2) sigma))), 0)
			//.   Or maybe something else? Then we can use Gaussian quadrature for the integral
			//.   Could also use Stokes theorm: \int_A FdA = \int_P Pdx + Qdy, where F = dQ/dx - dP/dy
			//.   Can use P = 0, Q = (...) e^(-y^2) * erf(x)
			//    Or better: (P, Q) = (1 - exp(-(x^2 + y^2)) / (2(x^2 + y^2)) * (-y, x)
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
