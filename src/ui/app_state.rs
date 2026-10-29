use parley::FontContext;

use super::reactive::{CreateContext, Var};
use super::{
    BuildContext, HostHandle, UiState, View, WidgetId, WindowId, layout::RecomputeLayout,
    render::WGPUSurface, task_queue::TaskQueue,
};
use crate::core::TextLayoutContext;
use crate::ui::TextContext;
use crate::ui::reactive::{Owner, ReadContext, ReadScope, WriteContext};
use crate::ui::render::GlyphCache;
use crate::ui::ui_state::WidgetHandlers;
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
    pub(super) ui: UiState,
    pub(super) widget_handlers: WidgetHandlers,
    pub(super) task_queue: TaskQueue,
    pub(crate) theme_signal: Var<WindowTheme>,
}

impl AppState {
    pub fn new(parameters: Rc<dyn AnyParameterMap>) -> Self {
        let task_queue = TaskQueue::default();
        let mut ui = UiState::new(parameters);
        let theme_signal = Var::new(
            &mut CreateContext::new_root_context(&mut ui),
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
            theme_signal,
            ui,
            task_queue,
            widget_handlers: Default::default(),
        }
    }

    pub fn parameters(&self) -> &dyn AnyParameterMap {
        self.ui.parameters.as_ref()
    }

    pub(crate) fn set_host_handle(&mut self, host_handle: Option<Box<dyn HostHandle>>) {
        self.ui.set_host_handle(host_handle);
    }

    #[allow(dead_code)]
    pub(crate) fn set_plain_parameter_value_from_host(
        &mut self,
        id: ParameterId,
        value: PlainValue,
    ) -> bool {
        let Some(param_ref) = self.ui.parameters.get_by_id(id) else {
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
        let Some(param_ref) = self.ui.parameters.get_by_id(id) else {
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
        let window_id = self.ui.allocate_window(handle, wgpu_surface);
        let root_widget_id = self.ui.window(window_id).root_widget;
        view.build(&mut BuildContext::new(
            root_widget_id,
            &mut self.ui,
            &mut self.widget_handlers,
        ));
        self.run_effects();
        self.ui.layout_window(
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
        self.ui.remove_window(id, &mut self.widget_handlers);
    }

    pub fn run_effects(&mut self) {
        loop {
            let ran_effect = self.ui.run_effects(&mut self.widget_handlers);

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

        self.ui.update_widget_props(&self.widget_handlers);

        // Layout if needed
        let window_ids: Vec<_> = self.ui.window_id_iter().collect();
        for window_id in window_ids {
            self.ui.layout_window(
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
            ui: &mut self.ui,
            scope,
            current_widget,
        }
    }

    pub fn write_context(&mut self) -> WriteContext<'_> {
        WriteContext { ui: &mut self.ui }
    }

    pub fn create_context(&mut self, owner: Owner) -> CreateContext<'_> {
        CreateContext {
            ui: &mut self.ui,
            owner,
        }
    }
}
