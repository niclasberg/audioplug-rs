use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use slotmap::Key;

use super::{
    AppState, OverlayOptions, View, Widget, WidgetData, WidgetFlags, WidgetId,
    app_state::WidgetInsertPos, render::invalidate_widget, style::Style,
};
use crate::{
    core::{Rect, diff::DiffOp},
    ui::{TypedWidgetId, Widgets, widgets::WidgetIdIter},
};

pub struct WidgetNotFound<'a, W: Widget + ?Sized>(
    WidgetMut<'a, W>
);

impl<'a, W: Widget + ?Sized> Debug for WidgetNotFound<'a, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("WidgetNotFound").field(&self.0.id).finish()
    }
}

pub struct WidgetRefIter<'a> {
    widgets: &'a Widgets,
    inner: WidgetIdIter,
}

impl<'a> Iterator for WidgetRefIter<'a> {
    type Item = WidgetRef<'a, dyn Widget>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next_id(self.widgets).map(|id| WidgetRef::new(self.widgets, id))
    }
}

impl<'a> DoubleEndedIterator for WidgetRefIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back_id(self.widgets).map(|id| WidgetRef::new(self.widgets, id))
    }
}

pub struct WidgetRef<'a, W: 'a + Widget + ?Sized> {
    pub(super) widgets: &'a Widgets,
    pub(super) id: WidgetId,
    _phantom: PhantomData<&'a W>,
}

