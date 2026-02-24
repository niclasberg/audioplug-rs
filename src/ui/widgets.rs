use std::{
    cell::Cell,
    collections::VecDeque,
    ops::{Deref, DerefMut},
};

use slotmap::{Key, SecondaryMap, SlotMap};

use crate::{
    core::{FxIndexSet, HAlign, Point, Rect, VAlign, Vec2, Zero},
    platform,
    ui::{
        OverlayAnchor, OverlayOptions, Widget, WidgetData, WidgetFlags, WidgetId, WidgetRef,
        WindowId,
        layout::RecomputeLayout,
        overlay::OverlayContainer,
        render::{GpuScene, WGPUSurface},
        widget_data::{ChildIdIter, SiblingWalker, WidgetDataMap},
    },
};

type ChildCache = SecondaryMap<WidgetId, Vec<WidgetId>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WidgetPos {
    Before(WidgetId),
    After(WidgetId),
    FirstChild(WidgetId),
    LastChild(WidgetId),
    Overlay(WidgetId, OverlayOptions),
}

pub(super) struct WindowState {
    pub(super) id: WindowId,
    pub(super) handle: platform::Handle,
    pub(super) wgpu_surface: WGPUSurface,
    pub(super) gpu_scene: GpuScene,
    pub(super) root_widget: WidgetId,
    pub(super) focus_widget: Option<WidgetId>,
    pub(super) overlays: OverlayContainer,
}

#[derive(Default)]
pub struct Widgets {
    /// Data (e.g. parent, layout, render scene etc.) associated with each widget
    pub(super) data: WidgetDataMap,
    /// Widget implementation. Should exist for each widget data.
    pub(super) widgets: SecondaryMap<WidgetId, Box<dyn Widget>>,
    windows: SlotMap<WindowId, WindowState>,
    /// (Lazy) cache of child ids. Taffy requires random access during layout.
    child_id_cache: ChildCache,
    /// Ids of all widgets that have had their child list changed. Cleared during call to rebuild_children.
    child_layout_changed: FxIndexSet<WidgetId>,
    /// Ids of all widgets that have requested animation. Cleared during each call to [drive_animations]
    pending_animations: FxIndexSet<WidgetId>,
    /// Temporary cache used to avoid allocations while performing traversals
    id_buffer: Cell<VecDeque<WidgetId>>,
    /// The widget that currently has mouse capture
    pub(super) mouse_capture_widget: Option<WidgetId>,
}

