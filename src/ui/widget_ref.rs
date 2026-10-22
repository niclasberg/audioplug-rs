use std::fmt::Debug;

use slotmap::Key;

use super::{OverlayOptions, View, WidgetData, WidgetFlags, WidgetId, style::Style};
use crate::{
    core::{Rect, diff::DiffOp},
    ui::{
        WidgetPos, Widgets,
        reactive::ReactiveGraph,
        style::{DisplayStyle, Length},
        text::TextData,
        widget_tree::SiblingWalker,
        widgets::WidgetHandlers,
    },
};

pub struct WidgetNotFound<'a>(WidgetMut<'a>);

impl<'a> Debug for WidgetNotFound<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("WidgetNotFound").field(&self.0.id).finish()
    }
}

pub struct WidgetRefIter<'a> {
    widgets: &'a Widgets,
    inner: SiblingWalker,
}

impl<'a> Iterator for WidgetRefIter<'a> {
    type Item = WidgetRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next_id(&self.widgets.tree)
            .map(|id| WidgetRef::new(self.widgets, id))
    }
}

impl<'a> DoubleEndedIterator for WidgetRefIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back_id(&self.widgets.tree)
            .map(|id| WidgetRef::new(self.widgets, id))
    }
}

pub struct WidgetRef<'a> {
    widgets: &'a Widgets,
    id: WidgetId,
}

impl<'a> WidgetRef<'a> {
    pub(super) fn new(widgets: &'a Widgets, id: WidgetId) -> Self {
        Self { widgets, id }
    }

    pub fn id(&self) -> WidgetId {
        self.id
    }

    pub fn data(&self) -> &WidgetData {
        self.widgets.tree.get(self.id).unwrap()
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
        self.widgets.has_focus(self.id)
    }

    pub fn has_mouse_capture(&self) -> bool {
        self.widgets.is_pressed(self.id)
    }

    pub fn child_iter(&self) -> WidgetRefIter<'_> {
        WidgetRefIter {
            inner: self.widgets.tree.child_id_walker(self.id),
            widgets: self.widgets,
        }
    }

    pub fn for_each_child_ref(&self, mut f: impl FnMut(WidgetRef<'_>)) {
        for_each_child_ref_impl(self.widgets, self.id, &mut f);
    }

    pub fn parent_id(&self) -> WidgetId {
        self.data().parent_id
    }

    pub fn parent<'s>(&'s self) -> Option<WidgetRef<'s>> {
        let parent_id = self.data().parent_id;
        if !parent_id.is_null() {
            Some(WidgetRef {
                widgets: self.widgets,
                id: parent_id,
            })
        } else {
            None
        }
    }
}

fn for_each_child_ref_impl(widgets: &Widgets, id: WidgetId, f: &mut dyn FnMut(WidgetRef<'_>)) {
    let mut child_iter = widgets.tree.child_id_walker(id);
    while let Some(child_id) = child_iter.next_id(&widgets.tree) {
        f(WidgetRef::new(widgets, child_id))
    }
}

pub struct WidgetMut<'a> {
    pub(super) widgets: &'a mut Widgets,
    pub(super) handlers: &'a mut WidgetHandlers,
    pub(super) reactive_graph: &'a mut ReactiveGraph,
    pub(super) id: WidgetId,
}

impl<'a> WidgetMut<'a> {
    pub(super) fn new(
        widgets: &'a mut Widgets,
        handlers: &'a mut WidgetHandlers,
        reactive_graph: &'a mut ReactiveGraph,
        id: WidgetId,
    ) -> Self {
        Self {
            widgets,
            id,
            reactive_graph,
            handlers,
        }
    }

    fn data(&self) -> &WidgetData {
        self.widgets.tree.get(self.id).unwrap()
    }

    fn data_mut(&mut self) -> &mut WidgetData {
        self.widgets.tree.get_mut(self.id).unwrap()
    }

    pub fn index_of_child(&self, id: WidgetId) -> Option<usize> {
        self.child_iter().position(|w| w.id == id)
    }

    pub fn find_child(self, id: WidgetId) -> Result<WidgetMut<'a>, WidgetNotFound<'a>> {
        if self.widgets.parent(id) == self.id {
            Ok(WidgetMut {
                widgets: self.widgets,
                id,
                handlers: self.handlers,
                reactive_graph: self.reactive_graph,
            })
        } else {
            Err(WidgetNotFound(self))
        }
    }

