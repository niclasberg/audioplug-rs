use crate::ui::{
    BuildContext, StyleExt, View, ViewProp, ViewSequence, ViewStyle, Widget,
    reactive::{Cached, ReactiveValue},
    style::{
        AlignItems, DisplayStyle, FlexDirection, FlexStyle, FlexWrap, GridStyle, JustifyContent,
        LayoutMode, Length,
    },
};

pub type Row<VS> = FlexContainer<VS, true>;
pub type Column<VS> = FlexContainer<VS, false>;

pub struct FlexContainer<VS, const IS_ROW: bool> {
    base: ViewStyle,
    view_seq: VS,
    spacing: ViewProp<Length>,
    wrap: ViewProp<FlexWrap>,
    align_items: Option<ViewProp<AlignItems>>,
    justify_content: Option<ViewProp<JustifyContent>>,
}

impl<VS: ViewSequence, const IS_ROW: bool> FlexContainer<VS, IS_ROW> {
    pub fn new(view_seq: VS) -> Self {
        Self {
            view_seq,
            spacing: ViewProp::Const(Length::ZERO),
            wrap: ViewProp::Const(Default::default()),
            align_items: None,
            justify_content: None,
            base: ViewStyle::DEFAULT,
        }
    }

    pub fn wrapping(mut self, value: impl Into<ViewProp<FlexWrap>>) -> Self {
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

    pub fn spacing(mut self, value: impl Into<ViewProp<Length>>) -> Self {
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
    pub fn h_align(mut self, value: impl Into<ViewProp<JustifyContent>>) -> Self {
        self.justify_content = Some(value.into());
        self
    }

    pub fn v_align(mut self, value: impl Into<ViewProp<AlignItems>>) -> Self {
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
    pub fn v_align(mut self, value: impl Into<ViewProp<AlignItems>>) -> Self {
        self.align_items = Some(value.into());
        self
    }

    pub fn h_align(mut self, value: impl Into<ViewProp<taffy::AlignContent>>) -> Self {
        self.justify_content = Some(value.into());
        self
    }
}

impl<VS, const IS_ROW: bool> StyleExt for FlexContainer<VS, IS_ROW> {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.base
    }
}

impl<VS: ViewSequence, const IS_ROW: bool> View for FlexContainer<VS, IS_ROW> {
    type Element = ContainerWidget;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        Container {
            base: self.base,
            view_seq: self.view_seq,
            display_style: Cached::new(cx, move |cx, _| {
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
            .into(),
        }
        .build(cx)
    }
}

pub struct Grid<VS> {
    view_seq: VS,
    columns: ViewProp<Vec<taffy::TrackSizingFunction>>,
    rows: ViewProp<Vec<taffy::TrackSizingFunction>>,
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
    base: ViewStyle,
    view_seq: VS,
    display_style: ViewProp<DisplayStyle>,
}

impl<VS: ViewSequence> Container<VS> {
    pub fn new(view_seq: VS) -> Self {
        Self {
            base: ViewStyle::DEFAULT,
            view_seq,
            display_style: ViewProp::Const(DisplayStyle::Block),
        }
    }
}

impl<VS: ViewSequence> View for Container<VS> {
    type Element = ContainerWidget;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        cx.add_children(self.view_seq);
        cx.apply_style(self.base);
        let display_style = self.display_style.get_and_bind(cx, |value, mut widget| {
            widget.set_display_style(value);
        });
        cx.set_display_style(display_style);

        ContainerWidget {}
    }
}

impl<VS> StyleExt for Container<VS> {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.base
    }
}

pub struct ContainerWidget;

impl Widget for ContainerWidget {
    fn debug_label(&self) -> &'static str {
        "Container"
    }
}
