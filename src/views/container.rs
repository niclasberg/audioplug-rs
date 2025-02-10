use taffy::{AlignContent, AlignItems, FlexDirection, FlexWrap};

use crate::{app::{Accessor, BuildContext, Memo, ViewSequence, Widget}, style::{DisplayStyle, FlexStyle, GridStyle, Length}};

use super::View;

pub type Row<VS> = FlexContainer<VS, true>;
pub type Column<VS> = FlexContainer<VS, false>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
	Start, 
	End,
	Center,
	Stretch
}

pub struct FlexContainer<VS, const IS_ROW: bool> {
    view_seq: VS,
    spacing: Accessor<Length>,
    wrap: Accessor<FlexWrap>,
	align_items: Accessor<Option<AlignItems>>,
	align_content: Accessor<Option<AlignContent>>
}

impl<VS: ViewSequence, const IS_ROW: bool> FlexContainer<VS, IS_ROW> {
    pub fn new(view_seq: VS) -> Self {
        Self {
            view_seq,
            spacing: Accessor::Const(Length::ZERO),
            wrap: Accessor::Const(Default::default()),
			align_items: Accessor::Const(None),
			align_content: Accessor::Const(None),
        }
    }

    pub fn spacing(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.spacing = value.into();
        self
    }
}

impl<VS: ViewSequence, const IS_ROW: bool> View for FlexContainer<VS, IS_ROW> {
    type Element = ContainerWidget;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
		Container {
			view_seq: self.view_seq,
			style: Memo::new(cx, move |cx, _| {
				ContainerStyle::Flex(FlexStyle { 
					direction: if IS_ROW { FlexDirection::Row } else { FlexDirection::Column }, 
					wrap: self.wrap.get(cx), 
					gap: self.spacing.get(cx),
					align_items: self.align_items.get(cx),
					align_content: self.align_content.get(cx),
				})
			}).into(),
		}.build(cx)
    }
}

pub struct Grid<VS> {
	view_seq: VS,
	columns: Accessor<Vec<taffy::TrackSizingFunction>>,
	rows: Accessor<Vec<taffy::TrackSizingFunction>>,
}

impl<VS: ViewSequence> Grid<VS> {
	pub fn new(f_rows: impl FnOnce(&mut GridStyleBuilder), f_cols: impl FnOnce(&mut GridStyleBuilder), view_seq: VS) -> Self {
		Self {
			view_seq,
			columns: Vec::new().into(),
			rows: Vec::new().into()
		}
	}
}

pub struct GridStyleBuilder {

}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerStyle {
	Block,
    Flex(FlexStyle),
	Grid(GridStyle)
}

pub struct Container<VS> {
	view_seq: VS,
	style: Accessor<ContainerStyle>
}

impl<VS: ViewSequence> Container<VS> {
	pub fn new(view_seq: VS) -> Self {
		Self {
			view_seq,
			style: Accessor::Const(ContainerStyle::Block)
		}
	}
}

impl<VS: ViewSequence> View for Container<VS> {
	type Element = ContainerWidget;

	fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
		self.view_seq.build_seq(cx);
		ContainerWidget {
			container_style: self.style.get_and_bind(cx, |value, mut widget| {
				widget.container_style = value;
			})
		}
	}
}

pub struct ContainerWidget {
	container_style: ContainerStyle
}

impl Widget for ContainerWidget {
	fn debug_label(&self) -> &'static str {
		"Container"
	}

	fn render(&mut self, ctx: &mut crate::app::RenderContext) {
		ctx.render_children()
	}

	fn display_style(&self) -> DisplayStyle {
		match &self.container_style {
			ContainerStyle::Block => DisplayStyle::Block,
			ContainerStyle::Flex(flex_style) => DisplayStyle::Flex(flex_style),
			ContainerStyle::Grid(grid_style) => DisplayStyle::Grid(grid_style),
		}
	}
}