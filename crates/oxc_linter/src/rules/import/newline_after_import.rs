use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Declaration, ExportDefaultDeclarationKind, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ast_util::is_global_require_call,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn newline_after_import_diagnostic(span: Span, count: usize, keyword: &str) -> OxcDiagnostic {
    let line_suffix = if count == 1 { "" } else { "s" };

    OxcDiagnostic::warn(format!(
        "Expected {count} empty line{line_suffix} after {keyword} statement not followed by another {keyword}."
    ))
        .with_label(span)
}

// <https://github.com/import-js/eslint-plugin-import/blob/v2.32.0/docs/rules/newline-after-import.md>
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NewlineAfterImport {
    // Number of empty lines required after import statements.
    count: u32,
    // Whether the number of empty lines must be exactly `count`.
    exact_count: bool,
    // Whether comments should be considered when counting empty lines.
    consider_comments: bool,
}

impl Default for NewlineAfterImport {
    fn default() -> Self {
        Self { count: 1, exact_count: false, consider_comments: false }
    }
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces having one or more empty lines after the last top-level import statement or require call.
    ///
    /// ### Why is this bad?
    ///
    /// Without a blank line, import/require declarations blend into the following logic,
    /// which hurts readability and makes changes harder to scan. A blank line clearly
    /// separates dependencies from implementation.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import * as foo from 'foo'
    /// const FOO = 'BAR';
    /// ```
    ///
    /// ```js
    /// import * as foo from 'foo';
    /// const FOO = 'BAR';
    ///
    /// import { bar }  from 'bar-lib';
    /// ```
    ///
    /// ```js
    /// const FOO = require('./foo');
    /// const BAZ = 1;
    /// const BAR = require('./bar');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import defaultExport from './foo';
    ///
    /// const FOO = 'BAR';
    /// ```
    ///
    /// ```js
    /// import defaultExport from './foo';
    /// import { bar }  from 'bar-lib';
    ///
    /// const FOO = 'BAR';
    /// ```
    ///
    /// ```js
    /// const FOO = require('./foo');
    /// const BAR = require('./bar');
    ///
    /// const BAZ = 1;
    /// ```
    ///
    /// With count set to 2 this will be considered valid:
    /// ```js
    /// import defaultExport from './foo';
    ///
    ///
    /// const FOO = 'BAR';
    /// ```
    ///
    /// ```js
    /// import defaultExport from './foo';
    ///
    ///
    ///
    /// const FOO = 'BAR';
    /// ```
    ///
    /// With count set to 2 these will be considered invalid:
    /// ```js
    /// import defaultExport from './foo';
    /// const FOO = 'BAR';
    /// ```
    ///
    /// ```js
    /// import defaultExport from './foo';
    ///
    /// const FOO = 'BAR';
    /// ```
    ///
    /// With count set to 2 and exactCount set to true this will be considered valid:
    /// ```js
    /// import defaultExport from './foo';
    ///
    ///
    /// const FOO = 'BAR';
    /// ```
    ///
    /// With count set to 2 and exactCount set to true these will be considered invalid:
    /// ```js
    /// import defaultExport from './foo';
    /// const FOO = 'BAR';
    /// ```
    ///
    /// ```js
    /// import defaultExport from './foo';
    ///
    /// const FOO = 'BAR';
    /// ```
    ///
    /// ```js
    /// import defaultExport from './foo';
    ///
    ///
    ///
    /// const FOO = 'BAR';
    /// ```
    ///
    /// ```js
    /// import defaultExport from './foo';
    ///
    ///
    ///
    ///
    /// const FOO = 'BAR';
    /// ```
    ///
    /// With considerComments set to false this will be considered valid:
    /// ```js
    /// import defaultExport from './foo'
    /// // some comment here.
    /// const FOO = 'BAR'
    /// ```
    ///
    /// With considerComments set to true this will be considered valid:
    /// ```js
    /// import defaultExport from './foo'
    ///
    /// // some comment here.
    /// const FOO = 'BAR'
    /// ```
    ///
    /// With considerComments set to true this will be considered invalid:
    /// ```js
    /// import defaultExport from './foo'
    /// // some comment here.
    /// const FOO = 'BAR'
    /// ```
    ///
    /// ### Example options usage
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///     "import/newline-after-import": ["error", { "count": 1 }]
    ///   }
    /// }
    /// ```
    NewlineAfterImport,
    import,
    style,
    fix,
    version = "next",
    config = NewlineAfterImport,
);

