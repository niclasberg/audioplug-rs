use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use super::{
    layout::request_layout, render::invalidate_widget, AppState, View, Widget, WidgetData,
    WidgetFlags, WidgetId,
};
use crate::{core::Rectangle, style::Style};

pub struct ChildIter<'a> {
    app_state: &'a AppState,
    current_id: *const WidgetId,
    end_id: *const WidgetId,
}

impl<'a> Iterator for ChildIter<'a> {
    type Item = WidgetRef<'a, dyn Widget>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_id == self.end_id {
            None
        } else {
            let id = unsafe { *self.current_id };
            self.current_id = unsafe { self.current_id.offset(1) };
            Some(WidgetRef::new(self.app_state, id))
        }
    }
}

pub struct ChildIterMut<'a> {
    app_state: &'a mut AppState,
    current_id: *const WidgetId,
    end_id: *const WidgetId,
}

impl<'a> Iterator for ChildIterMut<'a> {
    type Item = WidgetMut<'a, dyn Widget>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct WidgetRef<'a, W: 'a + Widget + ?Sized> {
    pub(super) app_state: &'a AppState,
    pub(super) id: WidgetId,
    _phantom: PhantomData<&'a W>,
}

impl<'a, W: 'a + Widget + ?Sized> WidgetRef<'a, W> {
    pub(super) fn new(app_state: &'a AppState, id: WidgetId) -> Self {
        Self {
            app_state,
            id,
            _phantom: PhantomData,
        }
    }

    pub fn data(&self) -> &WidgetData {
        &self.app_state.widget_data[self.id]
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

    pub(super) fn unchecked_cast<W2: 'a + Widget + ?Sized>(self) -> WidgetRef<'a, W2> {
        WidgetRef {
            app_state: self.app_state,
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl Deref for WidgetRef<'_, dyn Widget> {
    type Target = dyn Widget;

    fn deref(&self) -> &Self::Target {
        &self.app_state.widgets[self.id]
    }
}

impl<'a, W: 'a + Widget> Deref for WidgetRef<'a, W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        self.app_state.widgets[self.id].downcast_ref().unwrap()
    }
}

pub struct WidgetMut<'a, W: 'a + Widget + ?Sized> {
    pub(super) app_state: &'a mut AppState,
    pub(super) id: WidgetId,
    _phantom: PhantomData<&'a mut W>,
}

impl<'a, W: 'a + Widget + ?Sized> WidgetMut<'a, W> {
    pub(super) fn new(app_state: &'a mut AppState, id: WidgetId) -> Self {
        Self {
            app_state,
            id,
            _phantom: PhantomData,
        }
    }

    pub fn data(&self) -> &WidgetData {
        &self.app_state.widget_data[self.id]
    }

    pub fn data_mut(&mut self) -> &mut WidgetData {
        &mut self.app_state.widget_data[self.id]
    }

    pub fn child_count(&self) -> usize {
        self.data().children.len()
    }

    pub fn push_child<V: View>(&mut self, view: V) {
        let widget_id = self.app_state.add_widget(self.id, view, None);
        request_layout(self.app_state, widget_id);
    }

    pub fn insert_child<V: View>(&mut self, view: V, index: usize) {
        let widget_id = self.app_state.add_widget(self.id, view, Some(index));
        request_layout(self.app_state, widget_id);
    }

    pub fn remove_child(&mut self, i: usize) {
        let child_id = self.data().children[i];
        self.app_state.remove_widget(child_id);
        request_layout(self.app_state, self.id);
    }

    pub fn replace_child<V: View>(&mut self, index: usize, view: V) {
        let id = self.data().children[index];
        self.app_state.replace_widget(id, view);
    }

    pub fn swap_children(&mut self, from_index: usize, to_index: usize) {
        self.data_mut().children.swap(from_index, to_index);
    }

    pub fn index_of_child(&self, id: WidgetId) -> Option<usize> {
        self.data().children.iter().position(|x| *x == id)
    }

    pub fn child_iter(&self) -> ChildIter<'_> {
        let ptr_range = self.data().children.as_ptr_range();
        ChildIter {
            app_state: self.app_state,
            current_id: ptr_range.start,
            end_id: ptr_range.end,
        }
    }

    pub fn child_iter_mut(&mut self) -> ChildIterMut<'_> {
        let ptr_range = self.data_mut().children.as_mut_ptr_range();
        ChildIterMut {
            app_state: self.app_state,
            current_id: ptr_range.start,
            end_id: ptr_range.end,
        }
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
        invalidate_widget(self.app_state, self.id);
    }

    pub fn update_style(&mut self, f: impl FnOnce(&mut Style)) {
        f(&mut self.data_mut().style);
    }

    pub fn style(&self) -> &Style {
        &self.data().style
    }

    pub fn style_mut(&mut self) -> &mut Style {
        &mut self.data_mut().style
    }

    pub fn layout_requested(&self) -> bool {
        self.data().flag_is_set(WidgetFlags::NEEDS_LAYOUT)
    }

    pub(super) fn unchecked_cast<W2: 'a + Widget + ?Sized>(self) -> WidgetMut<'a, W2> {
        unsafe { std::mem::transmute(self) }
    }
}

impl Deref for WidgetMut<'_, dyn Widget> {
    type Target = dyn Widget;

    fn deref(&self) -> &Self::Target {
        &self.app_state.widgets[self.id]
    }
}

impl<'a, W: 'a + Widget> Deref for WidgetMut<'a, W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        self.app_state.widgets[self.id].downcast_ref().unwrap()
    }
}

impl DerefMut for WidgetMut<'_, dyn Widget> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.app_state.widgets[self.id]
    }
}

impl<'a, W: 'a + Widget> DerefMut for WidgetMut<'a, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.app_state.widgets[self.id].downcast_mut().unwrap()
    }
}

impl<'a, W: 'a + Widget + ?Sized> Drop for WidgetMut<'a, W> {
    fn drop(&mut self) {
        self.app_state.merge_widget_flags(self.id);
    }
}
