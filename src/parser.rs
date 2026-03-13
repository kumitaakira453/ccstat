use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Default)]
pub struct SessionEntry {
    pub write_lines: usize,
    pub edit_lines: usize,
    pub text_lines: usize,
    pub timestamp: Option<String>,
    pub session_id: Option<String>,
    pub title: Option<String>,
}

/// Truncate string to fit within `max_width` display columns (CJK=2, ASCII=1)
fn truncate_display(s: &str, max_width: usize) -> String {
    let mut width = 0;
    let mut result = String::new();
    for c in s.chars() {
        let cw = if c as u32 > 0x7F { 2 } else { 1 };
        if width + cw > max_width {
            result.push_str("...");
            return result;
        }
        width += cw;
        result.push(c);
    }
    result
}

fn count_lines(s: &str) -> usize {
    if s.is_empty() {
        0
    } else {
        s.chars().filter(|&c| c == '\n').count() + 1
    }
}

pub fn parse_jsonl_file(path: &Path) -> Vec<SessionEntry> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut title: Option<String> = None;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.is_empty() {
            continue;
        }
        let value: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let msg_type = value.get("type").and_then(|t| t.as_str()).unwrap_or("");

        // Extract title from first meaningful user message
        if msg_type == "user" && title.is_none() {
            if let Some(contents) = value
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_array())
            {
                for content in contents {
                    if content.get("type").and_then(|t| t.as_str()) == Some("text") {
                        if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                            let text = text.trim();
                            // Skip system/IDE context
                            if text.is_empty() || text.starts_with('<') {
                                continue;
                            }
                            let first_line = text.lines().next().unwrap_or(text);
                            let truncated = truncate_display(first_line, 20);
                            title = Some(truncated);
                            break;
                        }
                    }
                }
            }
            continue;
        }

        if msg_type != "assistant" {
            continue;
        }

        let timestamp = value.get("timestamp").and_then(|t| t.as_str()).map(String::from);
        let session_id = value.get("sessionId").and_then(|s| s.as_str()).map(String::from);

        let contents = match value.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_array()) {
            Some(arr) => arr,
            None => continue,
        };

        let mut entry = SessionEntry {
            timestamp,
            session_id,
            ..Default::default()
        };

        for content in contents {
            match content.get("type").and_then(|t| t.as_str()) {
                Some("tool_use") => {
                    let name = content.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    let input = content.get("input");
                    match name {
                        "Write" => {
                            if let Some(c) = input.and_then(|i| i.get("content")).and_then(|c| c.as_str()) {
                                entry.write_lines += count_lines(c);
                            }
                        }
                        "Edit" => {
                            if let Some(ns) = input.and_then(|i| i.get("new_string")).and_then(|s| s.as_str()) {
                                entry.edit_lines += count_lines(ns);
                            }
                        }
                        _ => {}
                    }
                }
                Some("text") => {
                    if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                        entry.text_lines += count_lines(text);
                    }
                }
                _ => {}
            }
        }

        if entry.write_lines > 0 || entry.edit_lines > 0 || entry.text_lines > 0 {
            if entries.is_empty() {
                entry.title = title.take();
            }
            entries.push(entry);
        }
    }

    // If title wasn't assigned yet (first entry had no title), assign to first
    if let Some(t) = title {
        if let Some(first) = entries.first_mut() {
            if first.title.is_none() {
                first.title = Some(t);
            }
        }
    }

    entries
}
