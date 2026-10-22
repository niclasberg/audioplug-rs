use super::{
    ViewSequence, WidgetId, WidgetPos,
    overlay::OverlayOptions,
    reactive::{CanCreate, CanRead, Owner, ReadScope},
};
use crate::{
    core::{Axis, Size},
    ui::{
        ButtonInteraction, EventContext, EventHandlers, KnobInteraction, RenderContext, WidgetData,
        WidgetProps, Widgets,
        interaction::{Interaction, KnobEvent, SliderEvent, SliderInteraction},
        reactive::{CreateContext, ReactiveGraph, ReadContext},
        style::DisplayStyle,
        widgets::WidgetHandlers,
    },
};

pub type AnyView = Box<dyn FnOnce(&mut BuildContext)>;

pub trait View: 'static {
    fn build(self, cx: &mut BuildContext);

    fn into_any_view(self) -> AnyView
    where
        Self: Sized + 'static,
    {
        Box::new(move |ctx| self.build(ctx))
    }
}

impl View for AnyView {
    fn build(self, ctx: &mut BuildContext) {
        self(ctx)
    }

    fn into_any_view(self) -> AnyView {
        self
    }
}

pub struct BuildContext<'a> {
    pub(crate) id: WidgetId,
    pub(crate) widgets: &'a mut Widgets,
    pub(crate) handlers: &'a mut WidgetHandlers,
    pub(crate) reactive_graph: &'a mut ReactiveGraph,
}

impl<'a> BuildContext<'a> {
    pub fn id(&self) -> WidgetId {
        self.id
    }

    fn widget_data_mut(&mut self) -> &mut WidgetData {
        &mut self.widgets.tree[self.id]
    }

    pub fn set_display_style(&mut self, display_style: DisplayStyle) {
        self.widget_data_mut().display_style = display_style;
    }

    pub fn intrinsic_size(&mut self, size: Size) -> &mut Self {
        self.widget_data_mut().intrinsic_size = Some(size);
        self
    }

    pub fn add_child(&mut self, view: impl View) -> WidgetId {
        self.widgets.add_widget(
            self.handlers,
            self.reactive_graph,
            WidgetPos::LastChild(self.id),
            view,
        )
    }

    pub fn add_children(&mut self, view_sequence: impl ViewSequence) {
        view_sequence.build_seq(&mut BuildContext {
            id: self.id,
            widgets: self.widgets,
            handlers: self.handlers,
            reactive_graph: self.reactive_graph,
        });
    }

    pub fn add_overlay(&mut self, view: impl View, options: OverlayOptions) -> WidgetId {
        self.widgets.add_widget(
            self.handlers,
            self.reactive_graph,
            WidgetPos::Overlay(self.id, options),
            view,
        )
    }

    pub fn props(&mut self, mut props: WidgetProps) -> &mut Self {
        // The props can contain both constant and reactive values.
        // We only need to retain the reactive values - we synchronize to those
        // values after handling events. We still want to apply the constant values,
        // so run apply for all props first.
        props.apply(&mut self.reactive_graph, self.id, &mut self.widgets);
        props.remove_const_props();
        self.handlers.widget_props.insert(self.id, props);
        self
    }

    pub fn interaction(&mut self, kind: Interaction) -> &mut Self {
        self.handlers.event_handlers.insert(self.id, kind);
        self
    }

    pub fn event_handlers(&mut self, event_handlers: EventHandlers) -> &mut Self {
        self.interaction(Interaction::Events(event_handlers))
    }

    pub fn button(&mut self, f: impl FnMut(&mut EventContext) + 'static) -> &mut Self {
        self.interaction(Interaction::Button(ButtonInteraction::new(f)))
    }

    pub fn knob(&mut self, f: impl FnMut(&mut EventContext, KnobEvent) + 'static) -> &mut Self {
        self.interaction(Interaction::Knob(KnobInteraction::new(f)))
    }

    pub fn slider(
        &mut self,
        f: impl FnMut(&mut EventContext, SliderEvent) + 'static,
        direction: Axis,
    ) -> &mut Self {
        self.interaction(Interaction::Slider(SliderInteraction::new(f, direction)))
    }

    pub fn render(&mut self, f: impl FnMut(&mut RenderContext) + 'static) -> &mut Self {
        self.handlers.renderers.insert(self.id, Box::new(f));
        self
    }
}

impl<'a> CanRead<'a> for BuildContext<'a> {
    fn read_context<'s2>(&'s2 mut self) -> ReadContext<'s2>
    where
        'a: 's2,
    {
        ReadContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            scope: ReadScope::Untracked,
            current_widget: Some(self.id),
        }
    }
}

impl<'s> CanCreate<'s> for BuildContext<'s> {
    fn create_context<'s2>(&'s2 mut self) -> CreateContext<'s2>
    where
        's: 's2,
    {
        CreateContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            owner: Owner::Widget(self.id),
        }
    }
}
