use std::ops::Range;

use super::{CreateContext, Memo, ReadContext, Readable};

pub fn signal_range<Idx: 'static + PartialEq>(
    cx: &mut impl CreateContext,
    f_start: impl Fn(&mut dyn ReadContext) -> Idx + 'static,
    f_end: impl Fn(&mut dyn ReadContext) -> Idx + 'static,
) -> impl Readable<Value = Range<Idx>> {
    Memo::new(cx, move |cx, _| {
        let start = f_start(cx);
        let end = f_end(cx);
        start..end
    })
}
