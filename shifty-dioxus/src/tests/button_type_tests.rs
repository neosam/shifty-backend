//! Regression guard: every raw `<button>` rendered via rsx must carry an
//! explicit `r#type: "button"` attribute.
//!
//! Background: in HTML a `<button>` without an explicit `type` defaults to
//! `type="submit"`. When such a button sits inside (or is later wrapped by) a
//! `<form>`, clicking it submits the form and triggers a full page reload —
//! exactly the "+" button bug observed in the shift plan. Forcing
//! `type="button"` on every raw button neutralises that default behaviour.
//!
//! This test scans the component/page source for raw `button {` openings and
//! fails if any of them lacks a `r#type:` attribute. The canonical
//! `atoms::btn::Btn` / `atoms::nav_btn` components are included in the scan as
//! well — they set the attribute themselves.

use std::fs;
use std::path::{Path, PathBuf};

/// Collect all `.rs` files under `src/`.
fn rust_sources() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read_dir src") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    out
}

/// Find `button {` openings whose attribute block does not contain `r#type:`.
///
/// Handles both the multiline form (`button {` on its own line) and the inline
/// form (`button { class: ..., "x" }`). Returns `file:line` offenders.
fn buttons_without_type(path: &Path) -> Vec<String> {
    let content = fs::read_to_string(path).expect("read source");
    let lines: Vec<&str> = content.lines().collect();
    let mut offenders = Vec::new();

    for (idx, raw) in lines.iter().enumerate() {
        let trimmed = raw.trim_start();
        if !trimmed.starts_with("button {") {
            continue;
        }

        // Inline form: everything for this button is on one line.
        if trimmed != "button {" {
            if !raw.contains("r#type:") {
                offenders.push(format!("{}:{}", path.display(), idx + 1));
            }
            continue;
        }

        // Multiline form: scan this button's direct attributes (depth == 1)
        // until it closes, looking for `r#type:` before any child opens.
        let mut depth = 1i32;
        let mut has_type = false;
        let mut j = idx + 1;
        while j < lines.len() && depth > 0 {
            let seg = lines[j];
            if depth == 1 && seg.contains("r#type:") {
                has_type = true;
                break;
            }
            depth += seg.matches('{').count() as i32 - seg.matches('}').count() as i32;
            if depth <= 0 {
                break;
            }
            j += 1;
        }
        if !has_type {
            offenders.push(format!("{}:{}", path.display(), idx + 1));
        }
    }
    offenders
}

#[test]
fn all_raw_buttons_have_explicit_type() {
    let mut offenders = Vec::new();
    for path in rust_sources() {
        offenders.extend(buttons_without_type(&path));
    }
    assert!(
        offenders.is_empty(),
        "raw <button> elements missing `r#type: \"button\"` (would default to \
         type=submit and reload the page inside a form):\n{}",
        offenders.join("\n")
    );
}
