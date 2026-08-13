use rustc_hash::FxHashMap;
use serde_json::{Value, json};

use oxc_formatter_core::{
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
///
/// The result envelope is `{ doc, refs, hasRootDedent }`:
/// - `refs` carries shared sub-trees (below)
/// - `hasRootDedent` tells the JS side a `dedent-to-root` needs its `-Infinity` restored (see [StartTagInfo::Dedent])
///
/// # Ref-based sharing
///
/// `oxc_formatter` IR uses [`FormatElement::Interned`] to share sub-trees by pointer.
/// Naively cloning the converted JSON for each reference duplicates content,
/// which explodes exponentially when nested inside [`FormatElement::BestFitting`] variants
/// (each variant emits its own copy of the inner Interned content in the Prettier Doc's `expandedStates`).
///
/// To preserve sharing across the JSON boundary, every Interned slice is emitted once
/// into a `refs` array and referenced via `{ "_REF": <index> }` placeholders.
/// The uppercase, prefixed key avoids any chance of collision with valid Prettier
/// Doc node keys (`type`, `contents`, `id`, etc.).
/// The JS-side plugin resolves these back into shared object references before
/// handing to Prettier as `Doc`, matching the original (memory-shared) structure exactly.
/// Output is unchanged because Prettier identifies groups by `id`, not by JS object identity.
pub fn format_elements_to_prettier_doc(
    elements: &[FormatElement],
    sorted_tailwind_classes: &[String],
) -> Result<Value, String> {
    let mut state = ConvertState::new(sorted_tailwind_classes);
    // The conversion starts at a line start (the printer's `line_width == 0`),
    // so a leading `Space` is dropped, not flushed.
    // (Fragment Docs are spliced mid-line by Prettier, but no fragment IR can start with a `Space`.)
    let children = convert_elements(elements, &mut state, LineState::Hardline)?;
    let doc = normalize_array(children);
    // The JS side skips the restore walk entirely when `hasRootDedent` is false,
    // so a desync would silently drop dedents; pin the contract in debug builds.
    debug_assert_eq!(
        state.has_root_dedent,
        contains_root_dedent(&doc) || state.refs.iter().any(contains_root_dedent),
        "`hasRootDedent` must be set iff an `align n: null` (dedent-to-root) is emitted"
    );
    Ok(json!({
        "doc": doc,
        "refs": Value::Array(state.refs),
        "hasRootDedent": state.has_root_dedent,
    }))
}

type InternedCacheKey = (usize, usize);

struct ConvertState<'a> {
    sorted_tailwind_classes: &'a [String],
    /// Maps `Interned` slice pointer to its ref id in `refs`.
    interned_to_ref: FxHashMap<InternedCacheKey, usize>,
    /// Converted content per ref id (index = id).
    refs: Vec<Value>,
    /// Whether any `dedent-to-root` was emitted:
    /// its `n: -Infinity` is unrepresentable in JSON (emitted as `null`),
    /// so the JS side must restore it (see `StartTagInfo::Dedent`).
    has_root_dedent: bool,
}

