use napi_derive::napi;

use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_diagnostics::OxcDiagnostic;
use oxc_linter::{Fixer, JsFix, Message, PossibleFixes, convert_and_merge_js_fixes};

/// Apply fixes to source text and return the fixed code.
///
/// - `source_text` is the original source code.
/// - `fixes_json` is a JSON string containing `Vec<Vec<JsFix>>` — an array of fix groups,
///    one group per diagnostic which provides fixes.
///    Each inner array should have length of 1 at minimum.
///
/// Each group's fixes are merged, then all merged fixes are applied to `source_text`.
///
/// Fix ranges are converted from UTF-16 code units to UTF-8 bytes.
#[napi]
#[allow(dead_code, clippy::needless_pass_by_value, clippy::allow_attributes)]
pub fn apply_fixes(source_text: String, fixes_json: String) -> Option<String> {
    // Deserialize fixes JSON
    let fix_groups: Vec<Vec<JsFix>> = serde_json::from_str(&fixes_json).ok()?;

    // Create `Utf8ToUtf16` converter
    let span_converter = Utf8ToUtf16::new(&source_text);

    // Merge fix groups into a single fix per group
    let messages = fix_groups
        .into_iter()
        .map(|group| {
            convert_and_merge_js_fixes(group, &source_text, &span_converter)
                .ok()
                .map(|fix| Message::new(OxcDiagnostic::error(""), PossibleFixes::Single(fix)))
        })
        .collect::<Option<Vec<_>>>()?;

    // Apply all the fixes
    let fixed_code = Fixer::new(&source_text, messages, None).fix().fixed_code.into_owned();

    Some(fixed_code)
}