impl<'a, W: 'a + Widget + ?Sized> WidgetRef<'a, W> {
    pub(super) fn new(widgets: &'a Widgets, id: WidgetId) -> Self {
        Self {
            widgets,
            id,
            _phantom: PhantomData,
        }
    }

    pub fn data(&self) -> &WidgetData {
        &self.widgets.data[self.id]
    }

    pub fn local_bounds(&self) -> Rect {
        self.data().local_bounds()
    }

    pub fn global_bounds(&self) -> Rect {
        self.data().global_bounds()
    }

    pub fn content_bounds(&self) -> Rect {
        self.data().content_bounds()
    }

    pub fn layout_requested(&self) -> bool {
        self.data().flag_is_set(WidgetFlags::NEEDS_LAYOUT)
    }

    pub fn has_focus(&self) -> bool {
        self.widgets.data[self.id].flag_is_set(WidgetFlags::HAS_FOCUS)
    }

    pub fn has_mouse_capture(&self) -> bool {
        self.widgets.data[self.id].flag_is_set(WidgetFlags::HAS_MOUSE_CAPTURE)
    }

    pub fn child_count(&self) -> usize {
        self.widgets.children_as_vec(self.id).len()
    }

    pub fn child_iter(&self) -> WidgetRefIter<'_> {
        WidgetRefIter {
            inner: WidgetIdIter::all_children(self.widgets, self.id),
            widgets: self.widgets,
        }
    }

    pub fn parent_id(&self) -> WidgetId {
        self.data().parent_id
    }

    pub fn parent(&self) -> Option<WidgetRef<'a, dyn Widget>> {
        let parent_id = self.data().parent_id;
        if !parent_id.is_null() {
            Some(WidgetRef {
                widgets: self.widgets,
                id: parent_id,
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }

    pub(super) fn unchecked_cast<W2: 'a + Widget + ?Sized>(self) -> WidgetRef<'a, W2> {
        WidgetRef {
            widgets: self.widgets,
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl Deref for WidgetRef<'_, dyn Widget> {
    type Target = dyn Widget;

    fn deref(&self) -> &Self::Target {
        &self.widgets.widgets[self.id]
    }
}

impl<'a, W: 'a + Widget> Deref for WidgetRef<'a, W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        self.widgets.widgets[self.id].downcast_ref().unwrap()
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
        &self.app_state.widgets.data[self.id]
    }

    pub fn data_mut(&mut self) -> &mut WidgetData {
        &mut self.app_state.widgets.data[self.id]
    }

    pub fn child_count(&self) -> usize {
        self.app_state.widgets.children_as_vec(self.id).len()
    }

    pub fn index_of_child(&self, id: WidgetId) -> Option<usize> {
        self.child_iter().position(|w| w.id == id)
    }

    pub fn find_child<W2: Widget + ?Sized>(self, id: TypedWidgetId<W2>) -> Result<WidgetMut<'a, W2>, WidgetNotFound<'a, W>> {
        let found = self.app_state.widgets.child_id_iter(self.id)
            .find(|&child_id| child_id == id.id);

        if let Some(child_id) = found {
            Ok(WidgetMut::new(self.app_state, child_id))
        } else {
            Err(WidgetNotFound(self))
        }
    }

    pub fn find_sibling<W2: Widget + ?Sized>(self, id: TypedWidgetId<W2>) -> Result<WidgetMut<'a, W2>, WidgetNotFound<'a, W>> {
        let found = self.app_state.widgets.sibling_id_iter(self.id)
            .find(|&child_id| child_id == id.id);
        
        if let Some(child_id) = found {
            Ok(WidgetMut::new(self.app_state, child_id))
        } else {
            Err(WidgetNotFound(self))
        }
    }

    pub fn child_iter(&self) -> WidgetRefIter<'_> {
        WidgetRefIter {
            inner: WidgetIdIter::all_children(&self.app_state.widgets, self.id),
            widgets: &self.app_state.widgets,
        }
    }

    pub fn for_each_child_ref(&self, mut f: impl FnMut(WidgetRef<'_, dyn Widget>)) {
        fn _impl(widgets: &Widgets, id: WidgetId, f: &mut dyn FnMut(WidgetRef<'_, dyn Widget>)) {
            let mut child_iter = WidgetIdIter::all_children(widgets, id);
            while let Some(child_id) = child_iter.next_id(widgets) {
                f(WidgetRef::new(widgets, child_id))
            }
        }
        _impl(&self.app_state.widgets, self.id, &mut f)
    }

    pub fn for_each_child_mut(&mut self, mut f: impl FnMut(WidgetMut<'_, dyn Widget>)) {
        fn _impl(app_state: &mut AppState, id: WidgetId, f: &mut dyn FnMut(WidgetMut<'_, dyn Widget>)) {
            let mut child_iter = WidgetIdIter::all_children(&app_state.widgets, id);
            let mut current = child_iter.next_id(&app_state.widgets);
            while let Some(child_id) = current {
                // This copy is needed - the widget might be removed while visited
                let next = child_iter.next_id(&app_state.widgets); 
                f(WidgetMut::new(app_state, child_id));
                current = next;
            }
        }
        _impl(self.app_state, self.id, &mut f)
    }
    
    pub fn remove(self) {
        self.app_state.remove_widget(self.id);
    }

    pub fn replace<V: View>(self, view: V) -> WidgetMut<'a, V::Element> {
        self.app_state.replace_widget(self.id, view);
        WidgetMut::new(self.app_state, self.id)
    }

    pub fn insert_before<V: View>(&mut self, view: V) -> WidgetId {
        todo!()
    }

    pub fn insert_after<V: View>(&mut self, view: V) -> WidgetId {
        todo!()
    }

    pub fn add_overlay<V: View>(&mut self, view: V, options: OverlayOptions) -> TypedWidgetId<V::Element> {
        let widget_id = self
            .app_state
            .add_widget(view, WidgetInsertPos::Overlay(self.id, options));
        TypedWidgetId::new(widget_id)
    }

    pub fn push_child<V: View>(&mut self, view: V) -> TypedWidgetId<V::Element> {
        let widget_id = self
            .app_state
            .add_widget(view, WidgetInsertPos::AfterLastChildOf(self.id));
        TypedWidgetId::new(widget_id)
    }

    /*pub fn remove_child(&mut self, i: usize) {
        let child_id = self.data().children[i];
        self.remove_child_by_id(child_id);
    }

    pub fn remove_child_by_id(&mut self, child_id: WidgetId) {
        invalidate_widget(self.app_state, child_id);
        self.app_state.remove_widget(child_id);
        request_layout(self.app_state, self.id);
    }

    pub fn replace_child<V: View>(&mut self, index: usize, view: V) {
        let id = self.data().children[index];
        self.app_state.replace_widget(id, view);
        request_layout(self.app_state, self.id);
    }

    pub fn swap_children(&mut self, from_index: usize, to_index: usize) {
        self.data_mut().children.swap(from_index, to_index);
        request_layout(self.app_state, self.id);
    }

    pub fn move_child(&mut self, from_index: usize, to_index: usize) {
        let children = &mut self.data_mut().children;
        let id = children.remove(from_index);
        self.data_mut().children.insert(to_index, id);
        request_layout(self.app_state, self.id);
    }*/

    pub fn local_bounds(&self) -> Rect {
        self.data().local_bounds()
    }

    pub fn global_bounds(&self) -> Rect {
        self.data().global_bounds()
    }

    pub fn request_layout(&mut self) {
        self.app_state.widgets.request_layout(self.id);
    }

    pub fn layout_requested(&self) -> bool {
        self.data().flag_is_set(WidgetFlags::NEEDS_LAYOUT)
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

    pub(super) fn unchecked_cast<W2: 'a + Widget + ?Sized>(self) -> WidgetMut<'a, W2> {
        unsafe { std::mem::transmute(self) }
    }
}

impl<'a> WidgetMut<'a, dyn Widget> {
    pub fn apply_diff_to_children<T, V: View>(
        &mut self,
        diff: DiffOp<'_, T>,
        view_fn: &dyn Fn(&T) -> V,
    ) {
        todo!()
        /*match diff {
            DiffOp::Remove { index, len } => {
                for i in (0..len).rev() {
                    self.remove_child(index + i);
                }
            }
            DiffOp::Replace { index, value } => self.replace_child(index, view_fn(value)),
            DiffOp::Insert { index, values } => {
                for (i, value) in values.iter().enumerate() {
                    self.insert_child(view_fn(value), index + i);
                }
            }
            DiffOp::Move { from, to } => {
                self.move_child(from, to);
            }
        }*/
    }
}

impl Deref for WidgetMut<'_, dyn Widget> {
    type Target = dyn Widget;

    fn deref(&self) -> &Self::Target {
        &self.app_state.widgets.widgets[self.id]
    }
}

impl<'a, W: 'a + Widget> Deref for WidgetMut<'a, W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        self.app_state.widgets.widgets[self.id].downcast_ref().unwrap()
    }
}

impl DerefMut for WidgetMut<'_, dyn Widget> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.app_state.widgets.widgets[self.id]
    }
}

impl<'a, W: 'a + Widget> DerefMut for WidgetMut<'a, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.app_state.widgets.widgets[self.id].downcast_mut().unwrap()
    }
}
