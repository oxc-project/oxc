use std::num::NonZeroU8;

use rustc_hash::FxHashMap;
use serde_json::Value;

use oxc_allocator::{Allocator, StringBuilder};
use oxc_formatter::{
    Align, Condition, DedentMode, FormatElement, Group, GroupId, GroupMode, IndentWidth, LineMode,
    PrintMode, Tag, TextWidth, UniqueGroupIdBuilder,
};

/// Marker string used to represent `-Infinity` in JSON.
/// JS side replaces `-Infinity` with this string before `JSON.stringify()`.
/// See `src-js/lib/apis.ts` for details.
const NEGATIVE_INFINITY_MARKER: &str = "__NEGATIVE_INFINITY__";

/// Converts a Prettier Doc JSON value into a flat `Vec<FormatElement<'a>>`,
/// with template-specific text escaping applied as post-processing.
pub fn to_format_elements_for_template<'a>(
    doc: &Value,
    allocator: &'a Allocator,
    group_id_builder: &UniqueGroupIdBuilder,
) -> Result<Vec<FormatElement<'a>>, String> {
    let mut ctx = FmtCtx::new(allocator, group_id_builder);
    let mut out = vec![];
    convert_doc(doc, &mut out, &mut ctx)?;

    postprocess(&mut out, |fe| {
        if let FormatElement::Text { text, width } = fe {
            // Some characters (e.g. backticks) should be escaped in template literals
            let escaped = escape_template_characters(text, allocator);
            if !std::ptr::eq(*text, escaped) {
                *text = escaped;
                // NOTE: `IndentWidth` only affects tab character width calculation.
                // If a `Doc = string` node contained `\t` (e.g. inside a string literal like `"\t"`?),
                // the width could be miscalculated when `options.indent_width` != 2.
                // However, the default value is sufficient in practice.
                *width = TextWidth::from_text(escaped, IndentWidth::default());
            }
        }
    });

    Ok(out)
}

// ---

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
    out: &mut Vec<FormatElement<'a>>,
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
                    convert_line(obj, out, ctx);
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
    out: &mut Vec<FormatElement<'a>>,
    ctx: &FmtCtx<'a, '_>,
) {
    let hard = obj.get("hard").and_then(Value::as_bool).unwrap_or(false);
    let soft = obj.get("soft").and_then(Value::as_bool).unwrap_or(false);
    let literal = obj.get("literal").and_then(Value::as_bool).unwrap_or(false);

    if hard && literal {
        let arena_text = ctx.allocator.alloc_str("\n");
        let width = TextWidth::multiline(0);
        out.push(FormatElement::Text { text: arena_text, width });
        out.push(FormatElement::ExpandParent);
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
    out: &mut Vec<FormatElement<'a>>,
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
    out: &mut Vec<FormatElement<'a>>,
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
    out: &mut Vec<FormatElement<'a>>,
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
                } else if i > 0 && i <= 255 {
                    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let count = i as u8;
                    if let Some(nz) = NonZeroU8::new(count) {
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
        _ => Err(format!("Unsupported align value: {n}")),
    }
}

fn convert_if_break<'a>(
    obj: &serde_json::Map<String, Value>,
    out: &mut Vec<FormatElement<'a>>,
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
    out: &mut Vec<FormatElement<'a>>,
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
    out: &mut Vec<FormatElement<'a>>,
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
    out: &mut Vec<FormatElement<'a>>,
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

// ---

/// Post-process `FormatElement`s:
/// - strip trailing hardline
/// - collapse consecutive hardlines into empty lines
///
/// And apply a per-element callback for custom transformations.
fn postprocess<'a>(ir: &mut Vec<FormatElement<'a>>, mut each: impl FnMut(&mut FormatElement<'a>)) {
    // Strip trailing `hardline` pattern from FormatElement output.
    // Trailing line is useless for embedded parts.
    if ir.len() >= 2
        && matches!(ir[ir.len() - 1], FormatElement::ExpandParent)
        && matches!(ir[ir.len() - 2], FormatElement::Line(LineMode::Hard))
    {
        ir.truncate(ir.len() - 2);
    }

    // Collapse consecutive `[Line(Hard), ExpandParent, Line(Hard), ExpandParent]` into `[Line(Empty), ExpandParent]`.
    //
    // In Prettier's Doc format, a blank line is represented as `hardline,
    // hardline` which expands to `[Line(Hard), ExpandParent, Line(Hard), ExpandParent]`.
    // However, `oxc_formatter`'s printer needs `Line(Empty)` instead.
    let mut write = 0;
    let mut read = 0;
    while read < ir.len() {
        if read + 3 < ir.len()
            && matches!(ir[read], FormatElement::Line(LineMode::Hard))
            && matches!(ir[read + 1], FormatElement::ExpandParent)
            && matches!(ir[read + 2], FormatElement::Line(LineMode::Hard))
            && matches!(ir[read + 3], FormatElement::ExpandParent)
        {
            ir[write] = FormatElement::Line(LineMode::Empty);
            ir[write + 1] = FormatElement::ExpandParent;
            write += 2;
            read += 4;
        } else {
            if write != read {
                ir[write] = ir[read].clone();
            }
            each(&mut ir[write]);
            write += 1;
            read += 1;
        }
    }

    ir.truncate(write);
}

/// Escape characters that would break template literal syntax.
///
/// Equivalent to Prettier's `uncookTemplateElementValue`:
/// `cookedValue.replaceAll(/([\\`]|\$\{)/gu, String.raw`\$1`);`
fn escape_template_characters<'a>(s: &'a str, allocator: &'a Allocator) -> &'a str {
    let bytes = s.as_bytes();
    let len = bytes.len();

    // Fast path: scan for characters that need escaping.
    let first_escape = (0..len).find(|&i| {
        let ch = bytes[i];
        ch == b'\\' || ch == b'`' || (ch == b'$' && i + 1 < len && bytes[i + 1] == b'{')
    });

    let Some(first) = first_escape else {
        return s;
    };

    // Slow path: build escaped string in the arena.
    let mut result = StringBuilder::with_capacity_in(len + 1, allocator);
    result.push_str(&s[..first]);

    let mut i = first;
    while i < len {
        let ch = bytes[i];
        if ch == b'\\' || ch == b'`' {
            result.push('\\');
            result.push(ch as char);
        } else if ch == b'$' && i + 1 < len && bytes[i + 1] == b'{' {
            result.push_str("\\${");
            i += 1; // skip '{'
        } else {
            result.push(ch as char);
        }
        i += 1;
    }

    result.into_str()
}
