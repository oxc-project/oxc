use oxc_ast::{
    AstKind,
    ast::{
        Argument, ArrowFunctionExpression, BlockStatement, CallExpression, Declaration, Decorator,
        ExportDefaultDeclarationKind, Function, ObjectExpression, Statement,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
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

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let import_like_node = match node.kind() {
            AstKind::ImportDeclaration(import_decl) => ImportLikeNode::Import(import_decl.span),
            AstKind::TSImportEqualsDeclaration(import_decl) => {
                ImportLikeNode::Import(import_decl.span)
            }
            AstKind::CallExpression(call_expr) => ImportLikeNode::Require(call_expr),
            _ => return,
        };

        let body = &ctx.nodes().program().body;
        let source_text = ctx.source_text();
        let count = self.count as usize;

        let target = match import_like_node {
            ImportLikeNode::Import(span) => SpacingTarget::from_import_span(body, span),
            ImportLikeNode::Require(call_expr) => {
                SpacingTarget::from_require_call(node, call_expr, ctx, body)
            }
        };
        let Some(target) = target else { return };
        let Some(next_stmt) = body.get(target.stmt_index + 1) else { return };

        let next_is_same_kind = match target.kind {
            ImportLikeKind::Import => is_import_statement(next_stmt),
            ImportLikeKind::Require => statement_has_top_level_static_require_call(ctx, next_stmt),
        };
        if next_is_same_kind && !self.consider_comments {
            return;
        }

        target.check(
            ctx,
            source_text,
            next_stmt,
            next_is_same_kind,
            count,
            self.exact_count,
            self.consider_comments,
        );
    }
}

#[derive(Clone, Copy)]
enum ImportLikeKind {
    Import,
    Require,
}

enum ImportLikeNode<'a> {
    Import(Span),
    Require(&'a CallExpression<'a>),
}

impl ImportLikeKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::Require => "require",
        }
    }
}

struct SpacingTarget {
    stmt_span: Span,
    stmt_index: usize,
    kind: ImportLikeKind,
}

impl SpacingTarget {
    fn from_import_span(body: &[Statement<'_>], span: Span) -> Option<Self> {
        let stmt_index = find_import_statement_index(body, span)?;
        Some(Self { stmt_span: span, stmt_index, kind: ImportLikeKind::Import })
    }

    fn from_require_call<'a>(
        node: &AstNode<'a>,
        call_expr: &CallExpression<'a>,
        ctx: &LintContext<'a>,
        body: &[Statement<'a>],
    ) -> Option<Self> {
        if !is_top_level_static_require_call(node, call_expr, ctx) {
            return None;
        }

        let (stmt_index, stmt) = find_containing_statement(body, call_expr.span)?;
        let stmt_span = stmt.span();
        is_last_top_level_static_require_call_in_statement(ctx, node, stmt).then_some(Self {
            stmt_span,
            stmt_index,
            kind: ImportLikeKind::Require,
        })
    }

    fn check(
        &self,
        ctx: &LintContext<'_>,
        source_text: &str,
        next_stmt: &Statement<'_>,
        next_is_same_kind: bool,
        count: usize,
        exact_count: bool,
        consider_comments: bool,
    ) {
        let next_start = next_statement_start(ctx, source_text, next_stmt);
        let keyword = self.kind.as_str();

        if consider_comments
            && let Some(comment_start) = find_comment_start_in_spacing_range(
                ctx,
                source_text,
                self.stmt_span,
                next_start,
                count,
            )
        {
            check_spacing(ctx, source_text, self.stmt_span, comment_start, count, keyword, false);
            return;
        }

        if next_is_same_kind {
            return;
        }

        check_spacing(ctx, source_text, self.stmt_span, next_start, count, keyword, exact_count);
    }
}

fn is_import_statement(stmt: &Statement<'_>) -> bool {
    matches!(stmt, Statement::ImportDeclaration(_) | Statement::TSImportEqualsDeclaration(_))
}

fn is_static_require_call(call_expr: &CallExpression, ctx: &LintContext<'_>) -> bool {
    if !is_global_require_call(call_expr, ctx.semantic()) {
        return false;
    }

    matches!(call_expr.arguments.first(), Some(Argument::StringLiteral(_)))
}

fn is_top_level_static_require_call(
    node: &AstNode<'_>,
    call_expr: &CallExpression,
    ctx: &LintContext<'_>,
) -> bool {
    is_static_require_call(call_expr, ctx) && is_top_level_require_call(node, ctx)
}

fn is_top_level_require_call(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    ctx.nodes().ancestor_kinds(node.id()).all(|kind| {
        !matches!(
            kind,
            AstKind::Function(_)
                | AstKind::ArrowFunctionExpression(_)
                | AstKind::BlockStatement(_)
                | AstKind::ObjectExpression(_)
                | AstKind::Decorator(_)
        )
    })
}

fn statement_has_top_level_static_require_call<'a>(
    ctx: &LintContext<'a>,
    stmt: &Statement<'a>,
) -> bool {
    RequireCallScanner::new(ctx).has_match(stmt)
}

