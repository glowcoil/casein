#![allow(dead_code)]

#[derive(Copy, Clone)]
pub enum Input {
    MouseMove,
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    Scroll(f32, f32),
    KeyDown(Key),
    KeyUp(Key),
    Char(char),
}

#[derive(Copy, Clone)]
pub struct InputState {
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub modifiers: Modifiers,
}

impl Default for InputState {
    fn default() -> InputState {
        InputState {
           mouse_x: 0.0,
           mouse_y: 0.0,
           modifiers: Modifiers::default(),
       }
    }
}

#[derive(Copy, Clone)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl Default for Modifiers {
    fn default() -> Modifiers {
        Modifiers {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Key {
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
    GraveAccent,
    Minus,
    Equals,
    LeftBracket,
    RightBracket,
    Backslash,
    Semicolon,
    Apostrophe,
    Comma,
    Period,
    Slash,
    Escape,
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
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    PrintScreen,
    ScrollLock,
    Pause,
    Backspace,
    Tab,
    CapsLock,
    Enter,
    Space,
    Insert,
    Delete,
    PageUp,
    PageDown,
    Home,
    End,
    Left,
    Right,
    Up,
    Down,
    NumLock,
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
    NumpadDecimal,
    NumpadDivide,
    NumpadMultiply,
    NumpadSubtract,
    NumpadAdd,
    NumpadEnter,
    NumpadEquals,
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftMeta,
    RightShift,
    RightControl,
    RightAlt,
    RightMeta,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}
