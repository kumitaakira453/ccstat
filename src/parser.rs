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

        if value.get("type").and_then(|t| t.as_str()) != Some("assistant") {
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
            entries.push(entry);
        }
    }

    entries
}
