//! IANA Timezone database
//!
//! This module provides a curated list of IANA timezone identifiers
//! for use in timezone selection, organized by region.

/// Get the list of available timezones, organized by region
pub fn get_timezone_list() -> Vec<String> {
    vec![
        // UTC/Special
        "UTC".to_string(),
        // Africa
        "Africa/Accra".to_string(),
        "Africa/Addis_Ababa".to_string(),
        "Africa/Algiers".to_string(),
        "Africa/Cairo".to_string(),
        "Africa/Casablanca".to_string(),
        "Africa/Dar_es_Salaam".to_string(),
        "Africa/Harare".to_string(),
        "Africa/Johannesburg".to_string(),
        "Africa/Kampala".to_string(),
        "Africa/Khartoum".to_string(),
        "Africa/Kinshasa".to_string(),
        "Africa/Lagos".to_string(),
        "Africa/Lusaka".to_string(),
        "Africa/Maputo".to_string(),
        "Africa/Nairobi".to_string(),
        "Africa/Tripoli".to_string(),
        "Africa/Tunis".to_string(),
        "Africa/Windhoek".to_string(),
        // Americas
        "America/Anchorage".to_string(),
        "America/Argentina/Buenos_Aires".to_string(),
        "America/Bogota".to_string(),
        "America/Chicago".to_string(),
        "America/Denver".to_string(),
        "America/Edmonton".to_string(),
        "America/Halifax".to_string(),
        "America/Lima".to_string(),
        "America/Los_Angeles".to_string(),
        "America/Manaus".to_string(),
        "America/Mexico_City".to_string(),
        "America/New_York".to_string(),
        "America/Phoenix".to_string(),
        "America/Santiago".to_string(),
        "America/Sao_Paulo".to_string(),
        "America/St_Johns".to_string(),
        "America/Toronto".to_string(),
        "America/Vancouver".to_string(),
        "America/Winnipeg".to_string(),
        // Asia
        "Asia/Almaty".to_string(),
        "Asia/Baghdad".to_string(),
        "Asia/Bangkok".to_string(),
        "Asia/Colombo".to_string(),
        "Asia/Dhaka".to_string(),
        "Asia/Dubai".to_string(),
        "Asia/Ho_Chi_Minh".to_string(),
        "Asia/Hong_Kong".to_string(),
        "Asia/Istanbul".to_string(),
        "Asia/Jakarta".to_string(),
        "Asia/Jerusalem".to_string(),
        "Asia/Karachi".to_string(),
        "Asia/Kathmandu".to_string(),
        "Asia/Kolkata".to_string(),
        "Asia/Kuala_Lumpur".to_string(),
        "Asia/Manila".to_string(),
        "Asia/Novosibirsk".to_string(),
        "Asia/Riyadh".to_string(),
        "Asia/Seoul".to_string(),
        "Asia/Shanghai".to_string(),
        "Asia/Singapore".to_string(),
        "Asia/Taipei".to_string(),
        "Asia/Tehran".to_string(),
        "Asia/Tokyo".to_string(),
        "Asia/Vladivostok".to_string(),
        "Asia/Yekaterinburg".to_string(),
        // Australia
        "Australia/Adelaide".to_string(),
        "Australia/Brisbane".to_string(),
        "Australia/Darwin".to_string(),
        "Australia/Hobart".to_string(),
        "Australia/Melbourne".to_string(),
        "Australia/Perth".to_string(),
        "Australia/Sydney".to_string(),
        // Europe
        "Europe/Amsterdam".to_string(),
        "Europe/Athens".to_string(),
        "Europe/Belgrade".to_string(),
        "Europe/Berlin".to_string(),
        "Europe/Brussels".to_string(),
        "Europe/Bucharest".to_string(),
        "Europe/Budapest".to_string(),
        "Europe/Copenhagen".to_string(),
        "Europe/Dublin".to_string(),
        "Europe/Helsinki".to_string(),
        "Europe/Kiev".to_string(),
        "Europe/Lisbon".to_string(),
        "Europe/London".to_string(),
        "Europe/Madrid".to_string(),
        "Europe/Moscow".to_string(),
        "Europe/Oslo".to_string(),
        "Europe/Paris".to_string(),
        "Europe/Prague".to_string(),
        "Europe/Rome".to_string(),
        "Europe/Sofia".to_string(),
        "Europe/Stockholm".to_string(),
        "Europe/Vienna".to_string(),
        "Europe/Vilnius".to_string(),
        "Europe/Warsaw".to_string(),
        "Europe/Zurich".to_string(),
        // Indian Ocean
        "Indian/Maldives".to_string(),
        "Indian/Mauritius".to_string(),
        // Pacific
        "Pacific/Auckland".to_string(),
        "Pacific/Fiji".to_string(),
        "Pacific/Guam".to_string(),
        "Pacific/Honolulu".to_string(),
    ]
}

/// Try to detect the system's current timezone.
/// Returns None if detection fails.
pub fn detect_timezone() -> Option<String> {
    // Try /etc/localtime symlink (most Linux systems)
    if let Ok(link) = std::fs::read_link("/etc/localtime") {
        let path = link.to_string_lossy();
        // Path looks like /usr/share/zoneinfo/America/New_York
        if let Some(pos) = path.find("zoneinfo/") {
            let tz = &path[pos + 9..];
            return Some(tz.to_string());
        }
    }

    // Try TZ environment variable
    if let Ok(tz) = std::env::var("TZ") {
        if !tz.is_empty() && tz.contains('/') {
            return Some(tz);
        }
    }

    // Try /etc/timezone (Debian-based)
    if let Ok(content) = std::fs::read_to_string("/etc/timezone") {
        let tz = content.trim().to_string();
        if !tz.is_empty() {
            return Some(tz);
        }
    }

    None
}
