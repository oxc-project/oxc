//! Prettier Doc JSON → FormatElement IR conversion primitives.
//!
//! [`convert_envelope`] is the public entry point:
//! it unwraps the `[doc, metadata]` envelope sent from the JS side
//! and converts the doc through the private `convert_*` walkers into a flat `FormatElement` IR.
//! Language-specific routing lives in `core::embed`;
//! [`postprocess`] here is the conversion's finishing pass (Prettier-fallback path only).

use std::num::NonZeroU8;

use rustc_hash::FxHashMap;
use serde_json::Value;

use oxc_allocator::{Allocator, ArenaStringBuilder, ArenaVec};
use oxc_formatter_core::{
    Align, Condition, DedentMode, FormatElement, Group, GroupId, GroupMode, IndentWidth, LineMode,
    PrintMode, Tag, TextWidth, UniqueGroupIdBuilder, format_element::BestFittingElement,
};

/// Marker string used to represent `-Infinity` in JSON.
/// JS side replaces `-Infinity` with this string before `JSON.stringify()`.
/// See `src-js/lib/apis.ts` for details.
const NEGATIVE_INFINITY_MARKER: &str = "__NEGATIVE_INFINITY__";

/// Unwrap a `[doc, metadata]` envelope and convert the doc JSON to IR.
///
/// Doc JSONs from the JS side always come wrapped in this uniform envelope
/// so the dispatcher can carry language-specific metadata alongside the doc itself.
///
/// Panics on invalid envelope format (internal protocol we control on both sides).
///
/// # Errors
/// Returns an error if the embedded doc JSON itself fails to convert
/// (unknown Doc type, unsupported construct, malformed group ID, ...).
pub fn convert_envelope<'a>(
    envelope: Value,
    allocator: &'a Allocator,
    group_id_builder: &UniqueGroupIdBuilder,
) -> Result<(ArenaVec<'a, FormatElement<'a>>, serde_json::Map<String, Value>), String> {
    let Value::Array(mut arr) = envelope else {
        unreachable!("Doc JSON envelope must be [doc, metadata]");
    };
    let metadata = match arr.pop() {
        Some(Value::Object(obj)) => obj,
        _ => serde_json::Map::new(),
    };
    let doc_json = arr.into_iter().next().expect("Doc JSON envelope must contain doc");

    let mut ctx = FmtCtx::new(allocator, group_id_builder);
    let mut ir = ArenaVec::new_in(&allocator);
    convert_doc(&doc_json, &mut ir, &mut ctx)?;
    Ok((ir, metadata))
}

/// Conversion context holding the allocator, group ID builder, and group ID mapping.
struct FmtCtx<'a, 'b> {
    allocator: &'a Allocator,
    group_id_builder: &'b UniqueGroupIdBuilder,
    /// Maps numeric group IDs from Prettier Doc JSON to real `GroupId`s.
    group_id_map: FxHashMap<u32, GroupId>,
}

impl<'a, 'b> FmtCtx<'a, 'b> {
    fn new(allocator: &'a Allocator, group_id_builder: &'b UniqueGroupIdBuilder) -> Self {
        Self { allocator, group_id_builder, group_id_map: FxHashMap::default() }
    }

    fn resolve_group_id(&mut self, id: u32) -> GroupId {
        *self.group_id_map.entry(id).or_insert_with(|| self.group_id_builder.group_id("xxx-in-js"))
    }
}

/// A `Text` element measured with the default `IndentWidth`.
///
/// NOTE: `IndentWidth` only affects tab character width calculation.
/// If the text contained `\t` (e.g. inside a string literal like `"\t"`?),
/// the width could be miscalculated when `options.indent_width` != 2.
/// However, the default value is sufficient in practice.
pub fn text_element(text: &str) -> FormatElement<'_> {
    let width = TextWidth::from_text(text, IndentWidth::default());
    FormatElement::Text { text, width }
}

