use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
    rc::Rc,
    time::Instant,
};

use slotmap::{Key, SecondaryMap, SlotMap};
use smallvec::SmallVec;

use crate::{
    core::{FxHashMap, FxHashSet, FxIndexSet, HAlign, Point, Rect, VAlign, Vec2, Zero},
    param::{AnyParameterMap, ParamRef, ParameterId},
    platform,
    ui::{
        BuildContext, HostHandle, OverlayAnchor, OverlayOptions, Sense, TaskQueue, View,
        WidgetFlags, WidgetId, WindowId,
        clipboard::Clipboard,
        interaction::Interactions,
        layout::{LayoutContext, RecomputeLayout},
        overlay::OverlayContainer,
        reactive::{
            EffectContext, NodeId, NodeState, NodeType, Owner, Source, WidgetPhase, WidgetStatus,
            WidgetStatusFlags,
        },
        reactive_graph::ReactiveGraph,
        render::{GpuScene, RendererMap, WGPUSurface},
        style::Style,
        text::{TextContext, TextData},
        widget_prop::WidgetPropMap,
        widget_tree::{ChildIdIter, WidgetTree},
    },
};

type ChildCache = SecondaryMap<WidgetId, Vec<WidgetId>>;
type LayoutCache = SecondaryMap<WidgetId, taffy::Cache>;

pub struct UiState {
    /// Data (e.g. parent/children, layout, position etc.) associated with each widget
    pub(crate) tree: WidgetTree,
    pub(super) layout_cache: LayoutCache,
    pub(super) texts: SecondaryMap<WidgetId, TextData>,
    pub(super) windows: SlotMap<WindowId, WindowState>,
    /// (Lazy) cache of child ids. Taffy requires random access during layout.
    child_id_cache: ChildCache,
    dirty_props_widgets: FxHashSet<WidgetId>,
    /// The current hovered widget, if any
    pub(super) hovered_widget: Option<WidgetId>,
    /// The current active mouse gesture, if any
    pub(super) gesture: Option<Gesture>,
    pub(crate) parameters: Rc<dyn AnyParameterMap>,
    pub(crate) reactive_graph: ReactiveGraph,
    pub(super) effects_to_check: FxIndexSet<NodeId>,
    node_id_buffer: VecDeque<NodeId>,
    pub(super) pending_animations: FxIndexSet<NodeId>,
    pub(super) host_handle: Option<Box<dyn HostHandle>>,
}

/// Contains widget callbacks for event handling, prop synchronization and rendering.
/// We keep this separate from the Widgets, since we need to mutably borrow
/// the handlers while doing dispatch
#[derive(Default)]
pub struct WidgetHandlers {
    pub(super) event_handlers: Interactions,
    pub(super) renderers: RendererMap,
    pub(super) widget_props: WidgetPropMap,
}

impl WidgetHandlers {
    pub fn remove(&mut self, widget_id: WidgetId) {
        self.event_handlers.remove(widget_id);
        self.renderers.remove(widget_id);
        self.widget_props.remove(widget_id);
    }
}

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

#[derive(Debug, Clone, Copy)]
pub struct Gesture {
    pub target: WidgetId,
    pub start: Point,
    pub last: Point,
    pub state: GestureState,
}

