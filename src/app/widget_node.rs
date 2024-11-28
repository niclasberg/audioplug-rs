use std::{collections::VecDeque, ops::{Deref, DerefMut}};

use bitflags::bitflags;
use slotmap::{new_key_type, Key, KeyData};

use crate::{core::{Point, Rectangle, Size}, style::{DisplayStyle, Style}};
use super::{app_state::Task, Widget, WindowId};

new_key_type! {
    pub struct WidgetId;
}

impl From<taffy::NodeId> for WidgetId {
    fn from(value: taffy::NodeId) -> Self {
        Self::from(KeyData::from_ffi(value.into()))
    }
}

impl Into<taffy::NodeId> for WidgetId {
    fn into(self) -> taffy::NodeId {
        self.0.as_ffi().into()
    }
}

bitflags!(
    #[derive(Debug, Clone, Copy)]
    pub struct WidgetFlags : u32 {
        const EMPTY = 0;
        const NEEDS_LAYOUT = 1 << 1;
        const NEEDS_RENDER = 1 << 2;
        const NEEDS_REBUILD = 1 << 3;
        const FOCUSABLE = 1 << 4;

        const UNDER_MOUSE_CURSOR = 1 << 8;
    }
);

pub struct WidgetData {
    pub(super) id: WidgetId,
    pub(super) window_id: WindowId,
    pub(super) parent_id: WidgetId,
    pub(super) children: Vec<WidgetId>,
    pub(super) style: Style,
	pub(super) display_style: DisplayStyle,
    pub(super) cache: taffy::Cache,
    pub(super) layout: taffy::Layout,
    pub(super) flags: WidgetFlags,
    pub(super) origin: Point,
}

impl WidgetData {
    pub fn new(window_id: WindowId, id: WidgetId) -> Self {
        Self {
            id,
            window_id,
            parent_id: WidgetId::null(),
            children: Vec::new(),
            style: Default::default(),
			display_style: Default::default(),
            cache: Default::default(),
            layout: Default::default(),
            flags: WidgetFlags::EMPTY,
            origin: Point::ZERO,
        }
    }

    pub fn with_parent(mut self, parent_id: WidgetId) -> Self {
        self.parent_id = parent_id;
        self
    }

    pub fn id(&self) -> WidgetId {
        self.id
    }

    /// Local bounds of the widget, relative to its parent 
    pub fn local_bounds(&self) -> Rectangle {
        Rectangle::new(self.offset(), self.size())
    }

    /// Bounds of the widget, in global coords, including borders and padding
    pub fn global_bounds(&self) -> Rectangle {
        Rectangle::new(self.origin(), self.size())
    }

    fn subtract_padding_and_border(&self, rect: Rectangle) -> Rectangle {
        Rectangle::from_ltrb(
            rect.left() + (self.layout.border.left + self.layout.padding.left) as f64, 
            rect.top() + (self.layout.border.top + self.layout.padding.top) as f64, 
            rect.right() - (self.layout.border.right + self.layout.padding.right) as f64, 
            rect.bottom() - (self.layout.border.bottom + self.layout.padding.bottom) as f64)
    }

    /// Bounds of the widget, in global coords, excluding borders and padding
    pub fn content_bounds(&self) -> Rectangle {
        self.subtract_padding_and_border(self.global_bounds())
    }

    pub fn border(&self) -> Rectangle {
        Rectangle::from_ltrb(
            self.layout.border.left as f64, 
            self.layout.border.top as f64, 
            self.layout.border.right as f64, 
            self.layout.border.bottom as f64)
    }

    pub fn padding(&self) -> Rectangle {
        Rectangle::from_ltrb(
            self.layout.padding.left as f64, 
            self.layout.padding.top as f64, 
            self.layout.padding.right as f64, 
            self.layout.padding.bottom as f64)
    }

    pub fn set_flag(&mut self, flag: WidgetFlags) {
        self.flags |= flag;
    }

    pub fn clear_flag(&mut self, flag: WidgetFlags) {
        self.flags &= !flag;
    }

    pub fn flag_is_set(& self, flag: WidgetFlags) -> bool{
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

    pub fn with_style(mut self, f: impl Fn(&mut Style)) -> Self {
        f(&mut self.style);
        self
    }

	pub fn with_display_style(mut self, f: impl Fn(&mut DisplayStyle)) -> Self {
		f(&mut self.display_style);
		self
	}

    pub fn is_hidden(&self) -> bool {
        self.style.hidden
    }
}

pub struct WidgetRef<'a, W: 'a + Widget + ?Sized> {
    pub(super) widget: &'a W,
    pub(super) data: &'a WidgetData
}

impl<'a, W: 'a + Widget + ?Sized> WidgetRef<'a, W> {
    pub(super) fn new(widget: &'a W, data: &'a WidgetData) -> Self {
        Self {
            widget,
            data
        }
    }

    pub fn data(&self) -> &WidgetData {
        &self.data
    }

    pub fn local_bounds(&self) -> Rectangle {
        self.data().local_bounds()
    }

    pub fn global_bounds(&self) -> Rectangle {
        self.data().global_bounds()
    }

    pub fn layout_requested(&self) -> bool {
        self.data().flag_is_set(WidgetFlags::NEEDS_LAYOUT)
    }
}

impl<'a, W: 'static + Widget> Deref for WidgetRef<'static, W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

pub struct WidgetMut<'a, W: 'a + Widget + ?Sized> {
    pub(super) widget: &'a mut W,
    pub(super) data: &'a mut WidgetData,
	pub(super) pending_tasks: &'a mut VecDeque<Task>
}

impl<'a, W: 'a + Widget + ?Sized> WidgetMut<'a, W> {
    pub(super) fn new(widget: &'a mut W, data: &'a mut WidgetData, pending_tasks: &'a mut VecDeque<Task>) -> Self {
        Self {
            widget,
            data,
			pending_tasks
        }
    }

    pub fn data(&self) -> &WidgetData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut WidgetData {
        &mut self.data
    }

    pub fn local_bounds(&self) -> Rectangle {
        self.data().local_bounds()
    }

    pub fn global_bounds(&self) -> Rectangle {
        self.data().global_bounds()
    }

    pub fn request_layout(&mut self) {
        self.data_mut().set_flag(WidgetFlags::NEEDS_LAYOUT);
    }

	pub fn request_render(&mut self) {
        self.pending_tasks.push_back(Task::InvalidateRect { 
			window_id: self.data.window_id, 
			rect: self.data.global_bounds()
		})
    }

	pub fn update_style(&mut self, f: impl FnOnce(&mut Style)) {
		f(&mut self.data.style);
	}

	pub fn update_display_style(&mut self, f: impl FnOnce(&mut DisplayStyle)) {
		f(&mut self.data.display_style);
	}

    pub fn layout_requested(&self) -> bool {
        self.data().flag_is_set(WidgetFlags::NEEDS_LAYOUT)
    }
}

impl<'a, W: 'a + Widget> Deref for WidgetMut<'a, W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<'a, W: 'a + Widget> DerefMut for WidgetMut<'a, W> {    
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}