fn convert_doc<'a>(
    doc: &Value,
    out: &mut ArenaVec<'a, FormatElement<'a>>,
    ctx: &mut FmtCtx<'a, '_>,
) -> Result<(), String> {
    match doc {
        Value::String(s) => {
            // A trailing space maps to `Space` (pending space), not text:
            // Prettier's printer trims trailing whitespace at every line break,
            // so a Doc string's trailing space is semantically "a space only if content follows on the same line".
            // Exactly the core printer's pending-space.
            // Kept as text it would leak before a soft break (the core printer never trims);
            // e.g. css-in-html `prop: ` before an `indent([softline, ...])` value.
            let (content, trailing_space) = match s.strip_suffix(' ') {
                Some(content) => (content, true),
                None => (s.as_str(), false),
            };
            if !content.is_empty() {
                out.push(text_element(ctx.allocator.alloc_str(content)));
            }
            if trailing_space {
                out.push(FormatElement::Space);
            }
            Ok(())
        }
        Value::Array(arr) => {
            for item in arr {
                convert_doc(item, out, ctx)?;
            }
            Ok(())
        }
        Value::Object(obj) => {
            let Some(doc_type) = obj.get("type").and_then(Value::as_str) else {
                return Err("Doc object missing 'type' field".to_string());
            };
            match doc_type {
                "line" => {
                    convert_line(obj, out);
                    Ok(())
                }
                "group" => convert_group(obj, out, ctx),
                "indent" => convert_indent(obj, out, ctx),
                "align" => convert_align(obj, out, ctx),
                "if-break" => convert_if_break(obj, out, ctx),
                "indent-if-break" => convert_indent_if_break(obj, out, ctx),
                "fill" => convert_fill(obj, out, ctx),
                "line-suffix" => convert_line_suffix(obj, out, ctx),
                "line-suffix-boundary" => {
                    out.push(FormatElement::LineSuffixBoundary);
                    Ok(())
                }
                "break-parent" => {
                    out.push(FormatElement::ExpandParent);
                    Ok(())
                }
                "label" => {
                    if let Some(contents) = obj.get("contents") {
                        convert_doc(contents, out, ctx)?;
                    }
                    Ok(())
                }
                "cursor" => Ok(()),
                "trim" => Err("Unsupported Doc type: 'trim'".to_string()),
                _ => Err(format!("Unknown Doc type: '{doc_type}'")),
            }
        }
        Value::Null => Ok(()),
        _ => Err(format!("Unexpected Doc value type: {doc}")),
    }
}

fn convert_line<'a>(
    obj: &serde_json::Map<String, Value>,
    out: &mut ArenaVec<'a, FormatElement<'a>>,
) {
    let hard = obj.get("hard").and_then(Value::as_bool).unwrap_or(false);
    let soft = obj.get("soft").and_then(Value::as_bool).unwrap_or(false);
    let literal = obj.get("literal").and_then(Value::as_bool).unwrap_or(false);

    if hard && literal {
        // NOTE: inherits the core printer's known divergence — a hard line directly
        // after a COLUMN-0 literal line is absorbed (Prettier prints both newlines).
        // This mechanical conversion cannot apply the `empty_line()` workaround;
        // see `hard_line_after_column_zero_literal_line_is_absorbed` in `oxc_formatter_core`.
        // Known gap: a bare `{line, hard, literal}` (Prettier's `literallineWithoutBreakParent`)
        // also lands here and over-propagates.
        // `Literal` expands enclosing groups and no non-propagating literal mode exists
        // (the paired `literalline` form is unaffected, its propagation rides the following `break-parent`).
        out.push(FormatElement::Line(LineMode::Literal));
    } else if hard {
        // `{line, hard}` alone is Prettier's `hardlineWithoutBreakParent`;
        // its `hardline` arrives as the `[{line, hard}, {break-parent}]` pair,
        // whose propagation the following `break-parent` → `ExpandParent` carries.
        out.push(FormatElement::Line(LineMode::HardWithoutExpand));
    } else if soft {
        out.push(FormatElement::Line(LineMode::Soft));
    } else {
        out.push(FormatElement::Line(LineMode::SoftOrSpace));
    }
}