impl Widgets {
    pub fn get(&self, widget_id: WidgetId) -> WidgetRef<'_, dyn Widget> {
        WidgetRef::new(self, widget_id)
    }

    pub fn contains(&self, widget_id: WidgetId) -> bool {
        self.data.contains(widget_id)
    }

    /// Iterator over the ids of all siblings of a node
    pub fn sibling_id_iter(&self, widget_id: WidgetId) -> ChildIdIter<'_> {
        self.data.sibling_id_iter(widget_id)
    }

    /// Iterator over the ids of all children of a node
    pub fn child_id_iter(&self, widget_id: WidgetId) -> ChildIdIter<'_> {
        self.data.child_id_iter(widget_id)
    }

    pub fn window_id_iter(&self) -> impl Iterator<Item = WindowId> {
        self.windows.iter().map(|(k, _)| k)
    }

    pub(super) fn allocate_window(
        &mut self,
        handle: platform::Handle,
        wgpu_surface: WGPUSurface,
    ) -> WindowId {
        let window_id = self.windows.insert_with_key(|window_id| {
            let root_widget = self.data.insert_root(window_id);
            self.child_id_cache.insert(root_widget, Vec::new());
            WindowState {
                id: window_id,
                handle,
                wgpu_surface,
                root_widget,
                focus_widget: None,
                gpu_scene: GpuScene::new(),
                overlays: OverlayContainer::default(),
            }
        });

        window_id
    }

    /// Allocate a new widget which does not have an implementation.
    /// Make sure to call [set_widget_impl] afterwards (or there will be panics later)
    pub(super) fn allocate_widget(&mut self, position: WidgetPos) -> WidgetId {
        let id = match position {
            WidgetPos::Before(widget_id) => self.data.insert_before(widget_id),
            WidgetPos::After(widget_id) => self.data.insert_after(widget_id),
            WidgetPos::FirstChild(widget_id) => self.data.insert_first_child(widget_id),
            WidgetPos::LastChild(widget_id) => self.data.insert_last_child(widget_id),
            WidgetPos::Overlay(parent_id, overlay_options) => {
                let id = self.data.insert_last_child(parent_id);
                let data = &mut self.data[id];
                data.set_flag(WidgetFlags::OVERLAY);
                self.windows[data.window_id]
                    .overlays
                    .insert_or_update(id, overlay_options);
                id
            }
        };
        self.child_id_cache.insert(id, Vec::new());
        id
    }

    fn move_widget_before(&mut self, widget_id: WidgetId, next_id: WidgetId) -> WidgetId {
        let next = &mut self.data[next_id];
        let prev_id = std::mem::replace(&mut next.prev_sibling_id, widget_id);
        let new_parent_id = next.parent_id;
        assert!(!new_parent_id.is_null());
        self.data[prev_id].next_sibling_id = widget_id;

        let current = &mut self.data[widget_id];
        current.next_sibling_id = next_id;
        current.prev_sibling_id = prev_id;
        new_parent_id
    }

    pub fn move_widget(&mut self, widget_id: WidgetId, destination: WidgetPos) {
        let current = self
            .data
            .get_mut(widget_id)
            .expect("Widget should exist when being moved");
        let old_parent_id = current.parent_id;
        assert!(!old_parent_id.is_null(), "Cannot move a root widget");

        let new_parent_id = match destination {
            WidgetPos::Before(next_id) => self.move_widget_before(widget_id, next_id),
            WidgetPos::After(prev_id) => {
                self.move_widget_before(widget_id, self.data[prev_id].next_sibling_id)
            }
            WidgetPos::FirstChild(parent_id) => {
                let parent = &mut self.data[parent_id];
                let first_child_id = std::mem::replace(&mut parent.first_child_id, widget_id);
                if !first_child_id.is_null() {
                    self.move_widget_before(widget_id, first_child_id);
                }
                parent_id
            }
            WidgetPos::LastChild(parent_id) => {
                let parent = &mut self.data[parent_id];
                let first_child_id = parent.first_child_id;
                if first_child_id.is_null() {
                    parent.first_child_id = widget_id;
                } else {
                    self.move_widget_before(widget_id, first_child_id);
                }
                parent_id
            }
            WidgetPos::Overlay(parent_id, overlay_options) => {
                let parent = &mut self.data[parent_id];
                let first_overlay_id = parent.first_overlay_id;
                self.windows[parent.window_id]
                    .overlays
                    .insert_or_update(widget_id, overlay_options);
                if first_overlay_id.is_null() {
                    parent.first_overlay_id = widget_id;
                } else {
                    self.move_widget_before(widget_id, first_overlay_id);
                }
                parent_id
            }
        };

        let new_parent = &mut self.data[new_parent_id];
        self.child_layout_changed.insert(new_parent_id);
        let window_id = new_parent.window_id;

        let current = &mut self.data[widget_id];
        current.parent_id = new_parent_id;
        current.window_id = window_id;
    }

    pub fn swap_widgets(&mut self, src_id: WidgetId, dst_id: WidgetId) {
        self.data.swap(src_id, dst_id);
        self.request_layout(src_id);
        self.request_layout(dst_id);
    }

    fn internal_remove(&mut self, widget_id: WidgetId) -> WidgetData {
        self.child_id_cache.remove(widget_id);
        self.widgets.remove(widget_id);

        let data = self
            .data
            .unchecked_remove(widget_id)
            .expect("Widget being removed should exist");

        self.windows[data.window_id].overlays.remove(widget_id);

        data
    }

    // Remove a widget and all its children. The `f` function is invoked after each removed widget.
    pub(super) fn remove_widget(&mut self, widget_id: WidgetId, f: &mut impl FnMut(WidgetData)) {
        let data = &self.data[widget_id];
        let parent_id = data.parent_id;
        if !parent_id.is_null() {
            let parent = &mut self.data[parent_id];
        }
        self.remove_children(widget_id, f);
        f(self.internal_remove(widget_id));
    }

    // Remove all children of a widget. The `f` function is invoked after each removed widget.
    pub(super) fn remove_children(&mut self, widget_id: WidgetId, f: &mut impl FnMut(WidgetData)) {
        let mut children_to_remove = self.id_buffer.take();
        children_to_remove.clear();
        children_to_remove.extend(self.child_id_iter(widget_id));
        let data = &mut self.data[widget_id];
        data.first_child_id = WidgetId::null();

        while let Some(child_id) = children_to_remove.pop_front() {
            children_to_remove.extend(self.child_id_iter(child_id));
            f(self.internal_remove(child_id));
        }

        self.id_buffer.set(children_to_remove);
    }

    pub(super) fn remove_window(&mut self, window_id: WindowId, f: &mut impl FnMut(WidgetData)) {
        let root_widget = self.windows[window_id].root_widget;
        self.remove_widget(root_widget, f);
        self.windows.remove(window_id);
    }

    pub(super) fn window(&self, id: WindowId) -> &WindowState {
        self.windows.get(id).expect("Window handle not found")
    }

    pub(super) fn window_for_widget(&self, id: WidgetId) -> &WindowState {
        self.windows
            .get(self.data[id].window_id)
            .expect("Window handle not found")
    }

    pub(super) fn window_mut(&mut self, id: WindowId) -> &mut WindowState {
        self.windows.get_mut(id).expect("Window handle not found")
    }

    pub(super) fn root_widget_for_window(&self, id: WindowId) -> WidgetId {
        self.windows[id].root_widget
    }

    #[inline(always)]
    pub fn get_parent(&self, widget_id: WidgetId) -> WidgetId {
        self.data[widget_id].parent_id
    }

    pub fn has_parent(&self, child_id: WidgetId, parent_id: WidgetId) -> bool {
        let mut id = child_id;
        while !id.is_null() {
            id = self.data[id].parent_id;
            if id == parent_id {
                return true;
            }
        }
        false
    }

    pub fn widget_has_focus(&self, id: WidgetId) -> bool {
        self.windows[self.data.get(id).unwrap().window_id]
            .focus_widget
            .as_ref()
            .is_some_and(|focus_widget_id| *focus_widget_id == id)
    }

    pub fn focus_widget_id(&self, window_id: WindowId) -> Option<WidgetId> {
        self.windows[window_id].focus_widget
    }

    pub fn widget_has_captured_mouse(&self, widget_id: WidgetId) -> bool {
        self.mouse_capture_widget.is_some_and(|id| id == widget_id)
    }

    pub(crate) fn request_animation(&mut self, widget_id: WidgetId) {
        self.pending_animations.insert(widget_id);
    }

    /// Returns the ids of all widgets that have requested an animation frame.
    /// May contain ids of widgets that have been removed.
    pub(super) fn take_requested_animations(&mut self) -> FxIndexSet<WidgetId> {
        std::mem::take(&mut self.pending_animations)
    }

    pub fn request_render(&mut self, widget_id: WidgetId) {
        let data = &mut self.data[widget_id];
        data.set_flag(WidgetFlags::NEEDS_RENDER);
        self.windows[data.window_id]
            .handle
            .invalidate(data.global_bounds());
    }

    pub fn invalidate_widget(&self, widget_id: WidgetId) {
        let bounds = self.data[widget_id].global_bounds();
        let window_id = self.data[widget_id].window_id;
        self.windows[window_id].handle.invalidate(bounds);
    }

    pub(super) fn lease_widget(&mut self, id: WidgetId) -> Option<LeasedWidget> {
        self.widgets.remove(id).map(|w| LeasedWidget(id, w))
    }

    pub(super) fn unlease_widget(&mut self, widget: LeasedWidget) {
        self.widgets.insert(widget.0, widget.1);
    }

    /// Gets children of a widget. The order is from the root and down the tree (draw order).
    pub(super) fn get_widgets_at(&self, window_id: WindowId, pos: Point) -> Vec<WidgetId> {
        let mut widgets = Vec::new();
        let window = &self.windows[window_id];
        let mut roots = vec![window.root_widget];
        roots.extend(window.overlays.iter());

        let mut stack = self.id_buffer.take();
        stack.clear();

        for root_id in roots {
            if self.data[root_id].global_bounds().contains(pos) {
                stack.push_back(root_id);
                while let Some(current) = stack.pop_front() {
                    widgets.push(current);

                    let mut widget_id_iter = SiblingWalker::all_children(&self.data, current);
                    while let Some(child_id) = widget_id_iter.next_id(&self.data) {
                        let data = &self.data[child_id];
                        if data.global_bounds().contains(pos) && !data.is_overlay() {
                            stack.push_back(child_id)
                        }
                    }
                }
            }
        }

        self.id_buffer.set(stack);
        widgets
    }
}

