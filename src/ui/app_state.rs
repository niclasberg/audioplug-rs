use super::{
    AnyView, BuildContext, CreateContext, HostHandle, ParamContext, ReactiveContext, ReadContext,
    ReadSignal, Var, View, Widget, WidgetContext, WidgetData, WidgetId, WidgetMut, WidgetRef,
    WindowId, WriteContext,
    clipboard::Clipboard,
    event_handling::{set_focus_widget, set_mouse_capture_widget},
    layout::RecomputeLayout,
    layout_window,
    overlay::OverlayContainer,
    reactive::{ReactiveGraph, WatchContext},
    style::StyleBuilder,
};
use crate::{
    core::{Point, WindowTheme},
    param::{AnyParameterMap, NormalizedValue, ParameterId, PlainValue},
    platform,
    ui::{
        OverlayOptions, Widgets,
        render::{GpuScene, WGPUSurface},
        task_queue::TaskQueue,
    },
};
use slotmap::{Key, SlotMap};
use std::{cell::Cell, rc::Rc};

pub(super) struct WindowState {
    pub(super) handle: platform::Handle,
    pub(super) wgpu_surface: WGPUSurface,
    pub(super) gpu_scene: GpuScene,
    pub(super) root_widget: WidgetId,
    pub(super) focus_widget: Option<WidgetId>,
    pub(super) theme_signal: Var<WindowTheme>,
    pub(super) overlays: OverlayContainer,
}

