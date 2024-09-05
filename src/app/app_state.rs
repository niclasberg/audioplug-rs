use std::{any::Any, collections::VecDeque, marker::PhantomData, ops::DerefMut, rc::Weak};
use slotmap::{Key, SecondaryMap, SlotMap};
use crate::{core::{Point, Rectangle}, param::{AnyParameterMap, NormalizedValue, ParameterId, ParameterMap, Params, PlainValue}, platform};

use super::{accessor::SourceId, binding::BindingState, contexts::BuildContext, memo::{Memo, MemoState}, widget_node::{WidgetData, WidgetFlags, WidgetId, WidgetMut, WidgetRef}, Accessor, HostHandle, ParamContext, Runtime, Scope, SignalContext, SignalGet, Widget, WindowId};
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
                app_state.reactive_context.with_scope(Scope::Effect(*id), |cx| {
                    if let Some(f) = f.upgrade() {
                        f(cx)
                    }
                })
            },
            Task::UpdateBinding { widget_id, f } => {
                if let Some(f) = f.upgrade() {
                    f(&mut app_state.reactive_context, app_state.widgets[*widget_id].deref_mut(), &mut app_state.widget_data[*widget_id]);
					app_state.merge_widget_flags(*widget_id);
                }
            },
			Task::InvalidateRect { window_id, rect } => {
				app_state.window(*window_id).handle.invalidate(*rect);
			}
        }
    }
}

pub(super) struct WindowState {
    pub(super) handle: platform::Handle,
    pub(super) root_widget: WidgetId,
}

pub struct AppState {
    parameters: Box<dyn AnyParameterMap>,
    windows: SlotMap<WindowId, WindowState>,
    pub(super) widget_data: SlotMap<WidgetId, WidgetData>,
    pub(super) widgets: SecondaryMap<WidgetId, Box<dyn Widget>>,
    widget_bindings: SecondaryMap<NodeId, WidgetId>,
    pub(super) mouse_capture_widget: Option<WidgetId>,
    pub(super) focus_widget: Option<WidgetId>,
    pub(super) reactive_context: Runtime,
    host_handle: Box<dyn HostHandle>,
}

impl AppState {
    pub fn new(parameters: impl Params + Any, host_handle: impl HostHandle + 'static) -> Self {
        let host_handle = Box::new(host_handle);
		let parameters = Box::new(ParameterMap::new(parameters));
        Self {
            parameters,
            widget_data: Default::default(),
            widgets: Default::default(),
            widget_bindings: Default::default(),
            windows: Default::default(),
            mouse_capture_widget: None,
            focus_widget: None,
            reactive_context: Default::default(),
            host_handle
        }
    }

	pub fn parameters(&self) -> &dyn AnyParameterMap {
		self.parameters.as_ref()
	}

    pub fn create_signal<T: Any>(&mut self, value: T) -> Signal<T> {
        let state = SignalState::new(value);
        let id = self.reactive_context.create_signal_node(state);
        Signal::new(id)
    }

