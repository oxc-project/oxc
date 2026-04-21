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

/// Converts parsed Prettier Doc JSON values into an [`EmbeddedDocResult`].
///
/// Handles language-specific processing:
/// - GraphQL: converts each doc independently → [`EmbeddedDocResult::MultipleDocs`]
/// - CSS, HTML: merges consecutive Text nodes, counts placeholders → [`EmbeddedDocResult::DocWithPlaceholders`]
///
/// All Doc JSONs use a uniform `[doc, metadata]` envelope from the JS side.
pub fn to_format_elements_for_template<'a>(
    language: &str,
    doc_jsons: Vec<Value>,
    allocator: &'a Allocator,
    group_id_builder: &UniqueGroupIdBuilder,
) -> Result<EmbeddedDocResult<'a>, String> {
    /// Unwrap `[doc, metadata]` envelope and convert doc JSON to IR.
    /// Panics on invalid envelope format (internal protocol we control on both sides).
    fn convert<'a>(
        envelope: Value,
        allocator: &'a Allocator,
        group_id_builder: &UniqueGroupIdBuilder,
    ) -> Result<(Vec<FormatElement<'a>>, serde_json::Map<String, Value>), String> {
        let Value::Array(mut arr) = envelope else {
            unreachable!("Doc JSON envelope must be [doc, metadata]");
        };
        let metadata = match arr.pop() {
            Some(Value::Object(obj)) => obj,
            _ => serde_json::Map::new(),
        };
        let doc_json = arr.into_iter().next().expect("Doc JSON envelope must contain doc");

        let mut ctx = FmtCtx::new(allocator, group_id_builder);
        let mut ir = vec![];
        convert_doc(&doc_json, &mut ir, &mut ctx)?;
        Ok((ir, metadata))
    }

    match language {
        "graphql" => {
            let irs = doc_jsons
                .into_iter()
                .map(|envelope| {
                    let (mut ir, _) = convert(envelope, allocator, group_id_builder)?;
                    postprocess(
                        &mut ir,
                        allocator,
                        // GraphQL uses `.cooked` values, so template chars need escaping
                        TemplateEscape::Full,
                        None,
                    );
                    Ok(ir)
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(EmbeddedDocResult::MultipleDocs(irs))
        }
        "css" => {
            let (mut ir, _) = convert(
                doc_jsons.into_iter().next().expect("Doc JSON for CSS"),
                allocator,
                group_id_builder,
            )?;
            let placeholder_count = postprocess(
                &mut ir,
                allocator,
                // CSS uses `.raw` values, so no template char escaping needed
                TemplateEscape::None,
                Some(("@prettier-placeholder-", "-id")),
            );
            Ok(EmbeddedDocResult::DocWithPlaceholders {
                ir,
                placeholder_count,
                html_has_multiple_root_elements: None,
            })
        }
        "html" | "angular" => {
            let (mut ir, metadata) = convert(
                doc_jsons.into_iter().next().expect("Doc JSON for HTML"),
                allocator,
                group_id_builder,
            )?;
            let html_has_multiple_root_elements =
                metadata.get("htmlHasMultipleRootElements").and_then(Value::as_bool);
            let placeholder_count = postprocess(
                &mut ir,
                allocator,
                // HTML/Angular use `.cooked` values, so template chars need escaping
                TemplateEscape::Full,
                Some(("PRETTIER_HTML_PLACEHOLDER_", "_IN_JS")),
            );
            Ok(EmbeddedDocResult::DocWithPlaceholders {
                ir,
                placeholder_count,
                html_has_multiple_root_elements,
            })
        }
        "markdown" => {
            let (mut ir, _) = convert(
                doc_jsons.into_iter().next().expect("Doc JSON for Markdown"),
                allocator,
                group_id_builder,
            )?;
            postprocess(
                &mut ir,
                allocator,
                // Markdown uses `.raw` values with backtick unescaping on Rust side
                TemplateEscape::RawBacktick,
                None,
            );
            Ok(EmbeddedDocResult::SingleDoc(ir))
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
            // `align({type: "root"}, ...)` = Prettier's `markAsRoot()`.
            // In Prettier, `markAsRoot` records the current indent position
            // so that a later `dedentToRoot` can return to it.
            // However, `oxc_formatter`'s `DedentMode::Root` always resets to absolute level 0
            // and has no way to store a custom root position.
            // Skipping the root capture is safe because
            // embedded language Docs are processed in their own context starting near level 0,
            // so `dedentToRoot` to absolute 0 produces the same result.
            //
            // NOTE: `markAsRoot` is used in Prettier for other cases.
            // e.g. JS comment printer, YAML block printer, and front-matter embed.
            // But none of those go through this Doc→IR path.
            if obj_val.get("type").and_then(Value::as_str) == Some("root") {
                if let Some(contents) = obj.get("contents") {
                    convert_doc(contents, out, ctx)?;
                }
                return Ok(());
            }
            Err(format!("Unsupported align value: {n}"))
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

#[derive(Clone, Copy)]
enum TemplateEscape {
    /// No escaping
    None,
    /// Full escaping: `\` → `\\`, `` ` `` → `` \` ``, `${` → `\${`.
    Full,
    /// Raw backtick escaping: `(\\*)\`` → `$1$1\\\``.
    RawBacktick,
}

/// Post-process FormatElements in a single compaction pass:
/// - strip trailing hardline (useless for embedded parts)
/// - collapse double-hardlines `[Hard, ExpandParent, Hard, ExpandParent]` → `[Empty, ExpandParent]`
/// - merge consecutive Text nodes (SCSS emits split strings like `"@"` + `"prettier-placeholder-0-id"`)
/// - escape template characters (mode determined by [`TemplateEscape`])
/// - count placeholders matching `(prefix)(digits)(_digits)?(suffix)` pattern
///
/// Returns the placeholder count (0 when `placeholder` is `None`).
fn postprocess<'a>(
    ir: &mut Vec<FormatElement<'a>>,
    allocator: &'a Allocator,
    escape: TemplateEscape,
    placeholder: Option<(&str, &str)>,
) -> usize {
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

            let text = if read - run_start == 1 {
                let FormatElement::Text { text, .. } = &ir[run_start] else { unreachable!() };
                text
            } else {
                let mut sb = StringBuilder::new_in(allocator);
                for element in &ir[run_start..read] {
                    if let FormatElement::Text { text, .. } = element {
                        sb.push_str(text);
                    }
                }
                sb.into_str()
            };
            let text = match escape {
                TemplateEscape::None => text,
                TemplateEscape::Full => escape_template_characters(text, allocator),
                TemplateEscape::RawBacktick => escape_backticks_raw_str(text, allocator),
            };
            let width = TextWidth::from_text(text, IndentWidth::default());
            ir[write] = FormatElement::Text { text, width };
            write += 1;

            // Count placeholders for this text if needed
            if let Some((prefix, suffix)) = placeholder {
                placeholder_count += count_placeholders(text, prefix, suffix);
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

/// Count placeholder occurrences matching `{prefix}{digits}(_{digits})?{suffix}` in text.
///
/// The optional `_{digits}` group allows matching both formats:
/// - CSS: `@prettier-placeholder-0-id` (no counter)
/// - HTML: `PRETTIER_HTML_PLACEHOLDER_0_0_IN_JS` (with counter)
fn count_placeholders(text: &str, prefix: &str, suffix: &str) -> usize {
    let mut count = 0;
    let mut remaining = text;
    while let Some(start) = remaining.find(prefix) {
        let after_prefix = &remaining[start + prefix.len()..];
        let digit_end =
            after_prefix.bytes().position(|b| !b.is_ascii_digit()).unwrap_or(after_prefix.len());
        if digit_end > 0 {
            let mut after_digits = &after_prefix[digit_end..];
            // Skip optional `_{digits}` (e.g., HTML counter)
            if let Some(after_underscore) = after_digits.strip_prefix('_') {
                let c = after_underscore
                    .bytes()
                    .position(|b| !b.is_ascii_digit())
                    .unwrap_or(after_underscore.len());
                if c > 0 {
                    after_digits = &after_underscore[c..];
                }
            }
            if let Some(rest) = after_digits.strip_prefix(suffix) {
                count += 1;
                remaining = rest;
                continue;
            }
        }
        remaining = &remaining[start + prefix.len()..];
    }
    count
}

/// Escape characters that would break template literal syntax.
///
/// Equivalent to Prettier's `uncookTemplateElementValue`:
/// `cookedValue.replaceAll(/([\\`]|\$\{)/gu, String.raw`\$1`);`
fn escape_template_characters<'a>(s: &'a str, allocator: &'a Allocator) -> &'a str {
    let bytes = s.as_bytes();
    let len = bytes.len();

    // Fast path: scan for the first character that needs escaping.
    // All characters of interest (`\`, `` ` ``, `$`, `{`) are single-byte ASCII,
    // so byte-indexed access is safe and avoids multi-byte decode overhead.
    let first_escape = (0..len).find(|&i| {
        let ch = bytes[i];
        ch == b'\\' || ch == b'`' || (ch == b'$' && i + 1 < len && bytes[i + 1] == b'{')
    });

    let Some(first) = first_escape else {
        return s;
    };

    // Slow path: build escaped string in the arena, reusing the clean prefix.
    let mut result = StringBuilder::with_capacity_in(len + 1, allocator);
    result.push_str(&s[..first]);

    // Iterate by chars (not bytes) to correctly handle multi-byte UTF-8.
    // All escape targets (`\`, `` ` ``, `${`) are ASCII, so this is straightforward.
    let mut chars = s[first..].chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' || ch == '`' {
            result.push('\\');
            result.push(ch);
        } else if ch == '$' && chars.peek() == Some(&'{') {
            result.push_str("\\${");
            chars.next(); // skip '{'
        } else {
            result.push(ch);
        }
    }

    result.into_str()
}

/// Escape backticks in raw mode for markdown-in-JS template literals.
///
/// Equivalent to Prettier's `escapeTemplateCharacters(doc, /* raw */ true)`:
/// <https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/print/template-literal.js#L277-L287>
/// `str.replaceAll(/(\\*)`/g, "$1$1\\`")`
///
/// For each backtick, doubles the preceding backslashes and adds `\` before the backtick:
/// - `` ` `` → `` \` ``
/// - `` \` `` → `` \\\` ``
/// - `` \\` `` → `` \\\\\` ``
fn escape_backticks_raw_str<'a>(s: &'a str, allocator: &'a Allocator) -> &'a str {
    if !s.contains('`') {
        return s;
    }
    let mut result = StringBuilder::with_capacity_in(s.len() + 1, allocator);
    let mut bs_count: usize = 0;
    for ch in s.chars() {
        if ch == '\\' {
            bs_count += 1;
            result.push('\\');
        } else if ch == '`' {
            // The backslash branch already emitted `bs_count` backslashes.
            // Emit another `bs_count` to double them, then add `\``.
            for _ in 0..bs_count {
                result.push('\\');
            }
            result.push('\\');
            result.push('`');
            bs_count = 0;
        } else {
            bs_count = 0;
            result.push(ch);
        }
    }
    result.into_str()
}
