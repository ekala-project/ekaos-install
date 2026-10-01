//! Checkbox list component
//!
//! Multi-select checkbox list. Pure data struct with navigation/toggle
//! methods and a `view()` method for rendering.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, IntoElement, SharedString};

use crate::ui::theme::{icons, spacing, AppTheme};

/// Multi-select checkbox list.
///
/// Pure data struct; the parent owns focus and routes keyboard events
/// to `toggle_current()`, `move_up()`, and `move_down()`.
pub struct CheckboxList {
    /// List title
    title: String,
    /// List items
    items: Vec<CheckboxItem>,
    /// Currently focused index
    focused_index: usize,
}

/// A checkbox item.
#[derive(Clone)]
pub struct CheckboxItem {
    /// Display label
    pub label: String,
    /// Optional description
    pub description: Option<String>,
    /// Whether this item is checked
    pub checked: bool,
    /// Whether this item is enabled (can be toggled)
    pub enabled: bool,
}

impl CheckboxItem {
    /// Create a new checkbox item.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
            checked: false,
            enabled: true,
        }
    }

    /// Set checked state.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Add a description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl CheckboxList {
    /// Create a new checkbox list.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
            focused_index: 0,
        }
    }

    /// Add an item.
    pub fn add_item(mut self, item: CheckboxItem) -> Self {
        self.items.push(item);
        self
    }

    /// Set items from a vector.
    pub fn with_items(mut self, items: Vec<CheckboxItem>) -> Self {
        self.items = items;
        self
    }

    /// Get all checked items.
    pub fn checked_items(&self) -> Vec<&CheckboxItem> {
        self.items.iter().filter(|item| item.checked).collect()
    }

    /// Toggle the currently focused item.
    pub fn toggle_current(&mut self) {
        if let Some(item) = self.items.get_mut(self.focused_index) {
            if item.enabled {
                item.checked = !item.checked;
            }
        }
    }

    /// Move focus up.
    pub fn move_up(&mut self) {
        if self.focused_index > 0 {
            self.focused_index -= 1;
        }
    }

    /// Move focus down.
    pub fn move_down(&mut self) {
        if self.focused_index < self.items.len().saturating_sub(1) {
            self.focused_index += 1;
        }
    }

    /// Get the focused index.
    pub fn focused_index(&self) -> usize {
        self.focused_index
    }

    /// Get the number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Build a gpui element tree for this list.
    ///
    /// `focused` indicates whether the parent considers this list focused.
    pub fn view(&self, focused: bool) -> impl IntoElement {
        let theme = AppTheme::new();

        let border_color = if focused { theme.primary } else { theme.border };

        let title: SharedString = self.title.clone().into();

        let mut container = div()
            .w_full()
            .border_1()
            .border_color(border_color)
            .rounded(px(4.0))
            .flex()
            .flex_col()
            .overflow_hidden()
            // Title
            .child(
                div()
                    .px(px(spacing::MEDIUM))
                    .py(px(spacing::SMALL))
                    .border_b_1()
                    .border_color(border_color)
                    .text_color(if focused {
                        theme.primary
                    } else {
                        theme.foreground
                    })
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .child(title),
            );

        // Item rows
        for (i, item) in self.items.iter().enumerate() {
            let is_item_focused = i == self.focused_index && focused;

            let checkbox: SharedString = if item.checked {
                icons::CHECKED.into()
            } else {
                icons::UNCHECKED.into()
            };

            let label: SharedString = item.label.clone().into();

            let text_color = if is_item_focused {
                theme.primary
            } else if !item.enabled {
                theme.muted
            } else {
                theme.foreground
            };

            let weight = if is_item_focused {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            };

            let mut row = div()
                .w_full()
                .px(px(spacing::MEDIUM))
                .py(px(spacing::SMALL))
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(spacing::SMALL))
                        .child(
                            div()
                                .text_color(text_color)
                                .font_weight(weight)
                                .child(checkbox),
                        )
                        .child(
                            div()
                                .text_color(text_color)
                                .font_weight(weight)
                                .child(label),
                        ),
                );

            // Description
            if let Some(desc) = &item.description {
                let desc_text: SharedString = format!("      {}", desc).into();
                row = row.child(div().text_color(theme.muted).text_sm().child(desc_text));
            }

            container = container.child(row);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_list_creation() {
        let list = CheckboxList::new("Select Options");
        assert_eq!(list.title, "Select Options");
        assert!(list.is_empty());
    }

    #[test]
    fn test_checkbox_list_with_items() {
        let list = CheckboxList::new("Select Options")
            .add_item(CheckboxItem::new("Option 1"))
            .add_item(CheckboxItem::new("Option 2").checked(true));

        assert_eq!(list.len(), 2);
        assert_eq!(list.checked_items().len(), 1);
    }

    #[test]
    fn test_checkbox_toggle() {
        let mut list = CheckboxList::new("Test").add_item(CheckboxItem::new("Item 1"));

        assert!(!list.items[0].checked);

        list.toggle_current();
        assert!(list.items[0].checked);

        list.toggle_current();
        assert!(!list.items[0].checked);
    }

    #[test]
    fn test_checkbox_navigation() {
        let mut list = CheckboxList::new("Test")
            .add_item(CheckboxItem::new("A"))
            .add_item(CheckboxItem::new("B"))
            .add_item(CheckboxItem::new("C"));

        assert_eq!(list.focused_index(), 0);

        list.move_down();
        assert_eq!(list.focused_index(), 1);

        list.move_down();
        assert_eq!(list.focused_index(), 2);

        // Should not go past end
        list.move_down();
        assert_eq!(list.focused_index(), 2);

        list.move_up();
        assert_eq!(list.focused_index(), 1);

        // Should not go before start
        list.move_up();
        list.move_up();
        assert_eq!(list.focused_index(), 0);
    }

    #[test]
    fn test_checkbox_toggle_disabled() {
        let mut list = CheckboxList::new("Test").add_item(CheckboxItem {
            label: "Disabled".to_string(),
            description: None,
            checked: false,
            enabled: false,
        });

        list.toggle_current();
        assert!(!list.items[0].checked); // Should remain unchecked
    }

    #[test]
    fn test_checkbox_checked_items() {
        let list = CheckboxList::new("Test")
            .add_item(CheckboxItem::new("A").checked(true))
            .add_item(CheckboxItem::new("B"))
            .add_item(CheckboxItem::new("C").checked(true));

        let checked = list.checked_items();
        assert_eq!(checked.len(), 2);
        assert_eq!(checked[0].label, "A");
        assert_eq!(checked[1].label, "C");
    }

    #[test]
    fn test_checkbox_item_builder() {
        let item = CheckboxItem::new("Test")
            .checked(true)
            .with_description("A description");

        assert_eq!(item.label, "Test");
        assert!(item.checked);
        assert_eq!(item.description.as_deref(), Some("A description"));
        assert!(item.enabled);
    }
}
