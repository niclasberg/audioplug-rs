use std::path::Path;

use crate::{
    core::{Color, ImageData, Size, Zero},
    ui::{
        RenderContext, Scene, StyleExt, View, ViewStyle, Widget,
        style::{AvailableSpace, LayoutMode, Length, Measure, Style},
    },
};

pub struct Image {
    style: ViewStyle,
    data: Option<ImageData>,
}

impl Image {
    pub fn from_file(path: &Path) -> Self {
        let data = ImageData::from_file(path).ok();

        Self {
            data,
            style: ViewStyle::default(),
        }
    }
}

impl View for Image {
    type Element = ImageWidget;

    fn build(self, cx: &mut crate::ui::BuildContext<Self::Element>) -> Self::Element {
        cx.apply_style(self.style);
        ImageWidget { source: self.data }
    }
}

impl StyleExt for Image {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.style
    }
}

pub struct ImageWidget {
    source: Option<ImageData>,
}

impl Measure for ImageWidget {
    fn measure(&self, width: AvailableSpace, height: AvailableSpace) -> Size {
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

    fn render(&mut self, ctx: &mut RenderContext) -> Scene {
        let mut scene = Scene::new();
        if let Some(source) = &self.source {
            scene.draw_bitmap(source, ctx.content_bounds())
        } else {
            scene.fill(ctx.content_bounds(), Color::RED)
        }
        scene
    }
}
