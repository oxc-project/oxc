use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
};

use oxc_span::Span;

use crate::json_parser::{JsonObject, JsonProperty, JsonValue};

pub(super) fn is_json_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == OsStr::new("json"))
}

pub(super) fn file_start_span(source_text: &str) -> Span {
    if source_text.is_empty() { Span::default() } else { Span::new(0, 1) }
}

/// Compute the span to delete for a JSON property, including its associated
/// comma and surrounding whitespace so that the remaining JSON stays valid.
#[expect(clippy::cast_possible_truncation)] // Span uses u32 by design
pub(super) fn property_deletion_span(
    source_text: &str,
    object: &JsonObject<'_>,
    prop: &JsonProperty<'_>,
    index: usize,
) -> Span {
    let prop_start = prop.span.start as usize;
    let prop_end = prop.span.end as usize;
    let obj_end = object.span.end as usize;

    // Try to consume trailing comma + whitespace
    let after = &source_text[prop_end..obj_end];
    let trimmed_after = after.trim_start();
    if trimmed_after.starts_with(',') {
        let comma_pos = prop_end + (after.len() - trimmed_after.len());
        let after_comma = &source_text[comma_pos + 1..obj_end];
        let ws_after_comma = after_comma.len() - after_comma.trim_start().len();
        return Span::new(prop_start as u32, (comma_pos + 1 + ws_after_comma) as u32);
    }

    // Last property — consume leading comma + whitespace before it
    if index > 0 {
        let prev_end = object.properties[index - 1].span.end as usize;
        let before = &source_text[prev_end..prop_start];
        let trimmed_before = before.trim_end();
        if trimmed_before.ends_with(',') {
            let comma_pos = prev_end + trimmed_before.len() - 1;
            return Span::new(comma_pos as u32, prop_end as u32);
        }
    }

    // Only property — just delete it
    Span::new(prop_start as u32, prop_end as u32)
}

fn join_object_path(parent: &str, key: &str) -> String {
    if parent.is_empty() { key.to_string() } else { format!("{parent}.{key}") }
}

fn join_array_path(parent: &str, index: usize) -> String {
    if parent.is_empty() { format!("[{index}]") } else { format!("{parent}[{index}]") }
}

fn display_path(path: &str) -> &str {
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
    reference: &JsonValue<'_>,
    candidate: &JsonValue<'_>,
    path: &str,
    diff: &mut JsonShapeDiff,
) {
    match (reference, candidate) {
        (JsonValue::Object(reference), JsonValue::Object(candidate)) => {
            for ref_prop in &reference.properties {
                let child_path = join_object_path(path, ref_prop.key);
                match candidate.get(ref_prop.key) {
                    Some(candidate_value) => {
                        compare_json_shapes(&ref_prop.value, candidate_value, &child_path, diff);
                    }
                    None => diff.missing.push(child_path),
                }
            }

            for cand_prop in &candidate.properties {
                if reference.get(cand_prop.key).is_none() {
                    diff.extra.push(join_object_path(path, cand_prop.key));
                }
            }
        }
        (JsonValue::Array(reference), JsonValue::Array(candidate)) => {
            let shared_len = reference.elements.len().min(candidate.elements.len());
            for index in 0..shared_len {
                compare_json_shapes(
                    &reference.elements[index],
                    &candidate.elements[index],
                    &join_array_path(path, index),
                    diff,
                );
            }

            for index in shared_len..reference.elements.len() {
                diff.missing.push(join_array_path(path, index));
            }

            for index in shared_len..candidate.elements.len() {
                diff.extra.push(join_array_path(path, index));
            }
        }
        (JsonValue::Object(_) | JsonValue::Array(_), _)
        | (_, JsonValue::Object(_) | JsonValue::Array(_)) => {
            diff.type_mismatches.push(display_path(path).to_string());
        }
        _ => {}
    }
}
