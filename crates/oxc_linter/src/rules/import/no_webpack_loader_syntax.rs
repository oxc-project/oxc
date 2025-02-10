use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_named_as_default_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected `!` in `{name}`."))
        .with_help("Do not use import syntax to configure webpack loaders")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoWebpackLoaderSyntax;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids using Webpack loader syntax directly in import or require statements.
    ///
    /// ### Why is this bad?
    ///
    /// This loader syntax is non-standard, so it couples the code to Webpack. The recommended way to
    /// specify Webpack loader configuration is in a [Webpack configuration file](https://webpack.js.org/concepts/loaders/#configuration).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import myModule from 'my-loader!my-module';
    /// import theme from 'style!css!./theme.css';
    ///
    /// var myModule = require('my-loader!./my-module');
    /// var theme = require('style!css!./theme.css');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import myModule from './my-module';
    /// import theme from './theme.css';
    ///
    /// var myModule = require('./my-module');
    /// var theme = require('./theme.css');
    /// ```
    NoWebpackLoaderSyntax,
    import,
    restriction,
);

impl Rule for NoWebpackLoaderSyntax {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // not in top level
        if node.scope_id() != ctx.scopes().root_scope_id() {
            return;
        }

        match node.kind() {
            AstKind::CallExpression(call_expr) => {
                if let Expression::Identifier(identifier) = &call_expr.callee {
                    if identifier.name != "require" {
                        return;
                    }

                    if call_expr.arguments.len() != 1 {
                        return;
                    }

                    let Argument::StringLiteral(ident) = &call_expr.arguments[0] else {
                        return;
                    };

                    if ident.value.contains('!') {
                        ctx.diagnostic(no_named_as_default_diagnostic(
                            ident.value.as_str(),
                            ident.span,
                        ));
                    }
                }
            }
            AstKind::ImportDeclaration(import_decl) => {
                if import_decl.source.value.contains('!') {
                    ctx.diagnostic(no_named_as_default_diagnostic(
                        &import_decl.source.value,
                        import_decl.source.span,
                    ));
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "import _ from 'lodash'",
        "import find from 'lodash.find'",
        "import foo from './foo.css'",
        "import data from '@scope/my-package/data.json'",
        "var _ = require('lodash')",
        "var find = require('lodash.find')",
        "var foo = require('./foo')",
        "var foo = require('../foo')",
        "var foo = require('foo')",
        "var foo = require('./')",
        "var foo = require('@scope/foo')",
    ];

    let fail = vec![
        "import _ from 'babel!lodash'",
        "import find from '-babel-loader!lodash.find'",
        "import foo from 'style!css!./foo.css'",
        "import data from 'json!@scope/my-package/data.json'",
        "var _ = require('babel!lodash')",
        "var find = require('-babel-loader!lodash.find')",
        "var foo = require('style!css!./foo.css')",
        "var data = require('json!@scope/my-package/data.json')",
    ];

    Tester::new(NoWebpackLoaderSyntax::NAME, NoWebpackLoaderSyntax::PLUGIN, pass, fail)
        .test_and_snapshot();
}
