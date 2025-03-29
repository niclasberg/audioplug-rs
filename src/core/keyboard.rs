use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Modifiers : u32 {
        const CONTROL = 1;
        const ALT = 1 << 1;
        const SHIFT = 1 << 2;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    Unknown,

    BackSpace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Escape,
    Tab,
    Enter,

    Left,
    Right,
    Up,
    Down,

    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,

    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Space,
    Period,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
}
