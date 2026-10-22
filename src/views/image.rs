use std::path::Path;

use crate::{
    core::{Color, ImageData, Size},
    ui::{
        BuildContext, StyleExt, View, ViewExt, WidgetProps,
        style::{AvailableSpace, Measure},
    },
};

pub struct Image {
    props: WidgetProps,
    data: Option<ImageData>,
}

impl Image {
    pub fn from_file(path: &Path) -> Self {
        let data = ImageData::from_file(path).ok();

        Self {
            data,
            props: Default::default(),
        }
    }
}

impl View for Image {
    fn build(self, cx: &mut BuildContext) {
        cx.props(self.props).render(move |cx| {
            if let Some(source) = &self.data {
                cx.draw_bitmap(source, cx.bounds())
            } else {
                cx.fill(cx.bounds(), Color::RED)
            }
        });
    }
}
impl ViewExt for Image {
    fn props_mut(&mut self) -> &mut WidgetProps {
        &mut self.props
    }
}
impl StyleExt for Image {}

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