impl Rule for NewlineAfterImport {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let body = &ctx.nodes().program().body;
        if body.len() < 2 {
            return;
        }

        let source_text = ctx.source_text();
        let mut newline_positions = None;
        let count = self.count as usize;

        for (i, stmt) in body.iter().enumerate() {
            if !is_import_statement(stmt) {
                continue;
            }

            let Some(next_stmt) = body.get(i + 1) else {
                continue;
            };

            SpacingTarget {
                stmt_span: stmt.span(),
                next_stmt,
                kind: ImportLikeKind::Import,
                next_is_same_kind: is_import_statement(next_stmt),
            }
            .check(
                ctx,
                source_text,
                &mut newline_positions,
                count,
                self.exact_count,
                self.consider_comments,
            );
        }

        let require_call_end_offsets = collect_require_call_end_offsets(ctx, body);

        for (i, stmt) in body.iter().enumerate() {
            if require_call_end_offsets[i].is_none() {
                continue;
            }
            let Some(next_stmt) = body.get(i + 1) else {
                continue;
            };
            let next_has_require = require_call_end_offsets[i + 1].is_some();

            SpacingTarget {
                stmt_span: stmt.span(),
                next_stmt,
                kind: ImportLikeKind::Require,
                next_is_same_kind: next_has_require,
            }
            .check(
                ctx,
                source_text,
                &mut newline_positions,
                count,
                self.exact_count,
                self.consider_comments,
            );
        }
    }
}

#[derive(Clone, Copy)]
enum ImportLikeKind {
    Import,
    Require,
}

impl ImportLikeKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::Require => "require",
        }
    }
}

struct SpacingTarget<'a> {
    stmt_span: Span,
    next_stmt: &'a Statement<'a>,
    kind: ImportLikeKind,
    next_is_same_kind: bool,
}

impl<'a> SpacingTarget<'a> {
    fn check(
        &self,
        ctx: &LintContext<'a>,
        source_text: &str,
        newline_positions: &mut Option<Vec<usize>>,
        count: usize,
        exact_count: bool,
        consider_comments: bool,
    ) {
        let next_start = next_statement_start(ctx, source_text, self.next_stmt);
        let keyword = self.kind.as_str();

        if consider_comments {
            let newlines = get_newline_positions(newline_positions, source_text);
            if let Some(comment_start) = find_comment_start_in_spacing_range(
                ctx,
                newlines,
                self.stmt_span,
                next_start,
                count,
            ) {
                check_spacing(ctx, newlines, self.stmt_span, comment_start, count, keyword, false);
                return;
            }
        }

        if self.next_is_same_kind {
            return;
        }

        let newlines = get_newline_positions(newline_positions, source_text);
        check_spacing(ctx, newlines, self.stmt_span, next_start, count, keyword, exact_count);
    }
}

fn is_import_statement(stmt: &Statement<'_>) -> bool {
    matches!(stmt, Statement::ImportDeclaration(_) | Statement::TSImportEqualsDeclaration(_))
}

