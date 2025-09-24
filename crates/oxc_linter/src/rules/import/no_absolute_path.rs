use std::path::Path;

use serde_json::Value;

use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_absolute_path_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not import modules using an absolute path").with_label(span)
}

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.31.0/docs/rules/no-absolute-path.md>
#[derive(Debug, Clone)]
pub struct NoAbsolutePath {
    esmodule: bool,
    commonjs: bool,
    amd: bool,
}

impl Default for NoAbsolutePath {
    fn default() -> Self {
        Self { esmodule: true, commonjs: true, amd: false }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule forbids the import of modules using absolute paths.
    ///
    /// ### Why is this bad?
    ///
    /// Node.js allows the import of modules using an absolute path such as `/home/xyz/file.js`.
    /// That is a bad practice as it ties the code using it to your computer,
    /// and therefore makes it unusable in packages distributed on npm for instance.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import f from '/foo';
    /// import f from '/some/path';
    /// var f = require('/foo');
    /// var f = require('/some/path');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import _ from 'lodash';
    /// import foo from 'foo';
    /// import foo from './foo';
    ///
    /// var _ = require('lodash');
    /// var foo = require('foo');
    /// var foo = require('./foo');
    /// ```
    ///
    /// Examples of **incorrect** code for the `{ amd: true }` option:
    /// ```js
    /// define('/foo', function(foo){})
    /// require('/foo', function(foo){})
    /// ```
    ///
    /// Examples of **correct** code for the `{ amd: true }` option:
    /// ```js
    /// define('./foo', function(foo){})
    /// require('./foo', function(foo){})
    /// ```
    ///
    /// ### Options
    ///
    /// By default, only ES6 imports and `CommonJS` require calls will have this rule enforced.
    /// You may provide an options object providing true/false for any of
    ///
    /// * `esmodule`: defaults to `true`
    /// * `commonjs`: defaults to `true`
    /// * `amd`: defaults to `false`
    ///
    /// If `{ amd: true }` is provided, dependency paths for AMD-style define and require calls will be resolved:
    ///
    /// ```js
    /// /*eslint import/no-absolute-path: ['error', { commonjs: false, amd: true }]*/
    /// define(['/foo'], function (foo) { /*...*/ }) // reported
    /// require(['/foo'], function (foo) { /*...*/ }) // reported
    ///
    /// const foo = require('/foo') // ignored because of explicit `commonjs: false`
    /// ```
    NoAbsolutePath,
    import,
    suspicious,
    pending
);

impl Rule for NoAbsolutePath {
    fn from_configuration(value: Value) -> Self {
        let obj = value.get(0);
        let esmodule = obj
            .and_then(|config| config.get("esmodule"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let commonjs = obj
            .and_then(|config| config.get("commonjs"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let amd = obj
            .and_then(|config| config.get("amd"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self { esmodule, commonjs, amd }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportDeclaration(import_decl) if self.esmodule => {
                if check_path_is_absolute(import_decl.source.value.as_str()) {
                    ctx.diagnostic(no_absolute_path_diagnostic(import_decl.source.span));
                }
            }
            AstKind::CallExpression(call_expr) => {
                let Expression::Identifier(ident) = &call_expr.callee else {
                    return;
                };
                let func_name = ident.name.as_str();
                let count = call_expr.arguments.len();
                if matches!(func_name, "require" | "define") && count > 0 {
                    match &call_expr.arguments[0] {
                        Argument::StringLiteral(str_literal)
                            if count == 1 && func_name == "require" && self.commonjs =>
                        {
                            if check_path_is_absolute(str_literal.value.as_str()) {
                                ctx.diagnostic(no_absolute_path_diagnostic(str_literal.span));
                            }
                        }
                        Argument::ArrayExpression(arr_expr) if count == 2 && self.amd => {
                            for el in &arr_expr.elements {
                                if let Some(el_expr) = el.as_expression()
                                    && matches!(el_expr, Expression::StringLiteral(literal) if check_path_is_absolute(literal.value.as_str()))
                                {
                                    ctx.diagnostic(no_absolute_path_diagnostic(el_expr.span()));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn check_path_is_absolute(path_str: &str) -> bool {
    Path::new(path_str).is_absolute()
}

#[test]
#[cfg(not(windows))] // `Path::is_absolute` is platform-dependent, so these tests fail on windows. https://doc.rust-lang.org/std/path/struct.Path.html#method.is_absolute
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        (r"import _ from 'lodash'", None),
        (r"import _ from '/lodash'", Some(json!([{ "esmodule": false }]))),
        (r"import _ from './lodash'", None),
        (r"import find from 'lodash.find'", None),
        (r"import foo from './foo'", None),
        (r"import foo from '../foo'", None),
        (r"import foo from './'", None),
        (r"import foo from '@scope/foo'", None),
        (r"var _ = require('lodash')", None),
        (r"var find = require('lodash.find')", None),
        (r"var foo = require('/foo')", Some(json!([{ "commonjs": false }]))),
        (r"var foo = require('foo')", None),
        (r"var foo = require('./foo')", None),
        (r"var foo = require('../foo')", None),
        (r"var foo = require('/foo', 2)", None),
        (r"var foo = require('./')", None),
        (r"var foo = require('@scope/foo')", None),
        (r"import events from 'events'", None),
        (r"import path from 'path'", None),
        (r"var events = require('events')", None),
        (r"var path = require('path')", None),
        (
            r"
                import path from 'path';
                import events from 'events'
            ",
            None,
        ),
        (
            r"
                var foo = require('/foo')
            ",
            Some(json!([{ "commonjs": false }])),
        ),
        (r"require(['/foo'], function(){})", None),
        (r"require(['/foo'], function(){})", Some(json!([{ "amd": false }]))),
        (r"require(['./foo'], function(){})", Some(json!([{ "amd": true }]))),
        (r"require(['./foo', 'boo'], function(){})", Some(json!([{ "amd": true }]))),
        (r"define(['./foo'], function(){})", Some(json!([{ "amd": true }]))),
        (r"define(['./foo'])", Some(json!([{ "amd": true }]))),
        (r"define([...['/12323']])", Some(json!([{ "amd": true }]))),
        (r"var foo = require()", None),
    ];

    let fail = vec![
        (r"import _ from '/lodash'", None),
        (r"import _ from '/lodash'", Some(json!([{ "esmodule": true }]))),
        (r"import f from '/foo/path'", None),
        (r"import f from '/foo/bar/baz.js'", None),
        (r"var foo = require('/foo')", None),
        (r"var f = require('/foo/some')", None),
        (r"var f = require('/foo/some/add')", Some(json!([{ "commonjs": true }]))),
        (r"require(['/foo'], function(){})", Some(json!([{ "amd": true }]))),
        (r"require(['./foo', '/boo'], function(){})", Some(json!([{ "amd": true }]))),
        (r"require(['/foo', '/boo'], function(){})", Some(json!([{ "amd": true }]))),
        (r"define(['/foo'], function(){})", Some(json!([{ "amd": true }]))),
    ];

    Tester::new(NoAbsolutePath::NAME, NoAbsolutePath::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
