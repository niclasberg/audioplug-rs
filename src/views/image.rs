use std::path::Path;

use crate::{app::Widget, core::{Color, Size}, platform, style::{AvailableSpace, DisplayStyle, Measure}};

use super::View;

pub struct Image {
    source: Option<platform::NativeImage>
}

impl Image {
    pub fn from_file(path: &Path) -> Self {
        let source = platform::NativeImage::from_file(path).ok();
        Self {
            source
        }
    }
}

impl View for Image {
    type Element = ImageWidget;

    fn build(self, _ctx: &mut crate::app::BuildContext<Self::Element>) -> Self::Element {
        ImageWidget {
            source: self.source
        }
    }
}

pub struct ImageWidget {
    source: Option<platform::NativeImage>
}

impl Measure for ImageWidget {
    fn measure(&self, _style: &crate::style::Style, width: AvailableSpace, height: AvailableSpace) -> Size 
    {
        let image_size = self.source.as_ref().map(|source| source.size())
            .unwrap_or(Size::new(20.0, 20.0));

        match (width, height) {
            (AvailableSpace::Exact(width), AvailableSpace::Exact(height)) => Size::new(width, height),
            (AvailableSpace::Exact(width), _) => {
                let height = if image_size.width > 1e-8 {
                    image_size.height * width / image_size.width
                } else {
                    0.0
                };
                Size::new(width, height)
            },
            (_, AvailableSpace::Exact(height)) => {
                let width = if image_size.height > 1e-8 {
                    image_size.width * height / image_size.height
                } else {
                    0.0
                };
                Size::new(width, height)
            },
            (_, _) => { 
                image_size
            }
        }
    }
}

impl Widget for ImageWidget {
    fn debug_label(&self) -> &'static str {
        "Image"
    }

    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Leaf(self)
    }

    fn render(&mut self, ctx: &mut crate::app::RenderContext) {
        if let Some(source) = &self.source {
            ctx.draw_bitmap(source, ctx.content_bounds())
        } else {
            ctx.fill(ctx.content_bounds(), Color::RED)
        }
    }
}