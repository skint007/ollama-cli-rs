use chrono::DateTime;

/// Convert bytes to human-readable format (e.g., "4.5GB", "256MB")
pub fn bytes_to_human(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{}B", bytes)
    } else {
        format!("{:.1}{}", size, UNITS[unit_idx])
    }
}

/// Format ISO 8601 datetime to "YYYY.MM.DD HH:MM"
pub fn format_datetime(iso: &str) -> String {
    DateTime::parse_from_rfc3339(iso)
        .map(|dt| dt.format("%Y.%m.%d %H:%M").to_string())
        .unwrap_or_else(|_| iso.to_string())
}

/// Convert nanoseconds to seconds
pub fn nanos_to_secs(nanos: u64) -> f64 {
    nanos as f64 / 1_000_000_000.0
}

/// Calculate tokens per second from eval_count and eval_duration (nanoseconds)
pub fn tokens_per_sec(eval_count: u64, eval_duration_nanos: u64) -> f64 {
    if eval_duration_nanos == 0 {
        return 0.0;
    }
    eval_count as f64 * 1_000_000_000.0 / eval_duration_nanos as f64
}
