use lazy_regex::{Lazy, Regex, lazy_regex};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::context::LintContext;

fn no_commented_out_tests_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Some tests appear to be inside comments.")
        .with_help("Remove or uncomment this test.")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

This rule raises a warning about commented-out tests. It's similar to the
`no-disabled-tests` rule.

### Why is this bad?

You may forget to uncomment some tests. This rule raises a warning about commented-out tests.

It is generally better to skip a test if it's flaky, or remove it if it's no longer needed.

### Examples

Examples of **incorrect** code for this rule:
```javascript
// describe('foo', () => {});
// it('foo', () => {});
// test('foo', () => {});

// describe.skip('foo', () => {});
// it.skip('foo', () => {});
// test.skip('foo', () => {});
```
";

//  /^\s*[xf]?(test|it|describe)(\.\w+|\[['"]\w+['"]\])?\s*\(/mu
static RE: Lazy<Regex> =
    lazy_regex!(r#"(?mu)^\s*[xf]?(test|it|describe)(\.\w+|\[['"]\w+['"]\])?\s*\("#);

/// Cheap prefilter: comments without these keywords cannot match the regex.
#[inline]
fn might_contain_commented_test(text: &str) -> bool {
    text.contains("test") || text.contains("it") || text.contains("describe")
}

pub fn run_once(ctx: &LintContext) {
    for comment in ctx.comments() {
        let text = ctx.source_range(comment.content_span());
        // Skip regex for the common case (ordinary comments without test APIs).
        if !might_contain_commented_test(text) {
            continue;
        }
        if RE.is_match(text) {
            ctx.diagnostic(no_commented_out_tests_diagnostic(comment.content_span()));
        }
    }
}
