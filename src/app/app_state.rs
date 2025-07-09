use super::{
    clipboard::Clipboard,
    effect::{BindingFn, EffectContext},
    layout_window,
    signal::ReadSignal,
    BuildContext, CreateContext, HostHandle, NodeId, ParamContext, ReactiveContext, ReadContext,
    Runtime, Signal, View, Widget, WidgetData, WidgetFlags, WidgetId, WidgetMut, WidgetRef,
    WindowId, WriteContext,
};
use crate::{
    app::{
        effect::{EffectFn, WatchContext},
        event_channel::HandleEventFn,
        event_handling::{set_focus_widget, set_mouse_capture_widget},
        AnyView, FxIndexSet, Scope, WidgetContext,
    },
    core::{Point, WindowTheme},
    param::{AnyParameterMap, NormalizedValue, ParameterId, PlainValue},
    platform,
    style::StyleBuilder,
};
use fxhash::FxBuildHasher;
use slotmap::{Key, SecondaryMap, SlotMap};
use std::{
    any::Any,
    cell::RefCell,
    rc::{Rc, Weak},
};

pub(super) struct WindowState {
    pub(super) handle: platform::Handle,
    pub(super) root_widget: WidgetId,
    pub(super) focus_widget: Option<WidgetId>,
    pub(super) pending_widget_animations: FxIndexSet<WidgetId>,
    pub(super) pending_node_animations: FxIndexSet<NodeId>,
    pub(super) theme_signal: Signal<WindowTheme>,
}

pub struct AppState {
    windows: SlotMap<WindowId, WindowState>,
    pub(super) widget_data: SlotMap<WidgetId, WidgetData>,
    pub(super) widgets: SecondaryMap<WidgetId, Box<dyn Widget>>,
    pub(super) mouse_capture_widget: Option<WidgetId>,
    pub(super) runtime: Runtime,
    host_handle: Option<Box<dyn HostHandle>>,
}

impl AppState {
    pub fn new(parameters: Rc<dyn AnyParameterMap>) -> Self {
        Self {
            widget_data: Default::default(),
            widgets: Default::default(),
            windows: Default::default(),
            mouse_capture_widget: None,
            runtime: Runtime::new(parameters),
            host_handle: None,
        }
    }

    pub fn parameters(&self) -> &dyn AnyParameterMap {
        self.runtime.parameters.as_ref()
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
        let Some(param_ref) = self.runtime.parameters.get_by_id(id) else {
            return false;
        };
        param_ref.set_value_plain(value);
        self.runtime.notify_parameter_subscribers(id);
        self.run_effects();
        true
    }

    pub(crate) fn set_normalized_parameter_value_from_host(
        &mut self,
        id: ParameterId,
        value: NormalizedValue,
    ) -> bool {
        let Some(param_ref) = self.runtime.parameters.get_by_id(id) else {
            return false;
        };
        param_ref.set_value_normalized(value);
        self.runtime.notify_parameter_subscribers(id);
        self.run_effects();
        true
    }

    pub fn add_window(&mut self, handle: platform::Handle, view: impl View) -> WindowId {
        let theme_signal = Signal::new(self, handle.theme());

        let window_id = self.windows.insert(WindowState {
            handle,
            root_widget: WidgetId::null(),
            focus_widget: None,
            pending_widget_animations: FxIndexSet::with_hasher(FxBuildHasher::new()),
            pending_node_animations: FxIndexSet::with_hasher(FxBuildHasher::new()),
            theme_signal,
        });

        let widget_id = self
            .widget_data
            .insert_with_key(|id| WidgetData::new(window_id, id));

        self.windows[window_id].root_widget = widget_id;
        self.build_and_insert_widget(widget_id, view);

        layout_window(self, window_id);

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
        self.widgets.insert(id, Box::new(widget));
    }

