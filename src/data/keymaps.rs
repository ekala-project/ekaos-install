//! Console keymap database
//!
//! This module provides a curated list of console keymaps
//! for use in keyboard layout selection.

/// Get the list of available console keymaps
pub fn get_keymap_list() -> Vec<(String, String)> {
    vec![
        // Common layouts first
        ("us".to_string(), "English (US)".to_string()),
        ("uk".to_string(), "English (UK)".to_string()),
        ("de".to_string(), "German".to_string()),
        ("fr".to_string(), "French".to_string()),
        ("es".to_string(), "Spanish".to_string()),
        ("it".to_string(), "Italian".to_string()),
        ("pt-latin1".to_string(), "Portuguese".to_string()),
        ("br-abnt2".to_string(), "Portuguese (Brazil)".to_string()),
        ("ru".to_string(), "Russian".to_string()),
        ("jp106".to_string(), "Japanese".to_string()),
        ("ko".to_string(), "Korean".to_string()),
        // European
        ("be-latin1".to_string(), "Belgian".to_string()),
        ("bg_bds-utf8".to_string(), "Bulgarian".to_string()),
        ("cf".to_string(), "Canadian French".to_string()),
        ("croat".to_string(), "Croatian".to_string()),
        ("cz-qwertz".to_string(), "Czech".to_string()),
        ("dk".to_string(), "Danish".to_string()),
        ("dvorak".to_string(), "Dvorak".to_string()),
        ("et".to_string(), "Estonian".to_string()),
        ("fi".to_string(), "Finnish".to_string()),
        ("gr".to_string(), "Greek".to_string()),
        ("hu".to_string(), "Hungarian".to_string()),
        ("is-latin1".to_string(), "Icelandic".to_string()),
        ("il".to_string(), "Israeli (Hebrew)".to_string()),
        ("lt.baltic".to_string(), "Lithuanian".to_string()),
        ("lv".to_string(), "Latvian".to_string()),
        ("mk".to_string(), "Macedonian".to_string()),
        ("nl".to_string(), "Dutch".to_string()),
        ("no".to_string(), "Norwegian".to_string()),
        ("pl2".to_string(), "Polish".to_string()),
        ("ro".to_string(), "Romanian".to_string()),
        ("sk-qwertz".to_string(), "Slovak".to_string()),
        ("si".to_string(), "Slovenian".to_string()),
        ("se".to_string(), "Swedish".to_string()),
        ("sg".to_string(), "Swiss German".to_string()),
        ("fr_CH".to_string(), "Swiss French".to_string()),
        ("trq".to_string(), "Turkish".to_string()),
        ("ua-utf".to_string(), "Ukrainian".to_string()),
        // Other
        ("colemak".to_string(), "Colemak".to_string()),
        ("la-latin1".to_string(), "Latin American".to_string()),
        ("us-acentos".to_string(), "US International".to_string()),
    ]
}
