use serde_json::Value;

use oxc_formatter::{EmbeddedIR, LineMode, PrintMode};

/// Marker string used to represent `-Infinity` in JSON.
/// JS side replaces `-Infinity` with this string before `JSON.stringify()`.
/// See `src-js/lib/apis.ts` for details.
const NEGATIVE_INFINITY_MARKER: &str = "__NEGATIVE_INFINITY__";

/// Converts a Prettier Doc JSON value into a flat `Vec<EmbeddedIR>`.
///
/// This is the reverse of `to_prettier_doc.rs` which converts `FormatElement` → Prettier Doc JSON.
/// The Doc JSON comes from Prettier's `__debug.printToDoc()` API.
pub fn doc_json_to_embedded_ir(doc: &Value) -> Result<Vec<EmbeddedIR>, String> {
    let mut out = vec![];
    convert_doc(doc, &mut out)?;

    strip_trailing_hardline(&mut out);
    collapse_consecutive_hardlines(&mut out);

    Ok(out)
}

fn convert_doc(doc: &Value, out: &mut Vec<EmbeddedIR>) -> Result<(), String> {
    match doc {
        Value::String(s) => {
            if !s.is_empty() {
                out.push(EmbeddedIR::Text(s.clone()));
            }
            Ok(())
        }
        Value::Array(arr) => {
            for item in arr {
                convert_doc(item, out)?;
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
                "group" => convert_group(obj, out),
                "indent" => convert_indent(obj, out),
                "align" => convert_align(obj, out),
                "if-break" => convert_if_break(obj, out),
                "indent-if-break" => convert_indent_if_break(obj, out),
                "fill" => convert_fill(obj, out),
                "line-suffix" => convert_line_suffix(obj, out),
                "line-suffix-boundary" => {
                    out.push(EmbeddedIR::LineSuffixBoundary);
                    Ok(())
                }
                "break-parent" => {
                    out.push(EmbeddedIR::ExpandParent);
                    Ok(())
                }
                "label" => {
                    // Label is transparent in Prettier's printer (just processes contents)
                    if let Some(contents) = obj.get("contents") {
                        convert_doc(contents, out)?;
                    }
                    Ok(())
                }
                "cursor" => Ok(()), // Ignore cursor markers
                "trim" => Err("Unsupported Doc type: 'trim'".to_string()),
                _ => Err(format!("Unknown Doc type: '{doc_type}'")),
            }
        }
        Value::Null => Ok(()),
        _ => Err(format!("Unexpected Doc value type: {doc}")),
    }
}

fn convert_line(obj: &serde_json::Map<String, Value>, out: &mut Vec<EmbeddedIR>) {
    let hard = obj.get("hard").and_then(Value::as_bool).unwrap_or(false);
    let soft = obj.get("soft").and_then(Value::as_bool).unwrap_or(false);
    let literal = obj.get("literal").and_then(Value::as_bool).unwrap_or(false);

    if hard && literal {
        // literalline: newline without indent, plus break-parent
        // Reverse of to_prettier_doc.rs: Text("\n") → literalline
        out.push(EmbeddedIR::Text("\n".to_string()));
        out.push(EmbeddedIR::ExpandParent);
    } else if hard {
        out.push(EmbeddedIR::Line(LineMode::Hard));
    } else if soft {
        out.push(EmbeddedIR::Line(LineMode::Soft));
    } else {
        out.push(EmbeddedIR::Line(LineMode::SoftOrSpace));
    }
}

fn convert_group(
    obj: &serde_json::Map<String, Value>,
    out: &mut Vec<EmbeddedIR>,
) -> Result<(), String> {
    // Bail out on expandedStates (conditionalGroup)
    // Even in Prettier, only JS and YAML use this.
    if obj.contains_key("expandedStates") {
        return Err("Unsupported: group with 'expandedStates' (conditionalGroup)".to_string());
    }

    let should_break = obj.get("break").and_then(Value::as_bool).unwrap_or(false);
    let id = extract_group_id(obj, "id")?;

    out.push(EmbeddedIR::StartGroup { id, should_break });
    if let Some(contents) = obj.get("contents") {
        convert_doc(contents, out)?;
    }
    out.push(EmbeddedIR::EndGroup);
    Ok(())
}