    /// Add a new widget
    pub fn add_widget<V: View>(
        &mut self,
        parent_id: WidgetId,
        view: V,
        child_index: Option<usize>,
    ) -> WidgetId {
        let window_id = self
            .widget_data
            .get(parent_id)
            .expect("Parent not found")
            .window_id;
        let id = self
            .widget_data
            .insert_with_key(|id| WidgetData::new(window_id, id).with_parent(parent_id));

        self.build_and_insert_widget(id, view);

        let parent_widget_data = self
            .widget_data
            .get_mut(parent_id)
            .expect("Parent does not exist");

        if let Some(child_index) = child_index {
            parent_widget_data.children.insert(child_index, id);
        } else {
            parent_widget_data.children.push(id);
        }

        id
    }

    pub fn remove_window(&mut self, id: WindowId) {
        let root_widget = self.window_mut(id).root_widget;
        let theme_signal_id = self.window_mut(id).theme_signal.id;
        self.remove_widget(root_widget);
        self.runtime.remove_node(theme_signal_id);
        self.windows.remove(id).expect("Window not found");
    }

    /// Remove a widget and all of its children and associated signals
    pub fn remove_widget(&mut self, id: WidgetId) {
        self.clear_mouse_capture_and_focus(id);

        let mut widget_data = self.do_remove_widget(id);
        // Must be removed from parent's child list
        if !widget_data.parent_id.is_null() {
            let parent_id = widget_data.parent_id;
            let parent_widget_data = self
                .widget_data
                .get_mut(parent_id)
                .expect("Parent does not exist");
            parent_widget_data
                .children
                .retain(|child_id| *child_id != id);
        }

        let mut children_to_remove = std::mem::take(&mut widget_data.children);
        while let Some(id) = children_to_remove.pop() {
            let widget_data = self.do_remove_widget(id);
            children_to_remove.extend(widget_data.children.into_iter());
        }
    }

    fn do_remove_widget(&mut self, id: WidgetId) -> WidgetData {
        let widget_data = self.widget_data.remove(id).unwrap();
        self.widgets.remove(id).expect("Widget already removed");
        self.runtime.clear_nodes_for_widget(id);
        widget_data
    }

    pub fn replace_widget<V: View>(&mut self, id: WidgetId, view: V) {
        self.clear_mouse_capture_and_focus(id);

        let parent_id = self.widget_data[id].parent_id;
        let window_id = self.widget_data[id].window_id;

        let mut children_to_remove = std::mem::take(&mut self.widget_data[id].children);
        while let Some(id) = children_to_remove.pop() {
            let widget_data = self.do_remove_widget(id);
            children_to_remove.extend(widget_data.children.into_iter());
        }

        self.widget_data[id] = WidgetData::new(window_id, id).with_parent(parent_id);
        self.build_and_insert_widget(id, view);
    }

    fn clear_mouse_capture_and_focus(&mut self, id: WidgetId) {
        if let Some(mouse_capture_widget) = self.mouse_capture_widget {
            if mouse_capture_widget == id || self.widget_has_parent(mouse_capture_widget, id) {
                set_mouse_capture_widget(self, None);
            }
        }

        let window_id = self.get_window_id_for_widget(id);
        if let Some(focus_widget) = self.window(window_id).focus_widget {
            if focus_widget == id || self.widget_has_parent(focus_widget, id) {
                set_focus_widget(self, window_id, None);
            }
        }
    }

