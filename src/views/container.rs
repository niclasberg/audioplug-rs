use crate::ui::{
    Accessor, BuildContext, Cached, ReactiveValue, View, ViewSequence, Widget,
    style::{
        AlignItems, FlexDirection, FlexStyle, FlexWrap, GridStyle, JustifyContent, LayoutMode,
        Length,
    },
};

pub type Row<VS> = FlexContainer<VS, true>;
pub type Column<VS> = FlexContainer<VS, false>;

pub struct FlexContainer<VS, const IS_ROW: bool> {
    view_seq: VS,
    spacing: Accessor<Length>,
    wrap: Accessor<FlexWrap>,
    align_items: Option<Accessor<AlignItems>>,
    justify_content: Option<Accessor<JustifyContent>>,
}

impl<VS: ViewSequence, const IS_ROW: bool> FlexContainer<VS, IS_ROW> {
    pub fn new(view_seq: VS) -> Self {
        Self {
            view_seq,
            spacing: Accessor::Const(Length::ZERO),
            wrap: Accessor::Const(Default::default()),
            align_items: None,
            justify_content: None,
        }
    }

    pub fn wrapping(mut self, value: impl Into<Accessor<FlexWrap>>) -> Self {
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

    pub fn spacing(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.spacing = value.into();
        self
    }

    pub fn center(mut self) -> Self {
        self.justify_content = Some(JustifyContent::Center.into());
        self.align_items = Some(AlignItems::Center.into());
        self
    }
}

impl<VS> Row<VS> {
    pub fn h_align(mut self, value: impl Into<Accessor<JustifyContent>>) -> Self {
        self.justify_content = Some(value.into());
        self
    }

    pub fn v_align(mut self, value: impl Into<Accessor<AlignItems>>) -> Self {
        self.align_items = Some(value.into());
        self
    }

    pub fn h_align_top(self) -> Self {
        self.h_align(taffy::JustifyContent::Start)
    }

    pub fn h_align_center(self) -> Self {
        self.h_align(taffy::JustifyContent::Center)
    }

    pub fn h_align_bottom(self) -> Self {
        self.h_align(taffy::JustifyContent::End)
    }

    pub fn h_align_space_around(self) -> Self {
        self.h_align(taffy::JustifyContent::SpaceAround)
    }

    pub fn h_align_space_between(self) -> Self {
        self.h_align(taffy::JustifyContent::SpaceBetween)
    }

    pub fn h_align_space_evenly(self) -> Self {
        self.h_align(taffy::JustifyContent::SpaceEvenly)
    }

    pub fn v_align_center(self) -> Self {
        self.v_align(taffy::AlignItems::Center)
    }
}

impl<VS> Column<VS> {
    pub fn v_align(mut self, value: impl Into<Accessor<AlignItems>>) -> Self {
        self.align_items = Some(value.into());
        self
    }

    pub fn h_align(mut self, value: impl Into<Accessor<taffy::AlignContent>>) -> Self {
        self.justify_content = Some(value.into());
        self
    }
}

impl<VS: ViewSequence, const IS_ROW: bool> View for FlexContainer<VS, IS_ROW> {
    type Element = ContainerWidget;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        Container {
            view_seq: self.view_seq,
            style: Cached::new(cx, move |cx, _| {
                ContainerStyle::Flex(FlexStyle {
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
    columns: Accessor<Vec<taffy::TrackSizingFunction>>,
    rows: Accessor<Vec<taffy::TrackSizingFunction>>,
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

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerStyle {
    Block,
    Stack,
    Flex(FlexStyle),
    Grid(GridStyle),
}

pub struct Container<VS> {
    view_seq: VS,
    style: Accessor<ContainerStyle>,
}

impl<VS: ViewSequence> Container<VS> {
    pub fn new(view_seq: VS) -> Self {
        Self {
            view_seq,
            style: Accessor::Const(ContainerStyle::Block),
        }
    }
}

impl<VS: ViewSequence> View for Container<VS> {
    type Element = ContainerWidget;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        cx.add_children(self.view_seq);
        ContainerWidget {
            container_style: self.style.get_and_bind(cx, |value, mut widget| {
                widget.container_style = value;
            }),
        }
    }
}

pub struct ContainerWidget {
    container_style: ContainerStyle,
}

impl Widget for ContainerWidget {
    fn debug_label(&self) -> &'static str {
        "Container"
    }

    fn layout_mode(&self) -> LayoutMode<'_> {
        match &self.container_style {
            ContainerStyle::Block => LayoutMode::Block,
            ContainerStyle::Stack => LayoutMode::Stack,
            ContainerStyle::Flex(flex_style) => LayoutMode::Flex(flex_style),
            ContainerStyle::Grid(grid_style) => LayoutMode::Grid(grid_style),
        }
    }
}
