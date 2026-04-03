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

/// Converts `oxc_formatter` IR (`FormatElement` slice) into Prettier Doc.
///
/// This is used for js-in-xxx formatting (both full and fragment)
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

/// Simulates a subset of the `oxc_formatter` printer's runtime state.
///
/// Our IR (`FormatElement`) is designed to be consumed by its own printer,
/// which applies several runtime optimizations
/// (space deduplication, consecutive hardline merging, etc.).
/// When converting this IR to Prettier `Doc` instead of printed text,
/// we must replicate these same optimizations,
/// otherwise the `Doc` would contain redundant spaces/lines that the printer would have suppressed.
///
/// Each field corresponds to a specific printer behavior in
/// `crates/oxc_formatter/src/formatter/printer/mod.rs`.
///
/// NOTE: If the printer gains new runtime optimizations that affect output,
/// this struct may need corresponding updates.
/// The conformance tests will catch most divergences,
/// but when debugging mismatches, compare against the printer's state machine.
struct PrinterState {
    /// Mirrors the printer's `pending_space` flag.
    /// See: `PrinterState::pending_space` in `printer/mod.rs`
    ///
    /// Set by `Space` elements.
    /// Consumed (flushed as `" "`) before the next visible content.
    /// Boolean semantics naturally deduplicates consecutive spaces.
    pending_space: bool,

    /// Mirrors the printer's `line_width > 0` guard for hardline emission
    /// and `has_empty_line` flag for empty line deduplication.
    /// See: `Printer::print_line()` in printer/mod.rs
    ///
    /// When true, consecutive `Hard` lines are suppressed (the line is already broken).
    /// `Empty` after a hardline emits only one additional `Hard` (the second newline).
    ///
    /// NOTE: Consecutive `Empty, Empty` won't fully collapse to a single empty line,
    /// but such sequences don't occur in practice since the IR generators already
    /// control line emission.
    last_was_hardline: bool,
}

impl PrinterState {
    fn new() -> Self {
        Self { pending_space: false, last_was_hardline: false }
    }

    /// Flushes the pending space (if any) by appending `" "` to the current scope's children.
    fn flush_pending_space(&mut self, stack: &mut [StackEntry]) -> Result<(), String> {
        if self.pending_space {
            concat_string(current_children_mut(stack)?, " ");
            self.pending_space = false;
        }
        Ok(())
    }
}

