use rustc_hash::FxHashMap;
use serde_json::{Value, json};

use oxc_formatter::{
    BestFittingElement, DedentMode, FormatElement, GroupId, LineMode, PrintMode, Tag,
};

// TODO: Currently, we build a Prettier Doc tree using `serde_json::Value`,
// then serialize it to a string.
//
// It might be more efficient to construct the Doc string directly.
// This would also allow for optimizations like:
// - caching interned elements
// - reusing constant Doc structures like `{type: "line", hard: true}`

/// Splits a printed string by newlines and joins with Prettier `hardline` docs.
pub fn printed_string_to_hardline_doc(text: &str) -> Value {
    // `lines()` will remove trailing newlines, but it is fine.
    // For js-in-xxx fragments, do not need to preserve trailing newlines.
    let lines: Vec<&str> = text.lines().collect();

    if lines.len() <= 1 {
        return Value::String(text.to_string());
    }

    let mut parts: Vec<Value> = Vec::with_capacity(lines.len() * 3);
    for (i, line) in lines.iter().enumerate() {
        if 0 < i {
            // hardline = [{ type: "line", hard: true } + break-parent]
            // https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/document/builders/line.js#L27
            parts.push(json!({"type": "line", "hard": true}));
            parts.push(json!({"type": "break-parent"}));
        }
        parts.push(Value::String((*line).to_string()));
    }

    normalize_array(parts)
}

/// Converts `oxc_formatter` IR (`FormatElement` slice) into Prettier Doc.
///
/// This is used for js-in-xxx fragment formatting
/// where the IR must be returned to Prettier as an unresolved Doc (rather than a printed string)
/// so that Prettier can handle line-wrapping in the parent context.
pub fn format_elements_to_prettier_doc(
    elements: &[FormatElement],
    sorted_tailwind_classes: &[String],
) -> Result<Value, String> {
    let mut state = ConvertState::new(sorted_tailwind_classes);
    let children = convert_elements(elements, &mut state)?;
    Ok(normalize_array(children))
}

type InternedCacheKey = (usize, usize);

struct ConvertState<'a> {
    sorted_tailwind_classes: &'a [String],
    interned_cache: FxHashMap<InternedCacheKey, Vec<Value>>,
}

impl<'a> ConvertState<'a> {
    fn new(sorted_tailwind_classes: &'a [String]) -> Self {
        Self { sorted_tailwind_classes, interned_cache: FxHashMap::default() }
    }
}

/// Stack entry that stores start tag info alongside accumulated children.
struct StackEntry {
    start_tag: Option<Tag>,
    start_info: Option<StartTagInfo>,
    children: Vec<Value>,
}

/// Information extracted from start tags, needed when processing the matching end tag.
enum StartTagInfo {
    Indent,
    Align(u8),
    Dedent(DedentMode),
    Group { id: Option<GroupId>, should_break: bool },
    ConditionalContent { mode: PrintMode, group_id: Option<GroupId> },
    IndentIfGroupBreaks(GroupId),
    Fill,
    Entry,
    LineSuffix,
    Labelled,
}

fn convert_elements(
    elements: &[FormatElement],
    state: &mut ConvertState,
) -> Result<Vec<Value>, String> {
    let mut stack: Vec<StackEntry> =
        vec![StackEntry { start_tag: None, start_info: None, children: vec![] }];

    for element in elements {
        match element {
            FormatElement::Space | FormatElement::HardSpace => {
                concat_string(current_children_mut(&mut stack)?, " ");
            }
            FormatElement::Token { text } => {
                concat_string(current_children_mut(&mut stack)?, text);
            }
            FormatElement::Text { text, .. } => {
                push_text(current_children_mut(&mut stack)?, text);
            }
            FormatElement::Line(mode) => {
                push_line(current_children_mut(&mut stack)?, *mode);
            }
            FormatElement::ExpandParent => {
                current_children_mut(&mut stack)?.push(json!({"type": "break-parent"}));
            }
            FormatElement::LineSuffixBoundary => {
                current_children_mut(&mut stack)?.push(json!({"type": "line-suffix-boundary"}));
            }
            FormatElement::Tag(tag) => {
                if tag.is_start() {
                    stack.push(StackEntry {
                        start_tag: Some(tag.clone()),
                        start_info: Some(extract_start_tag_info(tag)),
                        children: vec![],
                    });
                } else {
                    if stack.len() == 1 {
                        return Err(format!(
                            "Invalid formatter IR: found unmatched end tag `{tag:?}` while converting to Prettier Doc"
                        ));
                    }
                    let entry = stack.pop().ok_or_else(|| {
                        "Invalid formatter IR: stack underflow while processing end tag".to_string()
                    })?;
                    if let Some(start_tag) = &entry.start_tag
                        && start_tag.kind() != tag.kind()
                    {
                        return Err(format!(
                            "Invalid formatter IR: mismatched tags (start: `{start_tag:?}`, end: `{tag:?}`)"
                        ));
                    }
                    let doc = build_doc(entry.start_info.as_ref(), entry.children);
                    current_children_mut(&mut stack)?.push(doc);
                }
            }
            FormatElement::Interned(interned) => {
                let key = interned_cache_key(interned);
                if let Some(cached) = state.interned_cache.get(&key) {
                    current_children_mut(&mut stack)?.extend(cached.iter().cloned());
                } else {
                    let converted = convert_elements(interned, state)?;
                    state.interned_cache.insert(key, converted.clone());
                    current_children_mut(&mut stack)?.extend(converted);
                }
            }
            FormatElement::BestFitting(best_fitting) => {
                let doc = convert_best_fitting(best_fitting, state)?;
                current_children_mut(&mut stack)?.push(doc);
            }
            FormatElement::TailwindClass(index) => {
                if let Some(class) = state.sorted_tailwind_classes.get(*index) {
                    concat_string(current_children_mut(&mut stack)?, class);
                }
            }
        }
    }

    if stack.len() != 1 {
        if let Some(unclosed_tag) = stack.iter().rev().find_map(|entry| entry.start_tag.as_ref()) {
            return Err(format!(
                "Invalid formatter IR: unclosed start tag `{unclosed_tag:?}` while converting to Prettier Doc"
            ));
        }
        return Err(
            "Invalid formatter IR: unclosed tags while converting to Prettier Doc".to_string()
        );
    }

    stack.pop().map_or_else(
        || Err("Invalid formatter IR: missing root stack entry".to_string()),
        |e| Ok(e.children),
    )
}