fn convert_indent(
    obj: &serde_json::Map<String, Value>,
    out: &mut Vec<EmbeddedIR>,
) -> Result<(), String> {
    out.push(EmbeddedIR::StartIndent);
    if let Some(contents) = obj.get("contents") {
        convert_doc(contents, out)?;
    }
    out.push(EmbeddedIR::EndIndent);
    Ok(())
}

fn convert_align(
    obj: &serde_json::Map<String, Value>,
    out: &mut Vec<EmbeddedIR>,
) -> Result<(), String> {
    let n = &obj["n"];

    match n {
        // Numeric value
        Value::Number(num) => {
            if let Some(i) = num.as_i64() {
                if i == 0 {
                    // n=0: transparent (no-op), just emit contents
                    if let Some(contents) = obj.get("contents") {
                        convert_doc(contents, out)?;
                    }
                    return Ok(());
                } else if i == -1 {
                    // dedent (one level)
                    out.push(EmbeddedIR::StartDedent { to_root: false });
                    if let Some(contents) = obj.get("contents") {
                        convert_doc(contents, out)?;
                    }
                    out.push(EmbeddedIR::EndDedent { to_root: false });
                    return Ok(());
                } else if i > 0 && i <= 255 {
                    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let count = i as u8;
                    out.push(EmbeddedIR::StartAlign(count));
                    if let Some(contents) = obj.get("contents") {
                        convert_doc(contents, out)?;
                    }
                    out.push(EmbeddedIR::EndAlign);
                    return Ok(());
                }
            }
            // Fallthrough: n is a float or out of range
            Err(format!("Unsupported align value: {n}"))
        }
        // -Infinity marker string
        Value::String(s) if s == NEGATIVE_INFINITY_MARKER => {
            // dedentToRoot
            out.push(EmbeddedIR::StartDedent { to_root: true });
            if let Some(contents) = obj.get("contents") {
                convert_doc(contents, out)?;
            }
            out.push(EmbeddedIR::EndDedent { to_root: true });
            Ok(())
        }
        _ => Err(format!("Unsupported align value: {n}")),
    }
}

fn convert_if_break(
    obj: &serde_json::Map<String, Value>,
    out: &mut Vec<EmbeddedIR>,
) -> Result<(), String> {
    let group_id = extract_group_id(obj, "groupId")?;

    // Break branch
    out.push(EmbeddedIR::StartConditionalContent { mode: PrintMode::Expanded, group_id });
    if let Some(break_contents) = obj.get("breakContents") {
        convert_doc(break_contents, out)?;
    }
    out.push(EmbeddedIR::EndConditionalContent);

    // Flat branch
    out.push(EmbeddedIR::StartConditionalContent { mode: PrintMode::Flat, group_id });
    if let Some(flat_contents) = obj.get("flatContents") {
        convert_doc(flat_contents, out)?;
    }
    out.push(EmbeddedIR::EndConditionalContent);

    Ok(())
}

fn convert_indent_if_break(
    obj: &serde_json::Map<String, Value>,
    out: &mut Vec<EmbeddedIR>,
) -> Result<(), String> {
    // negate is not supported
    // Even in Prettier, HTML only uses `indentIfBreak()`, but `negate` is never used in the codebase!
    if obj.get("negate").and_then(Value::as_bool).unwrap_or(false) {
        return Err("Unsupported: indent-if-break with 'negate'".to_string());
    }
    let Some(group_id) = extract_group_id(obj, "groupId")? else {
        return Err("indent-if-break requires 'groupId'".to_string());
    };

    out.push(EmbeddedIR::StartIndentIfGroupBreaks(group_id));
    if let Some(contents) = obj.get("contents") {
        convert_doc(contents, out)?;
    }
    out.push(EmbeddedIR::EndIndentIfGroupBreaks(group_id));
    Ok(())
}

