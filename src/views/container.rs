use crate::{app::{Accessor, BuildContext, Memo, ViewSequence, Widget}, style::{DisplayStyle, FlexStyle, GridStyle, Length, FlexWrap, AlignItems, FlexDirection, JustifyContent}};

use super::View;

pub type Row<VS> = FlexContainer<VS, true>;
pub type Column<VS> = FlexContainer<VS, false>;

pub struct FlexContainer<VS, const IS_ROW: bool> {
    view_seq: VS,
    spacing: Accessor<Length>,
    wrap: Accessor<FlexWrap>,
	align_items: Option<Accessor<AlignItems>>,
	justify_content: Option<Accessor<JustifyContent>>
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

    pub fn spacing(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.spacing = value.into();
        self
    }
}

impl<VS> Row<VS> {
	pub fn v_align(mut self, value: impl Into<Accessor<taffy::JustifyContent>>) -> Self {
		self.justify_content = Some(value.into());
		self
	}

	pub fn h_align(mut self, value: impl Into<Accessor<taffy::AlignItems>>) -> Self {
		self.align_items = Some(value.into());
		self
	}

	pub fn v_align_top(self) -> Self {
		self.v_align(taffy::JustifyContent::Start)
	}

	pub fn v_align_center(self) -> Self {
		self.v_align(taffy::JustifyContent::Center)
	}

	pub fn v_align_bottom(self) -> Self {
		self.v_align(taffy::JustifyContent::End)
	}

	pub fn v_align_space_around(self) -> Self {
		self.v_align(taffy::JustifyContent::SpaceAround)
	}

	pub fn v_align_space_between(self) -> Self {
		self.v_align(taffy::JustifyContent::SpaceBetween)
	}

	pub fn v_align_space_evenly(self) -> Self {
		self.v_align(taffy::JustifyContent::SpaceEvenly)
	}

	pub fn h_align_center(self) -> Self {
		self.h_align(taffy::AlignItems::Center)
	}
}

impl<VS> Column<VS> {
	pub fn v_align(mut self, value: impl Into<Accessor<taffy::AlignItems>>) -> Self {
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
			style: Memo::new(cx, move |cx, _| {
				ContainerStyle::Flex(FlexStyle { 
					direction: if IS_ROW { FlexDirection::Row } else { FlexDirection::Column }, 
					wrap: self.wrap.get(cx), 
					gap: self.spacing.get(cx),
					align_items: self.align_items.as_ref().map(|x| x.get(cx)),
					align_content: self.justify_content.as_ref().map(|x| x.get(cx)),
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