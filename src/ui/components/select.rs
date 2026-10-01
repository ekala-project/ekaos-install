//! Selection list component
//!
//! `SelectList<T>` is a single-select list. It is a pure data struct with
//! navigation methods and a `view()` method that returns an `impl IntoElement`.
//! The parent screen is responsible for focus and keyboard routing.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, IntoElement, SharedString};

use crate::ui::theme::{icons, spacing, AppTheme};

/// Single-select list component.
///
/// This is a data struct, not a gpui entity. The parent screen owns focus
/// and calls navigation methods, then calls `view(focused)` to render.
pub struct SelectList<T> {
    /// List title
    title: String,
    /// List items
    items: Vec<SelectItem<T>>,
    /// Currently selected index
    selected: usize,
    /// Whether to show descriptions
    show_descriptions: bool,
}

/// An item in the selection list.
#[derive(Clone)]
pub struct SelectItem<T> {
    /// Display label
    pub label: String,
    /// Optional description
    pub description: Option<String>,
    /// Associated value
    pub value: T,
    /// Whether this item is enabled (selectable)
    pub enabled: bool,
}

impl<T> SelectItem<T> {
    /// Create a new select item.
    pub fn new(label: impl Into<String>, value: T) -> Self {
        Self {
            label: label.into(),
            description: None,
            value,
            enabled: true,
        }
    }

    /// Add a description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set enabled state.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl<T> SelectList<T> {
    /// Create a new selection list.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
            selected: 0,
            show_descriptions: false,
        }
    }

    /// Add an item to the list.
    pub fn add_item(mut self, item: SelectItem<T>) -> Self {
        self.items.push(item);
        self
    }

    /// Set items from a vector.
    pub fn with_items(mut self, items: Vec<SelectItem<T>>) -> Self {
        self.items = items;
        self
    }

    /// Enable description display.
    pub fn show_descriptions(mut self, show: bool) -> Self {
        self.show_descriptions = show;
        self
    }

    /// Get the currently selected item.
    pub fn selected(&self) -> Option<&SelectItem<T>> {
        self.items.get(self.selected)
    }

    /// Get the selected index.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Set the selected index.
    pub fn set_selected(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = index;
        }
    }

    /// Move selection up, skipping disabled items.
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            while self.selected > 0 && !self.items[self.selected].enabled {
                self.selected -= 1;
            }
        }
    }

    /// Move selection down, skipping disabled items.
    pub fn move_down(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
            while self.selected < self.items.len() - 1 && !self.items[self.selected].enabled {
                self.selected += 1;
            }
        }
    }

    /// Get number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if list is empty.
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
            // Title bar
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
            let is_selected = i == self.selected;

            let indicator: SharedString = if is_selected {
                format!("{} ", icons::SELECTION).into()
            } else {
                "  ".into()
            };

            let label: SharedString = item.label.clone().into();

            let (text_color, weight) = if !item.enabled {
                (theme.muted, FontWeight::NORMAL)
            } else if is_selected {
                (theme.primary, FontWeight::BOLD)
            } else {
                (theme.foreground, FontWeight::NORMAL)
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
                        .child(
                            div()
                                .text_color(if is_selected {
                                    theme.primary
                                } else {
                                    theme.foreground
                                })
                                .font_weight(FontWeight::BOLD)
                                .child(indicator),
                        )
                        .child(
                            div()
                                .text_color(text_color)
                                .font_weight(weight)
                                .child(label),
                        ),
                );

            // Description
            if self.show_descriptions {
                if let Some(desc) = &item.description {
                    let desc_text: SharedString = format!("    {}", desc).into();
                    row = row.child(div().text_color(theme.muted).text_sm().child(desc_text));
                }
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

        list.move_down();
        assert_eq!(list.selected_index(), 1);

        list.move_down();
        assert_eq!(list.selected_index(), 2);

        // Should not go beyond last item
        list.move_down();
        assert_eq!(list.selected_index(), 2);

        list.move_up();
        assert_eq!(list.selected_index(), 1);
    }

    #[test]
    fn test_select_disabled_items() {
        let mut list: SelectList<i32> = SelectList::new("Choose")
            .add_item(SelectItem::new("Option 1", 1))
            .add_item(SelectItem::new("Option 2", 2).with_enabled(false))
            .add_item(SelectItem::new("Option 3", 3));

        list.move_down();
        // Should skip disabled option 2
        assert_eq!(list.selected_index(), 2);
    }

    #[test]
    fn test_select_set_selected() {
        let mut list: SelectList<i32> = SelectList::new("Choose")
            .add_item(SelectItem::new("A", 1))
            .add_item(SelectItem::new("B", 2))
            .add_item(SelectItem::new("C", 3));

        list.set_selected(2);
        assert_eq!(list.selected_index(), 2);
        assert_eq!(list.selected().unwrap().label, "C");

        // Out of range should not change
        list.set_selected(10);
        assert_eq!(list.selected_index(), 2);
    }

    #[test]
    fn test_select_item_builder() {
        let item = SelectItem::new("Label", 42)
            .with_description("A description")
            .with_enabled(false);

        assert_eq!(item.label, "Label");
        assert_eq!(item.value, 42);
        assert_eq!(item.description.as_deref(), Some("A description"));
        assert!(!item.enabled);
    }

    #[test]
    fn test_select_move_up_at_top() {
        let mut list: SelectList<i32> = SelectList::new("Choose")
            .add_item(SelectItem::new("A", 1))
            .add_item(SelectItem::new("B", 2));

        list.move_up(); // Already at 0
        assert_eq!(list.selected_index(), 0);
    }

    #[test]
    fn test_select_empty() {
        let list: SelectList<i32> = SelectList::new("Empty");
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(list.selected().is_none());
    }

    #[test]
    fn test_select_show_descriptions() {
        let list: SelectList<i32> = SelectList::new("Choose")
            .show_descriptions(true)
            .add_item(SelectItem::new("A", 1).with_description("First"));
        assert!(list.show_descriptions);
    }
}