impl<'a> ConvertState<'a> {
    fn new(sorted_tailwind_classes: &'a [String]) -> Self {
        Self {
            sorted_tailwind_classes,
            interned_to_ref: FxHashMap::default(),
            refs: Vec::new(),
            has_root_dedent: false,
        }
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
    MarkAsRoot,
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
/// `crates/oxc_formatter_core/src/printer/mod.rs`.
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

    /// Mirrors the printer's end-of-line state for hardline collapsing and space suppression:
    /// its `line_width > 0` guard (`Content` vs the rest) and its `has_empty_line` cap (`Blank`).
    /// See the `Line` and `Space` arms of `Printer::print_element` in `printer/mod.rs`.
    ///
    /// The document start is a line start (`Hardline`);
    /// `Interned` / `BestFitting` contents convert with a `Content` (mid-line) assumption,
    /// so element sequences crossing those boundaries are approximated;
    /// the real IR shapes (straight-line emission) are what this mirrors faithfully.
    line: LineState,
}

/// End-of-line state for [PrinterState::line].
#[derive(Clone, Copy, Eq, PartialEq)]
enum LineState {
    /// Content was emitted since the last line break.
    Content,
    /// A line break was just emitted:
    /// further `Hard`s are suppressed (the line is already broken),
    /// an `Empty` emits only its second newline.
    Hardline,
    /// A blank line was just emitted: further `Empty`s emit nothing at all.
    Blank,
}

impl PrinterState {
    fn new(initial_line: LineState) -> Self {
        Self { pending_space: false, line: initial_line }
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
    initial_line: LineState,
) -> Result<Vec<Value>, String> {
    let mut stack: Vec<StackEntry> =
        vec![StackEntry { start_tag: None, start_info: None, children: vec![] }];
    let mut printer = PrinterState::new(initial_line);

    for element in elements {
        match element {
            FormatElement::Space => {
                // `Space` at the start of a line is dropped, never made pending
                if printer.line == LineState::Content {
                    printer.pending_space = true;
                }
            }
            FormatElement::Token { text } => {
                printer.flush_pending_space(&mut stack)?;
                concat_string(current_children_mut(&mut stack)?, text);
                printer.line = LineState::Content;
            }
            FormatElement::Text { text, .. } => {
                printer.flush_pending_space(&mut stack)?;
                push_text(current_children_mut(&mut stack)?, text);
                printer.line = LineState::Content;
            }
            FormatElement::Line(mode) => {
                match mode {
                    LineMode::SoftOrSpace | LineMode::Soft => {
                        if *mode == LineMode::SoftOrSpace {
                            // `SoftOrSpace` produces a space in flat mode, newline in expanded.
                            // If `pending_space` is already set,
                            // `SoftOrSpace` subsumes it (just like the printer's boolean idempotency).
                            printer.pending_space = false;
                        }
                        // else: Soft produces nothing in flat mode, newline in expanded.
                        // Keep `pending_space` as-is (mirroring the printer).

                        // Mirror the printer's newline suppression at a line start (its `line_width > 0` guard):
                        // a soft(-or-space) line right after a hardline emits nothing natively,
                        // but Prettier's printer would print a second newline
                        // (a spurious blank line, e.g. after a dangling comment in an otherwise-empty object).
                        // Dropping the soft line instead would leave the following content on the hardline's (deeper) indentation,
                        // so remove the trailing hardline and let the soft line provide the break at the current indentation.
                        // In the printer, the LAST line element decides the pending indention.
                        // The hardline's `break-parent` stays in place, keeping the enclosing groups broken so the soft line is guaranteed to break.
                        let children = current_children_mut(&mut stack)?;
                        let replaced_hard_line = printer.line != LineState::Content
                            && strip_trailing_hard_line(children);
                        push_line(children, *mode);
                        if !replaced_hard_line {
                            printer.line = LineState::Content;
                        }
                        // else: `printer.line` stays as-is — the output is still at a line start
                    }
                    LineMode::Hard | LineMode::HardWithoutExpand | LineMode::Empty => {
                        printer.flush_pending_space(&mut stack)?;
                        // Mimic the printer's `line_width > 0` guard and `has_empty_line` cap:
                        // the printer only emits a newline when the line has content,
                        // so consecutive `Hard, Hard` produces only one newline;
                        // for `Empty` after a hardline only the second newline is emitted
                        // (the first is redundant since the line is already broken),
                        // and nothing at all when a blank line was already emitted.
                        match printer.line {
                            LineState::Content => {
                                push_line(current_children_mut(&mut stack)?, *mode);
                                printer.line = if *mode == LineMode::Empty {
                                    LineState::Blank
                                } else {
                                    LineState::Hardline
                                };
                            }
                            LineState::Hardline => {
                                if *mode == LineMode::Empty {
                                    push_line(current_children_mut(&mut stack)?, LineMode::Hard);
                                    printer.line = LineState::Blank;
                                }
                                // `Hard` after `Hard` → skip (line already broken).
                                // Known gap: when the first hardline sits behind an `EndIndent`,
                                // the skip leaves the doc's last line on the inner indentation (the printer would re-anchor to the outer level).
                                // `children_are_only_hardlines` patches the no-content sub-case; no real IR is known to hit the rest.
                                // The Soft arm above handles its equivalent via `strip_trailing_hard_line`.
                            }
                            LineState::Blank => {}
                        }
                    }
                    LineMode::ExactLineBreaks(count) => {
                        printer.flush_pending_space(&mut stack)?;
                        // Exactly `count` breaks, exempt from the collapsing above
                        // (mirrors the printer's `ExactLineBreaks` arm; `push_line`
                        // expands this to `count` hardlines, which Prettier's own
                        // printer never collapses).
                        push_line(current_children_mut(&mut stack)?, *mode);
                        // A blank is left behind from the start of a line always,
                        // mid-line only when a break remains after the line-ending one.
                        // NOTE: `line != Content` approximates the printer's `line_width == 0`,
                        // they diverge right after a `Literal` (blank there, mid-line here).
                        // Real IR always emits `ExactLineBreaks` after content, where they agree.
                        printer.line = if printer.line != LineState::Content || count.get() > 1 {
                            LineState::Blank
                        } else {
                            LineState::Hardline
                        };
                    }
                    LineMode::Literal => {
                        // A literal line always prints (no hardline dedup) and
                        // preserves the pending space (the printer never trims it away),
                        // matching a `\n` embedded in `FormatElement::Text` below.
                        printer.flush_pending_space(&mut stack)?;
                        push_line(current_children_mut(&mut stack)?, *mode);
                        printer.line = LineState::Content;
                    }
                }
            }
            // `ExpandParent` is a directive (not visible content) — it forces the parent group
            // to break. The printer treats it as a no-op (expansion is propagated at IR level).
            // Neither `pending_space` nor `line` should be affected.
            FormatElement::ExpandParent => {
                current_children_mut(&mut stack)?.push(json!({"type": "break-parent"}));
            }
            FormatElement::LineSuffixBoundary => {
                printer.flush_pending_space(&mut stack)?;
                current_children_mut(&mut stack)?.push(json!({"type": "line-suffix-boundary"}));
                printer.line = LineState::Content;
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
                    if matches!(tag, Tag::StartDedent(DedentMode::Root)) {
                        state.has_root_dedent = true;
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
                    let doc = build_doc(entry.start_info.as_ref(), entry.children);
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
                let id = if let Some(&id) = state.interned_to_ref.get(&key) {
                    id
                } else {
                    // Reserve the slot index now:
                    // the recursive `convert_elements` call below may push more refs,
                    // so `state.refs.len()` would no longer equal this Interned's slot when we go to fill it.
                    let id = state.refs.len();
                    state.refs.push(Value::Null);
                    state.interned_to_ref.insert(key, id);
                    let converted = convert_shared_elements(interned, state)?;
                    state.refs[id] = normalize_array(converted);
                    id
                };
                current_children_mut(&mut stack)?.push(json!({ "_REF": id }));
                printer.line = LineState::Content;
            }
            FormatElement::BestFitting(best_fitting) => {
                printer.flush_pending_space(&mut stack)?;
                let doc = convert_best_fitting(best_fitting, state)?;
                current_children_mut(&mut stack)?.push(doc);
                printer.line = LineState::Content;
            }
            FormatElement::TailwindClass(index) => {
                printer.flush_pending_space(&mut stack)?;
                if let Some(class) = state.sorted_tailwind_classes.get(*index) {
                    concat_string(current_children_mut(&mut stack)?, class);
                }
                printer.line = LineState::Content;
            }
            FormatElement::EmbedPlaceholder(index) => {
                // The host splices `${expr}` for each marker before the IR is finalized,
                // so one should never reach Doc conversion.
                return Err(format!(
                    "Invalid formatter IR: unresolved EmbedPlaceholder({index}) reached \
                     Prettier Doc conversion (the host must splice the interpolation first)"
                ));
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

/// Converts `Interned` / `BestFitting` contents:
/// they are emitted once and reused at arbitrary positions,
/// so they convert under the mid-line (`Content`) assumption.
fn convert_shared_elements(
    elements: &[FormatElement],
    state: &mut ConvertState,
) -> Result<Vec<Value>, String> {
    convert_elements(elements, state, LineState::Content)
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
        LineMode::HardWithoutExpand => {
            children.push(json!({"type": "line", "hard": true}));
        }
        LineMode::Empty => {
            // hardline x2
            children.push(json!({"type": "line", "hard": true}));
            children.push(json!({"type": "break-parent"}));
            children.push(json!({"type": "line", "hard": true}));
            children.push(json!({"type": "break-parent"}));
        }
        LineMode::ExactLineBreaks(count) => {
            // hardline x count: Prettier's own printer never collapses hardlines,
            // so this reproduces exactly `count` breaks.
            for _ in 0..count.get() {
                children.push(json!({"type": "line", "hard": true}));
                children.push(json!({"type": "break-parent"}));
            }
        }
        LineMode::Literal => {
            push_literal_line(children);
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
        Tag::StartMarkAsRoot => StartTagInfo::MarkAsRoot,
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
        StartTagInfo::MarkAsRoot => {
            // Prettier's `markAsRoot()` = `align({type: "root"}, ...)`
            json!({"type": "align", "n": {"type": "root"}, "contents": normalize_array(children)})
        }
        StartTagInfo::Dedent(mode) => {
            let n: Value = match mode {
                DedentMode::Level => Value::from(-1),
                // Prettier expects `n: Number.NEGATIVE_INFINITY` for `dedent-to-root`,
                // but JSON cannot represent `Infinity`.
                // Emit `null` and let the JS side restore it to `Number.NEGATIVE_INFINITY`,
                // signaled by the top-level `hasRootDedent` flag.
                //
                // NOTE: `null` (not a self-describing marker string) on purpose:
                // if the restore is ever missed, Prettier's `makeAlign()` treats a falsy `n` as a no-op (a layout-only miss),
                // while a string `n` is a valid align form (literal indent text) and would inject the marker into the output.
                // The flag/`null` consistency is pinned by the `debug_assert_eq` in [format_elements_to_prettier_doc].
                DedentMode::Root => Value::Null,
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

    if variants.len() == 1 {
        let first_contents = normalize_array(convert_shared_elements(variants[0], state)?);
        return Ok(json!({"type": "group", "contents": first_contents}));
    }

    // `variants[0]` is rendered twice in Prettier's Doc.
    // - once as `contents` (the flat-mode candidate)
    // - and once as `expandedStates[0]` (the first break-mode candidate)
    // Convert it once and stash the result in `refs`,
    // then reference it from both positions via `{ _REF: id }` placeholders.
    // The JS-side resolver restores both to the same memory-shared object
    // (identity unaffected — Prettier identifies groups by `id`).
    //
    // Reserve the slot before recursing so any nested Interned refs pushed during conversion
    // get later ids; `state.refs[id]` is filled in after.
    let first_id = state.refs.len();
    state.refs.push(Value::Null);
    let first_content = normalize_array(convert_shared_elements(variants[0], state)?);
    state.refs[first_id] = first_content;
    let first_ref = json!({ "_REF": first_id });

    let mut expanded_states: Vec<Value> = Vec::with_capacity(variants.len());
    expanded_states.push(first_ref.clone());
    for v in &variants[1..] {
        expanded_states.push(normalize_array(convert_shared_elements(v, state)?));
    }

    Ok(json!({
        "type": "group",
        "contents": first_ref,
        "expandedStates": Value::Array(expanded_states),
    }))
}

/// Formats a `GroupId` as `"G<N>"` string for Prettier.
fn group_id_string(id: GroupId) -> String {
    format!("G{}", u32::from(id))
}

/// Whether the doc is a `break-parent` (as emitted by [push_line]).
fn is_break_parent(value: &Value) -> bool {
    matches!(value, Value::Object(map)
        if map.get("type").and_then(Value::as_str) == Some("break-parent"))
}

/// Whether the doc is a hardline (as emitted by [push_line]), excluding literal
/// lines (which always print and never take part in newline collapsing).
fn is_hard_line(value: &Value) -> bool {
    matches!(value, Value::Object(map)
        if map.get("type").and_then(Value::as_str) == Some("line")
            && map.get("hard") == Some(&Value::Bool(true))
            && !map.contains_key("literal"))
}

/// Removes the trailing hardline doc (leaving its `break-parent` in place) from the deep tail of `children`,
/// descending through transparent wrappers (arrays, `indent` / `align` / plain `group` contents).
/// Returns `true` on removal.
///
/// Only a hardline followed by its own `break-parent` (i.e. `LineMode::Hard`) is removed:
/// a bare hardline (`HardWithoutExpand`) must stay,
/// since replacing it with a soft line could let the enclosing group flatten and lose the break entirely.
/// `_REF` placeholders, `expandedStates` groups, fills, if-breaks, and line-suffixes are never descended into
/// (shared or conditionally-printed content).
fn strip_trailing_hard_line(children: &mut Vec<Value>) -> bool {
    fn descend(value: &mut Value) -> bool {
        match value {
            Value::Array(inner) => strip_trailing_hard_line(inner),
            Value::Object(map) => {
                let ty = map.get("type").and_then(Value::as_str);
                if matches!(ty, Some("indent" | "align"))
                    || (ty == Some("group") && !map.contains_key("expandedStates"))
                {
                    map.get_mut("contents").is_some_and(descend)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    // Scan back over trailing `break-parent`s to the last significant doc
    let Some(i) = children.iter().rposition(|child| !is_break_parent(child)) else {
        return false;
    };
    if is_hard_line(&children[i]) {
        // No `break-parent` behind it → a bare `HardWithoutExpand`, keep it
        if i + 1 == children.len() {
            return false;
        }
        children.remove(i);
        return true;
    }
    descend(&mut children[i])
}

/// Returns `true` if the children consist only of hardline docs and break-parent docs
/// (i.e., no visible content that would benefit from indentation).
fn children_are_only_hardlines(children: &[Value]) -> bool {
    children.iter().all(|child| is_break_parent(child) || is_hard_line(child))
}

/// Whether the doc (sub)tree contains a dedent-to-root (`align` with `n: null`).
/// Debug-only backstop for the `hasRootDedent` contract (see the `debug_assert_eq` in [format_elements_to_prettier_doc]).
/// Not `#[cfg(debug_assertions)]`: `debug_assert_eq!` type-checks its args even when off.
fn contains_root_dedent(value: &Value) -> bool {
    match value {
        Value::Array(items) => items.iter().any(contains_root_dedent),
        Value::Object(map) => {
            (map.get("type").and_then(Value::as_str) == Some("align")
                && map.get("n") == Some(&Value::Null))
                || map.values().any(contains_root_dedent)
        }
        _ => false,
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

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use serde_json::{Value, json};

    use oxc_formatter_core::{DedentMode, FormatElement, LineMode, Tag};

    use super::{format_elements_to_prettier_doc, is_hard_line};

    fn to_doc(elements: &[FormatElement]) -> Value {
        format_elements_to_prettier_doc(elements, &[]).unwrap()
    }

    /// Counts `{"type": "line", "hard": true}` nodes (excluding literal lines) in the Doc.
    fn count_hardlines(doc: &Value) -> usize {
        match doc {
            Value::Array(items) => items.iter().map(count_hardlines).sum(),
            Value::Object(map) => {
                usize::from(is_hard_line(doc)) + map.values().map(count_hardlines).sum::<usize>()
            }
            _ => 0,
        }
    }

    fn exact(count: u32) -> FormatElement<'static> {
        FormatElement::Line(LineMode::ExactLineBreaks(NonZeroU32::new(count).unwrap()))
    }
    const A: FormatElement<'static> = FormatElement::Token { text: "a" };
    const HARD: FormatElement<'static> = FormatElement::Line(LineMode::Hard);
    const EMPTY: FormatElement<'static> = FormatElement::Line(LineMode::Empty);
    const SPACE: FormatElement<'static> = FormatElement::Space;

    #[test]
    fn exact_line_breaks_expand_to_that_many_hardlines() {
        let doc = to_doc(&[A, exact(3), A]);
        assert_eq!(count_hardlines(&doc), 3);
    }

    #[test]
    fn empty_after_mid_line_single_exact_break_adds_a_blank() {
        // Mid-line count=1 leaves no blank behind: the following Empty may still add one.
        let doc = to_doc(&[A, exact(1), EMPTY, A]);
        assert_eq!(count_hardlines(&doc), 2);
    }

    #[test]
    fn empty_after_blank_leaving_exact_breaks_is_capped() {
        // Mid-line count=2 leaves a blank: the following Empty adds nothing.
        let doc = to_doc(&[A, exact(2), EMPTY, A]);
        assert_eq!(count_hardlines(&doc), 2);

        // From the start of a line even count=1 leaves a blank
        // (no break is consumed as a line ending), capping the Empty too.
        let doc = to_doc(&[A, HARD, exact(1), EMPTY, A]);
        assert_eq!(count_hardlines(&doc), 2);
    }

    #[test]
    fn hard_after_exact_line_breaks_is_absorbed() {
        let doc = to_doc(&[A, exact(2), HARD, A]);
        assert_eq!(count_hardlines(&doc), 2);
    }

    const SOFT: FormatElement<'static> = FormatElement::Line(LineMode::Soft);
    const HARD_NO_EXPAND: FormatElement<'static> = FormatElement::Line(LineMode::HardWithoutExpand);

    #[test]
    fn soft_after_hard_replaces_the_hardline() {
        // The printer suppresses the soft line's newline at a line start;
        // in the Doc the LAST line must survive (it decides the indention).
        let doc = to_doc(&[A, HARD, SOFT, A]);
        assert_eq!(
            doc["doc"],
            json!(["a", {"type": "break-parent"}, {"type": "line", "soft": true}, "a"])
        );
    }

    #[test]
    fn soft_after_hard_strips_through_a_closed_indent() {
        // Dangling comment in an otherwise-empty object:
        // `{` indent[softline, "// x", hardline] softline `}`
        let doc = to_doc(&[
            FormatElement::Token { text: "{" },
            FormatElement::Tag(Tag::StartIndent),
            SOFT,
            FormatElement::Token { text: "// x" },
            HARD,
            FormatElement::Tag(Tag::EndIndent),
            SOFT,
            FormatElement::Token { text: "}" },
        ]);
        assert_eq!(
            doc["doc"],
            json!([
                "{",
                {"type": "indent", "contents": [
                    {"type": "line", "soft": true},
                    "// x",
                    {"type": "break-parent"},
                ]},
                {"type": "line", "soft": true},
                "}"
            ])
        );
    }

    #[test]
    fn soft_after_hard_without_expand_keeps_both() {
        // A bare hardline (no break-parent) must stay:
        // replacing it with a soft line could let the enclosing group flatten and lose the break.
        let doc = to_doc(&[A, HARD_NO_EXPAND, SOFT, A]);
        assert_eq!(count_hardlines(&doc), 1);
    }

    #[test]
    fn root_dedent_is_null_and_flagged() {
        let doc = to_doc(&[
            FormatElement::Tag(Tag::StartDedent(DedentMode::Root)),
            A,
            FormatElement::Tag(Tag::EndDedent(DedentMode::Root)),
        ]);
        assert_eq!(doc["doc"], json!({"type": "align", "n": null, "contents": "a"}));
        assert_eq!(doc["hasRootDedent"], json!(true));

        let doc = to_doc(&[A]);
        assert_eq!(doc["hasRootDedent"], json!(false));
    }

    #[test]
    fn space_at_line_start_is_dropped() {
        // e.g. a comment-only program: the IR starts with the trailing-comment `Space`,
        // which the printer drops at `line_width == 0`.
        let doc = to_doc(&[SPACE, A]);
        assert_eq!(doc["doc"], "a");

        let doc = to_doc(&[A, HARD, SPACE, A]);
        assert_eq!(
            doc["doc"],
            json!(["a", {"type": "line", "hard": true}, {"type": "break-parent"}, "a"])
        );
        // The dropped space must not resurrect a hardline collapse either.
        let doc = to_doc(&[A, HARD, SPACE, HARD, A]);
        assert_eq!(count_hardlines(&doc), 1);
    }
}