// Layout
impl Widgets {
    pub(crate) fn request_layout(&mut self, widget_id: WidgetId) {
        let mut current = widget_id;
        // Mark the widget and all parents as needing layout until we hit the root,
        // an overlay or another node that already have marked as needing layout
        loop {
            if current.is_null() {
                break;
            }
            let data = &mut self.data.get(current).unwrap();
            if data.is_overlay() || data.flag_is_set(WidgetFlags::NEEDS_LAYOUT) {
                break;
            }

            data.set_flag(WidgetFlags::NEEDS_LAYOUT);
            current = data.parent_id;
        }
    }

    pub fn layout_window(&mut self, window_id: WindowId, mode: RecomputeLayout) {
        use super::layout::compute_root_layout;

        let window = &mut self.windows[window_id];
        let window_size = window.handle.global_bounds().size();
        let window_rect = Rect::from_origin(Point::ZERO, window_size);
        let root_id = window.root_widget;

        self.rebuild_children();

        // Need to layout root first, the overlay positions can depend on their parent positions
        if mode == RecomputeLayout::Force || self.data.get(root_id).unwrap().needs_layout() {
            let region_to_invalidate = compute_root_layout(self, root_id, window_size);
            self.data.update_node_origins(root_id, Point::ZERO);
            if let Some(region_to_invalidate) = region_to_invalidate {
                self.window(window_id)
                    .handle
                    .invalidate(region_to_invalidate);
            }
        }

        // This is currently in z-index order, we should probably iterate in insertion order
        // If an overlay's position depends on the position of a previously created overlay,
        // this will be wrong.
        let overlay_ids: Vec<_> = self.windows[window_id].overlays.iter().collect();
        for (i, overlay_id) in overlay_ids.into_iter().enumerate() {
            if mode == RecomputeLayout::Force || self.data.get(overlay_id).unwrap().needs_layout() {
                let region_to_invalidate = compute_root_layout(self, root_id, window_size);
                let options = self
                    .window(window_id)
                    .overlays
                    .get_overlay_options(i)
                    .unwrap();

                let offset = self.compute_overlay_offset(window_rect, overlay_id, options);
                self.data
                    .update_node_origins(overlay_id, offset.into_point());
                if let Some(region_to_invalidate) = region_to_invalidate {
                    self.window(window_id)
                        .handle
                        .invalidate(region_to_invalidate.offset(offset));
                }
            }
        }
    }

