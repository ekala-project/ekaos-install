//! Selection list component
//!
//! Single-select list with keyboard navigation

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use super::{Component, Focusable, InputEvent, Interactive};

/// Single-select list component
pub struct SelectList<T> {
    /// List title
    title: String,
    /// List items
    items: Vec<SelectItem<T>>,
    /// Currently selected index
    selected: usize,
    /// Whether the component is focused
    focused: bool,
    /// Whether the component is enabled
    enabled: bool,
    /// Show descriptions
    show_descriptions: bool,
}

/// An item in the selection list
#[derive(Clone)]
pub struct SelectItem<T> {
    /// Display label
    pub label: String,
    /// Optional description
    pub description: Option<String>,
    /// Associated value
    pub value: T,
    /// Whether this item is enabled
    pub enabled: bool,
}

impl<T> SelectItem<T> {
    /// Create a new select item
    pub fn new(label: impl Into<String>, value: T) -> Self {
        Self {
            label: label.into(),
            description: None,
            value,
            enabled: true,
        }
    }

    /// Add a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl<T> SelectList<T> {
    /// Create a new selection list
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
            selected: 0,
            focused: false,
            enabled: true,
            show_descriptions: false,
        }
    }

    /// Add an item to the list
    pub fn add_item(mut self, item: SelectItem<T>) -> Self {
        self.items.push(item);
        self
    }

    /// Add multiple items
    pub fn with_items(mut self, items: Vec<SelectItem<T>>) -> Self {
        self.items = items;
        self
    }

    /// Show descriptions for items
    pub fn show_descriptions(mut self, show: bool) -> Self {
        self.show_descriptions = show;
        self
    }

    /// Get the currently selected item
    pub fn selected(&self) -> Option<&SelectItem<T>> {
        self.items.get(self.selected)
    }

    /// Get the selected index
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Set the selected index
    pub fn set_selected(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = index;
        }
    }

    /// Move selection up
    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            // Skip disabled items
            while self.selected > 0 && !self.items[self.selected].enabled {
                self.selected -= 1;
            }
        }
    }

    /// Move selection down
    fn move_down(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
            // Skip disabled items
            while self.selected < self.items.len() - 1 && !self.items[self.selected].enabled {
                self.selected += 1;
            }
        }
    }

    /// Get number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if list is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T> Focusable for SelectList<T> {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl<T> Interactive for SelectList<T> {
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
            InputEvent::Enter => true,
            _ => false,
        }
    }
}

impl<T> Component for SelectList<T> {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let border_color = if !self.enabled {
            Color::DarkGray
        } else if self.focused {
            Color::Cyan
        } else {
            Color::White
        };

        let border_style = if self.focused {
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(border_color)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.as_str())
            .border_style(border_style);

        let items: Vec<ListItem<'_>> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == self.selected;

                let mut spans = Vec::new();

                // Selection indicator
                if is_selected {
                    spans.push(Span::styled(
                        "▶ ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::raw("  "));
                }

                // Label
                let label_style = if !item.enabled {
                    Style::default().fg(Color::DarkGray)
                } else if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                spans.push(Span::styled(item.label.clone(), label_style));

                let mut lines = vec![Line::from(spans)];

                // Description if shown and available
                if self.show_descriptions {
                    if let Some(desc) = &item.description {
                        lines.push(Line::from(Span::styled(
                            format!("    {}", desc),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }

                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items).block(block);

        frame.render_widget(list, area);
    }

    fn title(&self) -> Option<&str> {
        Some(&self.title)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_list_creation() {
        let list: SelectList<i32> = SelectList::new("Choose Option");
        assert_eq!(list.title, "Choose Option");
        assert!(list.is_empty());
        assert_eq!(list.selected_index(), 0);
    }

    #[test]
    fn test_select_list_with_items() {
        let list: SelectList<i32> = SelectList::new("Choose")
            .add_item(SelectItem::new("Option 1", 1))
            .add_item(SelectItem::new("Option 2", 2))
            .add_item(SelectItem::new("Option 3", 3));

        assert_eq!(list.len(), 3);
        assert_eq!(list.selected().unwrap().label, "Option 1");
    }

    #[test]
    fn test_select_navigation() {
        let mut list: SelectList<i32> = SelectList::new("Choose")
            .add_item(SelectItem::new("Option 1", 1))
            .add_item(SelectItem::new("Option 2", 2))
            .add_item(SelectItem::new("Option 3", 3));

        assert_eq!(list.selected_index(), 0);

        list.handle_input(InputEvent::Down);
        assert_eq!(list.selected_index(), 1);

        list.handle_input(InputEvent::Down);
        assert_eq!(list.selected_index(), 2);

        // Should not go beyond last item
        list.handle_input(InputEvent::Down);
        assert_eq!(list.selected_index(), 2);

        list.handle_input(InputEvent::Up);
        assert_eq!(list.selected_index(), 1);
    }

    #[test]
    fn test_select_disabled_items() {
        let mut list: SelectList<i32> = SelectList::new("Choose")
            .add_item(SelectItem::new("Option 1", 1))
            .add_item(SelectItem::new("Option 2", 2).with_enabled(false))
            .add_item(SelectItem::new("Option 3", 3));

        list.handle_input(InputEvent::Down);
        // Should skip disabled option 2
        assert_eq!(list.selected_index(), 2);
    }
}
