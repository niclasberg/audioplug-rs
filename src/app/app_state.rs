use std::{any::Any, collections::{HashSet, VecDeque}, ops::DerefMut, rc::{Rc, Weak}};
use slotmap::{Key, SecondaryMap, SlotMap};
use crate::{core::{Point, Rectangle}, param::{AnyParameter, AnyParameterMap, NormalizedValue, ParamRef, ParameterId, PlainValue}, platform};

use super::{accessor::SourceId, binding::BindingState, contexts::BuildContext, layout_window, memo::{Memo, MemoState}, widget_node::{WidgetData, WidgetFlags, WidgetId, WidgetMut, WidgetRef}, Accessor, HostHandle, ParamContext, ParamEditor, Runtime, Scope, SignalContext, SignalGet, SignalGetContext, Widget, WindowId};
use super::NodeId;
use super::signal::{Signal, SignalState};
use super::effect::EffectState;

pub(super) enum Task {
    RunEffect {
        id: NodeId,
        f: Weak<Box<dyn Fn(&mut Runtime)>>
    },
	UpdateBinding {
		widget_id: WidgetId,
    	f: Weak<Box<dyn Fn(&mut Runtime, &mut dyn Widget, &mut WidgetData)>>,
	},
	InvalidateRect {
		window_id: WindowId,
		rect: Rectangle
	}
}

impl Task {
    pub(super) fn run(&self, app_state: &mut AppState) {
        match self {
            Task::RunEffect { id, f } => {
                app_state.runtime.with_scope(Scope::Effect(*id), |cx| {
                    if let Some(f) = f.upgrade() {
                        f(cx)
                    }
                })
            },
            Task::UpdateBinding { widget_id, f } => {
                if let Some(f) = f.upgrade() {
                    // Widget might have been removed
                    if let Some(widget) = app_state.widgets.get_mut(*widget_id) {
                        f(&mut app_state.runtime, widget.deref_mut(), &mut app_state.widget_data[*widget_id]);
					    app_state.merge_widget_flags(*widget_id);
                    }
                }
            },
			Task::InvalidateRect { window_id, rect } => {
                if let Some(window) = app_state.windows.get(*window_id) {
                    window.handle.invalidate(*rect);
                }
			}
        }
    }
}

pub(super) struct WindowState {
    pub(super) handle: platform::Handle,
    pub(super) root_widget: WidgetId,
    pub(super) focus_widget: Option<WidgetId>,
}

pub struct AppState {
    windows: SlotMap<WindowId, WindowState>,
    pub(super) widget_data: SlotMap<WidgetId, WidgetData>,
    pub(super) widgets: SecondaryMap<WidgetId, Box<dyn Widget>>,
    widget_bindings: SecondaryMap<WidgetId, HashSet<NodeId>>,
    pub(super) mouse_capture_widget: Option<WidgetId>,
    pub(super) runtime: Runtime,
    host_handle: Option<Box<dyn HostHandle>>,
    executor: Rc<platform::Executor>
}

impl AppState {
    pub fn new(parameters: Rc<dyn AnyParameterMap>, executor: Rc<platform::Executor>) -> Self {
        Self {
            widget_data: Default::default(),
            widgets: Default::default(),
            widget_bindings: Default::default(),
            windows: Default::default(),
            mouse_capture_widget: None,
            runtime: Runtime::new(parameters),
            host_handle: None,
            executor
        }
    }

	pub fn parameters(&self) -> &dyn AnyParameterMap {
		self.runtime.parameters.as_ref()
	}

    pub fn create_signal<T: Any>(&mut self, value: T) -> Signal<T> {
        let state = SignalState::new(value);
        let id = self.runtime.create_signal_node(state);
        Signal::new(id)
    }

    fn add_widget_binding(&mut self, widget_id: WidgetId, node_id: NodeId) {
        self.widget_bindings.entry(widget_id).unwrap()
            .and_modify(|bindings| { bindings.insert(node_id); })
            .or_default();
    }