impl Gesture {
    pub fn new(target: WidgetId, start: Point, state: GestureState) -> Self {
        Self {
            target,
            start,
            last: start,
            state,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureState {
    Pressed,
    Dragging,
}

pub(crate) struct LeasedNode(NodeId, NodeType);
impl Deref for LeasedNode {
    type Target = NodeType;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl DerefMut for LeasedNode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

impl UiState {
    pub fn new(parameter_map: Rc<dyn AnyParameterMap>) -> Self {
        let mut parameter_subscriptions = FxHashMap::default();
        for parameter_id in parameter_map.parameter_ids() {
            parameter_subscriptions.insert(parameter_id, SmallVec::new());
        }
        let reactive_graph = ReactiveGraph::new(parameter_subscriptions);

        Self {
            parameters: parameter_map,
            node_id_buffer: Default::default(),
            pending_animations: Default::default(),
            host_handle: None,
            effects_to_check: Default::default(),
            tree: Default::default(),
            layout_cache: Default::default(),
            texts: Default::default(),
            windows: Default::default(),
            child_id_cache: Default::default(),
            dirty_props_widgets: Default::default(),
            hovered_widget: None,
            gesture: None,
            reactive_graph,
        }
    }

    pub(crate) fn set_host_handle(&mut self, host_handle: Option<Box<dyn HostHandle>>) {
        self.host_handle = host_handle;
    }

    pub fn host_handle(&self) -> &dyn HostHandle {
        let Some(host_handle) = self.host_handle.as_ref() else {
            panic!("Host handle not set")
        };
        host_handle.as_ref()
    }

    pub fn contains_widget(&self, widget_id: WidgetId) -> bool {
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

    /// Gets all widgets at a point in a window. The order is from the root and down the tree (draw order).
    pub fn get_widgets_at(&self, window_id: WindowId, pos: Point) -> Vec<WidgetId> {
        let mut widgets = Vec::new();

        let window = &self.windows[window_id];
        for root_id in std::iter::once(window.root_widget).chain(window.overlays.iter()) {
            let mut current = Some(root_id);
            while let Some(widget_id) = current {
                let data = &self.tree[widget_id];
                // Overlays are part of the root widget hierarchy (for event bubbling),
                // but we don't want to recurse into children that are overlays here - that
                // would lead to us visiting the overlays multiple times.
                let is_child_overlay = widget_id != root_id && data.is_overlay();
                if data.global_bounds().contains(pos) && !is_child_overlay {
                    widgets.push(widget_id);
                    current = self.tree.next_dfs(widget_id, root_id);
                } else {
                    current = self.tree.next_subtree_dfs(widget_id, root_id);
                };
            }
        }
        widgets
    }

    pub fn window_id_iter(&self) -> impl Iterator<Item = WindowId> {
        self.windows.iter().map(|(k, _)| k)
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
}

// Window, node and widget creation/removal
impl UiState {
    pub(super) fn allocate_window(
        &mut self,
        handle: platform::Handle,
        wgpu_surface: WGPUSurface,
    ) -> WindowId {
        self.windows.insert_with_key(|window_id| {
            let root_widget = self.tree.insert_root(window_id);
            self.child_id_cache.insert(root_widget, Vec::new());
            self.layout_cache.insert(root_widget, Default::default());
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

    pub fn add_widget(
        &mut self,
        handlers: &mut WidgetHandlers,
        position: WidgetPos,
        view: impl View,
    ) -> WidgetId {
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

        view.build(&mut BuildContext::new(id, self, handlers));

        self.request_layout(id);
        id
    }

    pub fn swap_widgets(&mut self, src_id: WidgetId, dst_id: WidgetId) {
        self.tree.swap(src_id, dst_id);
        self.request_layout(src_id);
        self.request_layout(dst_id);
    }

    // Remove a widget, all its children and all owned reactive nodes
    pub fn remove_widget(&mut self, widget_id: WidgetId, handlers: &mut WidgetHandlers) {
        // Request layout before removing, so that all parent widgets gets marked as needing layout
        self.request_layout(widget_id);
        self.request_render(widget_id);
        self.clear_mouse_capture_and_focus(widget_id);

        self.tree.remove(widget_id, |_, data| {
            self.child_id_cache.remove(data.id);
            self.layout_cache.remove(data.id);
            self.texts.remove(data.id);
            self.windows[data.window_id].overlays.remove(data.id);
            handlers.remove(data.id);
            self.reactive_graph.clear_widget(data.id);
        });
    }

    /// Resets a widget to its default, removing all children. Does not remove owned reactive nodes
    pub fn reset_widget(&mut self, widget_id: WidgetId, handlers: &mut WidgetHandlers) {
        self.clear_mouse_capture_and_focus(widget_id);
        self.tree.reset(widget_id, |data| {
            self.child_id_cache.remove(data.id);
            self.layout_cache.remove(data.id);
            self.texts.remove(data.id);
            self.windows[data.window_id].overlays.remove(data.id);
            handlers.remove(data.id);
            self.reactive_graph.clear_widget(data.id);
        });

        self.child_id_cache[widget_id].clear();
        self.layout_cache[widget_id].clear();
        self.texts.remove(widget_id);
        handlers.remove(widget_id);
    }

    pub fn replace_widget(
        &mut self,
        widget_id: WidgetId,
        handlers: &mut WidgetHandlers,
        view: impl View,
    ) {
        self.reset_widget(widget_id, handlers);
        self.reactive_graph.clear_widget(widget_id);
        view.build(&mut BuildContext::new(widget_id, self, handlers));
        self.request_layout(widget_id);
    }

    pub(super) fn remove_window(&mut self, window_id: WindowId, handlers: &mut WidgetHandlers) {
        let root_widget = self.windows[window_id].root_widget;
        self.remove_widget(root_widget, handlers);
        self.windows.remove(window_id);
    }

    /// Create a new reactive node
    pub(super) fn create_node(
        &mut self,
        node_type: NodeType,
        state: NodeState,
        owner: Owner,
    ) -> NodeId {
        self.reactive_graph.create_node(node_type, state, owner)
    }

    pub fn remove_node(&mut self, node_id: NodeId) {
        self.reactive_graph.remove_node(node_id);
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

    pub fn request_render(&self, widget_id: WidgetId) {
        let data = &self.tree[widget_id];
        data.set_flag(WidgetFlags::NEEDS_RENDER);
        self.windows[data.window_id]
            .handle
            .invalidate(data.global_bounds());
    }

    pub fn request_prop_update(&mut self, widget_id: WidgetId) {
        self.dirty_props_widgets.insert(widget_id);
    }

    pub fn style(&self, widget_id: WidgetId) -> &Style {
        &self.tree[widget_id].style
    }

    pub fn style_mut(&mut self, widget_id: WidgetId) -> &mut Style {
        &mut self.tree[widget_id].style
    }

    pub fn text_mut(&mut self, widget_id: WidgetId) -> &mut TextData {
        self.texts.entry(widget_id).unwrap().or_default()
    }

    pub fn hidden(&self, widget_id: WidgetId) -> bool {
        self.tree[widget_id].style.hidden
    }
}

// Focus and gestures
impl UiState {
    fn clear_mouse_capture_and_focus(&mut self, id: WidgetId) {
        if let Some(hovered_widget) = self.hovered_widget
            && (hovered_widget == id || self.has_parent(hovered_widget, id))
        {
            self.set_hovered_widget(None);
        }

        let window_id = self.window_for_widget(id).id;
        if let Some(focus_widget) = self.focus_widget_id(window_id)
            && (focus_widget == id || self.has_parent(focus_widget, id))
        {
            self.set_focus_widget(window_id, None);
        }
    }

    pub fn has_focus(&self, id: WidgetId) -> bool {
        self.windows[self.tree[id].window_id]
            .focus_widget
            .as_ref()
            .is_some_and(|focus_widget_id| *focus_widget_id == id)
    }

    pub fn focus_widget_id(&self, window_id: WindowId) -> Option<WidgetId> {
        self.windows[window_id].focus_widget
    }

    pub fn next_focusable_widget(
        &mut self,
        handlers: &Interactions,
        window_id: WindowId,
    ) -> Option<WidgetId> {
        let &WindowState {
            root_widget,
            focus_widget,
            ..
        } = &self.windows[window_id];

        let mut candidate = focus_widget
            .and_then(|focus_widget| self.tree.next_dfs(focus_widget, root_widget))
            .unwrap_or(root_widget);
        loop {
            if self.is_hidden(candidate) {
                candidate = self.tree.next_subtree_dfs(candidate, root_widget)?;
            } else if handlers.has_sense(candidate, Sense::FOCUSABLE) {
                return Some(candidate);
            } else {
                candidate = self.tree.next_dfs(candidate, root_widget)?;
            }
        }
    }

    pub fn prev_focusable_widget(
        &self,
        handlers: &Interactions,
        window_id: WindowId,
    ) -> Option<WidgetId> {
        let &WindowState {
            root_widget,
            focus_widget,
            ..
        } = &self.windows[window_id];
        let mut candidate = focus_widget
            .and_then(|focus_widget| self.tree.prev_dfs(focus_widget))
            .unwrap_or(root_widget);
        loop {
            if self.is_hidden(candidate) || !handlers.has_sense(candidate, Sense::FOCUSABLE) {
                candidate = self.tree.prev_dfs(candidate)?;
            } else {
                return Some(candidate);
            }
        }
    }

    pub fn set_focus_widget(&mut self, window_id: WindowId, new_focus_widget: Option<WidgetId>) {
        let old_focus_widget = self.focus_widget_id(window_id);
        if new_focus_widget != old_focus_widget {
            println!(
                "Focus change {:?}, {:?}",
                old_focus_widget, new_focus_widget
            );

            self.window_mut(window_id).focus_widget = new_focus_widget;

            if let Some(old_focus_widget) = old_focus_widget {
                self.notify_widget_status_changed(old_focus_widget, WidgetStatus::FOCUSED.mask);
            }

            if let Some(focus_gained_widget) = new_focus_widget {
                self.notify_widget_status_changed(focus_gained_widget, WidgetStatus::FOCUSED.mask);
            }
        }
    }

    pub fn is_pressed(&self, widget_id: WidgetId) -> bool {
        self.gesture
            .as_ref()
            .is_some_and(|gesture| gesture.target == widget_id)
    }

    pub fn is_hovered(&self, widget_id: WidgetId) -> bool {
        self.hovered_widget
            .is_some_and(|hovered_widget_id| hovered_widget_id == widget_id)
    }

    pub fn set_hovered_widget(&mut self, new_widget: Option<WidgetId>) {
        if new_widget != self.hovered_widget {
            let old_hovered_widget = std::mem::replace(&mut self.hovered_widget, new_widget);
            if let Some(old_hovered_widget) = old_hovered_widget {
                self.notify_widget_status_changed(old_hovered_widget, WidgetStatus::HOVERED.mask);
            }

            if let Some(new_hovered_widget) = new_widget {
                self.notify_widget_status_changed(new_hovered_widget, WidgetStatus::HOVERED.mask);
            }
        }
    }
}

// Widget properties
impl UiState {
    pub fn update_widget_props(&mut self, handlers: &WidgetHandlers) {
        for widget_id in std::mem::take(&mut self.dirty_props_widgets) {
            if let Some(props) = handlers.widget_props.get(widget_id) {
                let node_id = self
                    .reactive_graph
                    .widget_invalidation_node_id(widget_id, WidgetPhase::SyncWidget);
                self.clear_node_sources(node_id);
                props.apply(widget_id, self);
                self.mark_node_as_clean(node_id);
            }
        }
    }

    pub fn set_text(&mut self, widget_id: WidgetId, value: String) {
        if self.text_mut(widget_id).set_text(value) {
            self.request_layout(widget_id);
        }
    }

    pub fn set_enabled(&mut self, task_queue: &mut TaskQueue, widget_id: WidgetId, enabled: bool) {}

    pub fn is_hidden(&self, widget_id: WidgetId) -> bool {
        self.tree[widget_id].style.hidden
    }

    pub fn set_hidden(&mut self, widget_id: WidgetId, hidden: bool) {
        if hidden {
            self.clear_mouse_capture_and_focus(widget_id);
        }

        if hidden != self.tree[widget_id].style.hidden {
            self.tree[widget_id].style.hidden = hidden;
            self.request_layout(widget_id);
        }
    }
}

// Reactive graph
impl UiState {
    pub fn get_parameter_ref(&self, parameter_id: ParameterId) -> ParamRef<'_> {
        self.parameters
            .get_by_id(parameter_id)
            .expect("Parameter id should be a parameter that exists")
    }

    /// Temporarily remove a node and return it
    pub(crate) fn lease_node(&mut self, node_id: NodeId) -> Option<LeasedNode> {
        if let Some(node) = self.reactive_graph.nodes.get_mut(node_id) {
            Some(LeasedNode(
                node_id,
                std::mem::replace(&mut node.node_type, NodeType::TmpRemoved),
            ))
        } else {
            None
        }
    }

    /// Return a node that has previously been leased
    pub(crate) fn unlease_node(&mut self, mut node: LeasedNode) {
        std::mem::swap(
            &mut self.reactive_graph.nodes[node.0].node_type,
            &mut node.1,
        );
    }

    pub(crate) fn mark_node_as_clean(&mut self, node_id: NodeId) {
        self.reactive_graph
            .set_node_state(node_id, NodeState::Clean);
    }

    pub fn drive_animations(&mut self) {
        let node_ids = std::mem::take(&mut self.pending_animations);
        let now = Instant::now();
        for node_id in node_ids {
            let mut did_change = false;
            if let Some(node) = self.reactive_graph.nodes.get_mut(node_id) {
                did_change = match &mut node.node_type {
                    NodeType::Animation(animation) => animation.inner.drive(now),
                    NodeType::DerivedAnimation(animation) => animation.inner.drive(now),
                    _ => unreachable!(),
                }
            };

            if did_change {
                self.notify(node_id);
                // Re-queue the animation for the next frame
                self.pending_animations.insert(node_id);
            }
        }
    }

    /// Run all enqueued effects, returns true if any effect executed
    pub fn run_effects(&mut self, handlers: &mut WidgetHandlers) -> bool {
        let effects_to_check = std::mem::take(&mut self.effects_to_check);
        let effects_was_empty = effects_to_check.is_empty();
        for node_id in effects_to_check {
            if self.update_sources_if_necessary(node_id) == NodeState::Dirty {
                let mut node_type = self.lease_node(node_id).unwrap();
                match node_type.deref_mut() {
                    NodeType::Effect(state) => {
                        if state.dynamic_sources {
                            // Clear the sources, they will be re-populated while running the effect function
                            self.clear_node_sources(node_id);
                        }

                        (state.f)(&mut EffectContext {
                            effect_id: node_id,
                            track_reads: state.dynamic_sources,
                            handlers,
                            ui: self,
                        });
                    }
                    NodeType::DerivedAnimation(anim) => {
                        // Clear the sources, they will be re-populated while running the reset function
                        self.clear_node_sources(node_id);
                        if anim.reset(self) {
                            self.pending_animations.insert(node_id);
                        }
                    }
                    _ => unreachable!(),
                }
                self.unlease_node(node_type);
                self.mark_node_as_clean(node_id);
            }
        }
        !effects_was_empty
    }

    /// Given a node (effect, binding, derived animation, or cached value), check if any of its sources have changed.
    /// Any invalid cached value will be re-evaluated.
    fn update_sources_if_necessary(&mut self, node_id: NodeId) -> NodeState {
        let state = self.reactive_graph.get_node_state(node_id);
        if state == NodeState::Clean {
            return NodeState::Clean;
        }

        if state == NodeState::Check {
            for source_id in self.reactive_graph.sources[node_id].clone() {
                if let Source::Node(source_id) = source_id {
                    // Update the source value, if the value changed, this node will be marked as dirty
                    self.update_value_if_necessary(source_id);
                    if self.reactive_graph.get_node_state(node_id) == NodeState::Dirty {
                        return NodeState::Dirty;
                    }
                }
            }
            // No source marked this as dirty, so we must be clean
            self.reactive_graph
                .set_node_state(node_id, NodeState::Clean);
        }

        self.reactive_graph.get_node_state(node_id)
    }

    pub(super) fn update_value_if_necessary(&mut self, node_id: NodeId) {
        if self.update_sources_if_necessary(node_id) == NodeState::Dirty {
            let mut node_type = self.lease_node(node_id).unwrap();
            match node_type.deref_mut() {
                NodeType::Memo(memo) => {
                    // Clear the sources, they will be re-populated while running the memo function
                    self.clear_node_sources(node_id);
                    if memo.eval(node_id, self) {
                        // Memo eval returned true, meaning that it has changed.
                        self.reactive_graph.mark_observers_dirty(node_id);
                    }
                }
                NodeType::DerivedAnimation(anim) => {
                    // Clear the sources, they will be re-populated while running the reset function
                    self.clear_node_sources(node_id);
                    if anim.reset(self) {
                        self.pending_animations.insert(node_id);
                    }
                }
                _ => unreachable!(),
            }
            self.unlease_node(node_type);
        }
        self.mark_node_as_clean(node_id);
    }

    pub(crate) fn notify(&mut self, node_id: NodeId) {
        let mut observers = std::mem::take(&mut self.node_id_buffer);
        observers.clear();
        observers.extend(self.reactive_graph.node_observers_iter(node_id));
        self.notify_source_changed(observers);
    }

    fn notify_source_changed(&mut self, mut nodes_to_notify: VecDeque<NodeId>) {
        let direct_child_count = nodes_to_notify.len();
        let mut i = 0;
        while let Some(node_id) = nodes_to_notify.pop_front() {
            // Mark direct nodes as Dirty and grand-children as Check
            let new_state = if i < direct_child_count {
                NodeState::Dirty
            } else {
                NodeState::Check
            };
            let node = &mut self.reactive_graph.nodes[node_id];
            if node.state < new_state {
                node.state = new_state;
                match &node.node_type {
                    NodeType::Effect(_) | NodeType::DerivedAnimation(_) => {
                        self.effects_to_check.insert(node_id);
                    }
                    &NodeType::InvalidateWidget { widget_id, phase } => match phase {
                        WidgetPhase::SyncWidget => self.request_prop_update(widget_id),
                        WidgetPhase::Render => self.request_render(widget_id),
                    },
                    _ => {}
                }
                nodes_to_notify.extend(self.reactive_graph.node_observers_iter(node_id));
            }
            i += 1;
        }

        // Swap back the scratch buffer. Saves us from having to reallocate
        std::mem::swap(&mut self.node_id_buffer, &mut nodes_to_notify);
    }

    pub(crate) fn notify_parameter_subscribers(&mut self, param_id: ParameterId) {
        let mut nodes_to_notify = std::mem::take(&mut self.node_id_buffer);
        nodes_to_notify.clear();
        nodes_to_notify.extend(self.reactive_graph.parameter_observers_iter(param_id));
        self.notify_source_changed(nodes_to_notify);
    }

    pub(crate) fn notify_widget_status_changed(
        &mut self,
        widget_id: WidgetId,
        change_mask: WidgetStatusFlags,
    ) {
        let mut nodes_to_notify = std::mem::take(&mut self.node_id_buffer);
        nodes_to_notify.clear();
        nodes_to_notify.extend(
            self.reactive_graph
                .widget_status_observers_iter(widget_id, change_mask),
        );
        self.notify_source_changed(nodes_to_notify);
    }

    pub fn clear_node_sources(&mut self, node_id: NodeId) {
        self.reactive_graph.clear_node_sources(node_id);
    }

    pub(crate) fn add_subscription(&mut self, source_id: Source, observer_id: NodeId) {
        match source_id {
            Source::Parameter(parameter_id) => self
                .reactive_graph
                .add_parameter_subscription(parameter_id, observer_id),
            Source::Node(node_id) => self
                .reactive_graph
                .add_node_subscription(node_id, observer_id),
            Source::Widget(widget_id, status_mask) => self
                .reactive_graph
                .add_widget_status_subscription(widget_id, status_mask, observer_id),
        }
    }

    pub fn get_render_invalidation_node_id(&mut self, widget_id: WidgetId) -> NodeId {
        self.reactive_graph
            .widget_invalidation_node_id(widget_id, WidgetPhase::Render)
    }

    pub fn owner_widget(&self, node_id: NodeId) -> Option<WidgetId> {
        self.widget_from_owner(self.reactive_graph.nodes[node_id].owner)
    }

    pub fn widget_from_owner(&self, mut owner: Owner) -> Option<WidgetId> {
        loop {
            match owner {
                Owner::Root => return None,
                Owner::Widget(widget_id) => return Some(widget_id),
                Owner::Node(node_id) => owner = self.reactive_graph.nodes[node_id].owner,
            }
        }
    }
}

// Layout
impl UiState {
    pub(crate) fn request_layout(&mut self, widget_id: WidgetId) {
        let mut current = widget_id;
        // Mark the widget and all parents as needing layout until we hit the root,
        // an overlay or another node that already have marked as needing layout
        loop {
            if current.is_null() {
                break;
            }
            let data = &mut self.tree.get(current).unwrap();
            if data.flag_is_set(WidgetFlags::NEEDS_LAYOUT) {
                break;
            }
            data.set_flag(WidgetFlags::NEEDS_LAYOUT);
            if data.is_overlay() {
                break;
            }
            current = data.parent_id;
        }
    }

    pub fn layout_window(
        &mut self,
        text_context: &mut TextContext,
        window_id: WindowId,
        mode: RecomputeLayout,
    ) {
        let window = &mut self.windows[window_id];
        let window_size = window.handle.global_bounds().size();
        let window_rect = Rect::from_origin(Point::ZERO, window_size);
        let root_id = window.root_widget;
        self.rebuild_children();

        let mut did_layout = false;
        // Need to layout root first, the overlay positions can depend on their parent positions
        if mode == RecomputeLayout::Force || self.tree.get(root_id).unwrap().needs_layout() {
            LayoutContext::new(self, window_size, text_context).compute_root_layout(root_id);
            self.tree.update_node_origins(root_id, Point::ZERO);
            self.request_render(root_id);
            did_layout = true;
        }

        // This is currently in z-index order, we should probably iterate in insertion order
        // If an overlay's position depends on the position of a previously created overlay,
        // this will be wrong.
        let overlay_ids: Vec<_> = self.windows[window_id].overlays.iter().collect();
        for (i, overlay_id) in overlay_ids.into_iter().enumerate() {
            if mode == RecomputeLayout::Force || self.tree[overlay_id].needs_layout() {
                LayoutContext::new(self, window_size, text_context).compute_root_layout(overlay_id);
                let options = self
                    .window(window_id)
                    .overlays
                    .get_overlay_options(i)
                    .unwrap();

                let offset = self.compute_overlay_offset(window_rect, overlay_id, options);
                self.tree
                    .update_node_origins(overlay_id, offset.into_point());
                self.request_render(overlay_id);
                did_layout = true;
            }
        }

        if did_layout {
            self.print_tree(window_id);
        }
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

    pub fn print_tree(&self, window_id: WindowId) {
        fn _impl(this: &UiState, root_id: WidgetId, indent: usize) {
            println!(
                "{} {}({:?}), {:?}",
                "-".repeat(indent),
                this.tree[root_id].debug_label,
                root_id,
                this.tree[root_id].global_bounds()
            );
            let mut walker = this.tree.child_id_walker(root_id);
            while let Some(child_id) = walker.next_id(&this.tree) {
                _impl(this, child_id, indent + 1);
            }
        }
        let root_id = self.window(window_id).root_widget;
        _impl(self, root_id, 0);
        let overlay_ids: Vec<_> = self.windows[window_id].overlays.iter().collect();
        for id in overlay_ids {
            _impl(self, id, 0);
        }
    }
}
