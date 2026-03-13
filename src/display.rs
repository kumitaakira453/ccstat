use crate::stats::{ProjectStats, Stats};

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

pub fn display_summary(projects: &[ProjectStats]) {
    if projects.is_empty() {
        println!("No data found.");
        return;
    }

    let header = [
        "プロジェクト",
        "Write",
        "Edit",
        "チャット",
        "コード計",
        "総合計",
    ];

    // Calculate column widths
    let mut col_widths: Vec<usize> = header.iter().map(|h| display_width(h)).collect();

    let mut totals = Stats::default();

    for p in projects {
        col_widths[0] = col_widths[0].max(display_width(&p.name));
        col_widths[1] = col_widths[1].max(fmt_num(p.stats.write_lines).len());
        col_widths[2] = col_widths[2].max(fmt_num(p.stats.edit_lines).len());
        col_widths[3] = col_widths[3].max(fmt_num(p.stats.text_lines).len());
        col_widths[4] = col_widths[4].max(fmt_num(p.stats.code_total()).len());
        col_widths[5] = col_widths[5].max(fmt_num(p.stats.total()).len());
        totals.add(&p.stats);
    }

    // Also consider totals row
    col_widths[0] = col_widths[0].max(display_width("合計"));
    col_widths[1] = col_widths[1].max(fmt_num(totals.write_lines).len());
    col_widths[2] = col_widths[2].max(fmt_num(totals.edit_lines).len());
    col_widths[3] = col_widths[3].max(fmt_num(totals.text_lines).len());
    col_widths[4] = col_widths[4].max(fmt_num(totals.code_total()).len());
    col_widths[5] = col_widths[5].max(fmt_num(totals.total()).len());

    // Print table
    print_border(&col_widths, '┌', '┬', '┐');
    print_row_header(&header, &col_widths);
    print_border(&col_widths, '├', '┼', '┤');

    for p in projects {
        print_data_row(
            &p.name,
            &[
                fmt_num(p.stats.write_lines),
                fmt_num(p.stats.edit_lines),
                fmt_num(p.stats.text_lines),
                fmt_num(p.stats.code_total()),
                fmt_num(p.stats.total()),
            ],
            &col_widths,
        );
    }

    print_border(&col_widths, '├', '┼', '┤');
    print_data_row(
        "合計",
        &[
            fmt_num(totals.write_lines),
            fmt_num(totals.edit_lines),
            fmt_num(totals.text_lines),
            fmt_num(totals.code_total()),
            fmt_num(totals.total()),
        ],
        &col_widths,
    );
    print_border(&col_widths, '└', '┴', '┘');
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
        println!(
            "  {:<12} {:<12} {:>8} {:>8} {:>8} {:>8}",
            "日付", "Session", "Write", "Edit", "チャット", "合計"
        );

        for s in &p.sessions {
            println!(
                "  {:<12} {:<12} {:>8} {:>8} {:>8} {:>8}",
                s.date,
                s.session_id_short,
                fmt_num(s.stats.write_lines),
                fmt_num(s.stats.edit_lines),
                fmt_num(s.stats.text_lines),
                fmt_num(s.stats.total()),
            );
        }

        println!(
            "  {:<12} {:<12} {:>8} {:>8} {:>8} {:>8}",
            "", "合計",
            fmt_num(p.stats.write_lines),
            fmt_num(p.stats.edit_lines),
            fmt_num(p.stats.text_lines),
            fmt_num(p.stats.total()),
        );
    }
}

/// Calculate display width accounting for multi-byte characters (CJK = width 2)
fn display_width(s: &str) -> usize {
    s.chars()
        .map(|c| if c as u32 > 0x7F { 2 } else { 1 })
        .sum()
}

/// Pad string to target display width
fn pad_to_width(s: &str, target: usize) -> String {
    let w = display_width(s);
    if w >= target {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(target - w))
    }
}

fn print_border(widths: &[usize], left: char, mid: char, right: char) {
    print!("{}", left);
    for (i, w) in widths.iter().enumerate() {
        print!("{}", "─".repeat(w + 2));
        if i < widths.len() - 1 {
            print!("{}", mid);
        }
    }
    println!("{}", right);
}

fn print_row_header(cells: &[&str], widths: &[usize]) {
    print!("│");
    for (i, cell) in cells.iter().enumerate() {
        if i == 0 {
            print!(" {} │", pad_to_width(cell, widths[i]));
        } else {
            let val = cell.to_string();
            let pad = widths[i].saturating_sub(display_width(&val));
            print!(" {}{} │", " ".repeat(pad), val);
        }
    }
    println!();
}

fn print_data_row(name: &str, values: &[String], widths: &[usize]) {
    print!("│ {} │", pad_to_width(name, widths[0]));
    for (i, val) in values.iter().enumerate() {
        let pad = widths[i + 1].saturating_sub(val.len());
        print!(" {}{} │", " ".repeat(pad), val);
    }
    println!();
}
