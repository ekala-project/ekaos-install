//! Filterable select list component with fuzzy finding
//!
//! Provides a searchable list component where users can type to filter
//! items and select from the filtered results.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::{Component, Focusable, InputEvent, Interactive};

/// A filterable select list with search capability
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
    /// Whether the component is focused
    focused: bool,
    /// Whether the component is enabled
    enabled: bool,
    /// Scroll offset for list display
    scroll_offset: usize,
    /// Maximum visible items
    max_visible: usize,
}

/// An item in the filterable select list
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
    /// Create a new filterable select list
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
            filtered_indices: Vec::new(),
            query: String::new(),
            selected: 0,
            focused: false,
            enabled: true,
            scroll_offset: 0,
            max_visible: 8, // Default, will be adjusted based on available height
        }
    }

    /// Add an item to the list
    pub fn add_item(mut self, label: impl Into<String>, value: T) -> Self {
        let label = label.into();
        self.items.push(SelectItem { label, value });
        self.update_filter();
        self
    }

    /// Set items from a list
    pub fn with_items(mut self, items: Vec<(String, T)>) -> Self {
        self.items = items
            .into_iter()
            .map(|(label, value)| SelectItem { label, value })
            .collect();
        self.update_filter();
        self
    }

    /// Get the currently selected value
    pub fn selected_value(&self) -> Option<&T> {
        if self.filtered_indices.is_empty() {
            return None;
        }
        let filtered_idx = self
            .selected
            .min(self.filtered_indices.len().saturating_sub(1));
        let item_idx = self.filtered_indices[filtered_idx];
        Some(&self.items[item_idx].value)
    }

    /// Get the currently selected label
    pub fn selected_label(&self) -> Option<&str> {
        if self.filtered_indices.is_empty() {
            return None;
        }
        let filtered_idx = self
            .selected
            .min(self.filtered_indices.len().saturating_sub(1));
        let item_idx = self.filtered_indices[filtered_idx];
        Some(&self.items[item_idx].label)
    }

    /// Set the selected item by value (finds first match)
    pub fn set_selected_by_label(&mut self, label: &str) {
        // Find the item with matching label
        if let Some(item_idx) = self.items.iter().position(|item| item.label == label) {
            // Find this item in filtered list
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

    /// Update the filtered list based on current query
    fn update_filter(&mut self) {
        if self.query.is_empty() {
            // Show all items
            self.filtered_indices = (0..self.items.len()).collect();
        } else {
            // Filter items by fuzzy match
            let query_lower = self.query.to_lowercase();
            self.filtered_indices = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.label.to_lowercase().contains(&query_lower))
                .map(|(idx, _)| idx)
                .collect();
        }

        // Reset selection to first item
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Move selection up
    fn select_previous(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected = self.selected.saturating_sub(1);
            self.adjust_scroll();
        }
    }

    /// Move selection down
    fn select_next(&mut self) {
        if !self.filtered_indices.is_empty() {
            let max = self.filtered_indices.len().saturating_sub(1);
            self.selected = (self.selected + 1).min(max);
            self.adjust_scroll();
        }
    }

    /// Adjust scroll offset to keep selection visible
    fn adjust_scroll(&mut self) {
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + self.max_visible {
            self.scroll_offset = self.selected.saturating_sub(self.max_visible - 1);
        }
    }

    /// Clear the search query
    fn clear_query(&mut self) {
        self.query.clear();
        self.update_filter();
    }

    /// Add character to query
    fn add_char_to_query(&mut self, c: char) {
        self.query.push(c);
        self.update_filter();
    }

    /// Remove last character from query
    fn remove_char_from_query(&mut self) {
        self.query.pop();
        self.update_filter();
    }
}

impl<T> Focusable for FilterableSelectList<T>
where
    T: Clone,
{
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl<T> Interactive for FilterableSelectList<T>
where
    T: Clone,
{
    fn handle_input(&mut self, event: InputEvent) -> bool {
        if !self.enabled {
            return false;
        }

        match event {
            InputEvent::Char(c) => {
                self.add_char_to_query(c);
                true
            }
            InputEvent::Backspace => {
                self.remove_char_from_query();
                true
            }
            InputEvent::Up => {
                self.select_previous();
                true
            }
            InputEvent::Down => {
                self.select_next();
                true
            }
            InputEvent::Escape => {
                self.clear_query();
                true
            }
            _ => false,
        }
    }
}

impl<T> Component for FilterableSelectList<T>
where
    T: Clone,
{
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        // Adjust max visible based on available height
        // Reserve 4 lines for border + input + status
        self.max_visible = (area.height.saturating_sub(4) as usize).max(3);

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

        // Create main block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.as_str())
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height < 2 {
            return; // Not enough space to render content
        }

        // Split into input area and list area
        let input_height = 1;
        let status_height = 1;
        let list_height = inner.height.saturating_sub(input_height + status_height);

        let input_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: input_height,
        };

        let list_area = Rect {
            x: inner.x,
            y: inner.y + input_height,
            width: inner.width,
            height: list_height,
        };

        let status_area = Rect {
            x: inner.x,
            y: inner.y + input_height + list_height,
            width: inner.width,
            height: status_height,
        };

        // Render search input
        let query_display = if self.focused {
            format!("Filter: {}█", self.query)
        } else {
            format!("Filter: {}", self.query)
        };

        let input_para =
            Paragraph::new(query_display).style(Style::default().fg(if self.focused {
                Color::Cyan
            } else {
                Color::White
            }));
        frame.render_widget(input_para, input_area);

        // Render filtered list
        if self.filtered_indices.is_empty() {
            let no_results = Paragraph::new("No matches").style(Style::default().fg(Color::Red));
            frame.render_widget(no_results, list_area);
        } else {
            let visible_items: Vec<ListItem> = self
                .filtered_indices
                .iter()
                .skip(self.scroll_offset)
                .take(self.max_visible)
                .enumerate()
                .map(|(display_idx, &item_idx)| {
                    let actual_idx = self.scroll_offset + display_idx;
                    let item = &self.items[item_idx];
                    let is_selected = actual_idx == self.selected;

                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let prefix = if is_selected { "> " } else { "  " };
                    ListItem::new(Line::from(Span::styled(
                        format!("{}{}", prefix, item.label),
                        style,
                    )))
                })
                .collect();

            let list = List::new(visible_items);
            frame.render_widget(list, list_area);
        }

        // Render status line
        let status_text = if self.query.is_empty() {
            format!("{} items", self.filtered_indices.len())
        } else {
            format!("{} matches", self.filtered_indices.len())
        };

        let status_para = Paragraph::new(status_text).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(status_para, status_area);
    }

    fn title(&self) -> Option<&str> {
        Some(&self.title)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}
