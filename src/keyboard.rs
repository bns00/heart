//! Interface to the keyboard.
//!
//! See also:  
//! [key pressed][crate::HeartBuilder::with_key_pressed]  
//! [key released][crate::HeartBuilder::with_key_released]  

pub(crate) mod state;

/// Represents the physical location of a key on a keyboard.
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Scancode {
    /// The `` `~ `` key on a US keyboard.
    Backquote,

    /// The `` \| `` key on a US keyboard.
    Backslash,

    /// The `` [{ `` key on a US keyboard.
    BracketLeft,

    /// The `` ]} `` key on a US keyboard.
    BracketRight,

    /// The `` ,< `` key on a US keyboard.
    Comma,

    /// The `` 0) `` key on a US keyboard.
    Digit0,

    /// The `` 1! `` key on a US keyboard.
    Digit1,

    /// The `` 2@ `` key on a US keyboard.
    Digit2,

    /// The `` 3# `` key on a US keyboard.
    Digit3,

    /// The `` 4$ `` key on a US keyboard.
    Digit4,

    /// The `` 5% `` key on a US keyboard.
    Digit5,

    /// The `` 6^ `` key on a US keyboard.
    Digit6,

    /// The `` 7& `` key on a US keyboard.
    Digit7,

    /// The `` 8* `` key on a US keyboard.
    Digit8,

    /// The `` 9( `` key on a US keyboard.
    Digit9,

    /// The `` =+ `` key on a US keyboard.
    Equal,

    /// The `` \| `` key on a UK keyboard.
    IntlBackslash,

    /// The `` \ろ `` key on a Japanese keyboard.
    IntlRo,

    /// The `` ¥ `` key on a Japanese keyboard.
    IntlYen,

    /// The `` a `` key on a US keyboard.
    KeyA,

    /// The `` b `` key on a US keyboard.
    KeyB,

    /// The `` c `` key on a US keyboard.
    KeyC,

    /// The `` d `` key on a US keyboard.
    KeyD,

    /// The `` e `` key on a US keyboard.
    KeyE,

    /// The `` f `` key on a US keyboard.
    KeyF,

    /// The `` g `` key on a US keyboard.
    KeyG,

    /// The `` h `` key on a US keyboard.
    KeyH,

    /// The `` i `` key on a US keyboard.
    KeyI,

    /// The `` j `` key on a US keyboard.
    KeyJ,

    /// The `` k `` key on a US keyboard.
    KeyK,

    /// The `` l `` key on a US keyboard.
    KeyL,

    /// The `` m `` key on a US keyboard.
    KeyM,

    /// The `` n `` key on a US keyboard.
    KeyN,

    /// The `` o `` key on a US keyboard.
    KeyO,

    /// The `` p `` key on a US keyboard.
    KeyP,

    /// The `` q `` key on a US keyboard.
    KeyQ,

    /// The `` r `` key on a US keyboard.
    KeyR,

    /// The `` s `` key on a US keyboard.
    KeyS,

    /// The `` t `` key on a US keyboard.
    KeyT,

    /// The `` u `` key on a US keyboard.
    KeyU,

    /// The `` v `` key on a US keyboard.
    KeyV,

    /// The `` w `` key on a US keyboard.
    KeyW,

    /// The `` x `` key on a US keyboard.
    KeyX,

    /// The `` y `` key on a US keyboard.
    KeyY,

    /// The `` z `` key on a US keyboard.
    KeyZ,

    /// The `` -_ `` key on a US keyboard.
    Minus,

    /// The `` .> `` key on a US keyboard.
    Period,

    /// The `` '" `` key on a US keyboard.
    Quote,

    /// The `` ;: `` key on a US keyboard.
    Semicolon,

    /// The `` /? `` key on a US keyboard.
    Slash,

    /// The left `` alt `` key.
    AltLeft,

    /// The right `` alt `` key.
    AltRight,

    /// The `` backspace `` key.
    Backspace,

    /// The `` capslock `` key.
    CapsLock,

    /// The application context menu key.
    ContextMenu,

    /// The left `` control `` key.
    ControlLeft,

    /// The right `` control `` key.
    ControlRight,

    /// The `` enter `` key.
    Enter,

    /// The left meta key.
    MetaLeft,

    /// The right meta key.
    MetaRight,

    /// The left `` shift `` key.
    ShiftLeft,

    /// The right `` shift `` key.
    ShiftRight,

    /// The space key.
    Space,

    /// The `` tab `` key.
    Tab,

    /// The `` 変換 `` key on a Japanese keyboard.
    Convert,

    /// The `` カタカナ/ひらがな/ローマ字 `` key on a Japanese keyboard.
    KanaMode,

    /// The `` 無変換 `` key on a Japenase keyboard.
    NonConvert,

    /// The `` delete `` key.
    Delete,

    /// The `` end `` key.
    End,

    /// The `` help `` key.
    Help,

    /// The `` home `` key.
    Home,

    /// The `` insert `` key.
    Insert,

    /// The `` page down `` key.
    PageDown,

    /// The `` page up `` key.
    PageUp,

    /// The `` ↓ `` key.
    ArrowDown,

    /// The `` ← `` key.
    ArrowLeft,

    /// The `` → `` key.
    ArrowRight,

    /// The `` ↑ `` key.
    ArrowUp,

    /// The `` numlock `` key.
    NumLock,

    /// The `` 0 `` numpad key.
    Numpad0,

    /// The `` 1 `` numpad key.
    Numpad1,

    /// The `` 2 `` numpad key.
    Numpad2,

    /// The `` 3 `` numpad key.
    Numpad3,

    /// The `` 4 `` numpad key.
    Numpad4,

    /// The `` 5 `` numpad key.
    Numpad5,

    /// The `` 6 `` numpad key.
    Numpad6,

    /// The `` 7 `` numpad key.
    Numpad7,

    /// The `` 8 `` numpad key.
    Numpad8,

    /// The `` 9 `` numpad key.
    Numpad9,

    /// The `` + `` numpad key.
    NumpadAdd,

    /// The `` . `` numpad key.
    NumpadDecimal,

    /// The `` / `` numpad key.
    NumpadDivide,

    /// The `` enter `` numpad key.
    NumpadEnter,

    ///  The `` * `` numpad key.
    NumpadMultiply,

    /// The `` - `` numpad key.
    NumpadSubtract,

    /// The `` esc `` key.
    Escape,

    /// The `` F1 `` key.
    F1,

    /// The `` F2 `` key.
    F2,

    /// The `` F3 `` key.
    F3,

    /// The `` F4 `` key.
    F4,

    /// The `` F5 `` key.
    F5,

    /// The `` F6 `` key.
    F6,

    /// The `` F7 `` key.
    F7,

    /// The `` F8 `` key.
    F8,

    /// The `` F9 `` key.
    F9,

    /// The `` F10 `` key.
    F10,

    /// The `` F11 `` key.
    F11,

    /// The `` F12 `` key.
    F12,

    /// The `` print screen `` key.
    PrintScreen,

    /// The `` scroll lock `` key.
    ScrollLock,

    /// The `` pause `` key.
    Pause,

    /// Any other key.
    Unidentified,
}

/// Check if a key is pressed.
pub fn is_pressed(scancode: Scancode) -> bool {
    state::get_key(scancode)
}
