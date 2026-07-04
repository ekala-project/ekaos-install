//! Locale database
//!
//! This module provides a curated list of common system locales
//! for use in locale selection.

/// Get the list of available locales
pub fn get_locale_list() -> Vec<(String, String)> {
    vec![
        // English
        ("en_US.UTF-8".to_string(), "English (United States)".to_string()),
        ("en_GB.UTF-8".to_string(), "English (United Kingdom)".to_string()),
        ("en_AU.UTF-8".to_string(), "English (Australia)".to_string()),
        ("en_CA.UTF-8".to_string(), "English (Canada)".to_string()),
        ("en_IE.UTF-8".to_string(), "English (Ireland)".to_string()),
        ("en_NZ.UTF-8".to_string(), "English (New Zealand)".to_string()),
        ("en_ZA.UTF-8".to_string(), "English (South Africa)".to_string()),
        ("en_IN.UTF-8".to_string(), "English (India)".to_string()),
        // European
        ("de_DE.UTF-8".to_string(), "German (Germany)".to_string()),
        ("de_AT.UTF-8".to_string(), "German (Austria)".to_string()),
        ("de_CH.UTF-8".to_string(), "German (Switzerland)".to_string()),
        ("fr_FR.UTF-8".to_string(), "French (France)".to_string()),
        ("fr_BE.UTF-8".to_string(), "French (Belgium)".to_string()),
        ("fr_CA.UTF-8".to_string(), "French (Canada)".to_string()),
        ("fr_CH.UTF-8".to_string(), "French (Switzerland)".to_string()),
        ("es_ES.UTF-8".to_string(), "Spanish (Spain)".to_string()),
        ("es_MX.UTF-8".to_string(), "Spanish (Mexico)".to_string()),
        ("es_AR.UTF-8".to_string(), "Spanish (Argentina)".to_string()),
        ("es_CO.UTF-8".to_string(), "Spanish (Colombia)".to_string()),
        ("it_IT.UTF-8".to_string(), "Italian (Italy)".to_string()),
        ("pt_BR.UTF-8".to_string(), "Portuguese (Brazil)".to_string()),
        ("pt_PT.UTF-8".to_string(), "Portuguese (Portugal)".to_string()),
        ("nl_NL.UTF-8".to_string(), "Dutch (Netherlands)".to_string()),
        ("nl_BE.UTF-8".to_string(), "Dutch (Belgium)".to_string()),
        ("pl_PL.UTF-8".to_string(), "Polish".to_string()),
        ("cs_CZ.UTF-8".to_string(), "Czech".to_string()),
        ("sk_SK.UTF-8".to_string(), "Slovak".to_string()),
        ("hu_HU.UTF-8".to_string(), "Hungarian".to_string()),
        ("ro_RO.UTF-8".to_string(), "Romanian".to_string()),
        ("bg_BG.UTF-8".to_string(), "Bulgarian".to_string()),
        ("hr_HR.UTF-8".to_string(), "Croatian".to_string()),
        ("sl_SI.UTF-8".to_string(), "Slovenian".to_string()),
        ("sr_RS.UTF-8".to_string(), "Serbian".to_string()),
        ("el_GR.UTF-8".to_string(), "Greek".to_string()),
        ("tr_TR.UTF-8".to_string(), "Turkish".to_string()),
        ("uk_UA.UTF-8".to_string(), "Ukrainian".to_string()),
        ("ru_RU.UTF-8".to_string(), "Russian".to_string()),
        // Nordic
        ("da_DK.UTF-8".to_string(), "Danish".to_string()),
        ("fi_FI.UTF-8".to_string(), "Finnish".to_string()),
        ("nb_NO.UTF-8".to_string(), "Norwegian (Bokmal)".to_string()),
        ("nn_NO.UTF-8".to_string(), "Norwegian (Nynorsk)".to_string()),
        ("sv_SE.UTF-8".to_string(), "Swedish".to_string()),
        ("is_IS.UTF-8".to_string(), "Icelandic".to_string()),
        ("et_EE.UTF-8".to_string(), "Estonian".to_string()),
        ("lt_LT.UTF-8".to_string(), "Lithuanian".to_string()),
        ("lv_LV.UTF-8".to_string(), "Latvian".to_string()),
        // Asian
        ("ja_JP.UTF-8".to_string(), "Japanese".to_string()),
        ("ko_KR.UTF-8".to_string(), "Korean".to_string()),
        ("zh_CN.UTF-8".to_string(), "Chinese (Simplified)".to_string()),
        ("zh_TW.UTF-8".to_string(), "Chinese (Traditional)".to_string()),
        ("th_TH.UTF-8".to_string(), "Thai".to_string()),
        ("vi_VN.UTF-8".to_string(), "Vietnamese".to_string()),
        ("hi_IN.UTF-8".to_string(), "Hindi".to_string()),
        // Other
        ("ar_SA.UTF-8".to_string(), "Arabic (Saudi Arabia)".to_string()),
        ("he_IL.UTF-8".to_string(), "Hebrew".to_string()),
        ("id_ID.UTF-8".to_string(), "Indonesian".to_string()),
        ("ms_MY.UTF-8".to_string(), "Malay".to_string()),
    ]
}