    pub fn find_sibling(self, id: WidgetId) -> Result<WidgetMut<'a>, WidgetNotFound<'a>> {
        let found = self
            .widgets
            .sibling_id_iter(self.id)
            .any(|child_id| child_id == id);

        if found {
            Ok(WidgetMut {
                widgets: self.widgets,
                id,
                handlers: self.handlers,
                reactive_graph: self.reactive_graph,
            })
        } else {
            Err(WidgetNotFound(self))
        }
    }

    pub fn child_iter(&self) -> WidgetRefIter<'_> {
        WidgetRefIter {
            inner: self.widgets.tree.child_id_walker(self.id),
            widgets: &self.widgets,
        }
    }

    pub fn for_each_child_ref(&self, mut f: impl FnMut(WidgetRef<'_>)) {
        for_each_child_ref_impl(&self.widgets, self.id, &mut f);
    }

    pub fn for_each_child_mut(&mut self, mut f: impl FnMut(WidgetMut<'_>)) {
        let mut child_iter = self.widgets.tree.child_id_walker(self.id);
        let mut current = child_iter.next_id(&self.widgets.tree);
        while let Some(child_id) = current {
            // This copy is needed - the widget might be removed while visited
            let next = child_iter.next_id(&self.widgets.tree);
            f(WidgetMut {
                widgets: self.widgets,
                handlers: self.handlers,
                reactive_graph: self.reactive_graph,
                id: child_id,
            });
            current = next;
        }
    }

    pub fn remove(self) {
        self.widgets
            .remove_widget(self.id, self.reactive_graph, self.handlers);
    }

    pub fn replace(self, view: impl View) -> WidgetMut<'a> {
        self.widgets
            .replace_widget(self.id, self.reactive_graph, self.handlers, view);
        self
    }

    pub fn add_before(&mut self, view: impl View) -> WidgetId {
        self.add_widget(view, WidgetPos::Before(self.id))
    }

    fn add_widget(&mut self, view: impl View, pos: WidgetPos) -> WidgetId {
        self.widgets
            .add_widget(self.handlers, self.reactive_graph, pos, view)
    }

    pub fn add_after(&mut self, view: impl View) -> WidgetId {
        self.add_widget(view, WidgetPos::After(self.id))
    }

    pub fn push_child_front(&mut self, view: impl View) -> WidgetId {
        self.add_widget(view, WidgetPos::FirstChild(self.id))
    }

    pub fn push_child_back(&mut self, view: impl View) -> WidgetId {
        self.add_widget(view, WidgetPos::LastChild(self.id))
    }

    pub fn add_overlay(&mut self, view: impl View, options: OverlayOptions) -> WidgetId {
        self.add_widget(view, WidgetPos::Overlay(self.id, options))
    }

    pub fn swap_with(&mut self, widget_id: WidgetId) {
        self.widgets.swap_widgets(self.id, widget_id);
    }

    pub fn local_bounds(&self) -> Rect {
        self.data().local_bounds()
    }

    pub fn global_bounds(&self) -> Rect {
        self.data().global_bounds()
    }

    pub fn request_layout(&mut self) {
        self.widgets.request_layout(self.id);
    }

    pub fn layout_requested(&self) -> bool {
        self.data().flag_is_set(WidgetFlags::NEEDS_LAYOUT)
    }

    pub fn request_render(&mut self) {
        self.widgets.request_render(self.id);
    }

    pub fn update_style(&mut self, f: impl FnOnce(&mut Style)) {
        f(&mut self.data_mut().style);
    }

    pub fn update_text(&mut self, f: impl FnOnce(&mut TextData)) {
        if let Some(text_data) = self.widgets.texts.get_mut(self.id) {
            f(text_data)
        }
    }

    pub fn set_display_style(&mut self, display_style: DisplayStyle) {
        self.data_mut().display_style = display_style;
        self.widgets.request_layout(self.id);
    }

    pub fn style(&self) -> &Style {
        &self.data().style
    }
}

impl<'a> WidgetMut<'a> {
    pub fn apply_diff_to_children<T, V: View>(
        &mut self,
        _diff: DiffOp<'_, T>,
        _view_fn: &dyn Fn(&T) -> V,
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

pub enum WidgetPropValue {
    Height(Length),
    Width(Length),
}

impl WidgetPropValue {}