fn convert_elements(
    elements: &[FormatElement],
    state: &mut ConvertState,
) -> Result<Vec<Value>, String> {
    let mut stack: Vec<StackEntry> =
        vec![StackEntry { start_tag: None, start_info: None, children: vec![] }];
    let mut printer = PrinterState::new();

    for element in elements {
        match element {
            FormatElement::Space => {
                printer.pending_space = true;
                printer.last_was_hardline = false;
            }
            FormatElement::Token { text } => {
                printer.flush_pending_space(&mut stack)?;
                concat_string(current_children_mut(&mut stack)?, text);
                printer.last_was_hardline = false;
            }
            FormatElement::Text { text, .. } => {
                printer.flush_pending_space(&mut stack)?;
                push_text(current_children_mut(&mut stack)?, text);
                printer.last_was_hardline = false;
            }
            FormatElement::Line(mode) => {
                match mode {
                    LineMode::SoftOrSpace => {
                        // `SoftOrSpace` produces a space in flat mode, newline in expanded.
                        // If `pending_space` is already set,
                        // `SoftOrSpace` subsumes it (just like the printer's boolean idempotency).
                        printer.pending_space = false;
                        push_line(current_children_mut(&mut stack)?, *mode);
                    }
                    LineMode::Soft => {
                        // Soft produces nothing in flat mode, newline in expanded.
                        // Keep `pending_space` as-is (mirroring the printer).
                        push_line(current_children_mut(&mut stack)?, *mode);
                    }
                    LineMode::Hard | LineMode::Empty => {
                        printer.flush_pending_space(&mut stack)?;
                        // Mimic the printer's `line_width > 0` guard and `has_empty_line` dedup:
                        // - The printer only emits a newline when the line has content (`line_width > 0`).
                        //   Consecutive `Hard, Hard` produces only one newline.
                        // - For `Empty` after a hardline, only the second newline of `Empty` is emitted
                        //   (the first is redundant since the line is already broken).
                        if printer.last_was_hardline {
                            if *mode == LineMode::Empty {
                                push_line(current_children_mut(&mut stack)?, LineMode::Hard);
                            }
                            // `Hard` after `Hard` → skip (line already broken)
                        } else {
                            push_line(current_children_mut(&mut stack)?, *mode);
                        }
                    }
                }
                printer.last_was_hardline = matches!(mode, LineMode::Hard | LineMode::Empty);
            }
            // `ExpandParent` is a directive (not visible content) — it forces the parent group
            // to break. The printer treats it as a no-op (expansion is propagated at IR level).
            // Neither `pending_space` nor `last_was_hardline` should be affected.
            FormatElement::ExpandParent => {
                current_children_mut(&mut stack)?.push(json!({"type": "break-parent"}));
            }
            FormatElement::LineSuffixBoundary => {
                printer.flush_pending_space(&mut stack)?;
                current_children_mut(&mut stack)?.push(json!({"type": "line-suffix-boundary"}));
                printer.last_was_hardline = false;
            }
            FormatElement::Tag(tag) => {
                if tag.is_start() {
                    // Flush pending space to the parent scope before opening a new tag.
                    // The space belongs before the group/indent/conditional, not inside it.
                    // If it were flushed inside (e.g. into a `ConditionalContent(Flat)`),
                    // it could be lost when the group breaks.
                    //
                    // Exception: `LineSuffix` already contains a leading `Space` for separation.
                    // Flushing the outer `pending_space` here would create a double space
                    // (one from the flush + one from the LineSuffix's inner Space).
                    // The printer avoids this via `line_width > 0` guard and boolean idempotency.
                    // We discard the outer `pending_space` since the `LineSuffix`'s inner `Space` handles it.
                    if printer.pending_space {
                        if !matches!(tag, Tag::StartLineSuffix) {
                            concat_string(current_children_mut(&mut stack)?, " ");
                        }
                        printer.pending_space = false;
                    }
                    stack.push(StackEntry {
                        start_tag: Some(tag.clone()),
                        start_info: Some(extract_start_tag_info(tag)),
                        children: vec![],
                    });
                } else {
                    // For ConditionalContent, flush pending space into the closing scope's children
                    // so it stays within the conditional branch (otherwise it leaks into expanded mode).
                    // For all other tags (Indent, Group, Align, etc.), carry pending_space to the parent —
                    // these wrappers are transparent for spacing, and flushing inside would cause
                    // double-space bugs (e.g. `extends A, B  {` where indent's trailing " " + outer " ").
                    if printer.pending_space
                        && matches!(
                            tag,
                            Tag::EndConditionalContent | Tag::EndIndentIfGroupBreaks(_)
                        )
                    {
                        concat_string(current_children_mut(&mut stack)?, " ");
                        printer.pending_space = false;
                    }
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
                    let is_line_suffix = matches!(entry.start_info, Some(StartTagInfo::LineSuffix));
                    let mut doc = build_doc(entry.start_info.as_ref(), entry.children);

                    // The formatter always prepends a `Space` inside `LineSuffix` for separation
                    // from preceding code (e.g. `x = 1; // comment`).
                    // The printer guards this with `line_width > 0`,
                    // so the `Space` is suppressed when the line has no content yet.
                    // When the parent scope has no preceding content (e.g. comment-only `<script>` blocks),
                    // strip the leading space from the `lineSuffix` contents to avoid a spurious leading space.
                    if is_line_suffix
                        && current_children_mut(&mut stack)?.is_empty()
                        && let Value::Object(ref mut map) = doc
                        && let Some(contents) = map.get_mut("contents")
                    {
                        strip_leading_space(contents);
                    }

                    current_children_mut(&mut stack)?.push(doc);
                }
            }
            FormatElement::Interned(interned) => {
                if printer.pending_space {
                    // If the interned starts with a space-producing element,
                    // don't flush — the inner conversion already emits a leading space.
                    // This mirrors the printer's boolean idempotency (`Space + Space` = one space).
                    if !matches!(
                        interned.first(),
                        Some(FormatElement::Space | FormatElement::Line(LineMode::SoftOrSpace))
                    ) {
                        concat_string(current_children_mut(&mut stack)?, " ");
                    }
                    printer.pending_space = false;
                }
                let key = interned_cache_key(interned);
                if let Some(cached) = state.interned_cache.get(&key) {
                    current_children_mut(&mut stack)?.extend(cached.iter().cloned());
                } else {
                    let converted = convert_elements(interned, state)?;
                    state.interned_cache.insert(key, converted.clone());
                    current_children_mut(&mut stack)?.extend(converted);
                }
                printer.last_was_hardline = false;
            }
            FormatElement::BestFitting(best_fitting) => {
                printer.flush_pending_space(&mut stack)?;
                let doc = convert_best_fitting(best_fitting, state)?;
                current_children_mut(&mut stack)?.push(doc);
                printer.last_was_hardline = false;
            }
            FormatElement::TailwindClass(index) => {
                printer.flush_pending_space(&mut stack)?;
                if let Some(class) = state.sorted_tailwind_classes.get(*index) {
                    concat_string(current_children_mut(&mut stack)?, class);
                }
                printer.last_was_hardline = false;
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
            // If the indent scope contains only hardlines and break-parents (no actual content),
            // unwrap the indent wrapper. Otherwise Prettier applies indentation to the hardline,
            // producing spurious leading spaces (e.g. empty `switch` body: `{\n  }` instead of `{\n}`).
            if children_are_only_hardlines(&children) {
                normalize_array(children)
            } else {
                json!({"type": "indent", "contents": normalize_array(children)})
            }
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

/// Returns `true` if the children consist only of hardline docs and break-parent docs
/// (i.e., no visible content that would benefit from indentation).
fn children_are_only_hardlines(children: &[Value]) -> bool {
    children.iter().all(|child| {
        if let Value::Object(map) = child {
            let ty = map.get("type").and_then(Value::as_str);
            ty == Some("break-parent")
                || (ty == Some("line") && map.get("hard") == Some(&Value::Bool(true)))
        } else {
            false
        }
    })
}

/// Strips a leading space character from a `Value`.
/// Handles both plain strings (`" foo"` → `"foo"`) and arrays whose first element is a string.
fn strip_leading_space(value: &mut Value) {
    match value {
        Value::String(s) => {
            if s.starts_with(' ') {
                s.remove(0);
            }
        }
        Value::Array(arr) => {
            if let Some(Value::String(s)) = arr.first_mut()
                && s.starts_with(' ')
            {
                s.remove(0);
            }
        }
        _ => {}
    }
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
