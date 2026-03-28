//! Input conversion utilities

use crate::ui::components::InputEvent;
use crossterm::event::KeyCode;

/// Convert a KeyCode to an InputEvent
///
/// This function converts crossterm KeyCode events into the application's
/// InputEvent enum used by interactive components like InputField.
///
/// # Arguments
///
/// * `key` - The KeyCode to convert
///
/// # Returns
///
/// * `Some(InputEvent)` if the key can be converted to an input event
/// * `None` if the key is not handled by input components
///
/// # Examples
///
/// ```ignore
/// use crossterm::event::KeyCode;
///
/// let event = keycode_to_input_event(KeyCode::Char('a'));
/// assert!(event.is_some());
///
/// let event = keycode_to_input_event(KeyCode::F1);
/// assert!(event.is_none());
/// ```
pub fn keycode_to_input_event(key: KeyCode) -> Option<InputEvent> {
    match key {
        KeyCode::Char(c) => Some(InputEvent::Char(c)),
        KeyCode::Backspace => Some(InputEvent::Backspace),
        KeyCode::Delete => Some(InputEvent::Delete),
        KeyCode::Left => Some(InputEvent::Left),
        KeyCode::Right => Some(InputEvent::Right),
        KeyCode::Home => Some(InputEvent::Home),
        KeyCode::End => Some(InputEvent::End),
        KeyCode::Tab => Some(InputEvent::Tab),
        // BackTab doesn't exist in InputEvent, map to regular Tab
        KeyCode::BackTab => Some(InputEvent::Tab),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_conversion() {
        let event = keycode_to_input_event(KeyCode::Char('a'));
        assert!(matches!(event, Some(InputEvent::Char('a'))));
    }

    #[test]
    fn test_backspace_conversion() {
        let event = keycode_to_input_event(KeyCode::Backspace);
        assert!(matches!(event, Some(InputEvent::Backspace)));
    }

    #[test]
    fn test_navigation_keys() {
        assert!(matches!(
            keycode_to_input_event(KeyCode::Left),
            Some(InputEvent::Left)
        ));
        assert!(matches!(
            keycode_to_input_event(KeyCode::Right),
            Some(InputEvent::Right)
        ));
        assert!(matches!(
            keycode_to_input_event(KeyCode::Home),
            Some(InputEvent::Home)
        ));
        assert!(matches!(
            keycode_to_input_event(KeyCode::End),
            Some(InputEvent::End)
        ));
    }

    #[test]
    fn test_tab_keys() {
        assert!(matches!(
            keycode_to_input_event(KeyCode::Tab),
            Some(InputEvent::Tab)
        ));
        // BackTab maps to Tab
        assert!(matches!(
            keycode_to_input_event(KeyCode::BackTab),
            Some(InputEvent::Tab)
        ));
    }

    #[test]
    fn test_unhandled_keys() {
        assert!(keycode_to_input_event(KeyCode::F(1)).is_none());
        assert!(keycode_to_input_event(KeyCode::Enter).is_none());
        assert!(keycode_to_input_event(KeyCode::Esc).is_none());
        assert!(keycode_to_input_event(KeyCode::Up).is_none());
    }
}