    fn clear_widget_bindings(&mut self, widget_id: WidgetId) {
        if let Some(bindings) = self.widget_bindings.remove(widget_id) {
            for node_id in bindings {
                self.runtime.remove_node(node_id);
            }
        }
    }

    pub fn create_memo<T: PartialEq + 'static>(&mut self, f: impl Fn(&mut Self) -> T + 'static) -> Memo<T> {
        let state = MemoState::new(move |cx| Box::new(f(cx)));
        let id = self.runtime.create_memo_node(state);
        Memo::new(id)
    }

    pub fn create_binding<T: 'static, W: Widget + 'static>(&mut self, accessor: Accessor<T>, widget_id: WidgetId, f: impl Fn(&T, WidgetMut<'_, W>) + 'static) -> bool {
		let source_id = accessor.get_source_id();
		let create_state = move || -> BindingState {
			BindingState::new(widget_id, move |ctx, widget, data| {
				let tasks = accessor.with_ref(ctx, |value| {
					let mut tasks = VecDeque::new();
					let widget: &mut W = widget.downcast_mut().expect("Could not cast widget");
					let node = WidgetMut::new(widget, data, &mut tasks);
					f(value, node);
					tasks
				});
				ctx.pending_tasks.extend(tasks.into_iter());
			})
		};

		match source_id {
			SourceId::None => false,
			SourceId::Parameter(source_id) => {
				let node_id = self.runtime.create_parameter_binding_node(source_id, create_state());
                self.add_widget_binding(widget_id, node_id);
				true
			},
			SourceId::Node(source_id) => {
				let node_id = self.runtime.create_binding_node(source_id, create_state());
                self.add_widget_binding(widget_id, node_id);
				true
			}
		}
    }

    pub fn create_effect(&mut self, f: impl Fn(&mut Runtime) + 'static) {
        let id = self.runtime.create_effect_node(EffectState::new(f));
        self.runtime.notify(&id);
    }

    pub(crate) fn set_host_handle(&mut self, host_handle: Option<Box<dyn HostHandle>>) {
        self.host_handle = host_handle;
    }

    #[allow(dead_code)]
	pub(crate) fn set_plain_parameter_value_from_host(&mut self, id: ParameterId, value: PlainValue) -> bool {
		let Some(param_ref) = self.runtime.parameters.get_by_id(id) else { return false };
		param_ref.internal_set_value_plain(value);
		self.runtime.notify_parameter_subscribers(id);
		self.run_effects();
        true
    }

    pub(crate) fn set_normalized_parameter_value_from_host(&mut self, id: ParameterId, value: NormalizedValue) -> bool {
        let Some(param_ref) = self.runtime.parameters.get_by_id(id) else { return false };
		param_ref.internal_set_value_normalized(value);
		self.runtime.notify_parameter_subscribers(id);
		self.run_effects();
        true
    }

    pub fn add_window<W: Widget + 'static>(&mut self, handle: platform::Handle, f: impl FnOnce(&mut BuildContext<W>) -> W) -> WindowId {
		let window_id = self.windows.insert(
            WindowState {
                handle,
                root_widget: WidgetId::null(),
                focus_widget: None,
            });

		let widget_id = self.widget_data.insert_with_key(|id| {
			WidgetData::new(window_id, id)
		});

		self.windows[window_id].root_widget = widget_id;

		{
            let widget = f(&mut BuildContext::new(widget_id, self));
            self.widgets.insert(widget_id, Box::new(widget));
        }
        
		window_id
    }

    /// Add a new widget
    pub fn add_widget<W: Widget + 'static>(&mut self, parent_id: WidgetId, f: impl FnOnce(&mut BuildContext<W>) -> W) -> WidgetId {
		let window_id = self.widget_data.get(parent_id).expect("Parent not found").window_id;
        let id = self.widget_data.insert_with_key(|id| {
			WidgetData::new(window_id, id).with_parent(parent_id)
		});
        
		{
            let widget = f(&mut BuildContext::new(id, self));
            self.widgets.insert(id, Box::new(widget));
        }

        {
			let parent_widget_data = self.widget_data.get_mut(parent_id).expect("Parent does not exist");
			parent_widget_data.children.push(id);
		}

        id
    }

    pub fn remove_window(&mut self, id: WindowId) {
        let window = self.windows.remove(id).expect("Window not found");
        self.remove_widget(window.root_widget);
    }

    /// Remove a widget and all of its children and associated signals
    pub fn remove_widget(&mut self, id: WidgetId) {
        let mut widget_data = self.widget_data.remove(id).unwrap();
        self.widgets.remove(id).expect("Widget already removed");
        self.clear_widget_bindings(id);

        // Must be removed from the children of the parent
        if !widget_data.parent_id.is_null() {
            let parent_id = widget_data.parent_id;
            let parent_widget_data = self.widget_data.get_mut(parent_id).expect("Parent does not exist");
            parent_widget_data.children.retain(|id| *id != parent_id);
        }

        let mut children_to_remove = std::mem::take(&mut widget_data.children);
        while let Some(id) = children_to_remove.pop() {
            let widget_data = self.widget_data.remove(id).unwrap();
            self.widgets.remove(id).expect("Widget already removed");
            self.clear_widget_bindings(id);

            children_to_remove.extend(widget_data.children.into_iter());
        }
    }

    pub fn widget_ref(&self, id: WidgetId) -> WidgetRef<'_, dyn Widget> {
        WidgetRef::new(&*self.widgets[id], &self.widget_data[id])
    }

    pub fn widget_mut(&mut self, id: WidgetId) -> WidgetMut<'_, dyn Widget> {
        WidgetMut::new(&mut *self.widgets[id], &mut self.widget_data[id], &mut self.runtime.pending_tasks)
    }

    pub fn with_widget_mut<R>(&mut self, id: WidgetId, f: impl FnOnce(&mut Self, &mut dyn Widget) -> R) -> R {
        let Some(mut widget) = self.widgets.remove(id) else { panic!("Widget does not exist") };
        let value = f(self, widget.as_mut());
        self.widgets.insert(id, widget);
        value
    }

    pub fn widget_has_focus(&self, id: WidgetId) -> bool {
        self.window(self.widget_data_ref(id).window_id)
            .focus_widget.as_ref()
            .is_some_and(|focus_widget_id| *focus_widget_id == id)
    }

    pub fn widget_data_ref(&self, id: WidgetId) -> &WidgetData {
        self.widget_data.get(id).expect("Widget data not found")
    }

    pub fn widget_data_mut(&mut self, id: WidgetId) -> &mut WidgetData {
        self.widget_data.get_mut(id).expect("Widget data not found")
    }

    /// Calls `f` for each widget that conatains `pos`. The order is from the root and down the tree (depth first order)
    pub fn for_each_widget_at(&self, id: WindowId, pos: Point, mut f: impl FnMut(&Self, WidgetId) -> bool) {
        let mut stack = vec![self.windows[id].root_widget];
        while let Some(current) = stack.pop() {
            if !f(&self, current) {
                return;
            }

            let data = &self.widget_data[current];
            for child in self.widget_data[current].children.iter() {
                if data.global_bounds().contains(pos) {
                    stack.push(*child)
                }
            }
        }
    }

    pub fn for_each_widget_at_rev(&self, id: WindowId, pos: Point, f: impl FnMut(&Self, WidgetId) -> bool) {
		// TODO: implement
		self.for_each_widget_at(id, pos, f)
    }

	pub(super) fn merge_widget_flags(&mut self, source: WidgetId) {
		let mut current = source;
		let mut flags_to_apply = WidgetFlags::empty();
		while !current.is_null() {
			let data = self.widget_data_mut(current);
			data.flags = data.flags | flags_to_apply;
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

    pub fn get_window_id_for_widget(&self, widget_id: WidgetId) -> WindowId {
        self.widget_data[widget_id].window_id
    }

    pub fn run_effects(&mut self) {
        let mut tasks = self.runtime.take_tasks();

        if tasks.is_empty() {
            return;
        }

        loop {
            if let Some(task) = tasks.pop_front() {
                task.run(self);
                tasks.extend(self.runtime.take_tasks().into_iter());
            } else {
                break;
            }
        }

        // Layout if needed
        let windows_needing_layout: Vec<_> = self.windows.iter()
            .filter_map(|(window_id, window_state)|
                self.widget_data_ref(window_state.root_widget)
                    .flag_is_set(WidgetFlags::NEEDS_LAYOUT)
                    .then_some(window_id)
            )
            .collect();

        for window_to_layout in windows_needing_layout {
            layout_window(self, window_to_layout);
        }
    }
}

impl SignalGetContext for AppState {
    fn get_signal_value_ref_untracked<'a>(&'a self, signal_id: NodeId) -> &'a dyn Any {
        self.runtime.get_signal_value_ref_untracked(signal_id)
    }

    fn get_signal_value_ref<'a>(&'a mut self, signal_id: NodeId) -> &'a dyn Any {
        self.runtime.get_signal_value_ref(signal_id)
    }
	
	fn get_parameter_ref_untracked(&self, parameter_id: ParameterId) -> ParamRef {
		self.runtime.get_parameter_ref_untracked(parameter_id)
	}
	
	fn get_parameter_ref(&mut self, parameter_id: ParameterId) -> ParamRef {
		self.runtime.get_parameter_ref(parameter_id)
	}
}

