use parley::FontContext;

use super::reactive::{CreateContext, ReactiveGraph, Var};
use super::{
    BuildContext, HostHandle, View, WidgetId, Widgets, WindowId, layout::RecomputeLayout,
    render::WGPUSurface, task_queue::TaskQueue,
};
use crate::core::TextLayoutContext;
use crate::ui::TextContext;
use crate::ui::reactive::{Owner, ReadContext, ReadScope, WriteContext};
use crate::ui::render::GlyphCache;
use crate::ui::widgets::WidgetHandlers;
use crate::{
    core::WindowTheme,
    param::{AnyParameterMap, NormalizedValue, ParameterId, PlainValue},
    platform,
};
use std::rc::Rc;

pub struct AppState {
    pub(super) wgpu_instance: wgpu::Instance,
    pub(super) font_cx: parley::FontContext,
    pub(super) text_layout_cx: TextLayoutContext,
    pub(super) glyph_cache: GlyphCache,
    pub(super) widgets: Widgets,
    pub(super) widget_handlers: WidgetHandlers,
    pub(super) reactive_graph: ReactiveGraph,
    pub(super) task_queue: TaskQueue,
    pub(crate) theme_signal: Var<WindowTheme>,
}

impl AppState {
    pub fn new(parameters: Rc<dyn AnyParameterMap>) -> Self {
        let mut reactive_graph = ReactiveGraph::new(parameters);
        let task_queue = TaskQueue::default();
        let mut widgets = Widgets::default();
        let theme_signal = Var::new(
            &mut CreateContext::new_root_context(&mut widgets, &mut reactive_graph),
            WindowTheme::Dark,
        );

        Self {
            wgpu_instance: wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                ..Default::default()
            }),
            font_cx: FontContext::new(),
            text_layout_cx: TextLayoutContext::new(),
            glyph_cache: GlyphCache::new(),
            reactive_graph,
            theme_signal,
            widgets,
            task_queue,
            widget_handlers: Default::default(),
        }
    }

    pub fn parameters(&self) -> &dyn AnyParameterMap {
        self.reactive_graph.parameters.as_ref()
    }

    pub(crate) fn set_host_handle(&mut self, host_handle: Option<Box<dyn HostHandle>>) {
        self.reactive_graph.set_host_handle(host_handle);
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
        self.write_context().notify_parameter_subscribers(id);
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
        self.write_context().notify_parameter_subscribers(id);
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
        view.build(&mut BuildContext {
            id: root_widget_id,
            widgets: &mut self.widgets,
            handlers: &mut self.widget_handlers,
            reactive_graph: &mut self.reactive_graph,
        });
        self.run_effects();
        self.widgets.layout_window(
            &mut TextContext::new(
                &mut self.font_cx,
                &mut self.text_layout_cx,
                &mut self.glyph_cache,
            ),
            window_id,
            RecomputeLayout::Force,
        );
        window_id
    }

    pub fn remove_window(&mut self, id: WindowId) {
        self.widgets
            .remove_window(id, &mut self.widget_handlers, &mut self.reactive_graph);
    }

    pub fn run_effects(&mut self) {
        loop {
            let ran_effect = self
                .reactive_graph
                .run_effects(&mut self.widgets, &mut self.widget_handlers);

            let mut tasks = std::mem::take(&mut self.task_queue.tasks);
            let ran_task = !tasks.is_empty();
            while let Some(task) = tasks.pop_front() {
                task.run(self);
                tasks.extend(self.task_queue.tasks.drain(..));
            }

            if !(ran_effect || ran_task) {
                break;
            }
        }

        self.widgets
            .update_widget_props(&mut self.reactive_graph, &self.widget_handlers);

        // Layout if needed
        let window_ids: Vec<_> = self.widgets.window_id_iter().collect();
        for window_id in window_ids {
            self.widgets.layout_window(
                &mut TextContext::new(
                    &mut self.font_cx,
                    &mut self.text_layout_cx,
                    &mut self.glyph_cache,
                ),
                window_id,
                RecomputeLayout::IfNeeded,
            );
        }
    }

    pub fn read_context(
        &mut self,
        scope: ReadScope,
        current_widget: Option<WidgetId>,
    ) -> ReadContext<'_> {
        ReadContext {
            widgets: &self.widgets,
            reactive_graph: &mut self.reactive_graph,
            scope,
            current_widget,
        }
    }

    pub fn write_context(&mut self) -> WriteContext<'_> {
        WriteContext {
            widgets: &mut self.widgets,
            reactive_graph: &mut self.reactive_graph,
        }
    }

    pub fn create_context(&mut self, owner: Owner) -> CreateContext<'_> {
        CreateContext {
            widgets: &mut self.widgets,
            reactive_graph: &mut self.reactive_graph,
            owner,
        }
    }
}