fn collect_require_call_end_offsets<'a>(
    ctx: &LintContext<'a>,
    body: &[Statement<'a>],
) -> Vec<Option<u32>> {
    // This vector is index-aligned with `body`:
    // each slot stores the latest top-level static `require(...)` end offset in that statement.
    let mut require_call_end_offsets: Vec<Option<u32>> = vec![None; body.len()];
    let nodes = ctx.nodes();

    for node in nodes.iter() {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            continue;
        };

        if !is_static_require_call(call_expr, ctx) {
            continue;
        }

        let is_top_level = nodes.ancestor_kinds(node.id()).all(|kind| {
            !matches!(
                kind,
                AstKind::Function(_)
                    | AstKind::ArrowFunctionExpression(_)
                    | AstKind::BlockStatement(_)
                    | AstKind::ObjectExpression(_)
                    | AstKind::Decorator(_)
            )
        });
        if !is_top_level {
            continue;
        }

        let Some(stmt_index) = find_statement_index(body, call_expr.span) else {
            continue;
        };

        let call_end = call_expr.span.end;
        let entry = &mut require_call_end_offsets[stmt_index];
        if let Some(prev) = entry.as_mut() {
            *prev = (*prev).max(call_end);
        } else {
            *entry = Some(call_end);
        }
    }

    require_call_end_offsets
}

fn is_static_require_call(call_expr: &CallExpression, ctx: &LintContext<'_>) -> bool {
    if !is_global_require_call(call_expr, ctx.semantic()) {
        return false;
    }

    matches!(call_expr.arguments.first(), Some(Argument::StringLiteral(_)))
}

