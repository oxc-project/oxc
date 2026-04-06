use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
};

use oxc_span::Span;
use serde_json::Value;

pub(super) fn is_json_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == OsStr::new("json"))
}

pub(super) fn error_span(source_text: &str, line: usize, column: usize) -> Span {
    if source_text.is_empty() {
        return Span::default();
    }

    let line = line.max(1);
    let column = column.max(1);

    let mut current_line = 1usize;
    let mut line_start = 0usize;

    for segment in source_text.split_inclusive('\n') {
        if current_line == line {
            break;
        }
        line_start += segment.len();
        current_line += 1;
    }

    let line_end = source_text[line_start..]
        .find('\n')
        .map_or(source_text.len(), |offset| line_start + offset);
    let line_len = source_text[line_start..line_end].chars().count();
    let column_offset = column.saturating_sub(1).min(line_len);
    let start = line_start
        + source_text[line_start..line_end]
            .chars()
            .take(column_offset)
            .map(char::len_utf8)
            .sum::<usize>();

    let start = start.min(source_text.len().saturating_sub(1));
    Span::new(start as u32, (start + 1) as u32)
}

pub(super) fn file_start_span(source_text: &str) -> Span {
    if source_text.is_empty() { Span::default() } else { Span::new(0, 1) }
}

pub(super) fn join_object_path(parent: &str, key: &str) -> String {
    if parent.is_empty() { key.to_string() } else { format!("{parent}.{key}") }
}

pub(super) fn join_array_path(parent: &str, index: usize) -> String {
    if parent.is_empty() { format!("[{index}]") } else { format!("{parent}[{index}]") }
}

pub(super) fn display_path(path: &str) -> &str {
    if path.is_empty() { "<root>" } else { path }
}

pub(super) fn resolve_reference_path(current_file: &Path, raw_path: &str) -> PathBuf {
    let reference_path = Path::new(raw_path);
    if reference_path.is_absolute() {
        return normalize_path(reference_path);
    }

    let combined = current_file
        .parent()
        .map_or_else(|| reference_path.to_path_buf(), |parent| parent.join(reference_path));

    normalize_path(&combined)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
            _ => normalized.push(component.as_os_str()),
        }
    }

    normalized
}

#[derive(Debug, Default)]
pub(super) struct JsonShapeDiff {
    pub missing: Vec<String>,
    pub extra: Vec<String>,
    pub type_mismatches: Vec<String>,
}

pub(super) fn compare_json_shapes(
    reference: &Value,
    candidate: &Value,
    path: &str,
    diff: &mut JsonShapeDiff,
) {
    match (reference, candidate) {
        (Value::Object(reference), Value::Object(candidate)) => {
            let mut reference_keys = reference.keys().collect::<Vec<_>>();
            reference_keys.sort_unstable();

            let mut candidate_keys = candidate.keys().collect::<Vec<_>>();
            candidate_keys.sort_unstable();

            for key in &reference_keys {
                let child_path = join_object_path(path, key);
                match candidate.get(*key) {
                    Some(candidate_value) => {
                        let reference_value = reference.get(*key).expect("reference key exists");
                        compare_json_shapes(reference_value, candidate_value, &child_path, diff);
                    }
                    None => diff.missing.push(child_path),
                }
            }

            for key in candidate_keys {
                if !reference.contains_key(key) {
                    diff.extra.push(join_object_path(path, key));
                }
            }
        }
        (Value::Array(reference), Value::Array(candidate)) => {
            let shared_len = reference.len().min(candidate.len());
            for index in 0..shared_len {
                compare_json_shapes(
                    &reference[index],
                    &candidate[index],
                    &join_array_path(path, index),
                    diff,
                );
            }

            for index in shared_len..reference.len() {
                diff.missing.push(join_array_path(path, index));
            }

            for index in shared_len..candidate.len() {
                diff.extra.push(join_array_path(path, index));
            }
        }
        (Value::Object(_) | Value::Array(_), _) | (_, Value::Object(_) | Value::Array(_)) => {
            diff.type_mismatches.push(display_path(path).to_string());
        }
        _ => {}
    }
}
