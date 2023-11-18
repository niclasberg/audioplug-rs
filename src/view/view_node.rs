use crate::core::{Size, Point, Vector, Rectangle};
use bitflags::bitflags;

bitflags!(
    #[derive(Debug, Clone, Copy)]
    pub struct ViewFlags : u32 {
        const EMPTY = 0;
        const NEEDS_LAYOUT = 1 << 1;
        const NEEDS_RENDER = 1 << 2;
        const NEEDS_REBUILD = 1 << 3;

        const UNDER_MOUSE_CURSOR = 1 << 8;
    }
);

#[derive(Debug, Clone)]
pub struct ViewNode {
    /// Size of the view
    pub(crate) size: Size,
    /// Top-left corner of the node, in global coordinates
    pub(crate) origin: Point,
    /// Offset from parent view
    pub(crate) offset: Vector,
    pub(crate) flags: ViewFlags,
    pub(crate) children: Vec<ViewNode>
}

impl ViewNode {
    pub fn new() -> Self {
        Self {
            size: Size::ZERO,
            origin: Point::ZERO,
            offset: Vector::ZERO,
            flags: ViewFlags::EMPTY,
            children: Vec::new(),
        }
    }

    pub fn local_bounds(&self) -> Rectangle {
        Rectangle::new(Point::ZERO, self.size)
    }

	pub fn global_bounds(&self) -> Rectangle {
		Rectangle::new(self.origin, self.size)
	}

    pub fn set_flag(&mut self, flag: ViewFlags) {
        self.flags |= flag;
    }

    pub fn set_flag_recursive(&mut self, flag: ViewFlags) {
        self.set_flag(flag);
        for child in self.children.iter_mut() {
            child.set_flag_recursive(flag);
        }
    }

    pub fn clear_flag(&mut self, flag: ViewFlags) {
        self.flags &= !flag;
    }

    pub fn clear_flag_recursive(&mut self, flag: ViewFlags) {
        self.clear_flag(flag);
        for child in self.children.iter_mut() {
            child.clear_flag_recursive(flag);
        }
    }

    pub fn flag_is_set(&mut self, flag: ViewFlags) -> bool{
        self.flags.contains(flag)
    }

    pub fn combine_child_flags(&mut self, index: usize) {
        self.flags |= self.children[index].flags;
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn origin(&self) -> Point {
        self.origin
    }

    pub fn offset(&self) -> Vector {
        self.offset
    }

    pub fn set_size(&mut self, new_size: Size) {
        self.size = new_size;
    }

    pub(crate) fn set_origin(&mut self, new_origin: Point) {
        self.origin = new_origin;
        for child in self.children.iter_mut() {
            child.set_origin(new_origin + child.offset);
        }
    }

    pub fn set_offset(&mut self, new_offset: Vector) {
        self.offset = new_offset;
    }
}

