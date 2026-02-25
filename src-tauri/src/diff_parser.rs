use crate::types::{DiffLine, Hunk, ParsedDiff};

struct HunkBuilder {
    file_path: String,
    header: String,
    old_start: u32,
    old_lines: u32,
    new_start: u32,
    new_lines: u32,
    lines: Vec<DiffLine>,
    old_line: u32,
    new_line: u32,
}

fn flush_hunk(builder: HunkBuilder, counter: &mut u32, hunks: &mut Vec<Hunk>) {
    *counter += 1;
    hunks.push(Hunk {
        id: format!("H{}", counter),
        file_path: builder.file_path,
        header: builder.header,
        old_start: builder.old_start,
        old_lines: builder.old_lines,
        new_start: builder.new_start,
        new_lines: builder.new_lines,
        lines: builder.lines,
    });
}

#[tauri::command]
pub fn parse_diff(diff_text: String) -> Result<ParsedDiff, String> {
    let hunks = parse_unified_diff(&diff_text)?;
    Ok(ParsedDiff {
        hunks,
        raw: diff_text,
    })
}

fn parse_unified_diff(diff_text: &str) -> Result<Vec<Hunk>, String> {
    let hunk_header_re =
        regex::Regex::new(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@(.*)$").unwrap();

    let mut hunks: Vec<Hunk> = Vec::new();
    let mut current_file: Option<String> = None;
    let mut hunk_counter: u32 = 0;
    let mut current_hunk: Option<HunkBuilder> = None;

    for line in diff_text.lines() {
        if line.starts_with("diff --git ") || line.starts_with("diff --combined ") {
            if let Some(hb) = current_hunk.take() {
                flush_hunk(hb, &mut hunk_counter, &mut hunks);
            }
            current_file = None;
            continue;
        }

        if line.starts_with("+++ b/") {
            current_file = Some(line[6..].to_string());
            continue;
        }
        if line.starts_with("+++ /dev/null") {
            continue;
        }
        if line.starts_with("--- a/") || line.starts_with("--- /dev/null") {
            if current_file.is_none() && line.starts_with("--- a/") {
                current_file = Some(line[6..].to_string());
            }
            continue;
        }

        // Hunk header
        if let Some(caps) = hunk_header_re.captures(line) {
            if let Some(hb) = current_hunk.take() {
                flush_hunk(hb, &mut hunk_counter, &mut hunks);
            }

            let old_start: u32 = caps[1].parse().unwrap_or(0);
            let old_lines: u32 = caps.get(2).map_or(1, |m| m.as_str().parse().unwrap_or(1));
            let new_start: u32 = caps[3].parse().unwrap_or(0);
            let new_lines: u32 = caps.get(4).map_or(1, |m| m.as_str().parse().unwrap_or(1));

            let file_path = current_file.clone().unwrap_or_else(|| "unknown".to_string());

            current_hunk = Some(HunkBuilder {
                file_path,
                header: line.to_string(),
                old_start,
                old_lines,
                new_start,
                new_lines,
                lines: Vec::new(),
                old_line: old_start,
                new_line: new_start,
            });
            continue;
        }

        // Diff content lines
        if let Some(ref mut hb) = current_hunk {
            if line.starts_with('+') {
                hb.lines.push(DiffLine {
                    kind: "add".to_string(),
                    old_line: None,
                    new_line: Some(hb.new_line),
                    text: line[1..].to_string(),
                });
                hb.new_line += 1;
            } else if line.starts_with('-') {
                hb.lines.push(DiffLine {
                    kind: "remove".to_string(),
                    old_line: Some(hb.old_line),
                    new_line: None,
                    text: line[1..].to_string(),
                });
                hb.old_line += 1;
            } else if line.starts_with(' ') || line.is_empty() {
                let text = if line.is_empty() {
                    String::new()
                } else {
                    line[1..].to_string()
                };
                hb.lines.push(DiffLine {
                    kind: "context".to_string(),
                    old_line: Some(hb.old_line),
                    new_line: Some(hb.new_line),
                    text,
                });
                hb.old_line += 1;
                hb.new_line += 1;
            } else if line.starts_with('\\') {
                // "\ No newline at end of file" — skip
                continue;
            }
        }
    }

    // Flush last hunk
    if let Some(hb) = current_hunk.take() {
        flush_hunk(hb, &mut hunk_counter, &mut hunks);
    }

    Ok(hunks)
}