fn current_children_mut(stack: &mut [StackEntry]) -> Result<&mut Vec<Value>, String> {
    stack.last_mut().map(|entry| &mut entry.children).ok_or_else(|| {
        "Invalid formatter IR: stack underflow while appending converted doc".to_string()
    })
}

fn interned_cache_key(elements: &[FormatElement]) -> InternedCacheKey {
    (elements.as_ptr() as usize, elements.len())
}

fn push_text(children: &mut Vec<Value>, text: &str) {
    // `FormatElement::Text` may contain embedded newlines (e.g., template literals).
    // Convert them to Prettier's `literalline` docs instead of raw '\n' in strings,
    // so parent groups can remeasure and break correctly.
    if !text.as_bytes().iter().any(|b| *b == b'\n' || *b == b'\r') {
        concat_string(children, text);
        return;
    }

    let bytes = text.as_bytes();
    let mut segment_start = 0;
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                if segment_start < i {
                    concat_string(children, &text[segment_start..i]);
                }
                push_literal_line(children);
                i += 1;
                segment_start = i;
            }
            b'\r' => {
                if segment_start < i {
                    concat_string(children, &text[segment_start..i]);
                }
                push_literal_line(children);
                i += 1;
                // Treat CRLF as a single line break.
                if i < bytes.len() && bytes[i] == b'\n' {
                    i += 1;
                }
                segment_start = i;
            }
            _ => {
                i += 1;
            }
        }
    }

    if segment_start < bytes.len() {
        concat_string(children, &text[segment_start..]);
    }
}

fn push_line(children: &mut Vec<Value>, mode: LineMode) {
    match mode {
        LineMode::SoftOrSpace => {
            children.push(json!({"type": "line"}));
        }
        LineMode::Soft => {
            children.push(json!({"type": "line", "soft": true}));
        }
        LineMode::Hard => {
            children.push(json!({"type": "line", "hard": true}));
            children.push(json!({"type": "break-parent"}));
        }
        LineMode::Empty => {
            // hardline x2
            children.push(json!({"type": "line", "hard": true}));
            children.push(json!({"type": "break-parent"}));
            children.push(json!({"type": "line", "hard": true}));
            children.push(json!({"type": "break-parent"}));
        }
    }
}

fn push_literal_line(children: &mut Vec<Value>) {
    children.push(json!({"type": "line", "hard": true, "literal": true}));
    children.push(json!({"type": "break-parent"}));
}

fn extract_start_tag_info(tag: &Tag) -> StartTagInfo {
    match tag {
        Tag::StartIndent => StartTagInfo::Indent,
        Tag::StartAlign(align) => StartTagInfo::Align(align.count().get()),
        Tag::StartDedent(mode) => StartTagInfo::Dedent(*mode),
        Tag::StartGroup(group) => {
            StartTagInfo::Group { id: group.id(), should_break: !group.mode().is_flat() }
        }
        Tag::StartConditionalContent(condition) => StartTagInfo::ConditionalContent {
            mode: condition.mode(),
            group_id: condition.group_id(),
        },
        Tag::StartIndentIfGroupBreaks(gid) => StartTagInfo::IndentIfGroupBreaks(*gid),
        Tag::StartFill => StartTagInfo::Fill,
        Tag::StartEntry => StartTagInfo::Entry,
        Tag::StartLineSuffix => StartTagInfo::LineSuffix,
        Tag::StartLabelled(_) => StartTagInfo::Labelled,
        _ => unreachable!("end tags should not be passed to `extract_start_tag_info()`"),
    }
}

