//! Filterable select list component with search
//!
//! `FilterableSelectList<T>` is a searchable list. Pure data struct with
//! navigation/filter methods and a `view()` method for rendering.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, IntoElement, SharedString};

use crate::ui::theme::{spacing, AppTheme};

/// A filterable select list with search capability.
///
/// Pure data struct; the parent owns focus and routes keyboard events
/// to the appropriate methods.
pub struct FilterableSelectList<T> {
    /// List title
    title: String,
    /// All available items
    items: Vec<SelectItem<T>>,
    /// Indices of currently filtered items
    filtered_indices: Vec<usize>,
    /// Current search query
    query: String,
    /// Selected index in filtered list
    selected: usize,
    /// Scroll offset for list display
    scroll_offset: usize,
    /// Maximum visible items
    max_visible: usize,
}

/// An item in the filterable select list.
pub struct SelectItem<T> {
    /// Display label
    pub label: String,
    /// Associated value
    pub value: T,
}

impl<T> FilterableSelectList<T>
where
    T: Clone,
{
    /// Create a new filterable select list.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
            filtered_indices: Vec::new(),
            query: String::new(),
            selected: 0,
            scroll_offset: 0,
            max_visible: 8,
        }
    }

    /// Add an item to the list.
    pub fn add_item(mut self, label: impl Into<String>, value: T) -> Self {
        let label = label.into();
        self.items.push(SelectItem { label, value });
        self.update_filter();
        self
    }

    /// Set items from a vector of (label, value) pairs.
    pub fn with_items(mut self, items: Vec<(String, T)>) -> Self {
        self.items = items
            .into_iter()
            .map(|(label, value)| SelectItem { label, value })
            .collect();
        self.update_filter();
        self
    }

    /// Set the maximum number of visible items.
    pub fn with_max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    /// Get the currently selected value.
    pub fn selected_value(&self) -> Option<&T> {
        if self.filtered_indices.is_empty() {
            return None;
        }
        let idx = self
            .selected
            .min(self.filtered_indices.len().saturating_sub(1));
        let item_idx = self.filtered_indices[idx];
        Some(&self.items[item_idx].value)
    }

    /// Get the currently selected label.
    pub fn selected_label(&self) -> Option<&str> {
        if self.filtered_indices.is_empty() {
            return None;
        }
        let idx = self
            .selected
            .min(self.filtered_indices.len().saturating_sub(1));
        let item_idx = self.filtered_indices[idx];
        Some(&self.items[item_idx].label)
    }

    /// Set the selected item by label (finds first match).
    pub fn set_selected_by_label(&mut self, label: &str) {
        if let Some(item_idx) = self.items.iter().position(|item| item.label == label) {
            if let Some(filtered_idx) = self
                .filtered_indices
                .iter()
                .position(|&idx| idx == item_idx)
            {
                self.selected = filtered_idx;
                self.adjust_scroll();
            }
        }
    }

    /// Update the filtered list based on the current query.
    pub fn update_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered_indices = (0..self.items.len()).collect();
        } else {
            let query_lower = self.query.to_lowercase();
            self.filtered_indices = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.label.to_lowercase().contains(&query_lower))
                .map(|(idx, _)| idx)
                .collect();
        }
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Move selection up.
    pub fn select_previous(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected = self.selected.saturating_sub(1);
            self.adjust_scroll();
        }
    }

    /// Move selection down.
    pub fn select_next(&mut self) {
        if !self.filtered_indices.is_empty() {
            let max = self.filtered_indices.len().saturating_sub(1);
            self.selected = (self.selected + 1).min(max);
            self.adjust_scroll();
        }
    }

    /// Adjust scroll offset to keep selection visible.
    pub fn adjust_scroll(&mut self) {
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + self.max_visible {
            self.scroll_offset = self.selected.saturating_sub(self.max_visible - 1);
        }
    }

    /// Add a character to the search query.
    pub fn add_char_to_query(&mut self, c: char) {
        self.query.push(c);
        self.update_filter();
    }

    /// Remove the last character from the search query.
    pub fn remove_char_from_query(&mut self) {
        self.query.pop();
        self.update_filter();
    }

    /// Clear the search query.
    pub fn clear_query(&mut self) {
        self.query.clear();
        self.update_filter();
    }

    /// Get the current query string.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Get the number of filtered items.
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Get the total number of items.
    pub fn total_count(&self) -> usize {
        self.items.len()
    }

    /// Build a gpui element tree for this list.
    ///
    /// `focused` indicates whether the parent considers this list focused.
    pub fn view(&self, focused: bool) -> impl IntoElement {
        let theme = AppTheme::new();

        let border_color = if focused { theme.primary } else { theme.border };

        let title: SharedString = self.title.clone().into();

        let query_display: SharedString = if focused {
            format!("Filter: {}\u{2588}", self.query).into()
        } else {
            format!("Filter: {}", self.query).into()
        };

        let status_text: SharedString = if self.query.is_empty() {
            format!("{} items", self.filtered_indices.len()).into()
        } else {
            format!("{} matches", self.filtered_indices.len()).into()
        };

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
            )
            // Filter input
            .child(
                div()
                    .px(px(spacing::MEDIUM))
                    .py(px(spacing::SMALL))
                    .text_color(if focused {
                        theme.primary
                    } else {
                        theme.foreground
                    })
                    .text_sm()
                    .child(query_display),
            );

        // Filtered items
        if self.filtered_indices.is_empty() {
            container = container.child(
                div()
                    .px(px(spacing::MEDIUM))
                    .py(px(spacing::SMALL))
                    .text_color(theme.error)
                    .child("No matches"),
            );
        } else {
            for (display_idx, &item_idx) in self
                .filtered_indices
                .iter()
                .skip(self.scroll_offset)
                .take(self.max_visible)
                .enumerate()
            {
                let actual_idx = self.scroll_offset + display_idx;
                let item = &self.items[item_idx];
                let is_selected = actual_idx == self.selected;

                let prefix: SharedString = if is_selected {
                    "> ".into()
                } else {
                    "  ".into()
                };
                let label: SharedString = item.label.clone().into();

                let row = if is_selected {
                    div()
                        .w_full()
                        .px(px(spacing::MEDIUM))
                        .py(px(spacing::SMALL))
                        .bg(theme.primary)
                        .text_color(gpui::hsla(0.0, 0.0, 0.0, 1.0))
                        .font_weight(FontWeight::BOLD)
                        .flex()
                        .flex_row()
                        .child(prefix)
                        .child(label)
                } else {
                    div()
                        .w_full()
                        .px(px(spacing::MEDIUM))
                        .py(px(spacing::SMALL))
                        .text_color(theme.foreground)
                        .flex()
                        .flex_row()
                        .child(prefix)
                        .child(label)
                };

                container = container.child(row);
            }
        }

        // Status line
        container = container.child(
            div()
                .px(px(spacing::MEDIUM))
                .py(px(spacing::SMALL))
                .text_color(theme.muted)
                .text_sm()
                .child(status_text),
        );

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filterable_creation() {
        let list: FilterableSelectList<i32> = FilterableSelectList::new("Choose");
        assert_eq!(list.title, "Choose");
        assert_eq!(list.total_count(), 0);
        assert_eq!(list.filtered_count(), 0);
    }

    #[test]
    fn test_filterable_add_items() {
        let list = FilterableSelectList::new("Choose")
            .add_item("Alpha", 1)
            .add_item("Beta", 2)
            .add_item("Gamma", 3);

        assert_eq!(list.total_count(), 3);
        assert_eq!(list.filtered_count(), 3);
        assert_eq!(list.selected_label(), Some("Alpha"));
    }

    #[test]
    fn test_filterable_with_items() {
        let list = FilterableSelectList::new("Choose")
            .with_items(vec![("Alpha".to_string(), 1), ("Beta".to_string(), 2)]);

        assert_eq!(list.total_count(), 2);
        assert_eq!(list.selected_value(), Some(&1));
    }

    #[test]
    fn test_filterable_filter() {
        let mut list = FilterableSelectList::new("Choose")
            .add_item("Apple", 1)
            .add_item("Banana", 2)
            .add_item("Apricot", 3);

        list.add_char_to_query('a');
        list.add_char_to_query('p');
        // Should match "Apple" and "Apricot"
        assert_eq!(list.filtered_count(), 2);

        list.clear_query();
        assert_eq!(list.filtered_count(), 3);
    }

    #[test]
    fn test_filterable_navigation() {
        let mut list = FilterableSelectList::new("Choose")
            .add_item("A", 1)
            .add_item("B", 2)
            .add_item("C", 3);

        assert_eq!(list.selected, 0);

        list.select_next();
        assert_eq!(list.selected, 1);

        list.select_next();
        assert_eq!(list.selected, 2);

        // Should not go past end
        list.select_next();
        assert_eq!(list.selected, 2);

        list.select_previous();
        assert_eq!(list.selected, 1);

        // Should not go before start
        list.select_previous();
        list.select_previous();
        assert_eq!(list.selected, 0);
    }

    #[test]
    fn test_filterable_remove_char() {
        let mut list = FilterableSelectList::new("Choose")
            .add_item("Apple", 1)
            .add_item("Banana", 2);

        list.add_char_to_query('z');
        assert_eq!(list.filtered_count(), 0);

        list.remove_char_from_query();
        assert_eq!(list.filtered_count(), 2);
    }

    #[test]
    fn test_filterable_set_selected_by_label() {
        let mut list = FilterableSelectList::new("Choose")
            .add_item("Alpha", 1)
            .add_item("Beta", 2)
            .add_item("Gamma", 3);

        list.set_selected_by_label("Gamma");
        assert_eq!(list.selected_label(), Some("Gamma"));
        assert_eq!(list.selected_value(), Some(&3));
    }

    #[test]
    fn test_filterable_no_matches() {
        let mut list = FilterableSelectList::new("Choose").add_item("Apple", 1);

        list.add_char_to_query('z');
        assert_eq!(list.filtered_count(), 0);
        assert!(list.selected_value().is_none());
        assert!(list.selected_label().is_none());
    }

    #[test]
    fn test_filterable_case_insensitive() {
        let mut list = FilterableSelectList::new("Choose")
            .add_item("Apple", 1)
            .add_item("banana", 2);

        list.add_char_to_query('B');
        assert_eq!(list.filtered_count(), 1);
        assert_eq!(list.selected_label(), Some("banana"));
    }

    #[test]
    fn test_filterable_scroll_adjust() {
        let mut list = FilterableSelectList::new("Choose")
            .with_max_visible(2)
            .add_item("A", 1)
            .add_item("B", 2)
            .add_item("C", 3)
            .add_item("D", 4);

        // At start, scroll_offset = 0
        assert_eq!(list.scroll_offset, 0);

        list.select_next(); // selected = 1
        list.select_next(); // selected = 2, should scroll
        assert!(list.scroll_offset > 0);
    }

    #[test]
    fn test_filterable_query_accessor() {
        let mut list: FilterableSelectList<i32> = FilterableSelectList::new("Test");
        assert_eq!(list.query(), "");

        list.add_char_to_query('x');
        assert_eq!(list.query(), "x");
    }
}
