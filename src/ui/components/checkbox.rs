//! Checkbox list component
//!
//! Multi-select checkbox list with keyboard navigation

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use super::{Component, Focusable, InputEvent, Interactive};

/// Multi-select checkbox list
pub struct CheckboxList {
    /// List title
    title: String,
    /// List items
    items: Vec<CheckboxItem>,
    /// Currently focused index
    focused_index: usize,
    /// Whether the component is focused
    focused: bool,
    /// Whether the component is enabled
    enabled: bool,
}

/// A checkbox item
#[derive(Clone)]
pub struct CheckboxItem {
    /// Display label
    pub label: String,
    /// Optional description
    pub description: Option<String>,
    /// Whether this item is checked
    pub checked: bool,
    /// Whether this item is enabled
    pub enabled: bool,
}

impl CheckboxItem {
    /// Create a new checkbox item
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
            checked: false,
            enabled: true,
        }
    }

    /// Set checked state
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Add description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl CheckboxList {
    /// Create a new checkbox list
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
            focused_index: 0,
            focused: false,
            enabled: true,
        }
    }

    /// Add an item
    pub fn add_item(mut self, item: CheckboxItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add multiple items
    pub fn with_items(mut self, items: Vec<CheckboxItem>) -> Self {
        self.items = items;
        self
    }

    /// Get checked items
    pub fn checked_items(&self) -> Vec<&CheckboxItem> {
        self.items.iter().filter(|item| item.checked).collect()
    }

    /// Toggle current item
    fn toggle_current(&mut self) {
        if let Some(item) = self.items.get_mut(self.focused_index) {
            if item.enabled {
                item.checked = !item.checked;
            }
        }
    }

    /// Move focus up
    fn move_up(&mut self) {
        if self.focused_index > 0 {
            self.focused_index -= 1;
        }
    }

    /// Move focus down
    fn move_down(&mut self) {
        if self.focused_index < self.items.len().saturating_sub(1) {
            self.focused_index += 1;
        }
    }
}

impl Focusable for CheckboxList {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Interactive for CheckboxList {
    fn handle_input(&mut self, event: InputEvent) -> bool {
        if !self.enabled {
            return false;
        }

        match event {
            InputEvent::Up => {
                self.move_up();
                true
            }
            InputEvent::Down => {
                self.move_down();
                true
            }
            InputEvent::Char(' ') | InputEvent::Enter => {
                self.toggle_current();
                true
            }
            _ => false,
        }
    }
}

impl Component for CheckboxList {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let border_color = if self.focused {
            Color::Cyan
        } else {
            Color::White
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.as_str())
            .border_style(Style::default().fg(border_color));

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_focused = i == self.focused_index && self.focused;

                let checkbox = if item.checked { "[✓]" } else { "[ ]" };

                let style = if is_focused {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else if !item.enabled {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };

                let line = Line::from(vec![
                    Span::styled(checkbox, style),
                    Span::raw(" "),
                    Span::styled(item.label.clone(), style),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_list() {
        let list = CheckboxList::new("Select Options")
            .add_item(CheckboxItem::new("Option 1"))
            .add_item(CheckboxItem::new("Option 2").checked(true));

        assert_eq!(list.checked_items().len(), 1);
    }

    #[test]
    fn test_checkbox_toggle() {
        let mut list = CheckboxList::new("Test").add_item(CheckboxItem::new("Item 1"));

        assert!(!list.items[0].checked);

        list.handle_input(InputEvent::Enter);
        assert!(list.items[0].checked);

        list.handle_input(InputEvent::Enter);
        assert!(!list.items[0].checked);
    }
}