fn is_last_top_level_static_require_call_in_statement<'a>(
    ctx: &LintContext<'a>,
    current: &AstNode<'a>,
    stmt: &Statement<'a>,
) -> bool {
    !RequireCallScanner::after(ctx, current.span().end).has_match(stmt)
}

struct RequireCallScanner<'a, 'ctx> {
    ctx: &'ctx LintContext<'a>,
    after: Option<u32>,
    found: bool,
}

impl<'a, 'ctx> RequireCallScanner<'a, 'ctx> {
    fn new(ctx: &'ctx LintContext<'a>) -> Self {
        Self { ctx, after: None, found: false }
    }

    fn after(ctx: &'ctx LintContext<'a>, offset: u32) -> Self {
        Self { ctx, after: Some(offset), found: false }
    }

    fn has_match(mut self, stmt: &Statement<'a>) -> bool {
        self.visit_statement(stmt);
        self.found
    }
}

impl<'a> Visit<'a> for RequireCallScanner<'a, '_> {
    fn visit_call_expression(&mut self, call_expr: &CallExpression<'a>) {
        if self.found {
            return;
        }

        if self.after.is_none_or(|after| call_expr.span.end > after)
            && is_static_require_call(call_expr, self.ctx)
        {
            self.found = true;
            return;
        }

        walk::walk_call_expression(self, call_expr);
    }

    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _expr: &ArrowFunctionExpression<'a>) {}

    fn visit_block_statement(&mut self, _stmt: &BlockStatement<'a>) {}

    fn visit_object_expression(&mut self, _expr: &ObjectExpression<'a>) {}

    fn visit_decorator(&mut self, _decorator: &Decorator<'a>) {}
}

fn find_import_statement_index(body: &[Statement<'_>], span: Span) -> Option<usize> {
    let index = find_statement_index(body, span)?;
    let stmt = body.get(index)?;
    (is_import_statement(stmt) && stmt.span() == span).then_some(index)
}

fn find_containing_statement<'a, 'b>(
    body: &'b [Statement<'a>],
    span: Span,
) -> Option<(usize, &'b Statement<'a>)> {
    let index = find_statement_index(body, span)?;
    Some((index, body.get(index)?))
}

fn find_statement_index(body: &[Statement<'_>], span: Span) -> Option<usize> {
    let index = body.partition_point(|stmt| stmt.span().end <= span.start);
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
    source_text: &str,
    stmt_span: Span,
    next_start: u32,
    count: usize,
    keyword: &'static str,
    enforce_exact: bool,
) {
    let expected_line_diff = count + 1;
    let line_diff = line_difference(source_text, stmt_span, next_start);
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

fn line_difference(source_text: &str, current: Span, next_start: u32) -> usize {
    let start = current.end as usize;
    let end = next_start as usize;
    source_text[start..end].bytes().filter(|&byte| byte == b'\n').count()
}

fn find_comment_start_in_spacing_range(
    ctx: &LintContext<'_>,
    source_text: &str,
    stmt_span: Span,
    next_start: u32,
    count: usize,
) -> Option<u32> {
    let expected_line_diff = count + 1;
    ctx.comments_range(stmt_span.end..next_start)
        .find(|comment| {
            line_difference(source_text, stmt_span, comment.span.start) <= expected_line_diff
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