impl SignalContext for AppState {
    fn set_signal_value<T: Any>(&mut self, signal: &Signal<T>, value: T) {
        self.runtime.set_signal_value(signal, value)
    }
}

impl ParamContext for AppState {
    fn host_handle(&self) -> &dyn HostHandle {
        let Some(host_handle) = self.host_handle.as_ref() else { panic!("Host handle not set") };
        host_handle.as_ref()
    }
    
    fn get_parameter_as<'a, P: AnyParameter>(&'a self, param: &ParamEditor<P>) -> &'a P {
        if let Some(p) = self.runtime.parameters.get_by_id_as_any(param.id) {
            p.downcast_ref().expect("Parameter had wrong type")
        } else {
            unreachable!()
        }
    }
    
    fn get_parameter_ref<'a>(&'a self, id: ParameterId) -> Option<ParamRef<'a>> {
        self.runtime.parameters.get_by_id(id)
    }
}


/*#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_signal() {
        let mut cx = ReactiveGraph::new();
        let signal = cx.create_signal(0);
        let mapped_signal = signal.clone().map(|x| x * 2);    
        signal.set(&mut cx, 2);
        cx.create_effect(move |cx| {
            assert_eq!(mapped_signal.get(cx), 4);
        });
        assert_eq!(mapped_signal.get_untracked(&cx), 4);
    }

    #[test]
    fn effects_execute_upon_creation() {
        let mut cx = ReactiveGraph::new();
        let signal = cx.create_signal(0);
        cx.create_effect(move |cx| {
            signal.set(cx, 1);
        });
        
        assert_eq!(signal.get_untracked(&cx), 1);
    }

    #[test]
    fn effects_execute_when_signal_changes() {
        let mut cx = ReactiveGraph::new();
        let source_signal = cx.create_signal(0);
        let dest_signal = cx.create_signal(0);
        cx.create_effect(move |cx| {
            let new_value = source_signal.get(cx);
            dest_signal.set(cx, new_value);
        });

        source_signal.set(&mut cx, 1);
        assert_eq!(dest_signal.get_untracked(&cx), 1);

        source_signal.set(&mut cx, 2);
        assert_eq!(dest_signal.get_untracked(&cx), 2);
    }
}*/