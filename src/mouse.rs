//! Interface to the mouse.
//!
//! See also:  
//! [mouse pressed][crate::HeartBuilder::with_mouse_pressed]  
//! [mouse released][crate::HeartBuilder::with_mouse_released]  
//! [mouse moved][crate::HeartBuilder::with_mouse_moved]  

pub(crate) mod state;

/// Represents a button on a mouse.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Left,
    Right,
    Middle,
}

/// Check if a button is pressed.
pub fn is_pressed(button: Button) -> bool {
    state::get_button(button)
}

/// Get the x and y coordinates of the mouse.
pub fn get_position() -> (f32, f32) {
    state::get_position()
}