    pub fn widget_ref(&self, id: WidgetId) -> WidgetRef<'_, dyn Widget> {
        WidgetRef::new(self, id)
    }

    pub fn widget_mut(&mut self, id: WidgetId) -> WidgetMut<'_, dyn Widget> {
        WidgetMut::new(self, id)
    }

    pub fn with_widget_mut<R>(
        &mut self,
        id: WidgetId,
        f: impl FnOnce(&mut Self, &mut dyn Widget) -> R,
    ) -> R {
        let Some(mut widget) = self.widgets.remove(id) else {
            panic!("Widget does not exist")
        };
        let value = f(self, widget.as_mut());
        self.widgets.insert(id, widget);
        value
    }

    pub fn widget_has_parent(&self, child_id: WidgetId, parent_id: WidgetId) -> bool {
        let mut id = child_id;
        while !id.is_null() {
            id = self.widget_data[id].parent_id;
            if id == parent_id {
                return true;
            }
        }
        false
    }

    pub fn widget_has_focus(&self, id: WidgetId) -> bool {
        self.window(self.widget_data_ref(id).window_id)
            .focus_widget
            .as_ref()
            .is_some_and(|focus_widget_id| *focus_widget_id == id)
    }

    pub fn widget_has_captured_mouse(&self, widget_id: WidgetId) -> bool {
        self.mouse_capture_widget.is_some_and(|id| id == widget_id)
    }

    pub fn widget_data_ref(&self, id: WidgetId) -> &WidgetData {
        self.widget_data.get(id).expect("Widget data not found")
    }

    pub fn widget_data_mut(&mut self, id: WidgetId) -> &mut WidgetData {
        self.widget_data.get_mut(id).expect("Widget data not found")
    }

    /// Calls `f` for each widget that conatains `pos`. The order is from the root and down the tree (depth first order)
    pub fn for_each_widget_at(
        &self,
        id: WindowId,
        pos: Point,
        mut f: impl FnMut(&Self, WidgetId) -> bool,
    ) {
        let mut stack = vec![self.windows[id].root_widget];
        while let Some(current) = stack.pop() {
            if !f(self, current) {
                return;
            }

            for &child in self.widget_data[current].children.iter().rev() {
                if self.widget_data[child].global_bounds().contains(pos) {
                    stack.push(child)
                }
            }
        }
    }

    pub fn for_each_widget_at_rev(
        &self,
        id: WindowId,
        pos: Point,
        mut f: impl FnMut(&Self, WidgetId) -> bool,
    ) {
        enum Action {
            VisitChildren(WidgetId),
            Done(WidgetId),
        }

        let mut stack = vec![Action::VisitChildren(self.windows[id].root_widget)];
        while let Some(current) = stack.pop() {
            match current {
                Action::VisitChildren(widget_id) => {
                    let data = &self.widget_data[widget_id];
                    if data.children.is_empty() {
                        if !f(self, widget_id) {
                            return;
                        }
                    } else {
                        stack.push(Action::Done(widget_id));
                        for &child in data.children.iter() {
                            if self.widget_data[child].global_bounds().contains(pos) {
                                stack.push(Action::VisitChildren(child))
                            }
                        }
                    }
                }
                Action::Done(widget_id) => {
                    if !f(self, widget_id) {
                        return;
                    }
                }
            }
        }
    }

    pub(super) fn merge_widget_flags(&mut self, source: WidgetId) {
        let mut current = source;
        let mut flags_to_apply = WidgetFlags::empty();
        while !current.is_null() {
            let data = self.widget_data_mut(current);
            data.flags |= flags_to_apply;
            flags_to_apply = data.flags & (WidgetFlags::NEEDS_LAYOUT);
            current = data.parent_id;
        }
    }

    pub(super) fn window(&self, id: WindowId) -> &WindowState {
        self.windows.get(id).expect("Window handle not found")
    }

    pub(super) fn window_mut(&mut self, id: WindowId) -> &mut WindowState {
        self.windows.get_mut(id).expect("Window handle not found")
    }

    pub fn window_theme_signal(&self, id: WidgetId) -> ReadSignal<WindowTheme> {
        self.window(self.get_window_id_for_widget(id))
            .theme_signal
            .as_read_signal()
    }

    pub fn get_window_id_for_widget(&self, widget_id: WidgetId) -> WindowId {
        self.widget_data[widget_id].window_id
    }

    pub fn run_effects(&mut self) {
        let mut tasks = self.runtime.take_tasks();

        if tasks.is_empty() {
            return;
        }

        while let Some(task) = tasks.pop_front() {
            task.run(self);
            tasks.extend(self.runtime.take_tasks().into_iter());
        }

        // Layout if needed
        let windows_needing_layout: Vec<_> = self
            .windows
            .iter()
            .filter_map(|(window_id, window_state)| {
                self.widget_data_ref(window_state.root_widget)
                    .flag_is_set(WidgetFlags::NEEDS_LAYOUT)
                    .then_some(window_id)
            })
            .collect();

        for window_to_layout in windows_needing_layout {
            layout_window(self, window_to_layout);
        }
    }

    pub fn clipboard(&self, window_id: WindowId) -> Clipboard {
        Clipboard {
            handle: &self.window(window_id).handle,
        }
    }
}

