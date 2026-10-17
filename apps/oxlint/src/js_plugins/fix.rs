use napi::{Error, Result};
use napi_derive::napi;

use oxc_allocator::Allocator;
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_diagnostics::OxcDiagnostic;
use oxc_linter::{Fixer, JsFix, Message, PossibleFixes, convert_and_merge_js_fixes};
use oxc_napi::get_source_type;
use oxc_parser::{ParseOptions, Parser, config::RuntimeParserConfig};
use oxc_semantic::SemanticBuilder;

use super::parse::ParserOptions;

const BOM: &str = "\u{feff}";
#[expect(clippy::cast_possible_truncation)]
const BOM_LEN: u32 = BOM.len() as u32;

/// Apply fixes to source text and validate the fixed code with the original parsing options.
///
/// - `source_text` is the original source code.
/// - `fixes_json` is a JSON string containing `Vec<Vec<JsFix>>` — an array of fix groups,
///    one group per diagnostic which provides fixes.
///    Each inner array should have length of 1 at minimum.
///
/// If source text starts with a BOM, `JSFix`es must have offsets relative to the start
/// of the source text *without* the BOM.
///
/// Each group's fixes are merged, then all merged fixes are applied to `source_text`.
///
/// Fix ranges are converted from UTF-16 code units to UTF-8 bytes.
/// `filename` and `options` must match those used to parse the original source.
///
/// # Errors
///
/// Returns an error if the fixed code cannot be parsed.
#[napi]
#[allow(dead_code, clippy::needless_pass_by_value, clippy::allow_attributes)]
pub fn apply_fixes(
    source_text: String,
    fixes_json: String,
    filename: String,
    options: Option<ParserOptions>,
) -> Result<Option<String>> {
    let Some(fixed_code) = apply_fixes_impl(&source_text, &fixes_json) else {
        return Ok(None);
    };
    let options = options.unwrap_or_default();
    let source_type =
        get_source_type(&filename, options.lang.as_deref(), options.source_type.as_deref());
    let ignore_non_fatal_errors = options.ignore_non_fatal_errors.unwrap_or(false);

    // Use a separate allocator so validation cannot overwrite the original AST in the JS buffer.
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &fixed_code, source_type)
        .with_options(ParseOptions {
            parse_regular_expression: true,
            allow_return_outside_function: true,
            ..ParseOptions::default()
        })
        // Output validation does not need tokens. Share the linter's parser configuration.
        .with_config(RuntimeParserConfig::new(false))
        .parse();

    if ret.fatal_error || (!ignore_non_fatal_errors && !ret.diagnostics.is_empty()) {
        return Err(Error::from_reason(format!(
            "Parsing failed in fixed code: {}\n{fixed_code}",
            ret.diagnostics.first().map_or("Unknown parsing error", |error| error.message.as_ref()),
        )));
    }
    if !ignore_non_fatal_errors {
        let semantic_ret = SemanticBuilder::new_compiler().build(&ret.program);
        if let Some(error) = semantic_ret.diagnostics.first() {
            return Err(Error::from_reason(format!(
                "Parsing failed in fixed code: {}\n{fixed_code}",
                error.message,
            )));
        }
    }

    Ok(Some(fixed_code))
}

fn apply_fixes_impl(source_text: &str, fixes_json: &str) -> Option<String> {
    // Deserialize fixes JSON
    let fix_groups: Vec<Vec<JsFix>> = serde_json::from_str(fixes_json).ok()?;

    // Create `Utf8ToUtf16` converter.
    // If file has a BOM, trim it off start of the source text before creating the converter.
    let has_bom = source_text.starts_with(BOM);
    let span_converter = if has_bom {
        Utf8ToUtf16::new_with_offset(&source_text[BOM_LEN as usize..], BOM_LEN)
    } else {
        Utf8ToUtf16::new(source_text)
    };

    // Merge fix groups into a single fix per group
    let messages = fix_groups
        .into_iter()
        .map(|group| {
            convert_and_merge_js_fixes(group, source_text, &span_converter, has_bom)
                .ok()
                .map(|fix| Message::new(OxcDiagnostic::error(""), PossibleFixes::Single(fix)))
        })
        .collect::<Option<Vec<_>>>()?;

    // Apply all the fixes
    let fixed_code = Fixer::new(source_text, messages, None).fix().fixed_code.into_owned();

    Some(fixed_code)
}
