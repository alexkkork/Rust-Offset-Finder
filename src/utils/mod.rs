// Tue Jan 13 2026 - Alex

pub mod arm64;
pub mod binary;
pub mod config;
pub mod logging;
pub mod math;
pub mod process;
pub mod string;
pub mod testing;

pub use arm64::Arm64Utils;
pub use binary::BinaryUtils;
pub use logging::LoggingUtils;
pub use math::MathUtils;
pub use process::ProcessUtils;
pub use string::StringUtils;

use std::time::{Duration, Instant};

pub fn measure_time<F, T>(f: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();
    (result, elapsed)
}

pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs_f64();

    if total_secs < 0.001 {
        format!("{:.2}µs", duration.as_micros())
    } else if total_secs < 1.0 {
        format!("{:.2}ms", duration.as_millis())
    } else if total_secs < 60.0 {
        format!("{:.2}s", total_secs)
    } else {
        let mins = (total_secs / 60.0).floor();
        let secs = total_secs % 60.0;
        format!("{:.0}m {:.1}s", mins, secs)
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

pub fn hex_string(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn hex_string_spaced(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
}

pub fn parse_hex(s: &str) -> Option<Vec<u8>> {
    let s = s.replace(" ", "").replace("0x", "").replace("0X", "");

    if s.len() % 2 != 0 {
        return None;
    }

    let mut result = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let byte = u8::from_str_radix(&s[i..i+2], 16).ok()?;
        result.push(byte);
    }

    Some(result)
}

pub fn align_up(value: u64, alignment: u64) -> u64 {
    if alignment == 0 {
        return value;
    }
    (value + alignment - 1) & !(alignment - 1)
}

pub fn align_down(value: u64, alignment: u64) -> u64 {
    if alignment == 0 {
        return value;
    }
    value & !(alignment - 1)
}

pub fn is_aligned(value: u64, alignment: u64) -> bool {
    if alignment == 0 {
        return true;
    }
    (value & (alignment - 1)) == 0
}

pub fn safe_slice<T>(slice: &[T], start: usize, len: usize) -> &[T] {
    let end = start.saturating_add(len).min(slice.len());
    let start = start.min(slice.len());
    &slice[start..end]
}

pub fn clamp<T: Ord>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn percentage(current: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (current as f64 / total as f64) * 100.0
    }
}

pub fn ratio(current: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        current as f64 / total as f64
    }
}

pub fn pluralize(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{} {}", count, singular)
    } else {
        format!("{} {}", count, plural)
    }
}

pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len >= 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}

pub fn ensure_trailing_slash(path: &str) -> String {
    if path.ends_with('/') {
        path.to_string()
    } else {
        format!("{}/", path)
    }
}
