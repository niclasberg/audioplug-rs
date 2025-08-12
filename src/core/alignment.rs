use crate::core::{Rect, Vec2};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Align {
    pub fn compute_offset(&self, rect_to_align: Rect, bounds: Rect) -> Vec2 {
        let offset_x = match self.get_h_align() {
            HAlign::Left => bounds.left() - rect_to_align.left(),
            HAlign::Center => bounds.center().x - rect_to_align.center().x,
            HAlign::Right => bounds.right() - rect_to_align.right(),
        };
        let offset_y = match self.get_v_align() {
            VAlign::Top => bounds.top() - rect_to_align.top(),
            VAlign::Center => bounds.center().y - rect_to_align.center().y,
            VAlign::Bottom => bounds.bottom() - rect_to_align.bottom(),
        };
        Vec2::new(offset_x, offset_y)
    }

    pub fn get_v_align(&self) -> VAlign {
        match self {
            Align::TopLeft | Align::Top | Align::TopRight => VAlign::Top,
            Align::Left | Align::Center | Align::Right => VAlign::Center,
            Align::BottomLeft | Align::Bottom | Align::BottomRight => VAlign::Bottom,
        }
    }

    pub fn get_h_align(&self) -> HAlign {
        match self {
            Align::TopLeft | Align::Left | Align::BottomLeft => HAlign::Left,
            Align::Top | Align::Center | Align::Bottom => HAlign::Center,
            Align::Right | Align::TopRight | Align::BottomRight => HAlign::Right,
        }
    }
}

pub enum VAlign {
    Top,
    Center,
    Bottom,
}

pub enum HAlign {
    Left,
    Center,
    Right,
}
