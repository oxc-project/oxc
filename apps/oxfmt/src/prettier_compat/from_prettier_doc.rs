//! Prettier Doc JSON → FormatElement IR conversion primitives.
//!
//! [`convert_envelope`] is the public entry point:
//! it unwraps the `[doc, metadata]` envelope sent from the JS side
//! and converts the doc through the private `convert_*` walkers into a flat `FormatElement` IR.
//! Language-specific routing and postprocessing live in `core::embed`.

use std::num::NonZeroU8;

use rustc_hash::FxHashMap;
use serde_json::Value;

use oxc_allocator::{Allocator, ArenaVec};
use oxc_formatter_core::{
    Align, Condition, DedentMode, FormatElement, Group, GroupId, GroupMode, IndentWidth, LineMode,
    PrintMode, Tag, TextWidth, UniqueGroupIdBuilder,
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

fn convert_doc<'a>(
    doc: &Value,
    out: &mut ArenaVec<'a, FormatElement<'a>>,
    ctx: &mut FmtCtx<'a, '_>,
) -> Result<(), String> {
    match doc {
        Value::String(s) => {
            if !s.is_empty() {
                let text = ctx.allocator.alloc_str(s);
                // NOTE: `IndentWidth` only affects tab character width calculation.
                // If a `Doc = string` node contained `\t` (e.g. inside a string literal like `"\t"`?),
                // the width could be miscalculated when `options.indent_width` != 2.
                // However, the default value is sufficient in practice.
                let width = TextWidth::from_text(text, IndentWidth::default());
                out.push(FormatElement::Text { text, width });
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
        out.push(FormatElement::Line(LineMode::Literal));
    } else if hard {
        out.push(FormatElement::Line(LineMode::Hard));
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
    if obj.contains_key("expandedStates") {
        return Err("Unsupported: group with 'expandedStates' (conditionalGroup)".to_string());
    }

    let should_break = obj.get("break").and_then(Value::as_bool).unwrap_or(false);
    let id = extract_group_id(obj, "id")?;

    let gid = id.map(|n| ctx.resolve_group_id(n));
    let mode = if should_break { GroupMode::Expand } else { GroupMode::Flat };
    out.push(FormatElement::Tag(Tag::StartGroup(Group::new().with_id(gid).with_mode(mode))));
    if let Some(contents) = obj.get("contents") {
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
