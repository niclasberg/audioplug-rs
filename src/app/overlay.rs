use slotmap::Key;

use crate::app::WidgetId;

struct Overlay {
    widget_id: WidgetId,
    z_index: usize,
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

    pub fn add(&mut self, widget_id: WidgetId, z_index: usize) {
        let overlay = Overlay {
            widget_id,
            z_index,
            insertion_order: self.inserted_count,
        };
        self.inserted_count += 1;
        let insert_at = self
            .overlays
            .binary_search_by(|other| {
                overlay
                    .z_index
                    .cmp(&other.z_index)
                    .then(overlay.insertion_order.cmp(&other.insertion_order))
            })
            .unwrap_or_else(|e| e);
        self.overlays.insert(insert_at, overlay);
    }

    pub fn remove(&mut self, widget_id: WidgetId) {
        self.overlays
            .retain(|overlay| overlay.widget_id != widget_id);
    }
}
