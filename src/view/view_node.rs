use std::{any::Any, rc::Rc};

use crate::{app::{self, AppState, Binding}, core::{Point, Rectangle, Size}, event::KeyEvent, platform, window::WindowState, IdPath, MouseEvent};
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

pub(crate) enum ViewMessage {
    Mouse(MouseEvent),
    MouseEnterExit(bool),
    FocusChanged(bool)
}


pub struct WidgetData {
    id: IdPath,
    pub(super) style: taffy::Style,
    pub(super) cache: taffy::Cache,
    pub(super) layout: taffy::Layout,
    pub(super) flags: ViewFlags,
    pub(super) origin: Point,
    pub(super) bindings: Vec<Binding>
}

impl WidgetData {
    pub fn new(id: IdPath) -> Self {
        Self {
            id,
            style: Default::default(),
            cache: Default::default(),
            layout: Default::default(),
            flags: ViewFlags::EMPTY,
            origin: Point::ZERO,
            bindings: Default::default()
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
        if bounds.contains(event.position()) {
            ctx.with_child(&mut self.data, |ctx| {
                status = self.widget.mouse_event(event, ctx);
            });
        }
        status
    }

    pub fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
        let mut status = EventStatus::Ignored;
        ctx.with_child(&mut self.data, |ctx| {
            status = self.widget.key_event(event, ctx);
        });
        status
    }

    pub(crate) fn with_child(&mut self, destination: &IdPath, mut f: impl FnMut(&mut WidgetNode)) {
        fn with_child_impl(this: &mut WidgetNode, destination: &mut IdPath, f: &mut dyn FnMut(&mut WidgetNode)) {
            if let Some(child_id) = destination.pop_root() {
                let flags = {
                    let child = this.widget.get_child_mut(child_id.0);
                    with_child_impl(child, destination, f);
                    child.data.flags
                };
                this.data.flags |= flags & (ViewFlags::NEEDS_LAYOUT | ViewFlags::NEEDS_RENDER);
            } else {
                f(this)
            }
        }

        let mut destination = destination.clone();
        destination.pop_root();
        with_child_impl(self, &mut destination, &mut f)
    }

    pub(crate) fn handle_message(&mut self, destination: &IdPath, message: ViewMessage, window_state: &mut WindowState, handle: &mut platform::HandleRef, app_state: &mut AppState) {
        self.with_child(destination, |node: &mut WidgetNode| {
            let mut ctx = EventContext::new(&mut node.data, window_state, handle, app_state);
            match message {
                ViewMessage::Mouse(mouse_event) => {
                    node.widget.mouse_event(mouse_event, &mut ctx);
                },
                ViewMessage::FocusChanged(has_focus) => {
                    node.widget.focus_changed(has_focus, &mut ctx)
                },
                ViewMessage::MouseEnterExit(has_mouse_over) => {
                    node.widget.mouse_enter_exit(has_mouse_over, &mut ctx);
                }
            };
        })
    }

    // Traverse all views at a point 
    pub fn for_each_view_at(&self, point: Point, f: &mut impl FnMut(&Self) -> bool) -> bool {
        if !self.data.global_bounds().contains(point) {
            return true;
        }

        for i in (0..self.widget.child_count()).rev() {
            if !self.widget.get_child(i).for_each_view_at(point, f) {
                return false;
            }
        };

        f(self)
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
