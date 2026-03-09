use slotmap::{Key, SecondaryMap, SlotMap};

use crate::{
    core::{FxIndexSet, HAlign, Point, Rect, VAlign, Vec2, Zero},
    platform,
    ui::{
        OverlayAnchor, OverlayOptions, Scene, Widget, WidgetFlags, WidgetId, WindowId,
        app_state::WidgetMap,
        clipboard::Clipboard,
        layout::{LayoutContext, RecomputeLayout},
        overlay::OverlayContainer,
        reactive::ReactiveGraph,
        render::{GpuScene, WGPUSurface},
        widget_tree::{ChildIdIter, WidgetTree},
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
    /// Data (e.g. parent/children, layout, position etc.) associated with each widget
    pub(crate) tree: WidgetTree,
    pub(super) scenes: SecondaryMap<WidgetId, Scene>,
    pub(super) layout_cache: SecondaryMap<WidgetId, taffy::Cache>,
    pub(super) windows: SlotMap<WindowId, WindowState>,
    /// (Lazy) cache of child ids. Taffy requires random access during layout.
    child_id_cache: ChildCache,
    /// Ids of all widgets that have requested animation. Cleared during each call to [drive_animations]
    pending_animations: FxIndexSet<WidgetId>,
    /// The widget that currently has mouse capture
    pub(super) mouse_capture_widget: Option<WidgetId>,
}

impl Widgets {
    pub fn contains(&self, widget_id: WidgetId) -> bool {
        self.tree.contains(widget_id)
    }

    /// Iterator over the ids of all siblings of a node
    pub fn sibling_id_iter(&self, widget_id: WidgetId) -> ChildIdIter<'_> {
        self.tree.sibling_id_iter(widget_id)
    }

    /// Iterator over the ids of all children of a node
    pub fn child_id_iter(&self, widget_id: WidgetId) -> ChildIdIter<'_> {
        self.tree.child_id_iter(widget_id)
    }

    pub fn window_id_iter(&self) -> impl Iterator<Item = WindowId> {
        self.windows.iter().map(|(k, _)| k)
    }

    pub(super) fn allocate_window(
        &mut self,
        handle: platform::Handle,
        wgpu_surface: WGPUSurface,
    ) -> WindowId {
        self.windows.insert_with_key(|window_id| {
            let root_widget = self.tree.insert_root(window_id);
            self.child_id_cache.insert(root_widget, Vec::new());
            self.layout_cache.insert(root_widget, Default::default());
            self.scenes.insert(root_widget, Default::default());
            WindowState {
                id: window_id,
                handle,
                wgpu_surface,
                root_widget,
                focus_widget: None,
                gpu_scene: GpuScene::new(),
                overlays: OverlayContainer::default(),
            }
        })
    }

    /// Allocate a new widget which does not have an implementation.
    pub(super) fn allocate_widget(&mut self, position: WidgetPos) -> WidgetId {
        let id = match position {
            WidgetPos::Before(widget_id) => self.tree.insert_before(widget_id),
            WidgetPos::After(widget_id) => self.tree.insert_after(widget_id),
            WidgetPos::FirstChild(widget_id) => self.tree.insert_first_child(widget_id),
            WidgetPos::LastChild(widget_id) => self.tree.insert_last_child(widget_id),
            WidgetPos::Overlay(parent_id, overlay_options) => {
                let id = self.tree.insert_last_child(parent_id);
                let data = &mut self.tree[id];
                data.set_flag(WidgetFlags::OVERLAY);
                self.windows[data.window_id]
                    .overlays
                    .insert_or_update(id, overlay_options);
                id
            }
        };
        self.child_id_cache.insert(id, Vec::new());
        self.layout_cache.insert(id, Default::default());
        self.scenes.insert(id, Default::default());
        id
    }

    pub fn swap_widgets(&mut self, src_id: WidgetId, dst_id: WidgetId) {
        self.tree.swap(src_id, dst_id);
        self.request_layout(src_id);
        self.request_layout(dst_id);
    }

    // Remove a widget, all its children and all owned reactive nodes
    pub(super) fn remove_widget(
        &mut self,
        widget_id: WidgetId,
        reactive_graph: &mut ReactiveGraph,
        widget_impls: &mut WidgetMap,
    ) {
        self.tree.remove(widget_id, |data| {
            self.child_id_cache.remove(data.id);
            self.scenes.remove(data.id);
            self.layout_cache.remove(data.id);
            self.windows[data.window_id].overlays.remove(data.id);
            widget_impls.remove(widget_id);
            reactive_graph.remove_all_siblings(data.first_owned_node_id);
        });
    }

    pub fn reset_widget(
        &mut self,
        widget_id: WidgetId,
        reactive_graph: &mut ReactiveGraph,
        widget_impls: &mut WidgetMap,
    ) {
        self.tree.reset(widget_id, |data| {
            self.child_id_cache.remove(data.id);
            self.scenes.remove(data.id);
            self.layout_cache.remove(data.id);
            self.windows[data.window_id].overlays.remove(data.id);
            widget_impls.remove(widget_id);
            reactive_graph.remove_all_siblings(data.first_owned_node_id);
        });
        // self.scenes[widget_id]; <- clear
        self.layout_cache[widget_id].clear();
        self.request_layout(widget_id);
    }

    pub(super) fn remove_window(
        &mut self,
        window_id: WindowId,
        reactive_graph: &mut ReactiveGraph,
        widget_impls: &mut WidgetMap,
    ) {
        let root_widget = self.windows[window_id].root_widget;
        self.remove_widget(root_widget, reactive_graph, widget_impls);
        self.windows.remove(window_id);
    }

    pub(super) fn window(&self, id: WindowId) -> &WindowState {
        self.windows.get(id).expect("Window handle not found")
    }

    pub(super) fn window_for_widget(&self, id: WidgetId) -> &WindowState {
        self.windows
            .get(self.tree[id].window_id)
            .expect("Window handle not found")
    }

    pub(super) fn window_mut(&mut self, id: WindowId) -> &mut WindowState {
        self.windows.get_mut(id).expect("Window handle not found")
    }

    pub fn clipboard(&self, window_id: WindowId) -> Clipboard<'_> {
        Clipboard {
            handle: &self.window(window_id).handle,
        }
    }

    pub fn local_bounds(&self, id: WidgetId) -> Rect {
        self.tree[id].local_bounds()
    }

    pub fn global_bounds(&self, id: WidgetId) -> Rect {
        self.tree[id].global_bounds()
    }

    pub fn content_bounds(&self, id: WidgetId) -> Rect {
        self.tree[id].content_bounds()
    }

    #[inline(always)]
    pub fn parent(&self, widget_id: WidgetId) -> WidgetId {
        self.tree[widget_id].parent_id
    }

    pub fn has_parent(&self, child_id: WidgetId, parent_id: WidgetId) -> bool {
        let mut id = child_id;
        while !id.is_null() {
            id = self.tree[id].parent_id;
            if id == parent_id {
                return true;
            }
        }
        false
    }

    pub fn has_focus(&self, id: WidgetId) -> bool {
        self.windows[self.tree.get(id).unwrap().window_id]
            .focus_widget
            .as_ref()
            .is_some_and(|focus_widget_id| *focus_widget_id == id)
    }

    pub fn focus_widget_id(&self, window_id: WindowId) -> Option<WidgetId> {
        self.windows[window_id].focus_widget
    }

    pub fn has_mouse_capture(&self, widget_id: WidgetId) -> bool {
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
        let data = &mut self.tree[widget_id];
        data.set_flag(WidgetFlags::NEEDS_RENDER);
        self.windows[data.window_id]
            .handle
            .invalidate(data.global_bounds());
    }

    pub fn invalidate_widget(&self, widget_id: WidgetId) {
        let bounds = self.tree[widget_id].global_bounds();
        let window_id = self.tree[widget_id].window_id;
        self.windows[window_id].handle.invalidate(bounds);
    }

    /// Gets children of a widget. The order is from the root and down the tree (draw order).
    pub(super) fn get_widgets_at(&self, window_id: WindowId, pos: Point) -> Vec<WidgetId> {
        let mut widgets = Vec::new();

        let window = &self.windows[window_id];
        for root_id in std::iter::once(window.root_widget).chain(window.overlays.iter()) {
            let mut walker = self.tree.dfs_walker_with_pruning(root_id, |data| {
                data.global_bounds().contains(pos) && !data.is_overlay()
            });
            while let Some(widget_id) = walker.next(&self.tree) {
                widgets.push(widget_id);
            }
        }

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
            let data = &mut self.tree.get(current).unwrap();
            if data.is_overlay() || data.flag_is_set(WidgetFlags::NEEDS_LAYOUT) {
                break;
            }

            data.set_flag(WidgetFlags::NEEDS_LAYOUT);
            current = data.parent_id;
        }
    }

    pub fn layout_window(
        &mut self,
        widget_impls: &WidgetMap,
        window_id: WindowId,
        mode: RecomputeLayout,
    ) {
        let window = &mut self.windows[window_id];
        let window_size = window.handle.global_bounds().size();
        let window_rect = Rect::from_origin(Point::ZERO, window_size);
        let root_id = window.root_widget;
        self.rebuild_children();

        // Need to layout root first, the overlay positions can depend on their parent positions
        if mode == RecomputeLayout::Force || self.tree.get(root_id).unwrap().needs_layout() {
            let region_to_invalidate =
                LayoutContext::new(self, widget_impls, window_size).compute_root_layout(root_id);
            self.tree.update_node_origins(root_id, Point::ZERO);
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
            if mode == RecomputeLayout::Force || self.tree.get(overlay_id).unwrap().needs_layout() {
                let region_to_invalidate = LayoutContext::new(self, widget_impls, window_size)
                    .compute_root_layout(root_id);
                let options = self
                    .window(window_id)
                    .overlays
                    .get_overlay_options(i)
                    .unwrap();

                let offset = self.compute_overlay_offset(window_rect, overlay_id, options);
                self.tree
                    .update_node_origins(overlay_id, offset.into_point());
                if let Some(region_to_invalidate) = region_to_invalidate {
                    self.window(window_id)
                        .handle
                        .invalidate(region_to_invalidate.offset(offset));
                }
            }
        }

        self.print_tree(widget_impls, window_id);
    }

    fn rebuild_children(&mut self) {
        for (widget_id, child_ids) in self.child_id_cache.iter_mut() {
            child_ids.clear();
            child_ids.extend(self.tree.child_id_iter(widget_id));
        }
    }

    fn compute_overlay_offset(
        &self,
        window_rect: Rect,
        overlay_id: WidgetId,
        options: OverlayOptions,
    ) -> Vec2 {
        let current_bounds = self.tree.get(overlay_id).unwrap().local_bounds();
        let parent_id = self.tree.get(overlay_id).unwrap().parent_id;
        let parent_bounds = self.tree.get(parent_id).unwrap().global_bounds();
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
    pub(super) fn cached_child_ids(&self, widget_id: WidgetId) -> &[WidgetId] {
        self.child_id_cache[widget_id].as_slice()
    }

    pub fn print_tree(&self, widget_impls: &WidgetMap, window_id: WindowId) {
        fn _impl(this: &Widgets, widget_impls: &WidgetMap, root_id: WidgetId, indent: usize) {
            println!(
                "{} {}({:?})",
                "-".repeat(indent),
                widget_impls[root_id].debug_label(),
                root_id
            );
            let mut walker = this.tree.child_id_walker(root_id);
            while let Some(child_id) = walker.next_id(&this.tree) {
                _impl(this, widget_impls, child_id, indent + 1);
            }
        }
        let root_id = self.window(window_id).root_widget;
        _impl(self, widget_impls, root_id, 0);
    }
}
