use std::sync::LazyLock;

use crate::types::{DiffLine, Hunk, ParsedDiff};

static HUNK_HEADER_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@(.*)$")
        .expect("invalid hunk header regex")
});

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
    let hunk_header_re = &*HUNK_HEADER_RE;

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

        // File headers only appear outside of hunks
        if current_hunk.is_none() {
            if let Some(path) = line.strip_prefix("+++ b/") {
                current_file = Some(path.to_string());
                continue;
            }
            if line.starts_with("+++ /dev/null") {
                continue;
            }
            if line.starts_with("--- a/") || line.starts_with("--- /dev/null") {
                if current_file.is_none() {
                    if let Some(path) = line.strip_prefix("--- a/") {
                        current_file = Some(path.to_string());
                    }
                }
                continue;
            }
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

            let file_path = current_file
                .clone()
                .unwrap_or_else(|| "unknown".to_string());

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
            if let Some(text) = line.strip_prefix('+') {
                hb.lines.push(DiffLine {
                    kind: "add".to_string(),
                    old_line: None,
                    new_line: Some(hb.new_line),
                    text: text.to_string(),
                });
                hb.new_line += 1;
            } else if let Some(text) = line.strip_prefix('-') {
                hb.lines.push(DiffLine {
                    kind: "remove".to_string(),
                    old_line: Some(hb.old_line),
                    new_line: None,
                    text: text.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_file_single_hunk() {
        let diff = "\
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!(\"hello\");
     let x = 1;
 }";
        let hunks = parse_unified_diff(diff).unwrap();
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].id, "H1");
        assert_eq!(hunks[0].file_path, "src/main.rs");
        assert_eq!(hunks[0].old_start, 1);
        assert_eq!(hunks[0].old_lines, 3);
        assert_eq!(hunks[0].new_start, 1);
        assert_eq!(hunks[0].new_lines, 4);
        assert_eq!(hunks[0].lines.len(), 4);

        assert_eq!(hunks[0].lines[0].kind, "context");
        assert_eq!(hunks[0].lines[0].old_line, Some(1));
        assert_eq!(hunks[0].lines[0].new_line, Some(1));

        assert_eq!(hunks[0].lines[1].kind, "add");
        assert_eq!(hunks[0].lines[1].old_line, None);
        assert_eq!(hunks[0].lines[1].new_line, Some(2));
        assert_eq!(hunks[0].lines[1].text, "    println!(\"hello\");");

        assert_eq!(hunks[0].lines[2].kind, "context");
        assert_eq!(hunks[0].lines[3].kind, "context");
    }

    #[test]
    fn single_file_multiple_hunks() {
        let diff = "\
diff --git a/f.rs b/f.rs
--- a/f.rs
+++ b/f.rs
@@ -1,3 +1,4 @@
 a
+b
 c
 d
@@ -10,3 +11,4 @@
 x
+y
 z
 w";
        let hunks = parse_unified_diff(diff).unwrap();
        assert_eq!(hunks.len(), 2);
        assert_eq!(hunks[0].id, "H1");
        assert_eq!(hunks[1].id, "H2");
        assert_eq!(hunks[0].old_start, 1);
        assert_eq!(hunks[1].old_start, 10);
    }

    #[test]
    fn multiple_files() {
        let diff = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,2 +1,3 @@
 a
+b
 c
diff --git a/b.rs b/b.rs
--- a/b.rs
+++ b/b.rs
@@ -1,2 +1,3 @@
 x
+y
 z";
        let hunks = parse_unified_diff(diff).unwrap();
        assert_eq!(hunks.len(), 2);
        assert_eq!(hunks[0].file_path, "a.rs");
        assert_eq!(hunks[1].file_path, "b.rs");
    }

    #[test]
    fn global_hunk_id_counter() {
        let diff = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,2 +1,3 @@
 a
+b
 c
diff --git a/b.rs b/b.rs
--- a/b.rs
+++ b/b.rs
@@ -1,2 +1,3 @@
 x
+y
 z
@@ -10,2 +11,3 @@
 p
+q
 r";
        let hunks = parse_unified_diff(diff).unwrap();
        assert_eq!(hunks.len(), 3);
        assert_eq!(hunks[0].id, "H1");
        assert_eq!(hunks[1].id, "H2");
        assert_eq!(hunks[2].id, "H3");
    }

    #[test]
    fn empty_input() {
        let hunks = parse_unified_diff("").unwrap();
        assert!(hunks.is_empty());
    }

    #[test]
    fn no_newline_at_end_of_file_skipped() {
        let diff = "\
diff --git a/f.rs b/f.rs
--- a/f.rs
+++ b/f.rs
@@ -1,2 +1,2 @@
-old
+new
\\ No newline at end of file";
        let hunks = parse_unified_diff(diff).unwrap();
        assert_eq!(hunks.len(), 1);
        // The backslash line should be skipped — only remove + add = 2 lines
        assert_eq!(hunks[0].lines.len(), 2);
    }

    #[test]
    fn file_deletion_uses_minus_path() {
        let diff = "\
diff --git a/deleted.rs b/deleted.rs
--- a/deleted.rs
+++ /dev/null
@@ -1,2 +0,0 @@
-line1
-line2";
        let hunks = parse_unified_diff(diff).unwrap();
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].file_path, "deleted.rs");
    }

    #[test]
    fn hunk_header_with_function_name() {
        let diff = "\
diff --git a/f.rs b/f.rs
--- a/f.rs
+++ b/f.rs
@@ -10,3 +10,4 @@ fn some_function() {
 existing
+added
 end
 last";
        let hunks = parse_unified_diff(diff).unwrap();
        assert_eq!(hunks.len(), 1);
        assert!(hunks[0].header.contains("fn some_function()"));
    }

    #[test]
    fn hunk_header_no_comma_defaults_to_one_line() {
        let diff = "\
diff --git a/f.rs b/f.rs
--- a/f.rs
+++ b/f.rs
@@ -1 +1 @@
-old
+new";
        let hunks = parse_unified_diff(diff).unwrap();
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].old_lines, 1);
        assert_eq!(hunks[0].new_lines, 1);
    }

    #[test]
    fn remove_lines_have_old_line_numbers() {
        let diff = "\
diff --git a/f.rs b/f.rs
--- a/f.rs
+++ b/f.rs
@@ -5,3 +5,2 @@
 keep
-removed
 end";
        let hunks = parse_unified_diff(diff).unwrap();
        let rm = hunks[0].lines.iter().find(|l| l.kind == "remove").unwrap();
        assert_eq!(rm.old_line, Some(6));
        assert_eq!(rm.new_line, None);
        assert_eq!(rm.text, "removed");
    }

    #[test]
    fn empty_lines_are_context() {
        let diff = "\
diff --git a/f.rs b/f.rs
--- a/f.rs
+++ b/f.rs
@@ -1,3 +1,3 @@
 a

 b";
        let hunks = parse_unified_diff(diff).unwrap();
        assert_eq!(hunks[0].lines.len(), 3);
        assert_eq!(hunks[0].lines[1].kind, "context");
        assert_eq!(hunks[0].lines[1].text, "");
    }

    #[test]
    fn new_file_has_only_adds() {
        let diff = "\
diff --git a/new.rs b/new.rs
--- /dev/null
+++ b/new.rs
@@ -0,0 +1,2 @@
+line1
+line2";
        let hunks = parse_unified_diff(diff).unwrap();
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].file_path, "new.rs");
        assert!(hunks[0].lines.iter().all(|l| l.kind == "add"));
    }
}
