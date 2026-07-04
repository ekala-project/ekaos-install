//! Confirmation dialog component

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::{Component, Focusable, InputEvent, Interactive};

/// Confirmation dialog with Yes/No buttons
pub struct ConfirmDialog {
    /// Dialog title
    title: String,
    /// Dialog message
    message: String,
    /// Currently focused button (0=Yes, 1=No)
    focused_button: usize,
    /// Whether the dialog is focused
    focused: bool,
    /// Dialog type (affects color)
    dialog_type: DialogType,
}

/// Dialog type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    /// Normal confirmation (blue)
    Confirm,
    /// Warning confirmation (yellow)
    Warning,
    /// Danger confirmation (red)
    Danger,
}

impl DialogType {
    /// Get the color for this dialog type
    pub fn color(&self) -> Color {
        match self {
            DialogType::Confirm => Color::Blue,
            DialogType::Warning => Color::Yellow,
            DialogType::Danger => Color::Red,
        }
    }
}

impl ConfirmDialog {
    /// Create a new confirmation dialog
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            focused_button: 1, // Default to "No"
            focused: true,
            dialog_type: DialogType::Confirm,
        }
    }

    /// Set dialog type
    pub fn with_type(mut self, dialog_type: DialogType) -> Self {
        self.dialog_type = dialog_type;
        self
    }

    /// Create a warning dialog
    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message).with_type(DialogType::Warning)
    }

    /// Create a danger dialog
    pub fn danger(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message).with_type(DialogType::Danger)
    }

    /// Get whether "Yes" is currently focused
    pub fn is_yes_focused(&self) -> bool {
        self.focused_button == 0
    }

    /// Center a rect within another rect
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

impl Focusable for ConfirmDialog {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Interactive for ConfirmDialog {
    fn handle_input(&mut self, event: InputEvent) -> bool {
        match event {
            InputEvent::Left | InputEvent::Right | InputEvent::Tab => {
                self.focused_button = 1 - self.focused_button;
                true
            }
            InputEvent::Enter => true,
            InputEvent::Escape => {
                self.focused_button = 1; // Select "No" on escape
                true
            }
            _ => false,
        }
    }
}

impl Component for ConfirmDialog {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let color = self.dialog_type.color();

        // Create centered popup
        let popup_area = Self::centered_rect(60, 30, area);

        // Clear the area
        frame.render_widget(Clear, popup_area);

        // Main dialog block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .title(self.title.as_str());

        frame.render_widget(block, popup_area);

        // Split into message and buttons
        let inner = popup_area.inner(&ratatui::layout::Margin {
            vertical: 1,
            horizontal: 1,
        });

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(inner);

        // Render message
        let message = Paragraph::new(self.message.as_str())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        frame.render_widget(message, chunks[0]);

        // Render buttons
        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Yes button
        let yes_style = if self.focused_button == 0 {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };

        let yes_button = Paragraph::new("[ Yes ]")
            .alignment(Alignment::Center)
            .style(yes_style);
        frame.render_widget(yes_button, button_chunks[0]);

        // No button
        let no_style = if self.focused_button == 1 {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };

        let no_button = Paragraph::new("[ No ]")
            .alignment(Alignment::Center)
            .style(no_style);
        frame.render_widget(no_button, button_chunks[1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_dialog() {
        let dialog = ConfirmDialog::new("Confirm", "Are you sure?");
        assert_eq!(dialog.focused_button, 1); // Defaults to No
        assert_eq!(dialog.dialog_type, DialogType::Confirm);
    }

    #[test]
    fn test_dialog_navigation() {
        let mut dialog = ConfirmDialog::new("Test", "Message");

        assert_eq!(dialog.focused_button, 1);

        dialog.handle_input(InputEvent::Left);
        assert_eq!(dialog.focused_button, 0);

        dialog.handle_input(InputEvent::Right);
        assert_eq!(dialog.focused_button, 1);
    }

    #[test]
    fn test_dialog_types() {
        let warning = ConfirmDialog::warning("Warning", "Be careful");
        assert_eq!(warning.dialog_type, DialogType::Warning);

        let danger = ConfirmDialog::danger("Danger", "This is dangerous");
        assert_eq!(danger.dialog_type, DialogType::Danger);
    }
}