fn convert_group<'a>(
    obj: &serde_json::Map<String, Value>,
    out: &mut ArenaVec<'a, FormatElement<'a>>,
    ctx: &mut FmtCtx<'a, '_>,
) -> Result<(), String> {
    let should_break = obj.get("break").and_then(Value::as_bool).unwrap_or(false);
    let id = extract_group_id(obj, "id")?;
    let gid = id.map(|n| ctx.resolve_group_id(n));

    let Some(expanded_states) = obj.get("expandedStates") else {
        return convert_group_contents(obj.get("contents"), gid, should_break, out, ctx);
    };
    let Value::Array(expanded_states) = expanded_states else {
        return Err("group 'expandedStates' must be an array".to_string());
    };
    let contents = obj
        .get("contents")
        .ok_or_else(|| "group with 'expandedStates' missing 'contents'".to_string())?;

    // `conditionalGroup(states, options)` stores `states[0]` in `contents` as well as in `expandedStates`.
    // The first representation is therefore `contents`, followed by `expandedStates[1..]`.
    // A forced group skips fitting altogether and uses the final state.
    if should_break {
        let final_state = expanded_states.last().unwrap_or(contents);
        return convert_group_contents(Some(final_state), gid, true, out, ctx);
    }
    // A single state is just a regular group. BestFitting requires at least two variants
    if expanded_states.len() <= 1 {
        return convert_group_contents(Some(contents), gid, false, out, ctx);
    }

    let mut variants = ArenaVec::with_capacity_in(expanded_states.len(), &ctx.allocator);
    for (index, state) in
        std::iter::once(contents).chain(expanded_states.iter().skip(1)).enumerate()
    {
        let mode =
            if index + 1 == expanded_states.len() { GroupMode::Expand } else { GroupMode::Flat };
        let mut variant = ArenaVec::new_in(&ctx.allocator);
        variant.push(FormatElement::Tag(Tag::StartEntry));
        // `BestFitting` itself supplies the selected variant's print mode.
        // A wrapper group is only needed to publish that mode under Prettier's group ID for `if-break(groupId)` consumers.
        // Wrapping an ID-less variant would remeasure it after selection
        // and can incorrectly expand an intermediate state that Prettier prints flat.
        // (No core Prettier printer passes an id to `conditionalGroup`; this branch is defensive.
        // Note the same remeasure hazard reappears here via `propagate_expand` if a variant contains a top-level hard line,
        // so revisit before relying on it for real inputs.)
        if gid.is_some() {
            variant.push(FormatElement::Tag(Tag::StartGroup(
                Group::new().with_id(gid).with_mode(mode),
            )));
        }
        convert_doc(state, &mut variant, ctx)?;
        if gid.is_some() {
            variant.push(FormatElement::Tag(Tag::EndGroup));
        }
        variant.push(FormatElement::Tag(Tag::EndEntry));
        // The trailing `EndEntry` tag keeps postprocess's trailing-hardline strip from firing:
        // a variant retains its trailing hardline (content may follow the `BestFitting`).
        postprocess(&mut variant, ctx.allocator);
        variants.push(variant.into_arena_slice());
    }

    // SAFETY: `expanded_states.len() > 1`, and the loop emits exactly that many variants.
    out.push(FormatElement::BestFitting(unsafe {
        BestFittingElement::from_vec_unchecked(variants)
    }));
    Ok(())
}

fn convert_group_contents<'a>(
    contents: Option<&Value>,
    gid: Option<GroupId>,
    should_break: bool,
    out: &mut ArenaVec<'a, FormatElement<'a>>,
    ctx: &mut FmtCtx<'a, '_>,
) -> Result<(), String> {
    let mode = if should_break { GroupMode::Expand } else { GroupMode::Flat };
    out.push(FormatElement::Tag(Tag::StartGroup(Group::new().with_id(gid).with_mode(mode))));
    if let Some(contents) = contents {
        convert_doc(contents, out, ctx)?;
    }
    out.push(FormatElement::Tag(Tag::EndGroup));
    Ok(())
}

fn convert_indent<'a>(
    obj: &serde_json::Map<String, Value>,
    out: &mut ArenaVec<'a, FormatElement<'a>>,
    ctx: &mut FmtCtx<'a, '_>,
) -> Result<(), String> {
    out.push(FormatElement::Tag(Tag::StartIndent));
    if let Some(contents) = obj.get("contents") {
        convert_doc(contents, out, ctx)?;
    }
    out.push(FormatElement::Tag(Tag::EndIndent));
    Ok(())
}

