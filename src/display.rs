use crate::stats::{ProjectStats, Stats};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, CellAlignment, Color, Table};

fn fmt_num(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn num_cell(n: usize, bg: Option<Color>) -> Cell {
    let mut cell = Cell::new(fmt_num(n)).set_alignment(CellAlignment::Right);
    if let Some(color) = bg {
        cell = cell.bg(color);
    }
    cell
}

fn text_cell(s: &str, bg: Option<Color>) -> Cell {
    let mut cell = Cell::new(s);
    if let Some(color) = bg {
        cell = cell.bg(color);
    }
    cell
}

fn alt_bg(row_idx: usize) -> Option<Color> {
    if row_idx % 2 == 1 {
        Some(Color::Rgb { r: 40, g: 40, b: 40 })
    } else {
        None
    }
}

pub fn display_summary(projects: &[ProjectStats]) {
    if projects.is_empty() {
        println!("No data found.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("プロジェクト"),
        Cell::new("Write").set_alignment(CellAlignment::Right),
        Cell::new("Edit").set_alignment(CellAlignment::Right),
        Cell::new("チャット").set_alignment(CellAlignment::Right),
        Cell::new("コード計").set_alignment(CellAlignment::Right),
        Cell::new("総合計").set_alignment(CellAlignment::Right),
    ]);

    let mut totals = Stats::default();

    for (i, p) in projects.iter().enumerate() {
        let bg = alt_bg(i);
        table.add_row(vec![
            text_cell(&p.name, bg),
            num_cell(p.stats.write_lines, bg),
            num_cell(p.stats.edit_lines, bg),
            num_cell(p.stats.text_lines, bg),
            num_cell(p.stats.code_total(), bg),
            num_cell(p.stats.total(), bg),
        ]);
        totals.add(&p.stats);
    }

    table.add_row(vec![
        Cell::new("合計"),
        num_cell(totals.write_lines, None),
        num_cell(totals.edit_lines, None),
        num_cell(totals.text_lines, None),
        num_cell(totals.code_total(), None),
        num_cell(totals.total(), None),
    ]);

    println!("{table}");
}

pub fn display_sessions(projects: &[ProjectStats]) {
    if projects.is_empty() {
        println!("No data found.");
        return;
    }

    for p in projects {
        println!(
            "\n=== {} ({} sessions) ===",
            p.name,
            p.sessions.len()
        );

        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec![
            Cell::new("日付"),
            Cell::new("タイトル"),
            Cell::new("Write").set_alignment(CellAlignment::Right),
            Cell::new("Edit").set_alignment(CellAlignment::Right),
            Cell::new("チャット").set_alignment(CellAlignment::Right),
            Cell::new("合計").set_alignment(CellAlignment::Right),
        ]);

        for (i, s) in p.sessions.iter().enumerate() {
            let bg = alt_bg(i);
            table.add_row(vec![
                text_cell(&s.date, bg),
                text_cell(&s.title, bg),
                num_cell(s.stats.write_lines, bg),
                num_cell(s.stats.edit_lines, bg),
                num_cell(s.stats.text_lines, bg),
                num_cell(s.stats.total(), bg),
            ]);
        }

        table.add_row(vec![
            Cell::new(""),
            Cell::new("合計"),
            num_cell(p.stats.write_lines, None),
            num_cell(p.stats.edit_lines, None),
            num_cell(p.stats.text_lines, None),
            num_cell(p.stats.total(), None),
        ]);

        println!("{table}");
    }
}
