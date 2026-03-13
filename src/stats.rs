use crate::parser::{parse_jsonl_file, SessionEntry};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub write_lines: usize,
    pub edit_lines: usize,
    pub text_lines: usize,
}

impl Stats {
    pub fn code_total(&self) -> usize {
        self.write_lines + self.edit_lines
    }

    pub fn total(&self) -> usize {
        self.write_lines + self.edit_lines + self.text_lines
    }

    pub fn add(&mut self, other: &Stats) {
        self.write_lines += other.write_lines;
        self.edit_lines += other.edit_lines;
        self.text_lines += other.text_lines;
    }
}

#[derive(Debug)]
pub struct SessionStats {
    pub date: String,
    pub title: String,
    pub stats: Stats,
}

#[derive(Debug)]
pub struct ProjectStats {
    pub name: String,
    pub stats: Stats,
    pub sessions: Vec<SessionStats>,
}

fn extract_project_name(dir_name: &str) -> String {
    // Directory names like "-Users-kumitaakira-demia-works-wasurenai"
    // Extract last meaningful segment
    let parts: Vec<&str> = dir_name.split('-').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return dir_name.to_string();
    }
    // Find the last segment after "Users-username-..." pattern
    // Look for parts after known prefixes
    let skip = parts.iter().position(|&p| {
        !["Users", "kumitaakira", "demia", "works"].contains(&p)
            && !p.chars().all(|c| c.is_uppercase() || c == '_')
    });
    match skip {
        Some(idx) => parts[idx..].join("-"),
        None => parts.last().unwrap_or(&dir_name).to_string(),
    }
}

fn find_jsonl_files(project_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let entries = match fs::read_dir(project_dir) {
        Ok(e) => e,
        Err(_) => return files,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            files.push(path);
        } else if path.is_dir() {
            // Check for subagents directory
            let subagents_dir = path.join("subagents");
            if subagents_dir.is_dir() {
                if let Ok(sub_entries) = fs::read_dir(&subagents_dir) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                            files.push(sub_path);
                        }
                    }
                }
            }
        }
    }

    files
}

fn aggregate_entries(entries: &[SessionEntry]) -> (Stats, String, String, Option<String>) {
    let mut stats = Stats::default();
    let mut earliest_ts = String::new();
    let mut session_id = String::new();
    let mut title = None;

    for e in entries {
        stats.write_lines += e.write_lines;
        stats.edit_lines += e.edit_lines;
        stats.text_lines += e.text_lines;

        if let Some(ref ts) = e.timestamp {
            if earliest_ts.is_empty() || ts < &earliest_ts {
                earliest_ts = ts.clone();
            }
        }
        if session_id.is_empty() {
            if let Some(ref sid) = e.session_id {
                session_id = sid.clone();
            }
        }
        if title.is_none() {
            if let Some(ref t) = e.title {
                title = Some(t.clone());
            }
        }
    }

    (stats, earliest_ts, session_id, title)
}

fn extract_date(ts: &str) -> String {
    // ISO 8601: "2026-02-27T..." -> "2026-02-27"
    ts.get(..10).unwrap_or("unknown").to_string()
}

fn short_session_id(sid: &str) -> String {
    sid.get(..8).unwrap_or(sid).to_string()
}

pub fn collect_stats(projects_dir: &Path, project_filter: Option<&str>) -> Vec<ProjectStats> {
    // project_name -> (session_id -> (Stats, earliest_timestamp, title))
    let mut project_sessions: BTreeMap<String, BTreeMap<String, (Stats, String, Option<String>)>> =
        BTreeMap::new();

    let project_dirs = match fs::read_dir(projects_dir) {
        Ok(entries) => entries,
        Err(_) => {
            eprintln!("Cannot read directory: {}", projects_dir.display());
            return vec![];
        }
    };

    for entry in project_dirs.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();
        let project_name = extract_project_name(&dir_name);

        if let Some(filter) = project_filter {
            if !project_name.contains(filter) && !dir_name.contains(filter) {
                continue;
            }
        }

        let jsonl_files = find_jsonl_files(&path);

        for jsonl_path in &jsonl_files {
            let entries = parse_jsonl_file(jsonl_path);
            if entries.is_empty() {
                continue;
            }

            let (file_stats, earliest_ts, session_id, file_title) =
                aggregate_entries(&entries);
            if file_stats.total() == 0 {
                continue;
            }

            let sid = if session_id.is_empty() {
                jsonl_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            } else {
                session_id
            };

            let sessions = project_sessions
                .entry(project_name.clone())
                .or_default();

            let entry = sessions
                .entry(sid)
                .or_insert_with(|| (Stats::default(), earliest_ts.clone(), None));
            entry.0.add(&file_stats);
            if !earliest_ts.is_empty() && (entry.1.is_empty() || earliest_ts < entry.1) {
                entry.1 = earliest_ts;
            }
            if entry.2.is_none() {
                entry.2 = file_title;
            }
        }
    }

    // Build ProjectStats from grouped data
    let mut results: Vec<ProjectStats> = project_sessions
        .into_iter()
        .map(|(name, sessions)| {
            let mut project_stats = Stats::default();
            let mut session_list: Vec<SessionStats> = sessions
                .into_iter()
                .map(|(sid, (stats, ts, title))| {
                    project_stats.add(&stats);
                    let display_title = title.unwrap_or_else(|| short_session_id(&sid));
                    SessionStats {
                        date: extract_date(&ts),
                        title: display_title,
                        stats,
                    }
                })
                .collect();
            session_list.sort_by(|a, b| b.date.cmp(&a.date));
            ProjectStats {
                name,
                stats: project_stats,
                sessions: session_list,
            }
        })
        .collect();

    results.sort_by(|a, b| b.stats.total().cmp(&a.stats.total()));
    results
}