fn convert_align<'a>(
    obj: &serde_json::Map<String, Value>,
    out: &mut ArenaVec<'a, FormatElement<'a>>,
    ctx: &mut FmtCtx<'a, '_>,
) -> Result<(), String> {
    let n = &obj["n"];

    match n {
        Value::Number(num) => {
            if let Some(i) = num.as_i64() {
                if i == 0 {
                    if let Some(contents) = obj.get("contents") {
                        convert_doc(contents, out, ctx)?;
                    }
                    return Ok(());
                } else if i == -1 {
                    out.push(FormatElement::Tag(Tag::StartDedent(DedentMode::Level)));
                    if let Some(contents) = obj.get("contents") {
                        convert_doc(contents, out, ctx)?;
                    }
                    out.push(FormatElement::Tag(Tag::EndDedent(DedentMode::Level)));
                    return Ok(());
                } else if i > 0 {
                    debug_assert!(i <= 255, "align value {i} exceeds NonZeroU8 range");
                    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    if let Some(nz) = NonZeroU8::new(i as u8) {
                        out.push(FormatElement::Tag(Tag::StartAlign(Align::new(nz))));
                        if let Some(contents) = obj.get("contents") {
                            convert_doc(contents, out, ctx)?;
                        }
                        out.push(FormatElement::Tag(Tag::EndAlign));
                        return Ok(());
                    }
                }
            }
            Err(format!("Unsupported align value: {n}"))
        }
        Value::String(s) if s == NEGATIVE_INFINITY_MARKER => {
            out.push(FormatElement::Tag(Tag::StartDedent(DedentMode::Root)));
            if let Some(contents) = obj.get("contents") {
                convert_doc(contents, out, ctx)?;
            }
            out.push(FormatElement::Tag(Tag::EndDedent(DedentMode::Root)));
            Ok(())
        }
        Value::String(s) => {
            // String alignment (e.g., "  " for markdown list continuation indent).
            // Prettier uses the string length as the number of spaces to align by.
            if s.is_empty() {
                // Empty string → no alignment, just render contents
                if let Some(contents) = obj.get("contents") {
                    convert_doc(contents, out, ctx)?;
                }
                return Ok(());
            }
            debug_assert!(
                s.len() <= 255,
                "align string length {} exceeds NonZeroU8 range",
                s.len()
            );
            #[expect(clippy::cast_possible_truncation)]
            if let Some(nz) = NonZeroU8::new(s.len() as u8) {
                out.push(FormatElement::Tag(Tag::StartAlign(Align::new(nz))));
                if let Some(contents) = obj.get("contents") {
                    convert_doc(contents, out, ctx)?;
                }
                out.push(FormatElement::Tag(Tag::EndAlign));
                return Ok(());
            }
            Err(format!("Unsupported align value: {n}"))
        }
        Value::Object(obj_val) => {
            // `align({type: "root"}, ...)` = Prettier's `markAsRoot()`:
            // records the current indent position so that literal lines and
            // a later `dedentToRoot` return to it.
            if obj_val.get("type").and_then(Value::as_str) == Some("root") {
                out.push(FormatElement::Tag(Tag::StartMarkAsRoot));
                if let Some(contents) = obj.get("contents") {
                    convert_doc(contents, out, ctx)?;
                }
                out.push(FormatElement::Tag(Tag::EndMarkAsRoot));
                return Ok(());
            }
            Err(format!("Unsupported align value: {n}"))
        }
        _ => Err(format!("Unsupported align value: {n}")),
    }
}

fn convert_if_break<'a>(
    obj: &serde_json::Map<String, Value>,
    out: &mut ArenaVec<'a, FormatElement<'a>>,
    ctx: &mut FmtCtx<'a, '_>,
) -> Result<(), String> {
    let group_id_num = extract_group_id(obj, "groupId")?;
    let gid = group_id_num.map(|n| ctx.resolve_group_id(n));

    // Break branch
    out.push(FormatElement::Tag(Tag::StartConditionalContent(
        Condition::new(PrintMode::Expanded).with_group_id(gid),
    )));
    if let Some(break_contents) = obj.get("breakContents") {
        convert_doc(break_contents, out, ctx)?;
    }
    out.push(FormatElement::Tag(Tag::EndConditionalContent));

    // Flat branch
    out.push(FormatElement::Tag(Tag::StartConditionalContent(
        Condition::new(PrintMode::Flat).with_group_id(gid),
    )));
    if let Some(flat_contents) = obj.get("flatContents") {
        convert_doc(flat_contents, out, ctx)?;
    }
    out.push(FormatElement::Tag(Tag::EndConditionalContent));

    Ok(())
}

fn convert_indent_if_break<'a>(
    obj: &serde_json::Map<String, Value>,
    out: &mut ArenaVec<'a, FormatElement<'a>>,
    ctx: &mut FmtCtx<'a, '_>,
) -> Result<(), String> {
    if obj.get("negate").and_then(Value::as_bool).unwrap_or(false) {
        return Err("Unsupported: indent-if-break with 'negate'".to_string());
    }
    let Some(group_id_num) = extract_group_id(obj, "groupId")? else {
        return Err("indent-if-break requires 'groupId'".to_string());
    };
    let gid = ctx.resolve_group_id(group_id_num);

    out.push(FormatElement::Tag(Tag::StartIndentIfGroupBreaks(gid)));
    if let Some(contents) = obj.get("contents") {
        convert_doc(contents, out, ctx)?;
    }
    out.push(FormatElement::Tag(Tag::EndIndentIfGroupBreaks(gid)));
    Ok(())
}

