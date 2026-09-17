use crate::ui::{BuildContext, StyleExt, TextExt, View, ViewProp, ViewStyle, ViewText, Widget};

pub struct Label {
    style: ViewStyle,
    text: ViewText,
}

impl Label {
    pub fn new(str: impl Into<ViewProp<String>>) -> Self {
        Self {
            style: Default::default(),
            text: ViewText {
                text: Some(str.into()),
                ..Default::default()
            },
        }
    }
}

impl View for Label {
    type Element = TextWidget;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        ctx.apply_style(self.style);
        ctx.apply_text(self.text);
        TextWidget {}
    }
}

impl StyleExt for Label {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.style
    }
}

impl TextExt for Label {
    fn text_mut(&mut self) -> &mut ViewText {
        &mut self.text
    }
}

pub struct TextWidget {}

/*impl Measure for TextWidget {
    fn measure(&self, available_width: AvailableSpace, height: AvailableSpace) -> Size {
        let mut text_layout = self.text_layout.borrow_mut();
        let widths = text_layout.calculate_content_widths();

        let width = match available_width {
            AvailableSpace::MinContent => widths.min,
            AvailableSpace::MaxContent => widths.max,
            AvailableSpace::Exact(width) => width as f32,
        }
        .ceil();

        text_layout.break_all_lines(Some(width));
        let height = if let AvailableSpace::Exact(height) = height {
            height as f32
        } else {
            text_layout.height()
        };
        Size::new(width as _, height as _)
    }
}*/

impl Widget for TextWidget {
    fn debug_label(&self) -> &'static str {
        "Label"
    }

    /*fn render(&mut self, ctx: &mut RenderContext) -> Scene {
        let mut scene = Scene::new();
        let mut text_layout = self.text_layout.borrow_mut();
        let bounds = ctx.content_bounds();
        text_layout.break_all_lines(Some(bounds.width() as _));
        scene.draw_text(&text_layout, bounds.top_left());
        scene
    }*/
}
