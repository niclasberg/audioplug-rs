use std::path::Path;

use crate::{app::Widget, core::{Color, Size}, platform};

use super::View;

pub struct Image {
    source: Option<platform::ImageSource>
}

impl Image {
    pub fn from_file(path: &Path) -> Self {
        let source = platform::ImageSource::from_file(path).ok();
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
    source: Option<platform::ImageSource>
}

impl Widget for ImageWidget {
    fn debug_label(&self) -> &'static str {
        "Image"
    }

    fn style(&self) -> taffy::Style {
        taffy::Style::default()
    }

    fn measure(&self, _style: &taffy::Style, known_dimensions: taffy::Size<Option<f32>>, _available_space: taffy::Size<taffy::AvailableSpace>) -> taffy::Size<f32> {
        let image_size = self.source.as_ref().map(|source| source.size())
            .unwrap_or(Size::new(20.0, 20.0));

        match (known_dimensions.width, known_dimensions.height) {
            (Some(width), Some(height)) => taffy::Size { width, height },
            (Some(width), None) => {
                let height = if image_size.width > 1e-8 {
                    image_size.height * width as f64 / image_size.width
                } else {
                    0.0
                } as f32;
                taffy::Size { width, height }
            },
            (None, Some(height)) => {
                let width = if image_size.height > 1e-8 {
                    image_size.width * height as f64 / image_size.height
                } else {
                    0.0
                } as f32;
                taffy::Size { width, height }
            },
            (None, None) => { 
                taffy::Size { width: image_size.width as f32, height: image_size.height as f32 }
            }
        }
    }

    fn render(&mut self, ctx: &mut crate::app::RenderContext) {
        if let Some(source) = &self.source {
            ctx.draw_bitmap(source, ctx.content_bounds())
        } else {
            ctx.fill(ctx.content_bounds(), Color::RED)
        }
    }
}