    fn rebuild_children(&mut self) {
        for widget_id in std::mem::take(&mut self.child_layout_changed) {
            let mut children = std::mem::take(&mut self.child_id_cache[widget_id]);
            children.clear();
            children.extend(self.child_id_iter(widget_id));
            self.child_id_cache[widget_id] = children;
        }
    }

    fn compute_overlay_offset(
        &self,
        window_rect: Rect,
        overlay_id: WidgetId,
        options: OverlayOptions,
    ) -> Vec2 {
        let current_bounds = self.data.get(overlay_id).unwrap().local_bounds();
        let parent_id = self.data.get(overlay_id).unwrap().parent_id;
        let parent_bounds = self.data.get(parent_id).unwrap().global_bounds();
        let alignment_offset = match options.anchor {
            OverlayAnchor::Fixed => options.align.compute_offset(current_bounds, window_rect),
            OverlayAnchor::InsideParent => {
                options.align.compute_offset(current_bounds, parent_bounds)
            }
            OverlayAnchor::OutsideParent => {
                let mut result = options.align.compute_offset(current_bounds, parent_bounds);
                match options.align.get_h_align() {
                    HAlign::Left => result.x -= current_bounds.width(),
                    HAlign::Right => result.x += current_bounds.width(),
                    _ => {}
                };
                match options.align.get_v_align() {
                    VAlign::Top => result.y -= current_bounds.height(),
                    VAlign::Bottom => result.y += current_bounds.height(),
                    _ => {}
                }
                result
            }
        };

        alignment_offset + options.offset
    }

    /// Get the (cached) child ids for a widget. You must call rebuild_children before using this method.
    pub(super) fn cached_child_ids(&self, widget_id: WidgetId) -> &Vec<WidgetId> {
        debug_assert!(self.child_layout_changed.is_empty());
        &self.child_id_cache[widget_id]
    }
}

pub struct LeasedWidget(WidgetId, Box<dyn Widget>);

impl Deref for LeasedWidget {
    type Target = dyn Widget;

    fn deref(&self) -> &Self::Target {
        self.1.deref()
    }
}

impl DerefMut for LeasedWidget {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.1.deref_mut()
    }
}