pub struct EffectContextImpl<'a> {
    pub(super) effect_id: NodeId,
    pub(super) app_state: &'a mut AppState,
}

impl EffectContext for EffectContextImpl<'_> {}

impl WidgetContext for EffectContextImpl<'_> {
    fn widget_ref_dyn(&self, id: WidgetId) -> WidgetRef<'_, dyn Widget> {
        WidgetRef::new(&self.app_state, id)
    }

    fn widget_mut_dyn(&mut self, id: WidgetId) -> WidgetMut<'_, dyn Widget> {
        WidgetMut::new(&mut self.app_state, id)
    }

    fn replace_widget_dun(&mut self, id: WidgetId, view: AnyView) {
        self.app_state.replace_widget(id, view);
    }
}

impl ReactiveContext for EffectContextImpl<'_> {
    fn runtime(&self) -> &Runtime {
        self.app_state.runtime()
    }

    fn runtime_mut(&mut self) -> &mut Runtime {
        self.app_state.runtime_mut()
    }
}

impl ReadContext for EffectContextImpl<'_> {
    fn scope(&self) -> Scope {
        Scope::Node(self.effect_id)
    }
}

impl WriteContext for EffectContextImpl<'_> {}

pub(super) enum Task {
    RunEffect {
        id: NodeId,
        f: Weak<RefCell<EffectFn>>,
    },
    UpdateBinding {
        f: Weak<RefCell<BindingFn>>,
        node_id: NodeId,
    },
    HandleEvent {
        f: Weak<HandleEventFn>,
        event: Rc<dyn Any>,
    },
    UpdateAnimation {
        node_id: NodeId,
        window_id: WindowId,
    },
}

impl Task {
    pub(super) fn run(self, app_state: &mut AppState) {
        match self {
            Task::RunEffect { id, f } => {
                if let Some(f) = f.upgrade() {
                    let mut cx = EffectContextImpl {
                        effect_id: id,
                        app_state,
                    };
                    (RefCell::borrow_mut(&f))(&mut cx);
                    app_state.runtime.mark_node_as_clean(id);
                }
            }
            Task::UpdateBinding { f, node_id } => {
                if let Some(f) = f.upgrade() {
                    (RefCell::borrow_mut(&f))(app_state);
                    app_state.runtime.mark_node_as_clean(node_id);
                }
            }
            Task::UpdateAnimation { node_id, window_id } => {
                app_state
                    .window_mut(window_id)
                    .pending_node_animations
                    .insert(node_id);
            }
            Task::HandleEvent { f, event } => {
                if let Some(f) = f.upgrade() {
                    f(app_state, &event);
                }
            }
        }
    }
}

impl ReactiveContext for AppState {
    fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    fn runtime_mut(&mut self) -> &mut Runtime {
        &mut self.runtime
    }
}

impl ReadContext for AppState {
    fn scope(&self) -> super::Scope {
        super::Scope::Root
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
        WidgetRef::new(self, id)
    }

    fn widget_mut_dyn(&mut self, id: WidgetId) -> WidgetMut<'_, dyn Widget> {
        WidgetMut::new(self, id)
    }

    fn replace_widget_dun(&mut self, id: WidgetId, view: AnyView) {
        self.replace_widget(id, view);
    }
}

impl WatchContext for AppState {}
