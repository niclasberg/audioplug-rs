use crate::{core::{Point, Rectangle, Size}, event::KeyEvent, IdPath, MouseEvent};
use bitflags::bitflags;

use super::{EventContext, EventStatus, LayoutContext, RenderContext, Widget};

bitflags!(
    #[derive(Debug, Clone, Copy)]
    pub struct ViewFlags : u32 {
        const EMPTY = 0;
        const NEEDS_LAYOUT = 1 << 1;
        const NEEDS_RENDER = 1 << 2;
        const NEEDS_REBUILD = 1 << 3;
        const FOCUSABLE = 1 << 4;

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

    pub fn mouse_event(&mut self, event: MouseEvent, ctx: &mut EventContext) -> EventStatus {
        let bounds = self.data.global_bounds();
        let mut status = EventStatus::Ignored;
        match event {
            MouseEvent::Moved { position } if bounds.contains(position) => {
                if !self.data.flag_is_set(ViewFlags::UNDER_MOUSE_CURSOR) {
                    self.data.set_flag(ViewFlags::UNDER_MOUSE_CURSOR);
                    ctx.with_child(&mut self.data, |ctx| {
                        self.widget.mouse_event(MouseEvent::Enter, ctx);
                        ctx.view_flags()
                    });
                }

                ctx.with_child(&mut self.data, |ctx| {
                    status = self.widget.mouse_event(event, ctx);
                    ctx.view_flags()
                });
            },
            // Mouse exited the child's parent, or the mouse moved
            // and the mouse is not over the child
            MouseEvent::Moved { .. } | MouseEvent::Exit => {
                if self.data.flag_is_set(ViewFlags::UNDER_MOUSE_CURSOR) {
                    self.data.clear_flag(ViewFlags::UNDER_MOUSE_CURSOR);
                    ctx.with_child(&mut self.data, |ctx| {
                        self.widget.mouse_event(MouseEvent::Exit, ctx);
                        ctx.view_flags()
                    });
                }
                ctx.with_child(&mut self.data, |ctx| {
                    status = self.widget.mouse_event(event, ctx);
                    ctx.view_flags()
                });
            },
            // Filter these out
            MouseEvent::Enter => {},
            _ => {
                ctx.with_child(&mut self.data, |ctx| {
                    status = self.widget.mouse_event(event, ctx);
                    ctx.view_flags()
                });
            }
        };
        status
    }

    pub fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
        let mut status = EventStatus::Ignored;
        ctx.with_child(&mut self.data, |ctx| {
            status = self.widget.key_event(event, ctx);
            ctx.view_flags()
        });
        status
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