fn convert_fill<'a>(
    obj: &serde_json::Map<String, Value>,
    out: &mut ArenaVec<'a, FormatElement<'a>>,
    ctx: &mut FmtCtx<'a, '_>,
) -> Result<(), String> {
    out.push(FormatElement::Tag(Tag::StartFill));
    if let Some(Value::Array(parts)) = obj.get("parts") {
        for part in parts {
            out.push(FormatElement::Tag(Tag::StartEntry));
            convert_doc(part, out, ctx)?;
            out.push(FormatElement::Tag(Tag::EndEntry));
        }
    }
    out.push(FormatElement::Tag(Tag::EndFill));
    Ok(())
}

fn convert_line_suffix<'a>(
    obj: &serde_json::Map<String, Value>,
    out: &mut ArenaVec<'a, FormatElement<'a>>,
    ctx: &mut FmtCtx<'a, '_>,
) -> Result<(), String> {
    out.push(FormatElement::Tag(Tag::StartLineSuffix));
    if let Some(contents) = obj.get("contents") {
        convert_doc(contents, out, ctx)?;
    }
    out.push(FormatElement::Tag(Tag::EndLineSuffix));
    Ok(())
}

/// Extracts a numeric group ID from a Doc object field.
/// The ID may be a number (from Symbol→numeric conversion in JS) or a string like "G123".
fn extract_group_id(
    obj: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<Option<u32>, String> {
    match obj.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => n
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .map(Some)
            .ok_or_else(|| format!("Invalid group ID: {n}")),
        Some(other) => Err(format!("Invalid group ID: {other}")),
    }
}

