use lazy_regex::{Lazy, Regex, lazy_regex};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::context::LintContext;

fn no_commented_out_tests_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Some tests appear to be inside comments.")
        .with_help("Remove or uncomment this test.")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r#"### What it does

This rule raises a warning about commented out tests. It's similar to the
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

This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/v1.1.9/docs/rules/no-commented-out-tests.md),
to use it, add the following configuration to your `.oxlintrc.json`:

```json
{
  "rules": {
     "vitest/no-commented-out-tests": "error"
  }
}
```
"#;

//  /^\s*[xf]?(test|it|describe)(\.\w+|\[['"]\w+['"]\])?\s*\(/mu
static RE: Lazy<Regex> =
    lazy_regex!(r#"(?mu)^\s*[xf]?(test|it|describe)(\.\w+|\[['"]\w+['"]\])?\s*\("#);

pub fn run_once(ctx: &LintContext) {
    let comments = ctx.comments();
    let commented_tests = comments.iter().filter_map(|comment| {
        let text = ctx.source_range(comment.content_span());
        if RE.is_match(text) { Some(comment.content_span()) } else { None }
    });
    for span in commented_tests {
        ctx.diagnostic(no_commented_out_tests_diagnostic(span));
    }
}
