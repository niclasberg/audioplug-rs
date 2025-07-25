use crate::{
    ui::WidgetId,
    core::{Align, Vec2},
};

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum OverlayAnchor {
    #[default]
    Fixed,
    InsideParent,
    OutsideParent,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct OverlayOptions {
    pub anchor: OverlayAnchor,
    pub align: Align,
    pub offset: Vec2,
    pub z_index: usize,
}

impl OverlayOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

struct Overlay {
    widget_id: WidgetId,
    options: OverlayOptions,
    insertion_order: usize,
}

pub struct OverlayIter<'a> {
    inner: std::slice::Iter<'a, Overlay>,
}

impl Iterator for OverlayIter<'_> {
    type Item = WidgetId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| x.widget_id)
    }
}

impl DoubleEndedIterator for OverlayIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|x| x.widget_id)
    }
}

#[derive(Default)]
pub struct OverlayContainer {
    overlays: Vec<Overlay>,
    inserted_count: usize,
}

impl OverlayContainer {
    pub fn iter(&self) -> OverlayIter {
        OverlayIter {
            inner: self.overlays.iter(),
        }
    }

    pub fn insert_or_update(&mut self, widget_id: WidgetId, options: OverlayOptions) {
        let overlay = Overlay {
            widget_id,
            options,
            insertion_order: self.inserted_count,
        };
        self.inserted_count += 1;
        let insert_at = self
            .overlays
            .binary_search_by(|other| {
                other
                    .options
                    .z_index
                    .cmp(&overlay.options.z_index)
                    .then(other.insertion_order.cmp(&overlay.insertion_order))
            })
            .unwrap_or_else(|e| e);
        self.overlays.insert(insert_at, overlay);
    }

    pub fn remove(&mut self, widget_id: WidgetId) {
        self.overlays
            .retain(|overlay| overlay.widget_id != widget_id);
    }

    pub fn get_overlay_options(&self, index: usize) -> Option<OverlayOptions> {
        self.overlays.get(index).map(|o| o.options)
    }
}