pub struct AppState {
    pub(super) wgpu_instance: wgpu::Instance,
    pub(super) windows: SlotMap<WindowId, WindowState>,
    pub(super) widgets: Widgets,
    pub(super) mouse_capture_widget: Option<WidgetId>,
    pub(super) reactive_graph: ReactiveGraph,
    host_handle: Option<Box<dyn HostHandle>>,
    id_buffer: Cell<Vec<WidgetId>>,
    pub(super) task_queue: TaskQueue,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WidgetInsertPos {
    Before(WidgetId),
    After(WidgetId),
    BeforeFirstChildOf(WidgetId),
    AfterLastChildOf(WidgetId),
    Overlay(WidgetId, OverlayOptions),
}

impl AppState {
    pub fn new(parameters: Rc<dyn AnyParameterMap>) -> Self {
        Self {
            wgpu_instance: wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                ..Default::default()
            }),
            widgets: Default::default(),
            windows: Default::default(),
            mouse_capture_widget: None,
            reactive_graph: ReactiveGraph::new(parameters),
            host_handle: None,
            id_buffer: Default::default(),
            task_queue: Default::default(),
        }
    }

    pub fn parameters(&self) -> &dyn AnyParameterMap {
        self.reactive_graph.parameters.as_ref()
    }

    pub(crate) fn set_host_handle(&mut self, host_handle: Option<Box<dyn HostHandle>>) {
        self.host_handle = host_handle;
    }

    #[allow(dead_code)]
    pub(crate) fn set_plain_parameter_value_from_host(
        &mut self,
        id: ParameterId,
        value: PlainValue,
    ) -> bool {
        let Some(param_ref) = self.reactive_graph.parameters.get_by_id(id) else {
            return false;
        };
        param_ref.set_value_plain(value);
        super::reactive::notify_parameter_subscribers(self, id);
        self.run_effects();
        true
    }

    pub(crate) fn set_normalized_parameter_value_from_host(
        &mut self,
        id: ParameterId,
        value: NormalizedValue,
    ) -> bool {
        let Some(param_ref) = self.reactive_graph.parameters.get_by_id(id) else {
            return false;
        };
        param_ref.set_value_normalized(value);
        super::reactive::notify_parameter_subscribers(self, id);
        self.run_effects();
        true
    }

    pub fn add_window(
        &mut self,
        handle: platform::Handle,
        wgpu_surface: WGPUSurface,
        view: impl View,
    ) -> WindowId {
        let theme_signal = Var::new(self, handle.theme());

        let window_id = self.windows.insert(WindowState {
            handle,
            wgpu_surface,
            root_widget: WidgetId::null(),
            focus_widget: None,
            theme_signal,
            overlays: Default::default(),
            gpu_scene: GpuScene::new(),
        });

        let widget_id = self.widgets.allocate_widget(window_id);
        self.windows[window_id].root_widget = widget_id;
        self.build_and_insert_widget(widget_id, view);

        layout_window(self, window_id, RecomputeLayout::Force);

        window_id
    }

    fn build_and_insert_widget<V: View>(&mut self, id: WidgetId, view: V) {
        let mut styles = StyleBuilder::default();
        let widget = view.build(&mut BuildContext::new(id, self, &mut styles));
        styles.apply_styles(&mut BuildContext::new(
            id,
            self,
            &mut StyleBuilder::default(),
        ));
        self.widgets.widgets.insert(id, Box::new(widget));
    }

    /// Add a new widget
    pub fn add_widget<V: View>(&mut self, view: V, position: WidgetInsertPos) -> WidgetId {
        let id = self.widgets.allocate_widget(WindowId::null());
        self.build_and_insert_widget(id, view);

        /*match position {
            WidgetInsertPos::Overlay(_, options) => {
                self.window_mut(window_id)
                    .overlays
                    .insert_or_update(id, options);
            }
            _ => {}
        };*/

        /*WidgetInsertPos::Index(index) => self.widgets.children[parent_id].insert(index, id),
        WidgetInsertPos::End => self.widgets.children[parent_id].push(id),
        WidgetInsertPos::Overlay(options) => {
            self.widgets.overlays[parent_id].push(id);
            self.widgets.data[id].set_flag(WidgetFlags::OVERLAY);

        }*/

        id
    }

    pub fn remove_window(&mut self, id: WindowId) {
        let root_widget = self.window_mut(id).root_widget;
        let theme_signal_id = self.window_mut(id).theme_signal.id;
        self.remove_widget(root_widget);
        self.reactive_graph.remove_node(theme_signal_id);
        self.windows.remove(id).expect("Window not found");
    }

    /// Remove a widget and all of its children and associated signals
    pub fn remove_widget(&mut self, id: WidgetId) {
        self.clear_mouse_capture_and_focus(id);

        let widget_data = self.do_remove_widget(id);
        let parent_id = widget_data.parent_id;
        if !parent_id.is_null() {
            // Must be removed from parent's child list
            self.widgets.children[parent_id].retain(|child_id| *child_id != id);
            self.widgets.overlays[parent_id].retain(|child_id| *child_id != id);
        }

        self.with_id_buffer_mut(|app_state, children_to_remove| {
            children_to_remove.extend(widget_data.children);
            children_to_remove.extend(widget_data.overlays);
            while let Some(id) = children_to_remove.pop() {
                let widget_data = app_state.do_remove_widget(id);
                children_to_remove.extend(widget_data.children.into_iter());
                children_to_remove.extend(widget_data.overlays.into_iter());
            }
        });
    }

    fn do_remove_widget(&mut self, id: WidgetId) -> WidgetData {
        let widget_data = self.widget_data.remove(id).unwrap();
        if widget_data.is_overlay() {
            self.window_mut(widget_data.window_id).overlays.remove(id);
        }
        self.widgets.remove(id).expect("Widget already removed");
        self.reactive_graph.clear_nodes_for_widget(id);
        widget_data
    }

    pub fn replace_widget<V: View>(&mut self, id: WidgetId, view: V) {
        self.clear_mouse_capture_and_focus(id);

        let WidgetData {
            parent_id,
            window_id,
            next_sibling_id,
            prev_sibling_id,
            ..
        } = self.widgets.data[id];

        self.widgets.remove_children(id, |_| {});

        self.widgets.data[id] = WidgetData::new(window_id, id)
            .with_parent(parent_id)
            .with_siblings(prev_sibling_id, next_sibling_id);
        self.build_and_insert_widget(id, view);
    }

    fn clear_mouse_capture_and_focus(&mut self, id: WidgetId) {
        if let Some(mouse_capture_widget) = self.mouse_capture_widget
            && (mouse_capture_widget == id || self.widget_has_parent(mouse_capture_widget, id))
        {
            set_mouse_capture_widget(self, None);
        }

        let window_id = self.get_window_id_for_widget(id);
        if let Some(focus_widget) = self.window(window_id).focus_widget
            && (focus_widget == id || self.widget_has_parent(focus_widget, id))
        {
            set_focus_widget(self, window_id, None);
        }
    }

    pub fn widget_ref(&self, id: WidgetId) -> WidgetRef<'_, dyn Widget> {
        WidgetRef::new(&self.widgets, id)
    }

    pub fn widget_mut(&mut self, id: WidgetId) -> WidgetMut<'_, dyn Widget> {
        WidgetMut::new(self, id)
    }

    pub fn with_widget_mut<R>(
        &mut self,
        id: WidgetId,
        f: impl FnOnce(&mut Self, &mut dyn Widget) -> R,
    ) -> R {
        let mut widget = self.widgets.lease_widget(id);
        let value = f(self, widget.as_mut());
        self.widgets.unlease_widget(id, widget);
        value
    }

    pub fn widget_has_parent(&self, child_id: WidgetId, parent_id: WidgetId) -> bool {
        let mut id = child_id;
        while !id.is_null() {
            id = self.widgets.data[id].parent_id;
            if id == parent_id {
                return true;
            }
        }
        false
    }

    pub fn widget_has_focus(&self, id: WidgetId) -> bool {
        self.window(self.widgets.data[id].window_id)
            .focus_widget
            .as_ref()
            .is_some_and(|focus_widget_id| *focus_widget_id == id)
    }

    pub fn widget_has_captured_mouse(&self, widget_id: WidgetId) -> bool {
        self.mouse_capture_widget.is_some_and(|id| id == widget_id)
    }

    pub(super) fn with_id_buffer(&self, f: impl FnOnce(&Self, &mut Vec<WidgetId>)) {
        let mut scratch = self.id_buffer.replace(Vec::new());
        scratch.clear();
        f(self, &mut scratch);
        self.id_buffer.set(scratch);
    }

    pub(super) fn with_id_buffer_mut(&mut self, f: impl FnOnce(&mut Self, &mut Vec<WidgetId>)) {
        let mut scratch = self.id_buffer.replace(Vec::new());
        scratch.clear();
        f(self, &mut scratch);
        self.id_buffer.set(scratch);
    }

    /// Calls `f` for each widget that contains `pos`. The order is from the root and down the tree (draw order).
    /// Iteration continues until all widgets have been visited, or `f` returns false
    pub fn for_each_widget_at(
        &self,
        id: WindowId,
        pos: Point,
        mut f: impl FnMut(&Self, WidgetId) -> bool,
    ) {
        let mut root_and_overlays = vec![self.windows[id].root_widget];
        root_and_overlays.extend(self.windows[id].overlays.iter());
        for root in root_and_overlays {
            for id in self.widgets.get_widgets_at(root, pos) {
                if !f(self, id) {
                    return;
                }
            }
        }
    }

    pub fn for_each_widget_at_mut(
        &mut self,
        id: WindowId,
        pos: Point,
        mut f: impl FnMut(&mut Self, WidgetId) -> bool,
    ) {
        let mut root_and_overlays = vec![self.windows[id].root_widget];
        root_and_overlays.extend(self.windows[id].overlays.iter());
        for root in root_and_overlays {
            for id in self.widgets.get_widgets_at(root, pos) {
                if !f(self, id) {
                    return;
                }
            }
        }
    }

    /// Calls `f` for each widget that contains `pos`. The traversal order is from the top leaf back to the root (reverse draw order)
    pub fn for_each_widget_at_rev(
        &self,
        id: WindowId,
        pos: Point,
        mut f: impl FnMut(&Self, WidgetId) -> bool,
    ) {
        // Overlays first (in reverse order) and then the "regular" view hierarchy
        let mut root_and_overlays: Vec<_> = self.windows[id].overlays.iter().rev().collect();
        root_and_overlays.push(self.windows[id].root_widget);
        for root in root_and_overlays {
            for id in self.widgets.get_widgets_at(root, pos).into_iter().rev() {
                if !f(self, id) {
                    return;
                }
            }
        }
    }

    pub fn for_each_widget_at_rev_mut(
        &mut self,
        id: WindowId,
        pos: Point,
        mut f: impl FnMut(&mut Self, WidgetId) -> bool,
    ) {
        let mut root_and_overlays: Vec<_> = self.windows[id].overlays.iter().rev().collect();
        root_and_overlays.push(self.windows[id].root_widget);
        for root in root_and_overlays {
            for id in self.widgets.get_widgets_at(root, pos).into_iter().rev() {
                if !f(self, id) {
                    return;
                }
            }
        }
    }

    pub(super) fn window(&self, id: WindowId) -> &WindowState {
        self.windows.get(id).expect("Window handle not found")
    }

    pub(super) fn window_for_widget(&self, id: WidgetId) -> &WindowState {
        self.windows
            .get(self.widgets.data[id].window_id)
            .expect("Window handle not found")
    }

    pub(super) fn window_mut(&mut self, id: WindowId) -> &mut WindowState {
        self.windows.get_mut(id).expect("Window handle not found")
    }

    pub fn window_theme_signal(&self, id: WidgetId) -> ReadSignal<WindowTheme> {
        self.window_for_widget(id).theme_signal.as_read_signal()
    }

    pub fn focus_widget(&self, window_id: WindowId) -> Option<WidgetRef<'_, dyn Widget>> {
        self.window(window_id)
            .focus_widget
            .map(|id| WidgetRef::new(&self.widgets, id))
    }

    pub fn focus_widget_mut(&mut self, window_id: WindowId) -> Option<WidgetMut<'_, dyn Widget>> {
        self.window(window_id)
            .focus_widget
            .map(|id| WidgetMut::new(self, id))
    }

    pub fn get_window_id_for_widget(&self, widget_id: WidgetId) -> WindowId {
        self.widgets.data[widget_id].window_id
    }

    pub fn run_effects(&mut self) {
        let mut tasks = std::mem::take(&mut self.task_queue.0);

        if tasks.is_empty() {
            return;
        }

        while let Some(task) = tasks.pop_front() {
            task.run(self);
            tasks.extend(self.task_queue.0.drain(..));
        }

        // Layout if needed
        let window_ids: Vec<_> = self.windows.iter().map(|window| window.0).collect();
        for window_id in window_ids {
            layout_window(self, window_id, RecomputeLayout::IfNeeded);
        }
    }

    pub fn clipboard(&self, window_id: WindowId) -> Clipboard<'_> {
        Clipboard {
            handle: &self.window(window_id).handle,
        }
    }
}