fn build_doc(start_info: Option<&StartTagInfo>, children: Vec<Value>) -> Value {
    let Some(info) = start_info else {
        return normalize_array(children);
    };

    match info {
        StartTagInfo::Indent => {
            json!({"type": "indent", "contents": normalize_array(children)})
        }
        StartTagInfo::Align(count) => {
            json!({"type": "align", "n": *count, "contents": normalize_array(children)})
        }
        StartTagInfo::Dedent(mode) => {
            let n: Value = match mode {
                DedentMode::Level => Value::from(-1),
                // NOTE: Prettier expects `n: Number.NEGATIVE_INFINITY` for `dedent-to-root`,
                // but JSON cannot represent `Infinity`.
                // It will be `null`, which makes Prettier's `makeAlign()` treat it as a no-op.
                // If that becomes a problem, we need a JSON reviver to fix it back to `-Infinity` on the JS side.
                DedentMode::Root => Value::from(f64::NEG_INFINITY),
            };
            json!({"type": "align", "n": n, "contents": normalize_array(children)})
        }
        StartTagInfo::Group { id, should_break } => {
            let contents = normalize_array(children);
            let mut doc = json!({"type": "group", "contents": contents});
            if *should_break {
                doc["break"] = json!(true);
            }
            if let Some(gid) = id {
                doc["id"] = Value::String(group_id_string(*gid));
            }
            doc
        }
        StartTagInfo::ConditionalContent { mode, group_id } => {
            let contents = normalize_array(children);
            let mut doc = match mode {
                PrintMode::Expanded => {
                    json!({"type": "if-break", "breakContents": contents, "flatContents": ""})
                }
                PrintMode::Flat => {
                    json!({"type": "if-break", "breakContents": "", "flatContents": contents})
                }
            };
            if let Some(gid) = group_id {
                doc["groupId"] = Value::String(group_id_string(*gid));
            }
            doc
        }
        // NOTE: Prettier's `indent-if-break` also supports a `negate` property
        // that inverts the behavior (indent when flat, no indent when broken).
        // oxc_formatter's `Tag::StartIndentIfGroupBreaks` doesn't have this option yet.
        StartTagInfo::IndentIfGroupBreaks(gid) => {
            json!({
                "type": "indent-if-break",
                "contents": normalize_array(children),
                "groupId": group_id_string(*gid)
            })
        }
        StartTagInfo::Fill => {
            // Children from Entry pairs become fill parts
            json!({"type": "fill", "parts": Value::Array(children)})
        }
        StartTagInfo::Entry => {
            // Entry contents are returned directly
            normalize_array(children)
        }
        StartTagInfo::LineSuffix => {
            json!({"type": "line-suffix", "contents": normalize_array(children)})
        }
        // NOTE: Prettier's `label` doc wraps contents
        // with an arbitrary label value used by doc utilities (e.g., `stripTrailingHardline`).
        // We drop the label here since `LabelId` is opaque and the printer ignores it.
        StartTagInfo::Labelled => normalize_array(children),
    }
}

fn convert_best_fitting(
    best_fitting: &BestFittingElement,
    state: &mut ConvertState,
) -> Result<Value, String> {
    let variants = best_fitting.variants();
    if variants.is_empty() {
        return Ok(json!({"type": "group", "contents": ""}));
    }

    let first_contents = normalize_array(convert_elements(variants[0], state)?);

    if variants.len() == 1 {
        return Ok(json!({"type": "group", "contents": first_contents}));
    }

    let expanded_states: Vec<Value> = variants
        .iter()
        .map(|v| convert_elements(v, state).map(normalize_array))
        .collect::<Result<_, _>>()?;

    Ok(json!({
        "type": "group",
        "contents": first_contents,
        "expandedStates": Value::Array(expanded_states)
    }))
}

/// Formats a `GroupId` as `"G<N>"` string for Prettier.
fn group_id_string(id: GroupId) -> String {
    format!("G{}", u32::from(id))
}

/// Concatenates a string onto the last element if it's also a string,
/// otherwise pushes a new string value.
fn concat_string(children: &mut Vec<Value>, s: &str) {
    if let Some(Value::String(last)) = children.last_mut() {
        last.push_str(s);
    } else {
        children.push(Value::String(s.to_string()));
    }
}

// ---

/// Normalizes a `Vec<Value>` into a single `Value`:
/// - Empty vec -> empty string
/// - Single element -> that element
/// - Multiple -> JSON array
fn normalize_array(mut arr: Vec<Value>) -> Value {
    match arr.len() {
        0 => Value::String(String::new()),
        1 => arr.pop().unwrap(),
        _ => Value::Array(arr),
    }
}
