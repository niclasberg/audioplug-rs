use std::sync::atomic::{AtomicUsize, Ordering};

pub struct WidgetId(usize);

impl WidgetId {
    pub fn next() -> WidgetId {
        static WIDGET_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);
        let id = WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(id)
    }    
}

impl From<usize> for WidgetId {
    fn from(value: usize) -> Self {
        WidgetId(value)
    }
}