impl ReactiveContext for AppState {
    fn components(&self) -> (&ReactiveGraph, &Widgets) {
        (&self.reactive_graph, &self.widgets)
    }

    fn components_mut(&mut self) -> (&mut ReactiveGraph, &mut Widgets, &mut TaskQueue) {
        (
            &mut self.reactive_graph,
            &mut self.widgets,
            &mut self.task_queue,
        )
    }
}

impl ReadContext for AppState {
    fn scope(&self) -> super::ReadScope {
        super::ReadScope::Untracked
    }
}

impl WriteContext for AppState {}

impl ParamContext for AppState {
    fn host_handle(&self) -> &dyn HostHandle {
        let Some(host_handle) = self.host_handle.as_ref() else {
            panic!("Host handle not set")
        };
        host_handle.as_ref()
    }
}

impl CreateContext for AppState {
    fn owner(&self) -> Option<super::Owner> {
        None
    }
}

impl WidgetContext for AppState {
    fn widget_ref_dyn(&self, id: WidgetId) -> WidgetRef<'_, dyn Widget> {
        WidgetRef::new(&self.widgets, id)
    }

    fn widget_mut_dyn(&mut self, id: WidgetId) -> WidgetMut<'_, dyn Widget> {
        WidgetMut::new(self, id)
    }

    fn replace_widget_dyn(&mut self, id: WidgetId, view: AnyView) {
        self.replace_widget(id, view);
    }
}

impl WatchContext for AppState {}
