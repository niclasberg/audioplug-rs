use std::{collections::VecDeque, ops::{Deref, DerefMut}};

use crate::{core::Rectangle, style::Style};
use super::{app_state::Task, Widget, WidgetData, WidgetFlags};

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

	pub fn style(&self) -> &Style {
		&self.data.style
	}

	pub fn style_mut(&mut self) -> &mut Style {
		&mut self.data.style
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