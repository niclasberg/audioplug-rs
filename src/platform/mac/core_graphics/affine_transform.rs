use objc2_foundation::CGFloat;

use crate::core::Transform;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CGAffineTransform {
	/// The value at position [1,1] in the matrix.
    pub a: CGFloat,
	/// The value at position [1,2] in the matrix.
    pub b: CGFloat,
	/// The value at position [2,1] in the matrix.
    pub c: CGFloat,
	/// The value at position [2,2] in the matrix.
    pub d: CGFloat,
	/// The value at position [3,1] in the matrix.
    pub tx: CGFloat,
	/// The value at position [3,2] in the matrix.
    pub ty: CGFloat,
}

impl CGAffineTransform {
	pub fn scale(sx: CGFloat, sy: CGFloat) -> Self {
		Self { a: sx, b: 0.0, c: 0.0, d: sy, tx: 0.0, ty: 0.0 }
	}

	pub fn translate(tx: CGFloat, ty: CGFloat) -> Self {
		Self { a: 0.0, b: 0.0, c: 0.0, d: 0.0, tx, ty }
	}
}

impl From<Transform> for CGAffineTransform {
    fn from(value: Transform) -> Self {
        CGAffineTransform { a: value.m11, b: value.m12, c: value.m21, d: value.m22, tx: value.tx, ty: value.ty }
    }
}

impl From<CGAffineTransform> for Transform {
    fn from(value: CGAffineTransform) -> Self {
        Transform { m11: value.a, m12: value.b, m21: value.c, m22: value.d, tx: value.tx, ty: value.ty }
    }
}