/// Post-process converted FormatElements in a single compaction pass —
/// the finishing step of the Doc→IR conversion (Prettier-fallback path only;
/// Rust formatters write IR that never needs it):
/// - strip trailing hardline (useless for embedded parts)
/// - collapse double-hardlines `[HardWithoutExpand, ExpandParent, HardWithoutExpand, ExpandParent]` → `[Empty, ExpandParent]`
/// - merge consecutive Text nodes (the Prettier Doc path can emit adjacent `Text`s)
/// - trim a Text's trailing spaces/tabs when a hard/empty line follows:
///   Prettier's own printer trims at every line break,
///   so a Doc can rightfully carry them, but the core printer does not.
///   Untrimmed they would leak into the output verbatim.
///   (A single trailing space before a MAY-break line is already mapped to `Space` at conversion,
///   see `convert_doc`'s String arm; this pass covers the statically-known hard breaks,
///   where full runs and tabs can be dropped.)
pub fn postprocess<'a>(ir: &mut ArenaVec<'a, FormatElement<'a>>, allocator: &'a Allocator) {
    // Strip trailing hardline
    if ir.len() >= 2
        && matches!(ir[ir.len() - 1], FormatElement::ExpandParent)
        && matches!(ir[ir.len() - 2], FormatElement::Line(LineMode::HardWithoutExpand))
    {
        let new_len = ir.len() - 2;
        ir.truncate(new_len);
    }

    let mut write = 0;
    let mut read = 0;
    while read < ir.len() {
        // Collapse double-hardline → empty line
        if read + 3 < ir.len()
            && matches!(ir[read], FormatElement::Line(LineMode::HardWithoutExpand))
            && matches!(ir[read + 1], FormatElement::ExpandParent)
            && matches!(ir[read + 2], FormatElement::Line(LineMode::HardWithoutExpand))
            && matches!(ir[read + 3], FormatElement::ExpandParent)
        {
            ir[write] = FormatElement::Line(LineMode::Empty);
            ir[write + 1] = FormatElement::ExpandParent;
            write += 2;
            read += 4;
        } else if matches!(ir[read], FormatElement::Text { .. }) {
            // Merge consecutive Text nodes
            let run_start = read;
            read += 1;
            while read < ir.len() && matches!(ir[read], FormatElement::Text { .. }) {
                read += 1;
            }
            let single = read - run_start == 1;
            let text: &str = if single {
                let FormatElement::Text { text, .. } = ir[run_start] else { unreachable!() };
                text
            } else {
                let mut sb = ArenaStringBuilder::new_in(allocator);
                for element in &ir[run_start..read] {
                    if let FormatElement::Text { text, .. } = element {
                        sb.push_str(text);
                    }
                }
                sb.into_str()
            };
            // Prettier's own printer trims at every line break regardless of the doc structure around it,
            // so a break hiding behind tags (`Text("a  "), StartIndent, <hard line>`
            // from `["a  ", indent([hardline, ..])]`) still trims,
            // look through tag/expand-parent markers for it (only when there is anything to trim in the first place).
            let trimmed = if text.ends_with([' ', '\t'])
                && ir[read..]
                    .iter()
                    .find(|el| !matches!(el, FormatElement::Tag(_) | FormatElement::ExpandParent))
                    .is_some_and(|el| {
                        matches!(
                            el,
                            FormatElement::Line(LineMode::HardWithoutExpand | LineMode::Empty)
                        )
                    }) {
                text.trim_end_matches([' ', '\t'])
            } else {
                text
            };
            if single && trimmed.len() == text.len() {
                if write != run_start {
                    ir[write] = ir[run_start].clone();
                }
            } else {
                ir[write] = text_element(trimmed);
            }
            write += 1;
        } else {
            if write != read {
                ir[write] = ir[read].clone();
            }
            write += 1;
            read += 1;
        }
    }
    ir.truncate(write);
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_formatter_core::{Document, PrintWidth, PrinterOptions, UniqueGroupIdBuilder};
    use serde_json::{Value, json};

    use super::{convert_envelope, postprocess};

    fn print_doc(doc: &Value, print_width: u32) -> String {
        let allocator = Allocator::default();
        let group_id_builder = UniqueGroupIdBuilder::default();
        let (mut ir, _) =
            convert_envelope(json!([doc, {}]), &allocator, &group_id_builder).unwrap();
        postprocess(&mut ir, &allocator);
        Document::new(ir, vec![])
            .print(0, PrinterOptions::default().with_print_width(PrintWidth::new(print_width)))
            .unwrap()
            .into_code()
    }

    #[test]
    fn conditional_group_selects_the_first_fitting_state() {
        let group = json!({
            "type": "group",
            "contents": "1234567890",
            "expandedStates": [
                "1234567890",
                ["12345", { "type": "line" }, "678"],
                ["1234", { "type": "line" }, "5678"]
            ]
        });

        assert_eq!(print_doc(&group, 10), "1234567890");
        assert_eq!(print_doc(&group, 9), "12345 678");
        assert_eq!(print_doc(&group, 4), "1234\n5678");
    }

    #[test]
    fn conditional_group_with_one_state_is_a_regular_group() {
        let group = json!({
            "type": "group",
            "contents": ["a", { "type": "line" }, "b"],
            "expandedStates": [["a", { "type": "line" }, "b"]]
        });

        assert_eq!(print_doc(&group, 80), "a b");
    }

    #[test]
    fn forced_conditional_group_uses_only_the_final_state() {
        let group = json!({
            "type": "group",
            "break": true,
            "contents": "flat",
            "expandedStates": [
                "flat",
                ["final", { "type": "line" }, "state"]
            ]
        });

        assert_eq!(print_doc(&group, 80), "final\nstate");
    }

    #[test]
    fn conditional_group_id_exposes_the_selected_mode_to_if_break() {
        let flat_if_break = json!({
            "type": "if-break",
            "breakContents": "B",
            "flatContents": "F",
            "groupId": 1
        });
        let group = json!({
            "type": "group",
            "id": 1,
            "contents": ["flat", flat_if_break],
            "expandedStates": [
                ["flat", flat_if_break],
                ["x", { "type": "line" }, "y", flat_if_break]
            ]
        });

        assert_eq!(print_doc(&group, 80), "flatF");
        assert_eq!(print_doc(&group, 1), "x\nyB");
    }

    #[test]
    fn conditional_group_variant_keeps_its_trailing_hardline() {
        let hardline = json!([
            { "type": "line", "hard": true },
            { "type": "break-parent" }
        ]);
        let doc = json!([
            {
                "type": "group",
                "contents": ["flat", hardline],
                "expandedStates": [
                    ["flat", hardline],
                    ["expanded", hardline]
                ]
            },
            "after"
        ]);

        assert_eq!(print_doc(&doc, 80), "flat\nafter");
    }
}