fn convert_fill(
    obj: &serde_json::Map<String, Value>,
    out: &mut Vec<EmbeddedIR>,
) -> Result<(), String> {
    out.push(EmbeddedIR::StartFill);
    if let Some(Value::Array(parts)) = obj.get("parts") {
        for part in parts {
            out.push(EmbeddedIR::StartEntry);
            convert_doc(part, out)?;
            out.push(EmbeddedIR::EndEntry);
        }
    }
    out.push(EmbeddedIR::EndFill);
    Ok(())
}

fn convert_line_suffix(
    obj: &serde_json::Map<String, Value>,
    out: &mut Vec<EmbeddedIR>,
) -> Result<(), String> {
    out.push(EmbeddedIR::StartLineSuffix);
    if let Some(contents) = obj.get("contents") {
        convert_doc(contents, out)?;
    }
    out.push(EmbeddedIR::EndLineSuffix);
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

/// Strip trailing `hardline` pattern from the IR.
///
/// Prettier's internal `textToDoc()` behavior which calls `stripTrailingHardline()` before returning.
/// `__debug.printToDoc()` does not do this, so we need to handle it here.
/// <https://github.com/prettier/prettier/blob/96596de06d9421e8cfffce21430b904ffbea73f1/src/main/multiparser.js#L131>
///
/// Prettier's `hardline` is `[line(hard), break-parent]`,
/// which maps to `[Line(Hard), ExpandParent]` in EmbeddedIR.
fn strip_trailing_hardline(ir: &mut Vec<EmbeddedIR>) {
    if ir.len() >= 2
        && matches!(ir[ir.len() - 1], EmbeddedIR::ExpandParent)
        && matches!(ir[ir.len() - 2], EmbeddedIR::Line(LineMode::Hard))
    {
        ir.truncate(ir.len() - 2);
    }
}

/// Collapse consecutive `[Line(Hard), ExpandParent, Line(Hard), ExpandParent]` into `[Line(Empty), ExpandParent]`.
///
/// In Prettier's Doc format, a blank line is represented as `hardline,
/// hardline` which expands to `[Line(Hard), ExpandParent, Line(Hard), ExpandParent]`.
/// However, oxc_formatter's printer needs `Line(Empty)` to produce a blank line (double newline).
fn collapse_consecutive_hardlines(ir: &mut Vec<EmbeddedIR>) {
    if ir.len() < 4 {
        return;
    }

    let mut write = 0;
    let mut read = 0;
    while read < ir.len() {
        // Check for the 4-element pattern: Line(Hard), ExpandParent, Line(Hard), ExpandParent
        if read + 3 < ir.len()
            && matches!(ir[read], EmbeddedIR::Line(LineMode::Hard))
            && matches!(ir[read + 1], EmbeddedIR::ExpandParent)
            && matches!(ir[read + 2], EmbeddedIR::Line(LineMode::Hard))
            && matches!(ir[read + 3], EmbeddedIR::ExpandParent)
        {
            ir[write] = EmbeddedIR::Line(LineMode::Empty);
            ir[write + 1] = EmbeddedIR::ExpandParent;
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

// ---

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_simple_string() {
        let doc = json!("hello");
        let ir = doc_json_to_embedded_ir(&doc).unwrap();
        assert_eq!(ir.len(), 1);
        assert!(matches!(&ir[0], EmbeddedIR::Text(s) if s == "hello"));
    }

    #[test]
    fn test_array() {
        let doc = json!(["a", "b"]);
        let ir = doc_json_to_embedded_ir(&doc).unwrap();
        assert_eq!(ir.len(), 2);
    }

    #[test]
    fn test_line_modes() {
        let soft = json!({"type": "line", "soft": true});
        let ir = doc_json_to_embedded_ir(&soft).unwrap();
        assert!(matches!(ir[0], EmbeddedIR::Line(LineMode::Soft)));

        let hard = json!({"type": "line", "hard": true});
        let ir = doc_json_to_embedded_ir(&hard).unwrap();
        assert!(matches!(ir[0], EmbeddedIR::Line(LineMode::Hard)));

        let literal = json!({"type": "line", "hard": true, "literal": true});
        let ir = doc_json_to_embedded_ir(&literal).unwrap();
        assert!(matches!(&ir[0], EmbeddedIR::Text(s) if s == "\n"));
        assert!(matches!(ir[1], EmbeddedIR::ExpandParent));
    }

    #[test]
    fn test_group() {
        let doc = json!({"type": "group", "contents": "hello", "break": true, "id": 1});
        let ir = doc_json_to_embedded_ir(&doc).unwrap();
        assert!(matches!(&ir[0], EmbeddedIR::StartGroup { id: Some(1), should_break: true }));
        assert!(matches!(&ir[1], EmbeddedIR::Text(s) if s == "hello"));
        assert!(matches!(ir[2], EmbeddedIR::EndGroup));
    }

    #[test]
    fn test_indent() {
        let doc = json!({"type": "indent", "contents": "x"});
        let ir = doc_json_to_embedded_ir(&doc).unwrap();
        assert!(matches!(ir[0], EmbeddedIR::StartIndent));
        assert!(matches!(ir[2], EmbeddedIR::EndIndent));
    }

    #[test]
    fn test_if_break_two_branches() {
        let doc = json!({
            "type": "if-break",
            "breakContents": "broken",
            "flatContents": "flat"
        });
        let ir = doc_json_to_embedded_ir(&doc).unwrap();
        // Break branch
        assert!(matches!(
            &ir[0],
            EmbeddedIR::StartConditionalContent { mode: PrintMode::Expanded, group_id: None }
        ));
        assert!(matches!(&ir[1], EmbeddedIR::Text(s) if s == "broken"));
        assert!(matches!(ir[2], EmbeddedIR::EndConditionalContent));
        // Flat branch
        assert!(matches!(
            &ir[3],
            EmbeddedIR::StartConditionalContent { mode: PrintMode::Flat, group_id: None }
        ));
        assert!(matches!(&ir[4], EmbeddedIR::Text(s) if s == "flat"));
        assert!(matches!(ir[5], EmbeddedIR::EndConditionalContent));
    }

    #[test]
    fn test_align_dedent() {
        let doc = json!({"type": "align", "n": -1, "contents": "x"});
        let ir = doc_json_to_embedded_ir(&doc).unwrap();
        assert!(matches!(ir[0], EmbeddedIR::StartDedent { to_root: false }));
    }

    #[test]
    fn test_align_dedent_to_root() {
        let doc = json!({"type": "align", "n": "__NEGATIVE_INFINITY__", "contents": "x"});
        let ir = doc_json_to_embedded_ir(&doc).unwrap();
        assert!(matches!(ir[0], EmbeddedIR::StartDedent { to_root: true }));
    }

    #[test]
    fn test_label_transparent() {
        let doc = json!({"type": "label", "label": {"hug": false}, "contents": "inner"});
        let ir = doc_json_to_embedded_ir(&doc).unwrap();
        assert_eq!(ir.len(), 1);
        assert!(matches!(&ir[0], EmbeddedIR::Text(s) if s == "inner"));
    }

    #[test]
    fn test_unknown_type_bail_out() {
        let doc = json!({"type": "unknown_thing"});
        assert!(doc_json_to_embedded_ir(&doc).is_err());
    }

    #[test]
    fn test_trim_bail_out() {
        let doc = json!({"type": "trim"});
        assert!(doc_json_to_embedded_ir(&doc).is_err());
    }

    #[test]
    fn test_expanded_states_bail_out() {
        let doc = json!({"type": "group", "contents": "", "expandedStates": []});
        assert!(doc_json_to_embedded_ir(&doc).is_err());
    }

    #[test]
    fn test_strip_trailing_hardline() {
        // hardline = [line(hard), break-parent]
        let doc = json!(["hello", {"type": "line", "hard": true}, {"type": "break-parent"}]);
        let ir = doc_json_to_embedded_ir(&doc).unwrap();
        // Trailing hardline should be stripped
        assert_eq!(ir.len(), 1);
        assert!(matches!(&ir[0], EmbeddedIR::Text(s) if s == "hello"));
    }

    #[test]
    fn test_no_strip_when_not_trailing_hardline() {
        // Only Line(Hard) without ExpandParent — should not strip
        let doc = json!(["hello", {"type": "line", "hard": true}]);
        let ir = doc_json_to_embedded_ir(&doc).unwrap();
        assert_eq!(ir.len(), 2);
        assert!(matches!(ir[1], EmbeddedIR::Line(LineMode::Hard)));
    }

    #[test]
    fn test_fill() {
        let doc = json!({"type": "fill", "parts": ["a", {"type": "line"}, "b"]});
        let ir = doc_json_to_embedded_ir(&doc).unwrap();
        assert!(matches!(ir[0], EmbeddedIR::StartFill));
        assert!(matches!(ir[1], EmbeddedIR::StartEntry));
        assert!(matches!(&ir[2], EmbeddedIR::Text(s) if s == "a"));
        assert!(matches!(ir[3], EmbeddedIR::EndEntry));
        // separator
        assert!(matches!(ir[4], EmbeddedIR::StartEntry));
        assert!(matches!(ir[5], EmbeddedIR::Line(LineMode::SoftOrSpace)));
        assert!(matches!(ir[6], EmbeddedIR::EndEntry));
        // second content
        assert!(matches!(ir[7], EmbeddedIR::StartEntry));
        assert!(matches!(&ir[8], EmbeddedIR::Text(s) if s == "b"));
        assert!(matches!(ir[9], EmbeddedIR::EndEntry));
        assert!(matches!(ir[10], EmbeddedIR::EndFill));
    }

    #[test]
    fn test_collapse_consecutive_hardlines_to_empty_line() {
        // Two hardlines in sequence: [Line(Hard), ExpandParent, Line(Hard), ExpandParent]
        // should collapse to [Line(Empty), ExpandParent]
        let doc = json!([
            "hello",
            {"type": "line", "hard": true},
            {"type": "break-parent"},
            {"type": "line", "hard": true},
            {"type": "break-parent"},
            "world"
        ]);
        let ir = doc_json_to_embedded_ir(&doc).unwrap();
        // "hello" + Line(Empty) + ExpandParent + "world"
        assert_eq!(ir.len(), 4);
        assert!(matches!(&ir[0], EmbeddedIR::Text(s) if s == "hello"));
        assert!(matches!(ir[1], EmbeddedIR::Line(LineMode::Empty)));
        assert!(matches!(ir[2], EmbeddedIR::ExpandParent));
        assert!(matches!(&ir[3], EmbeddedIR::Text(s) if s == "world"));
    }

    #[test]
    fn test_single_hardline_not_collapsed() {
        // Single hardline should remain as-is
        let doc = json!([
            "hello",
            {"type": "line", "hard": true},
            {"type": "break-parent"},
            "world"
        ]);
        let ir = doc_json_to_embedded_ir(&doc).unwrap();
        assert_eq!(ir.len(), 4);
        assert!(matches!(&ir[0], EmbeddedIR::Text(s) if s == "hello"));
        assert!(matches!(ir[1], EmbeddedIR::Line(LineMode::Hard)));
        assert!(matches!(ir[2], EmbeddedIR::ExpandParent));
        assert!(matches!(&ir[3], EmbeddedIR::Text(s) if s == "world"));
    }
}
