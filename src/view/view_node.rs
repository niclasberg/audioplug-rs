use crate::{core::{Point, Rectangle, Size, Vector}, Event, IdPath, MouseEvent};
use bitflags::bitflags;

use super::{EventContext, LayoutContext, RenderContext, Widget};

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


pub struct WidgetData {
    id: IdPath,
    pub(super) style: taffy::Style,
    pub(super) cache: taffy::Cache,
    pub(super) layout: taffy::Layout,
    pub(super) flags: ViewFlags,
    pub(super) origin: Point
}

impl WidgetData {
    pub fn new(id: IdPath) -> Self {
        Self {
            id,
            style: Default::default(),
            cache: Default::default(),
            layout: Default::default(),
            flags: ViewFlags::EMPTY,
            origin: Point::ZERO
        }
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id
    }

    pub fn local_bounds(&self) -> Rectangle {
        Rectangle::new(self.offset(), self.size())
    }

    pub fn global_bounds(&self) -> Rectangle {
        Rectangle::new(self.origin(), self.size())
    }

    pub fn set_flag(&mut self, flag: ViewFlags) {
        self.flags |= flag;
    }

    pub fn clear_flag(&mut self, flag: ViewFlags) {
        self.flags &= !flag;
    }

    pub fn flag_is_set(& self, flag: ViewFlags) -> bool{
        self.flags.contains(flag)
    }

    pub fn size(&self) -> Size {
        self.layout.size.map(|x| x as f64).into()
    }

    pub fn offset(&self) -> Point {
        self.layout.location.map(|x| x as f64).into()
    }

    pub fn origin(&self) -> Point {
        self.origin
    }

    pub fn with_style(mut self, f: impl Fn(&mut taffy::Style)) -> Self {
        f(&mut self.style);
        self
    }
}

pub struct WidgetNode {
    pub(crate) widget: Box<dyn Widget>,
    pub(crate) data: WidgetData,
}

impl WidgetNode {
    pub fn data(&self) -> &WidgetData {
        &self.data
    }

    pub fn render(&mut self, ctx: &mut RenderContext) {
        ctx.with_child(&mut self.data, |ctx| self.widget.render(ctx))
    }

    pub fn layout(&mut self, inputs: taffy::LayoutInput, ctx: &mut LayoutContext) -> taffy::LayoutOutput {
        if self.data.flag_is_set(ViewFlags::NEEDS_LAYOUT) {
            self.data.cache.clear();
            self.data.clear_flag(ViewFlags::NEEDS_LAYOUT);
        }
        self.widget.layout(inputs, ctx)
    }

    pub fn event(&mut self, event: Event, ctx: &mut EventContext) {
        if ctx.is_handled() {
            return;
        }

        let bounds = self.data.global_bounds();

        match event {
            Event::Mouse(mouse_event) => {
                match mouse_event {
                    MouseEvent::Moved { position } if bounds.contains(position) => {
                        if !self.data.flag_is_set(ViewFlags::UNDER_MOUSE_CURSOR) {
                            self.data.set_flag(ViewFlags::UNDER_MOUSE_CURSOR);
                            ctx.with_child(&mut self.data, |ctx| {
                                self.widget.event(Event::Mouse(MouseEvent::Enter), ctx);
                                ctx.view_flags()
                            });
                        }

                        ctx.with_child(&mut self.data, |ctx| {
                            self.widget.event(Event::Mouse(mouse_event), ctx);
                            ctx.view_flags()
                        });
                    },
                    // Mouse exited the child's parent, or the mouse moved
                    // and the mouse is not over the child
                    MouseEvent::Moved { .. } | MouseEvent::Exit => {
                        if self.data.flag_is_set(ViewFlags::UNDER_MOUSE_CURSOR) {
                            self.data.clear_flag(ViewFlags::UNDER_MOUSE_CURSOR);
                            ctx.with_child(&mut self.data, |ctx| {
                                self.widget.event(Event::Mouse(MouseEvent::Exit), ctx);
                                ctx.view_flags()
                            });
                        }
                        ctx.with_child(&mut self.data, |ctx| {
                            self.widget.event(Event::Mouse(mouse_event), ctx);
                            ctx.view_flags()
                        });
                    },
                    // Filter these out
                    MouseEvent::Enter => {},
                    _ => {
                        ctx.with_child(&mut self.data, |ctx| {
                            self.widget.event(Event::Mouse(mouse_event), ctx);
                            ctx.view_flags()
                        });
                    }
                };
            },
            other => {
                ctx.with_child(&mut self.data, |ctx| {
                    self.widget.event(other, ctx);
                    ctx.view_flags()
                });
            }
        };
    }

    pub(crate) fn set_origin(&mut self, parent_offset: Point) {
        let origin = parent_offset + self.data.offset();
        self.data.origin = origin;
        for i in 0..self.widget.child_count() {
            let child = self.widget.get_child_mut(i);
            child.set_origin(origin);
        }
    }

    pub fn layout_requested(&self) -> bool {
        self.data.flag_is_set(ViewFlags::NEEDS_LAYOUT)
    }
}

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

