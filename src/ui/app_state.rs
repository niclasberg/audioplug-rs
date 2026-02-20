use super::{
    AnyView, BuildContext, CreateContext, HostHandle, ParamContext, ReactiveContext, ReadContext,
    Var, View, Widget, WidgetContext, WidgetData, WidgetId, WidgetMut, WidgetRef, WindowId,
    WriteContext,
    clipboard::Clipboard,
    event_handling::{set_focus_widget, set_mouse_capture_widget},
    layout::RecomputeLayout,
    reactive::{ReactiveGraph, WatchContext},
    style::StyleBuilder,
};
use crate::{
    core::WindowTheme,
    param::{AnyParameterMap, NormalizedValue, ParameterId, PlainValue},
    platform,
    ui::{
        Owner, Widgets,
        reactive::{LocalCreateContext, ReactiveContextMut},
        render::WGPUSurface,
        task_queue::TaskQueue,
        widgets::WidgetPos,
    },
};
use std::{cell::Cell, rc::Rc};

pub struct AppState {
    pub(super) wgpu_instance: wgpu::Instance,
    pub(super) widgets: Widgets,
    pub(super) reactive_graph: ReactiveGraph,
    host_handle: Option<Box<dyn HostHandle>>,
    id_buffer: Cell<Vec<WidgetId>>,
    pub(super) task_queue: TaskQueue,
    pub(crate) theme_signal: Var<WindowTheme>,
}

impl AppState {
    pub fn new(parameters: Rc<dyn AnyParameterMap>) -> Self {
        let mut reactive_graph = ReactiveGraph::new(parameters);
        let mut task_queue = TaskQueue::default();
        let mut widgets = Widgets::default();
        let theme_signal = Var::new(
            &mut LocalCreateContext::new_root_context(
                &mut widgets,
                &mut reactive_graph,
                &mut task_queue,
            ),
            WindowTheme::Dark,
        );

        Self {
            wgpu_instance: wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                ..Default::default()
            }),
            reactive_graph,
            host_handle: None,
            id_buffer: Default::default(),
            theme_signal,
            widgets,
            task_queue,
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
        let window_id = self.widgets.allocate_window(handle, wgpu_surface);
        let root_widget_id = self.widgets.window(window_id).root_widget;
        self.build_and_insert_widget(root_widget_id, view);
        self.widgets
            .layout_window(window_id, RecomputeLayout::Force);
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
    pub fn add_widget<V: View>(&mut self, view: V, position: WidgetPos) -> WidgetId {
        let id = self.widgets.allocate_widget(position);
        self.build_and_insert_widget(id, view);
        id
    }

    pub fn remove_window(&mut self, id: WindowId) {
        self.clear_mouse_capture_and_focus(self.widgets.window(id).root_widget);
        let reactive_graph = &mut self.reactive_graph;
        self.widgets.remove_window(id, &mut |data| {
            reactive_graph.clear_nodes_for_widget(data.id);
        });
    }

    /// Remove a widget and all of its children and associated signals
    pub fn remove_widget(&mut self, id: WidgetId) {
        self.clear_mouse_capture_and_focus(id);
        let reactive_graph = &mut self.reactive_graph;
        self.widgets.remove_widget(id, &mut |data| {
            reactive_graph.clear_nodes_for_widget(data.id);
        });
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

        let reactive_graph = &mut self.reactive_graph;
        reactive_graph.clear_nodes_for_widget(id);
        self.widgets.remove_children(id, &mut |data| {
            reactive_graph.clear_nodes_for_widget(data.id);
        });

        self.widgets.data[id] = WidgetData::new(window_id, id)
            .with_parent(parent_id)
            .with_siblings(prev_sibling_id, next_sibling_id);
        self.build_and_insert_widget(id, view);
    }

    fn clear_mouse_capture_and_focus(&mut self, id: WidgetId) {
        if let Some(mouse_capture_widget) = self.widgets.mouse_capture_widget
            && (mouse_capture_widget == id || self.widgets.has_parent(mouse_capture_widget, id))
        {
            set_mouse_capture_widget(self, None);
        }

        let window_id = self.get_window_id_for_widget(id);
        if let Some(focus_widget) = self.widgets.focus_widget_id(window_id)
            && (focus_widget == id || self.widgets.has_parent(focus_widget, id))
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

    pub(super) fn with_id_buffer_mut(&mut self, f: impl FnOnce(&mut Self, &mut Vec<WidgetId>)) {
        let mut scratch = self.id_buffer.replace(Vec::new());
        scratch.clear();
        f(self, &mut scratch);
        self.id_buffer.set(scratch);
    }

    pub fn focus_widget(&self, window_id: WindowId) -> Option<WidgetRef<'_, dyn Widget>> {
        self.widgets
            .window(window_id)
            .focus_widget
            .map(|id| WidgetRef::new(&self.widgets, id))
    }

    pub fn focus_widget_mut(&mut self, window_id: WindowId) -> Option<WidgetMut<'_, dyn Widget>> {
        self.widgets
            .window(window_id)
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
        let window_ids: Vec<_> = self.widgets.window_id_iter().collect();
        for window_id in window_ids {
            self.widgets
                .layout_window(window_id, RecomputeLayout::IfNeeded);
        }
    }

    pub fn clipboard(&self, window_id: WindowId) -> Clipboard<'_> {
        Clipboard {
            handle: &self.widgets.window(window_id).handle,
        }
    }
}

impl ReactiveContext for AppState {
    fn reactive_graph_and_widgets(&self) -> (&ReactiveGraph, &Widgets) {
        (&self.reactive_graph, &self.widgets)
    }

    fn reactive_graph_mut_and_widgets(&mut self) -> (&mut ReactiveGraph, &Widgets) {
        (&mut self.reactive_graph, &mut self.widgets)
    }
}

impl ReactiveContextMut for AppState {
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
    fn owner(&self) -> Owner {
        Owner::Root
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
