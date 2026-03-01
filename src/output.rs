use chrono::{DateTime, Utc};
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Table};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Json,
}

impl OutputMode {
    pub fn from_flag(json: bool) -> Self {
        if json {
            Self::Json
        } else {
            Self::Human
        }
    }
}

/// Print a value as JSON or as a formatted table.
pub fn print_output<T: serde::Serialize>(
    mode: OutputMode,
    value: &T,
    table_fn: impl FnOnce(&T) -> Table,
) {
    match mode {
        OutputMode::Json => {
            let json = serde_json::to_string_pretty(value).expect("JSON serialization failed");
            println!("{json}");
        }
        OutputMode::Human => {
            let table = table_fn(value);
            println!("{table}");
        }
    }
}

/// Print a list of values as JSON array or as a table with rows.
pub fn print_list<T: serde::Serialize>(
    mode: OutputMode,
    items: &[T],
    headers: &[&str],
    row_fn: impl Fn(&T) -> Vec<String>,
) {
    match mode {
        OutputMode::Json => {
            let json = serde_json::to_string_pretty(items).expect("JSON serialization failed");
            println!("{json}");
        }
        OutputMode::Human => {
            if items.is_empty() {
                println!("No results.");
                return;
            }
            let mut table = new_table();
            table.set_header(headers.iter().copied());
            for item in items {
                table.add_row(row_fn(item));
            }
            println!("{table}");
        }
    }
}

pub fn new_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);
    table
}

pub fn format_time(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn format_duration_secs(secs: Option<f64>) -> String {
    match secs {
        Some(s) if s < 60.0 => format!("{s:.1}s"),
        Some(s) => {
            let mins = (s / 60.0).floor();
            let remainder = s - mins * 60.0;
            format!("{mins:.0}m {remainder:.1}s")
        }
        None => "-".to_string(),
    }
}

pub fn short_uuid(id: &uuid::Uuid) -> String {
    let s = id.to_string();
    s[..8].to_string()
}

pub fn format_bytes(bytes: i64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    #[expect(
        clippy::cast_precision_loss,
        reason = "file sizes don't need i64 precision"
    )]
    let b = bytes as f64;
    if b < KB {
        format!("{bytes} B")
    } else if b < MB {
        format!("{:.1} KB", b / KB)
    } else if b < GB {
        format!("{:.1} MB", b / MB)
    } else {
        format!("{:.2} GB", b / GB)
    }
}
