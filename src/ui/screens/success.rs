//! Installation success/complete screen
//!
//! Displays installation summary and next steps.

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::ui::{theme::AppTheme, utils::render_navigation_hints};

use super::{Screen, ScreenAction};

/// Success screen state
pub struct SuccessScreen {
    /// Application theme
    theme: AppTheme,
    /// Selected option (0 = View Summary, 1 = Reboot)
    selected: usize,
}

impl SuccessScreen {
    /// Create a new success screen
    pub fn new() -> Self {
        Self {
            theme: AppTheme::new(),
            selected: 1, // Default to Reboot
        }
    }

    /// Select next option
    fn select_next(&mut self) {
        self.selected = (self.selected + 1) % 2;
    }

    /// Select previous option
    fn select_previous(&mut self) {
        self.selected = if self.selected == 0 { 1 } else { 0 };
    }
}

impl Default for SuccessScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for SuccessScreen {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),  // Success message
                Constraint::Length(12), // Installation summary
                Constraint::Length(10), // Next steps
                Constraint::Min(0),     // Spacer
                Constraint::Length(3),  // Navigation
            ])
            .split(area);

        // Success message
        let success_lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "✓ Installation Complete!",
                Style::default()
                    .fg(self.theme.success)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("NixOS has been successfully installed on your system."),
        ];

        let success_para =
            Paragraph::new(success_lines).alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(success_para, chunks[0]);

        // Installation summary
        let summary_block = Block::default()
            .borders(Borders::ALL)
            .title(" Installation Summary ")
            .border_style(self.theme.border_style());

        let summary_items = vec![
            ListItem::new(Line::from(vec![
                Span::styled("✓ ", Style::default().fg(self.theme.success)),
                Span::raw("System partitioned and formatted"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("✓ ", Style::default().fg(self.theme.success)),
                Span::raw("NixOS configuration generated"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("✓ ", Style::default().fg(self.theme.success)),
                Span::raw("System packages installed"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("✓ ", Style::default().fg(self.theme.success)),
                Span::raw("Bootloader configured"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("✓ ", Style::default().fg(self.theme.success)),
                Span::raw("User account created"),
            ])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::styled("Configuration: ", self.theme.text_muted()),
                Span::raw("/mnt/etc/nixos/configuration.nix"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("Installation log: ", self.theme.text_muted()),
                Span::raw("/tmp/ekaos-install.log"),
            ])),
        ];

        let summary_list = List::new(summary_items).block(summary_block);
        frame.render_widget(summary_list, chunks[1]);

        // Next steps
        let next_steps_block = Block::default()
            .borders(Borders::ALL)
            .title(" Next Steps ")
            .border_style(self.theme.border_style());

        let next_steps_items = vec![
            ListItem::new(Line::from("1. Remove the installation media")),
            ListItem::new(Line::from("2. Reboot your computer")),
            ListItem::new(Line::from("3. Log in with your user account")),
            ListItem::new(Line::from("4. Customize your system:")),
            ListItem::new(Line::from("   • Edit /etc/nixos/configuration.nix")),
            ListItem::new(Line::from("   • Run 'sudo nixos-rebuild switch' to apply changes")),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::raw("Learn more: "),
                Span::styled(
                    "https://nixos.org/manual/nixos/stable/",
                    Style::default().fg(self.theme.primary),
                ),
            ])),
        ];

        let next_steps_list = List::new(next_steps_items).block(next_steps_block);
        frame.render_widget(next_steps_list, chunks[2]);

        // Navigation
        render_navigation_hints(
            frame,
            &[("Enter", "Reboot Now"), ("q", "Exit to shell")],
            &self.theme,
            chunks[4],
        );
    }

    fn handle_input(&mut self, key: KeyCode) -> ScreenAction {
        // Try standard handlers first (quit, help - though help not shown on success)
        if let Some(action) = self.handle_standard_input(key) {
            return action;
        }

        // Handle screen-specific keys
        match key {
            KeyCode::Up => {
                self.select_previous();
                ScreenAction::None
            }
            KeyCode::Down => {
                self.select_next();
                ScreenAction::None
            }
            KeyCode::Enter => {
                // For now, just exit. In a real implementation, this would trigger a reboot
                ScreenAction::Exit
            }
            _ => ScreenAction::None,
        }
    }

    fn title(&self) -> &str {
        "Installation Complete"
    }

    fn help_content(&self) -> Vec<String> {
        vec![
            "# Installation Complete".to_string(),
            "".to_string(),
            "Congratulations! NixOS has been successfully installed.".to_string(),
            "".to_string(),
            "## What Was Installed".to_string(),
            "".to_string(),
            "- NixOS base system with your chosen configuration".to_string(),
            "- Bootloader (systemd-boot or GRUB)".to_string(),
            "- Your user account with sudo privileges".to_string(),
            "- Network Manager for connectivity".to_string(),
            "- Essential system packages".to_string(),
            "".to_string(),
            "## Configuration Files".to_string(),
            "".to_string(),
            "Your system configuration is located at:".to_string(),
            "- /etc/nixos/configuration.nix (your main config)".to_string(),
            "- /etc/nixos/hardware-configuration.nix (hardware-specific settings)".to_string(),
            "".to_string(),
            "You can edit these files to customize your system.".to_string(),
            "".to_string(),
            "## Next Steps".to_string(),
            "".to_string(),
            "1. Remove Installation Media".to_string(),
            "   - Remove the USB drive or installation disk".to_string(),
            "   - This ensures your system boots from the hard drive".to_string(),
            "".to_string(),
            "2. Reboot".to_string(),
            "   - Press Enter to reboot now".to_string(),
            "   - Or press 'q' to exit and reboot manually later".to_string(),
            "".to_string(),
            "3. First Boot".to_string(),
            "   - You'll see the bootloader menu".to_string(),
            "   - Select NixOS to boot".to_string(),
            "   - Log in with your username and password".to_string(),
            "".to_string(),
            "4. Customize Your System".to_string(),
            "   - Edit /etc/nixos/configuration.nix".to_string(),
            "   - Add packages, enable services, change settings".to_string(),
            "   - Run 'sudo nixos-rebuild switch' to apply changes".to_string(),
            "".to_string(),
            "## Learning Resources".to_string(),
            "".to_string(),
            "- NixOS Manual: https://nixos.org/manual/nixos/stable/".to_string(),
            "- Package Search: https://search.nixos.org/packages".to_string(),
            "- Options Search: https://search.nixos.org/options".to_string(),
            "- Wiki: https://nixos.wiki/".to_string(),
            "- Community: https://discourse.nixos.org/".to_string(),
            "".to_string(),
            "## Troubleshooting".to_string(),
            "".to_string(),
            "If you encounter issues:".to_string(),
            "- Check /tmp/ekaos-install.log for installation details".to_string(),
            "- Boot from the installation media to fix problems".to_string(),
            "- Ask for help on the NixOS Discourse forum".to_string(),
            "".to_string(),
            "## Welcome to NixOS!".to_string(),
            "".to_string(),
            "You're now part of the NixOS community. Enjoy your reproducible,".to_string(),
            "declarative system configuration!".to_string(),
        ]
    }

    fn can_proceed(&self) -> bool {
        true
    }

    fn can_go_back(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_screen_creation() {
        let screen = SuccessScreen::new();
        assert_eq!(screen.title(), "Installation Complete");
        assert_eq!(screen.selected, 1);
    }

    #[test]
    fn test_success_screen_navigation() {
        let mut screen = SuccessScreen::new();
        assert_eq!(screen.selected, 1);

        screen.select_next();
        assert_eq!(screen.selected, 0);

        screen.select_next();
        assert_eq!(screen.selected, 1);

        screen.select_previous();
        assert_eq!(screen.selected, 0);
    }

    #[test]
    fn test_success_screen_cannot_go_back() {
        let screen = SuccessScreen::new();
        assert!(!screen.can_go_back());
    }

    #[test]
    fn test_help_content() {
        let screen = SuccessScreen::new();
        let help = screen.help_content();
        assert!(!help.is_empty());
        assert!(help[0].contains("Installation Complete"));
    }
}
