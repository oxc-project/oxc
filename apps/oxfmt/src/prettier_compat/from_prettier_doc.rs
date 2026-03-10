use std::num::NonZeroU8;

use rustc_hash::FxHashMap;
use serde_json::Value;

use oxc_allocator::{Allocator, StringBuilder};
use oxc_formatter::{
    Align, Condition, DedentMode, EmbeddedDocResult, FormatElement, Group, GroupId, GroupMode,
    IndentWidth, LineMode, PrintMode, Tag, TextWidth, UniqueGroupIdBuilder,
};

/// Marker string used to represent `-Infinity` in JSON.
/// JS side replaces `-Infinity` with this string before `JSON.stringify()`.
/// See `src-js/lib/apis.ts` for details.
const NEGATIVE_INFINITY_MARKER: &str = "__NEGATIVE_INFINITY__";

/// Converts a Prettier Doc JSON value into [`FormatElement`]s for an entire file.
///
/// Unlike [`to_format_elements_for_template`], this does not apply template-specific
/// postprocessing (escape template characters, count placeholders, strip trailing hardlines).
pub fn to_format_elements_for_file<'a>(
    doc_json: &Value,
    allocator: &'a Allocator,
    group_id_builder: &UniqueGroupIdBuilder,
) -> Result<Vec<FormatElement<'a>>, String> {
    let mut ctx = FmtCtx::new(allocator, group_id_builder);
    let mut out = vec![];
    convert_doc(doc_json, &mut out, &mut ctx)?;
    collapse_double_hardlines(&mut out);
    Ok(out)
}

/// Collapse double-hardline sequences `[Hard, ExpandParent, Hard, ExpandParent]`
/// into `[Empty, ExpandParent]` (empty line).
///
/// This is the subset of [`postprocess`] that applies to file-level formatting.
fn collapse_double_hardlines(ir: &mut Vec<FormatElement>) {
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
            write += 1;
            read += 1;
        }
    }
    ir.truncate(write);
}

/// Converts parsed Prettier Doc JSON values into an [`EmbeddedDocResult`].
///
/// Handles language-specific processing:
/// - GraphQL: converts each doc independently → [`EmbeddedDocResult::MultipleDocs`]
/// - CSS, HTML: merges consecutive Text nodes, counts placeholders → [`EmbeddedDocResult::DocWithPlaceholders`]
pub fn to_format_elements_for_template<'a>(
    language: &str,
    doc_jsons: &[Value],
    allocator: &'a Allocator,
    group_id_builder: &UniqueGroupIdBuilder,
) -> Result<EmbeddedDocResult<'a>, String> {
    let convert = |doc_json: &Value| -> Result<(Vec<FormatElement<'a>>, usize), String> {
        let mut ctx = FmtCtx::new(allocator, group_id_builder);
        let mut out = vec![];
        convert_doc(doc_json, &mut out, &mut ctx)?;
        let placeholder_count = postprocess(&mut out, allocator);
        Ok((out, placeholder_count))
    };

    match language {
        "tagged-css" => {
            let doc_json = doc_jsons
                .first()
                .ok_or_else(|| "Expected exactly one Doc JSON for CSS".to_string())?;
            let (ir, count) = convert(doc_json)?;
            Ok(EmbeddedDocResult::DocWithPlaceholders(ir, count))
        }
        "tagged-graphql" => {
            let irs = doc_jsons
                .iter()
                .map(|doc_json| {
                    let (ir, _) = convert(doc_json)?;
                    Ok(ir)
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(EmbeddedDocResult::MultipleDocs(irs))
        }
        _ => unreachable!("Unsupported embedded_doc language: {language}"),
    }
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

/// Post-process FormatElements in a single compaction pass:
/// - strip trailing hardline (useless for embedded parts)
/// - collapse double-hardlines `[Hard, ExpandParent, Hard, ExpandParent]` → `[Empty, ExpandParent]`
/// - merge consecutive Text nodes (SCSS emits split strings like `"@"` + `"prettier-placeholder-0-id"`)
/// - escape template characters (`\`, `` ` ``, `${`)
/// - count `@prettier-placeholder-N-id` patterns
///
/// Returns the placeholder count (0 for non-CSS languages).
fn postprocess<'a>(ir: &mut Vec<FormatElement<'a>>, allocator: &'a Allocator) -> usize {
    const PREFIX: &str = "@prettier-placeholder-";
    const SUFFIX: &str = "-id";

    // Strip trailing hardline
    if ir.len() >= 2
        && matches!(ir[ir.len() - 1], FormatElement::ExpandParent)
        && matches!(ir[ir.len() - 2], FormatElement::Line(LineMode::Hard))
    {
        ir.truncate(ir.len() - 2);
    }

    let mut placeholder_count = 0;
    let mut write = 0;
    let mut read = 0;
    while read < ir.len() {
        // Collapse double-hardline → empty line
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
        } else if matches!(ir[read], FormatElement::Text { .. }) {
            // Merge consecutive Text nodes + escape + count placeholders
            let run_start = read;
            read += 1;
            while read < ir.len() && matches!(ir[read], FormatElement::Text { .. }) {
                read += 1;
            }

            let escaped = if read - run_start == 1 {
                let FormatElement::Text { text, .. } = &ir[run_start] else { unreachable!() };
                escape_template_characters(text, allocator)
            } else {
                let mut sb = StringBuilder::new_in(allocator);
                for element in &ir[run_start..read] {
                    if let FormatElement::Text { text, .. } = element {
                        sb.push_str(text);
                    }
                }
                escape_template_characters(sb.into_str(), allocator)
            };
            let width = TextWidth::from_text(escaped, IndentWidth::default());
            ir[write] = FormatElement::Text { text: escaped, width };
            write += 1;

            // Count placeholders
            let mut remaining = escaped;
            while let Some(start) = remaining.find(PREFIX) {
                let after_prefix = &remaining[start + PREFIX.len()..];
                let digit_end = after_prefix
                    .bytes()
                    .position(|b| !b.is_ascii_digit())
                    .unwrap_or(after_prefix.len());
                if digit_end > 0
                    && let Some(rest) = after_prefix[digit_end..].strip_prefix(SUFFIX)
                {
                    placeholder_count += 1;
                    remaining = rest;
                    continue;
                }
                remaining = &remaining[start + PREFIX.len()..];
            }
        } else {
            if write != read {
                ir[write] = ir[read].clone();
            }
            write += 1;
            read += 1;
        }
    }
    ir.truncate(write);
    placeholder_count
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
