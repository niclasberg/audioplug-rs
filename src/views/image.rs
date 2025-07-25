use std::path::Path;

use crate::{
    core::{Color, Size},
    platform,
    ui::{
        View, Widget,
        style::{AvailableSpace, LayoutMode, Measure, Style},
    },
};

pub struct Image {
    source: Option<platform::Bitmap>,
}

impl Image {
    pub fn from_file(path: &Path) -> Self {
        let source = platform::Bitmap::from_file(path).ok();
        Self { source }
    }
}

impl View for Image {
    type Element = ImageWidget;

    fn build(self, _cx: &mut crate::ui::BuildContext<Self::Element>) -> Self::Element {
        ImageWidget {
            source: self.source,
        }
    }
}

pub struct ImageWidget {
    source: Option<platform::Bitmap>,
}

impl Measure for ImageWidget {
    fn measure(&self, _style: &Style, width: AvailableSpace, height: AvailableSpace) -> Size {
        let image_size = self
            .source
            .as_ref()
            .map(|source| source.size())
            .unwrap_or(Size::new(20.0, 20.0));

        match (width, height) {
            (AvailableSpace::Exact(width), AvailableSpace::Exact(height)) => {
                Size::new(width, height)
            }
            (AvailableSpace::Exact(width), _) => {
                let height = if image_size.width > 1e-8 {
                    image_size.height * width / image_size.width
                } else {
                    0.0
                };
                Size::new(width, height)
            }
            (_, AvailableSpace::Exact(height)) => {
                let width = if image_size.height > 1e-8 {
                    image_size.width * height / image_size.height
                } else {
                    0.0
                };
                Size::new(width, height)
            }
            (_, _) => image_size,
        }
    }
}

impl Widget for ImageWidget {
    fn debug_label(&self) -> &'static str {
        "Image"
    }

    fn layout_mode(&self) -> LayoutMode {
        LayoutMode::Leaf(self)
    }

    fn render(&mut self, ctx: &mut crate::ui::RenderContext) {
        if let Some(source) = &self.source {
            ctx.draw_bitmap(source, ctx.content_bounds())
        } else {
            ctx.fill(ctx.content_bounds(), Color::RED)
        }
    }
}