    pub fn create_memo<T: PartialEq + 'static>(&mut self, f: impl Fn(&mut Self) -> T + 'static) -> Memo<T> {
        let state = MemoState::new(move |cx| Box::new(f(cx)));
        let id = self.reactive_context.create_memo_node(state);
        Memo::new(id)
    }

    pub fn create_binding<T: 'static, W: Widget + 'static>(&mut self, accessor: Accessor<T>, widget_id: WidgetId, f: impl Fn(&T, WidgetMut<'_, W>) + 'static) -> bool {
		match accessor.get_source_id() {
			SourceId::None => false,
			SourceId::Parameter(_) => todo!(),
			SourceId::Node(source_id) => {
				let state = BindingState::new(widget_id, move |ctx, widget, data| {
					let tasks = accessor.with_ref(ctx, |value| {
						let mut tasks = VecDeque::new();
						let widget: &mut W = widget.downcast_mut().expect("Could not cast widget");
						let node = WidgetMut::new(widget, data, &mut tasks);
						f(value, node);
						tasks
					});
					ctx.pending_tasks.extend(tasks.into_iter());
				});
				self.reactive_context.create_binding_node(source_id, state);
				true
			}
		}
    }

    pub fn create_effect(&mut self, f: impl Fn(&mut Runtime) + 'static) {
        let id = self.reactive_context.create_effect_node(EffectState::new(f));
        self.reactive_context.notify(&id);
    }

	pub(crate) fn set_plain_parameter_value_from_host(&mut self, id: ParameterId, value: PlainValue) {

    }

    pub(crate) fn set_parameter_value_from_host(&mut self, id: ParameterId, value: NormalizedValue) {

    }

    pub fn add_window<W: Widget + 'static>(&mut self, handle: platform::Handle, f: impl FnOnce(&mut BuildContext<W>) -> W) -> WindowId {
		let window_id = self.windows.insert_with_key(|window_id| {
            WindowState {
                handle,
                root_widget: WidgetId::null()
            }
        });

		let widget_id = self.widget_data.insert_with_key(|id| {
			WidgetData::new(window_id, id)
		});

		self.windows[window_id].root_widget = widget_id;

		{
            let widget = f(&mut BuildContext::new(widget_id, self));
			self.widget_data[widget_id].style = widget.style();
            self.widgets.insert(widget_id, Box::new(widget));
        }
        
		window_id
    }

    pub fn remove_window(&mut self, id: WindowId) {
        let window = self.windows.remove(id).expect("Window not found");
        self.remove_widget(window.root_widget);
    }

    /// Add a new widget
    pub fn add_widget<W: Widget + 'static>(&mut self, parent_id: WidgetId, f: impl FnOnce(&mut BuildContext<W>) -> W) -> WidgetId {
		let window_id = self.widget_data.get(parent_id).expect("Parent not found").window_id;
        let id = self.widget_data.insert_with_key(|id| {
			WidgetData::new(window_id, id).with_parent(parent_id)
		});
        
		{
            let widget = f(&mut BuildContext::new(id, self));
			self.widget_data[id].style = widget.style();
            self.widgets.insert(id, Box::new(widget));
        }

        {
			let parent_widget_data = self.widget_data.get_mut(parent_id).expect("Parent does not exist");
			parent_widget_data.children.push(id);
		}

        id
    }

    /// Remove a widget and all of its children and associated signals
    pub fn remove_widget(&mut self, id: WidgetId) {
        let mut widget_data = self.widget_data.remove(id).unwrap();
        self.widgets.remove(id).expect("Widget already removed");

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

            children_to_remove.extend(widget_data.children.into_iter());
        }
    }

    pub fn widget_ref(&self, id: WidgetId) -> WidgetRef<'_, dyn Widget> {
        WidgetRef::new(&*self.widgets[id], &self.widget_data[id])
    }

    pub fn widget_mut(&mut self, id: WidgetId) -> WidgetMut<'_, dyn Widget> {
        WidgetMut::new(&mut *self.widgets[id], &mut self.widget_data[id], &mut self.reactive_context.pending_tasks)
    }

    pub fn widget_has_focus(&self, id: WidgetId) -> bool {
        self.focus_widget.as_ref()
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
            for child in self.widget_data[current].children.iter().rev() {
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
			data.flags = data.flags & flags_to_apply;
			flags_to_apply = data.flags & (WidgetFlags::NEEDS_LAYOUT);
			current = data.parent_id;
		}
	}

    pub(super) fn window(&self, id: WindowId) -> &WindowState {
        self.windows.get(id).expect("Window handle not found")
    }

    pub fn get_window_id_for_widget(&self, widget_id: WidgetId) -> WindowId {
        self.widget_data[widget_id].window_id
    }

    pub fn run_effects(&mut self) {
        let mut tasks = self.reactive_context.take_tasks();
        loop {
            if let Some(task) = tasks.pop_front() {
                task.run(self);
                tasks.extend(self.reactive_context.take_tasks().into_iter());
            } else {
                break;
            }
        }
    }
}

impl SignalContext for AppState {
    fn get_signal_value_ref_untracked<'a, T: Any>(&'a self, signal: &Signal<T>) -> &'a T {
        self.reactive_context.get_signal_value_ref_untracked(signal)
    }

    fn get_signal_value_ref<'a, T: Any>(&'a mut self, signal: &Signal<T>) -> &'a T {
        self.reactive_context.get_signal_value_ref(signal)
    }

    fn set_signal_value<T: Any>(&mut self, signal: &Signal<T>, value: T) {
        self.reactive_context.set_signal_value(signal, value)
    }
}

impl ParamContext for AppState {
    fn host_handle(&self) -> &dyn HostHandle {
        self.host_handle.as_ref()
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