fn find_statement_index(body: &[Statement<'_>], span: Span) -> Option<usize> {
    let index = body.partition_point(|stmt| stmt.span().end < span.start);
    body.get(index).is_some_and(|stmt| stmt.span().contains_inclusive(span)).then_some(index)
}

fn next_statement_start(ctx: &LintContext<'_>, source_text: &str, stmt: &Statement<'_>) -> u32 {
    // For decorated classes, spacing should be measured to the first decorator line.
    let mut start = first_decorator_start(stmt).unwrap_or(stmt.span().start);
    let bytes = source_text.as_bytes();
    let len = u32::try_from(bytes.len()).unwrap();

    while start < len {
        while start < len && bytes[start as usize].is_ascii_whitespace() {
            start += 1;
        }
        if let Some(comment) = ctx.semantic().get_comment_at(start) {
            start = comment.span.end;
            continue;
        }
        break;
    }

    start
}

fn first_decorator_start(stmt: &Statement<'_>) -> Option<u32> {
    match stmt {
        Statement::ClassDeclaration(class) => {
            class.decorators.first().map(|decorator| decorator.span.start)
        }
        Statement::ExportDefaultDeclaration(export_default) => match &export_default.declaration {
            ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                class.decorators.first().map(|decorator| decorator.span.start)
            }
            _ => None,
        },
        Statement::ExportNamedDeclaration(export_named) => {
            match export_named.declaration.as_ref() {
                Some(Declaration::ClassDeclaration(class)) => {
                    class.decorators.first().map(|decorator| decorator.span.start)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn check_spacing(
    ctx: &LintContext<'_>,
    newline_positions: &[usize],
    stmt_span: Span,
    next_start: u32,
    count: usize,
    keyword: &'static str,
    enforce_exact: bool,
) {
    let expected_line_diff = count + 1;
    let line_diff = line_difference(newline_positions, stmt_span, next_start);
    let should_report =
        line_diff < expected_line_diff || (enforce_exact && line_diff != expected_line_diff);

    if !should_report {
        return;
    }

    let diagnostic = newline_after_import_diagnostic(stmt_span, count, keyword);
    if line_diff < expected_line_diff {
        let missing = expected_line_diff - line_diff;
        let fix_text = "\n".repeat(missing);
        ctx.diagnostic_with_fix(diagnostic, |fixer| {
            fixer
                .insert_text_after_range(stmt_span, fix_text)
                .with_message(format!("Add empty line(s) after {keyword}"))
        });
    } else {
        ctx.diagnostic(diagnostic);
    }
}

fn line_difference(newline_positions: &[usize], current: Span, next_start: u32) -> usize {
    let current_line = line_number_at_with_newlines(newline_positions, current.end);
    let next_line = line_number_at_with_newlines(newline_positions, next_start);
    next_line.saturating_sub(current_line)
}

fn collect_newline_positions(source_text: &str) -> Vec<usize> {
    source_text
        .bytes()
        .enumerate()
        .filter_map(|(index, byte)| (byte == b'\n').then_some(index))
        .collect()
}

fn get_newline_positions<'a>(
    newline_positions: &'a mut Option<Vec<usize>>,
    source_text: &str,
) -> &'a [usize] {
    newline_positions.get_or_insert_with(|| collect_newline_positions(source_text)).as_slice()
}

fn line_number_at_with_newlines(newlines: &[usize], offset: u32) -> usize {
    let offset = offset as usize;
    let count = newlines.partition_point(|&pos| pos < offset);
    count + 1
}

fn find_comment_start_in_spacing_range(
    ctx: &LintContext<'_>,
    newline_positions: &[usize],
    stmt_span: Span,
    next_start: u32,
    count: usize,
) -> Option<u32> {
    let expected_line_diff = count + 1;
    ctx.comments_range(stmt_span.end..next_start)
        .find(|comment| {
            line_difference(newline_positions, stmt_span, comment.span.start) <= expected_line_diff
        })
        .map(|comment| comment.span.start)
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (
            r"var path = require('path');
var foo = require('foo');",
            None,
        ),
        (r"require('foo');", None),
        (r"switch ('foo') { case 'bar': require('baz'); }", None),
        (r"const x = () => require('baz'), y = () => require('bar');", None),
        (
            r"const x = () => require('baz'), y = () => require('bar');

// some comment here",
            Some(json!([{ "considerComments": true }])),
        ),
        (r"const x = () => require('baz') && require('bar');", None),
        (
            r"const x = () => require('baz') && require('bar');

// Some random single line comment
var bar = 42;",
            Some(json!([{ "considerComments": true }])),
        ),
        (
            r"const x = () => require('baz') && require('bar');

// Some random single line comment
var bar = 42;",
            Some(json!([{ "considerComments": true, "count": 1, "exactCount": true }])),
        ),
        (
            r"const x = () => require('baz') && require('bar');
/**
 * some multiline comment here
 * another line of comment
**/
var bar = 42;",
            None,
        ),
        (r"function x() { require('baz'); }", None),
        (r"a(require('b'), require('c'), require('d'));", None),
        (
            r"function foo() {
  switch (renderData.modalViewKey) {
    case 'value':
      var bar = require('bar');
      return bar(renderData, options)
    default:
      return renderData.mainModalContent.clone()
  }
}",
            None,
        ),
        (
            r"function bar() {
  switch (foo) {
    case '1':
      return require('../path/to/file1.jst.hbs')(renderData, options);
    case '2':
      return require('../path/to/file2.jst.hbs')(renderData, options);
    case '3':
      return require('../path/to/file3.jst.hbs')(renderData, options);
    case '4':
      return require('../path/to/file4.jst.hbs')(renderData, options);
    case '5':
      return require('../path/to/file5.jst.hbs')(renderData, options);
    case '6':
      return require('../path/to/file6.jst.hbs')(renderData, options);
    case '7':
      return require('../path/to/file7.jst.hbs')(renderData, options);
    case '8':
      return require('../path/to/file8.jst.hbs')(renderData, options);
    case '9':
      return require('../path/to/file9.jst.hbs')(renderData, options);
    case '10':
      return require('../path/to/file10.jst.hbs')(renderData, options);
    case '11':
      return require('../path/to/file11.jst.hbs')(renderData, options);
    case '12':
      return something();
    default:
      return somethingElse();
  }
}",
            None,
        ),
        (
            r"import path from 'path';
import foo from 'foo';",
            None,
        ),
        (r"import path from 'path';import foo from 'foo';", None),
        (
            r"import path from 'path';import foo from 'foo';

var bar = 42;",
            None,
        ),
        (
            r"import foo from 'foo';

var bar = 'bar';",
            None,
        ),
        (
            r"import foo from 'foo';


var bar = 'bar';",
            Some(json!([{ "count": 2 }])),
        ),
        (
            r"import foo from 'foo';


var bar = 'bar';",
            Some(json!([{ "count": 2, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';

var bar = 'bar';",
            Some(json!([{ "count": 1, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';

// Some random comment
var bar = 'bar';",
            Some(json!([{ "count": 2, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';
// Some random comment
var bar = 'bar';",
            Some(json!([{ "count": 1, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';


// Some random comment
var bar = 'bar';",
            Some(json!([{ "count": 2, "exactCount": true, "considerComments": true }])),
        ),
        (
            r"import foo from 'foo';

// Some random comment
var bar = 'bar';",
            Some(json!([{ "count": 1, "exactCount": true, "considerComments": true }])),
        ),
        (
            r"/**
 * A leading comment
 */
import foo from 'foo';

// Some random comment
export {foo};",
            Some(json!([{ "count": 2, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';


var bar = 'bar';",
            Some(json!([{ "count": 1 }])),
        ),
        (
            r"import foo from 'foo';




var bar = 'bar';",
            Some(json!([{ "count": 4 }])),
        ),
        (
            r"var foo = require('foo-module');

var foo = 'bar';",
            None,
        ),
        (
            r"var foo = require('foo-module');


var foo = 'bar';",
            Some(json!([{ "count": 2 }])),
        ),
        (
            r"var foo = require('foo-module');




var foo = 'bar';",
            Some(json!([{ "count": 4 }])),
        ),
        (
            r"var foo = require('foo-module');




var foo = 'bar';",
            Some(json!([{ "count": 4, "exactCount": true }])),
        ),
        (
            r"var foo = require('foo-module');

// Some random comment


var foo = 'bar';",
            Some(json!([{ "count": 4, "exactCount": true }])),
        ),
        (
            r"var foo = require('foo-module');




// Some random comment
var foo = 'bar';",
            Some(json!([{ "count": 4, "exactCount": true, "considerComments": true }])),
        ),
        (
            r"require('foo-module');

var foo = 'bar';",
            None,
        ),
        (
            r"import foo from 'foo';
import { bar } from './bar-lib';",
            None,
        ),
        (
            r"import foo from 'foo';

var a = 123;

import { bar } from './bar-lib';",
            None,
        ),
        (
            r"var foo = require('foo-module');

var a = 123;

var bar = require('bar-lib');",
            None,
        ),
        (
            r"function foo() {
  var foo = require('foo');
  foo();
}",
            None,
        ),
        (
            r"if (true) {
  var foo = require('foo');
  foo();
}",
            None,
        ),
        (
            r"function a() {
  var assign = Object.assign || require('object-assign');
  var foo = require('foo');
  var bar = 42;
}",
            None,
        ),
        (
            r"export default
@SomeDecorator(require('./some-file'))
class App {}",
            None,
        ),
        (
            r"var foo = require('foo');

@SomeDecorator(foo)
class Foo {}",
            None,
        ),
        (
            r"import foo from 'foo';

@SomeDecorator(foo)
export default class Test {}",
            None,
        ),
        (
            r"const foo = require('foo');

@SomeDecorator(foo)
export default class Test {}",
            None,
        ),
        (
            r"import { ExecaReturnValue } from 'execa';
import execa = require('execa');",
            None,
        ),
        (
            r"import execa = require('execa');
import { ExecaReturnValue } from 'execa';",
            None,
        ),
        (
            r"import { ExecaReturnValue } from 'execa';
import execa = require('execa');
import { ExecbReturnValue } from 'execb';",
            None,
        ),
        (
            r"import execa = require('execa');
import { ExecaReturnValue } from 'execa';
import execb = require('execb');",
            None,
        ),
        (
            r"export import a = obj;
f(a);",
            None,
        ),
        (
            r#"import { a } from "./a";

export namespace SomeNamespace {
  export import a2 = a;
  f(a);
}"#,
            None,
        ),
        (
            r"import stub from './stub';

export {
  stub
}",
            None,
        ),
        (
            r"import { ns } from 'namespace';
import Bar = ns.baz.foo.Bar;

export import Foo = ns.baz.bar.Foo;",
            None,
        ),
        (
            r"import stub from './stub';

export {
  stub
}",
            None,
        ),
        (
            r"import path from 'path';
import foo from 'foo';
/**
 * some multiline comment here
 * another line of comment
**/
var bar = 42;",
            None,
        ),
        (
            r"import path from 'path';import foo from 'foo';

/**
 * some multiline comment here
 * another line of comment
**/
var bar = 42;",
            Some(json!([{ "considerComments": true }])),
        ),
        (
            r"import path from 'path';
import foo from 'foo';

// Some random single line comment
var bar = 42;",
            None,
        ),
        (
            r"var foo = require('foo-module');


// Some random comment
var foo = 'bar';",
            Some(json!([{ "count": 2, "considerComments": true }])),
        ),
        (
            r"var foo = require('foo-module');


/**
 * Test comment
 */
var foo = 'bar';",
            Some(json!([{ "count": 2, "considerComments": true }])),
        ),
        (
            r"const foo = require('foo');


// some random comment
const bar = function() {};",
            Some(json!([{ "count": 2, "exactCount": true, "considerComments": true }])),
        ),
    ];

    let fail = vec![
        (
            r"import { A, B, C, D } from
'../path/to/my/module/in/very/far/directory'
// some comment
var foo = 'bar';",
            Some(json!([{ "considerComments": true }])),
        ),
        (
            r"import path from 'path';
import foo from 'foo';
/**
 * some multiline comment here
 * another line of comment
**/
var bar = 42;",
            Some(json!([{ "considerComments": true }])),
        ),
        (
            r"import path from 'path';
import foo from 'foo';
// Some random single line comment
var bar = 42;",
            Some(json!([{ "considerComments": true, "count": 1 }])),
        ),
        (
            r"import foo from 'foo';
export default function() {};",
            None,
        ),
        (
            r"import foo from 'foo';

export default function() {};",
            Some(json!([{ "count": 2 }])),
        ),
        (
            r"var foo = require('foo-module');
var something = 123;",
            None,
        ),
        (
            r"import foo from 'foo';
export default function() {};",
            Some(json!([{ "count": 1 }])),
        ),
        (
            r"import foo from 'foo';
var a = 123;

import { bar } from './bar-lib';
var b=456;",
            None,
        ),
        (
            r"var foo = require('foo-module');
var a = 123;

var bar = require('bar-lib');
var b=456;",
            None,
        ),
        (
            r"var foo = require('foo-module');
var a = 123;

require('bar-lib');
var b=456;",
            None,
        ),
        (
            r"var path = require('path');
var foo = require('foo');
var bar = 42;",
            None,
        ),
        (
            r"var assign = Object.assign || require('object-assign');
var foo = require('foo');
var bar = 42;",
            None,
        ),
        (
            r"require('a');
foo(require('b'), require('c'), require('d'));
require('d');
var foo = 'bar';",
            None,
        ),
        (
            r"require('a');
foo(
require('b'),
require('c'),
require('d')
);
var foo = 'bar';",
            None,
        ),
        (
            r"import path from 'path';
import foo from 'foo';
var bar = 42;",
            None,
        ),
        (r"import path from 'path';import foo from 'foo';var bar = 42;", None),
        (
            r"import foo from 'foo';
@SomeDecorator(foo)
class Foo {}",
            None,
        ),
        (
            r"var foo = require('foo');
@SomeDecorator(foo)
class Foo {}",
            None,
        ),
        (
            r"import foo from 'foo';
@SomeDecorator(foo)
export default class Test {}",
            None,
        ),
        (
            r"const foo = require('foo');
@SomeDecorator(foo)
export default class Test {}",
            None,
        ),
        (
            r"import { map } from 'rxjs/operators';
@Component({})
export class Test {}",
            None,
        ),
        (
            r"import foo from 'foo';

export default function() {};",
            Some(json!([{ "count": 2, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';



export default function() {};",
            Some(json!([{ "count": 2, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';




export default function() {};",
            Some(json!([{ "count": 2, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';
// some random comment
export default function() {};",
            Some(json!([{ "count": 2, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';
// some random comment


export default function() {};",
            Some(json!([{ "count": 2, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';
// some random comment



export default function() {};",
            Some(json!([{ "count": 2, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';
// some random comment
export default function() {};",
            Some(json!([{ "count": 2, "exactCount": true, "considerComments": true }])),
        ),
        (
            r"import foo from 'foo';

// some random comment
export default function() {};",
            Some(json!([{ "count": 2, "exactCount": true, "considerComments": true }])),
        ),
        (
            r"import foo from 'foo';



// some random comment
export default function() {};",
            Some(json!([{ "count": 2, "exactCount": true, "considerComments": true }])),
        ),
        (
            r"import foo from 'foo';


// Some random single line comment
var bar = 42;",
            Some(json!([{ "considerComments": true, "count": 1, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';export default function() {};",
            Some(json!([{ "count": 1, "exactCount": true }])),
        ),
        (
            r"const foo = require('foo');



const bar = function() {};",
            Some(json!([{ "count": 2, "exactCount": true }])),
        ),
        (
            r"const foo = require('foo');



// some random comment
const bar = function() {};",
            Some(json!([{ "count": 2, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';// some random comment
export default function() {};",
            Some(json!([{ "count": 1, "exactCount": true, "considerComments": true }])),
        ),
        (
            r"var foo = require('foo-module');
var foo = require('foo-module');

// Some random comment
var foo = 'bar';",
            Some(json!([{ "considerComments": true, "count": 2 }])),
        ),
        (
            r"var foo = require('foo-module');

/**
 * Test comment
 */
var foo = 'bar';",
            Some(json!([{ "considerComments": true, "count": 2 }])),
        ),
    ];

    let fix = vec![
        (
            r"import { A, B, C, D } from
'../path/to/my/module/in/very/far/directory'
// some comment
var foo = 'bar';",
            r"import { A, B, C, D } from
'../path/to/my/module/in/very/far/directory'

// some comment
var foo = 'bar';",
            Some(json!([{ "considerComments": true }])),
        ),
        (
            r"import path from 'path';
import foo from 'foo';
/**
 * some multiline comment here
 * another line of comment
**/
var bar = 42;",
            r"import path from 'path';
import foo from 'foo';

/**
 * some multiline comment here
 * another line of comment
**/
var bar = 42;",
            Some(json!([{ "considerComments": true }])),
        ),
        (
            r"import path from 'path';
import foo from 'foo';
// Some random single line comment
var bar = 42;",
            r"import path from 'path';
import foo from 'foo';

// Some random single line comment
var bar = 42;",
            Some(json!([{ "considerComments": true, "count": 1 }])),
        ),
        (
            r"import foo from 'foo';
export default function() {};",
            r"import foo from 'foo';

export default function() {};",
            None,
        ),
        (
            r"import foo from 'foo';

export default function() {};",
            r"import foo from 'foo';


export default function() {};",
            Some(json!([{ "count": 2 }])),
        ),
        (
            r"var foo = require('foo-module');
var something = 123;",
            r"var foo = require('foo-module');

var something = 123;",
            None,
        ),
        (
            r"import foo from 'foo';
export default function() {};",
            r"import foo from 'foo';

export default function() {};",
            Some(json!([{ "count": 1 }])),
        ),
        (
            r"import foo from 'foo';
var a = 123;

import { bar } from './bar-lib';
var b=456;",
            r"import foo from 'foo';

var a = 123;

import { bar } from './bar-lib';

var b=456;",
            None,
        ),
        (
            r"var foo = require('foo-module');
var a = 123;

var bar = require('bar-lib');
var b=456;",
            r"var foo = require('foo-module');

var a = 123;

var bar = require('bar-lib');

var b=456;",
            None,
        ),
        (
            r"var foo = require('foo-module');
var a = 123;

require('bar-lib');
var b=456;",
            r"var foo = require('foo-module');

var a = 123;

require('bar-lib');

var b=456;",
            None,
        ),
        (
            r"var path = require('path');
var foo = require('foo');
var bar = 42;",
            r"var path = require('path');
var foo = require('foo');

var bar = 42;",
            None,
        ),
        (
            r"var assign = Object.assign || require('object-assign');
var foo = require('foo');
var bar = 42;",
            r"var assign = Object.assign || require('object-assign');
var foo = require('foo');

var bar = 42;",
            None,
        ),
        (
            r"require('a');
foo(require('b'), require('c'), require('d'));
require('d');
var foo = 'bar';",
            r"require('a');
foo(require('b'), require('c'), require('d'));
require('d');

var foo = 'bar';",
            None,
        ),
        (
            r"require('a');
foo(
require('b'),
require('c'),
require('d')
);
var foo = 'bar';",
            r"require('a');
foo(
require('b'),
require('c'),
require('d')
);

var foo = 'bar';",
            None,
        ),
        (
            r"import path from 'path';
import foo from 'foo';
var bar = 42;",
            r"import path from 'path';
import foo from 'foo';

var bar = 42;",
            None,
        ),
        (
            r"import path from 'path';import foo from 'foo';var bar = 42;",
            r"import path from 'path';import foo from 'foo';

var bar = 42;",
            None,
        ),
        (
            r"import foo from 'foo';
@SomeDecorator(foo)
class Foo {}",
            r"import foo from 'foo';

@SomeDecorator(foo)
class Foo {}",
            None,
        ),
        (
            r"var foo = require('foo');
@SomeDecorator(foo)
class Foo {}",
            r"var foo = require('foo');

@SomeDecorator(foo)
class Foo {}",
            None,
        ),
        (
            r"import foo from 'foo';
@SomeDecorator(foo)
export default class Test {}",
            r"import foo from 'foo';

@SomeDecorator(foo)
export default class Test {}",
            None,
        ),
        (
            r"const foo = require('foo');
@SomeDecorator(foo)
export default class Test {}",
            r"const foo = require('foo');

@SomeDecorator(foo)
export default class Test {}",
            None,
        ),
        (
            r"import { map } from 'rxjs/operators';
@Component({})
export class Test {}",
            r"import { map } from 'rxjs/operators';

@Component({})
export class Test {}",
            None,
        ),
        (
            r"import foo from 'foo';

export default function() {};",
            r"import foo from 'foo';


export default function() {};",
            Some(json!([{ "count": 2, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';
// some random comment
export default function() {};",
            r"import foo from 'foo';

// some random comment
export default function() {};",
            Some(json!([{ "count": 2, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';
// some random comment
export default function() {};",
            r"import foo from 'foo';


// some random comment
export default function() {};",
            Some(json!([{ "count": 2, "exactCount": true, "considerComments": true }])),
        ),
        (
            r"import foo from 'foo';

// some random comment
export default function() {};",
            r"import foo from 'foo';


// some random comment
export default function() {};",
            Some(json!([{ "count": 2, "exactCount": true, "considerComments": true }])),
        ),
        (
            r"import foo from 'foo';export default function() {};",
            r"import foo from 'foo';

export default function() {};",
            Some(json!([{ "count": 1, "exactCount": true }])),
        ),
        (
            r"import foo from 'foo';// some random comment
export default function() {};",
            r"import foo from 'foo';

// some random comment
export default function() {};",
            Some(json!([{ "count": 1, "exactCount": true, "considerComments": true }])),
        ),
        (
            r"var foo = require('foo-module');
var foo = require('foo-module');

// Some random comment
var foo = 'bar';",
            r"var foo = require('foo-module');
var foo = require('foo-module');


// Some random comment
var foo = 'bar';",
            Some(json!([{ "considerComments": true, "count": 2 }])),
        ),
        (
            r"var foo = require('foo-module');

/**
 * Test comment
 */
var foo = 'bar';",
            r"var foo = require('foo-module');


/**
 * Test comment
 */
var foo = 'bar';",
            Some(json!([{ "considerComments": true, "count": 2 }])),
        ),
    ];

    Tester::new(NewlineAfterImport::NAME, NewlineAfterImport::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
