use crate::ui::{
    BuildContext, EventHandlerExt, EventHandlers, Prop, StyleExt, View, ViewExt, ViewSequence,
    WidgetProps,
    reactive::{Cached, ReactiveValue},
    style::{AlignItems, DisplayStyle, FlexDirection, FlexStyle, FlexWrap, JustifyContent, Length},
};

pub type Row<VS> = FlexContainer<VS, true>;
pub type Column<VS> = FlexContainer<VS, false>;

pub struct FlexContainer<VS, const IS_ROW: bool> {
    props: WidgetProps,
    view_seq: VS,
    spacing: Prop<Length>,
    wrap: Prop<FlexWrap>,
    align_items: Option<Prop<AlignItems>>,
    justify_content: Option<Prop<JustifyContent>>,
    event_handlers: EventHandlers,
}

impl<VS: ViewSequence, const IS_ROW: bool> FlexContainer<VS, IS_ROW> {
    pub fn new(view_seq: VS) -> Self {
        Self {
            view_seq,
            spacing: Prop::Const(Length::ZERO),
            wrap: Prop::Const(Default::default()),
            align_items: None,
            justify_content: None,
            props: Default::default(),
            event_handlers: Default::default(),
        }
    }

    pub fn wrapping(mut self, value: impl Into<Prop<FlexWrap>>) -> Self {
        self.wrap = value.into();
        self
    }

    /// Allow children to wrap to a new line (for column) or column (for rows) instead of overflowing
    pub fn wrap(self) -> Self {
        self.wrapping(FlexWrap::Wrap)
    }

    /// Allow children to wrap in reverse order
    pub fn wrap_reverse(self) -> Self {
        self.wrapping(FlexWrap::WrapReverse)
    }

    pub fn spacing(mut self, value: impl Into<Prop<Length>>) -> Self {
        self.spacing = value.into();
        self
    }

    pub fn center(mut self) -> Self {
        self.justify_content = Some(JustifyContent::CENTER.into());
        self.align_items = Some(AlignItems::CENTER.into());
        self
    }
}

impl<VS> Row<VS> {
    pub fn h_align(mut self, value: impl Into<Prop<JustifyContent>>) -> Self {
        self.justify_content = Some(value.into());
        self
    }

    pub fn v_align(mut self, value: impl Into<Prop<AlignItems>>) -> Self {
        self.align_items = Some(value.into());
        self
    }

    pub fn h_align_top(self) -> Self {
        self.h_align(taffy::JustifyContent::START)
    }

    pub fn h_align_center(self) -> Self {
        self.h_align(taffy::JustifyContent::CENTER)
    }

    pub fn h_align_bottom(self) -> Self {
        self.h_align(taffy::JustifyContent::END)
    }

    pub fn h_align_space_around(self) -> Self {
        self.h_align(taffy::JustifyContent::SPACE_AROUND)
    }

    pub fn h_align_space_between(self) -> Self {
        self.h_align(taffy::JustifyContent::SPACE_BETWEEN)
    }

    pub fn h_align_space_evenly(self) -> Self {
        self.h_align(taffy::JustifyContent::SPACE_EVENLY)
    }

    pub fn v_align_center(self) -> Self {
        self.v_align(taffy::AlignItems::CENTER)
    }
}

impl<VS> Column<VS> {
    pub fn v_align(mut self, value: impl Into<Prop<AlignItems>>) -> Self {
        self.align_items = Some(value.into());
        self
    }

    pub fn h_align(mut self, value: impl Into<Prop<taffy::AlignContent>>) -> Self {
        self.justify_content = Some(value.into());
        self
    }
}

impl<VS, const IS_ROW: bool> ViewExt for FlexContainer<VS, IS_ROW> {
    fn props_mut(&mut self) -> &mut WidgetProps {
        &mut self.props
    }
}
impl<VS, const IS_ROW: bool> StyleExt for FlexContainer<VS, IS_ROW> {}

impl<VS: ViewSequence, const IS_ROW: bool> View for FlexContainer<VS, IS_ROW> {
    fn build(self, cx: &mut BuildContext) {
        cx.props(self.props).event_handlers(self.event_handlers);
        let display_style = Cached::new(cx, move |cx, _| {
            DisplayStyle::Flex(FlexStyle {
                direction: if IS_ROW {
                    FlexDirection::Row
                } else {
                    FlexDirection::Column
                },
                wrap: self.wrap.get(cx),
                gap: self.spacing.get(cx),
                align_items: self.align_items.as_ref().map(|x| x.get(cx)),
                align_content: self.justify_content.as_ref().map(|x| x.get(cx)),
            })
        })
        .into();
        build_container(cx, self.view_seq, display_style);
    }
}

fn build_container<VS: ViewSequence>(
    cx: &mut BuildContext,
    children: VS,
    display_style: Prop<DisplayStyle>,
) {
    cx.add_children(children);
    let display_style = display_style.get_and_bind(cx, |value, mut widget| {
        widget.set_display_style(value);
    });
    cx.set_display_style(display_style);
}

pub struct Grid<VS> {
    view_seq: VS,
    columns: Prop<Vec<taffy::TrackSizingFunction>>,
    rows: Prop<Vec<taffy::TrackSizingFunction>>,
}

impl<VS: ViewSequence> Grid<VS> {
    pub fn new(
        f_rows: impl FnOnce(&mut GridStyleBuilder),
        f_cols: impl FnOnce(&mut GridStyleBuilder),
        view_seq: VS,
    ) -> Self {
        Self {
            view_seq,
            columns: Vec::new().into(),
            rows: Vec::new().into(),
        }
    }
}

pub struct GridStyleBuilder {}

pub struct Container<VS> {
    props: WidgetProps,
    view_seq: VS,
    display_style: Prop<DisplayStyle>,
    event_handlers: EventHandlers,
}

impl<VS: ViewSequence> Container<VS> {
    pub fn new(view_seq: VS) -> Self {
        Self {
            props: Default::default(),
            view_seq,
            display_style: Prop::Const(DisplayStyle::Block),
            event_handlers: Default::default(),
        }
    }
}

impl<VS: ViewSequence> View for Container<VS> {
    fn build(self, cx: &mut BuildContext) {
        cx.props(self.props);
        build_container(cx, self.view_seq, self.display_style);
    }
}

impl<VS> ViewExt for Container<VS> {
    fn props_mut(&mut self) -> &mut WidgetProps {
        &mut self.props
    }
}
impl<VS> StyleExt for Container<VS> {}

impl<VS> EventHandlerExt for Container<VS> {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}
