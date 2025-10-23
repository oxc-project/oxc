use schemars::JsonSchema;
use serde_json::Value;

use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_unassigned_import_diagnostic(span: Span, msg: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(msg.to_string())
        .with_help("Consider assigning the import to a variable or removing it if it's unused.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnassignedImport(Box<NoUnassignedImportConfig>);

#[derive(Debug, Default, Clone, JsonSchema)]
pub struct NoUnassignedImportConfig {
    /// A list of glob patterns to allow unassigned imports for specific modules.
    /// For example:
    /// `{ "allow": ["*.css"] }` will allow unassigned imports for any module ending with `.css`.
    #[serde(rename = "allow", default)]
    globs: Vec<CompactStr>,
}

impl std::ops::Deref for NoUnassignedImport {
    type Target = NoUnassignedImportConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule aims to remove modules with side-effects by reporting when a module is imported but not assigned.
    ///
    /// ### Why is this bad?
    ///
    /// With both CommonJS' require and the ES6 modules' import syntax,
    /// it is possible to import a module but not to use its result.
    /// This can be done explicitly by not assigning the module to a variable.
    /// Doing so can mean either of the following things:
    /// * The module is imported but not used
    /// * The module has side-effects. Having side-effects,
    /// makes it hard to know whether the module is actually used or can be removed.
    /// It can also make it harder to test or mock parts of your application.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import 'should'
    /// require('should')
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import _ from 'foo'
    /// import _, {foo} from 'foo'
    /// import _, {foo as bar} from 'foo'
    /// const _ = require('foo')
    /// const {foo} = require('foo')
    /// const {foo: bar} = require('foo')
    /// bar(require('foo'))
    /// ```
    NoUnassignedImport,
    import,
    suspicious,
    config = NoUnassignedImportConfig
);

impl Rule for NoUnassignedImport {
    fn from_configuration(value: Value) -> Self {
        let obj = value.get(0);
        let globs = obj
            .and_then(|v| v.get("allow"))
            .and_then(Value::as_array)
            .map(|v| v.iter().filter_map(Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();
        Self(Box::new(NoUnassignedImportConfig { globs }))
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportDeclaration(import_decl) => {
                if import_decl.specifiers.is_some() {
                    return;
                }
                if !self.is_match_allow_globs(import_decl.source.value.as_str()) {
                    ctx.diagnostic(no_unassigned_import_diagnostic(
                        import_decl.span,
                        "Imported module should be assigned",
                    ));
                }
            }
            AstKind::ExpressionStatement(statement) => {
                let Expression::CallExpression(call_expr) = &statement.expression else {
                    return;
                };
                if !call_expr.is_require_call() {
                    return;
                }
                let first_arg = &call_expr.arguments[0];
                let Argument::StringLiteral(source_str) = first_arg else {
                    return;
                };
                if !self.is_match_allow_globs(source_str.value.as_str()) {
                    ctx.diagnostic(no_unassigned_import_diagnostic(
                        call_expr.span,
                        "A `require()` style import is forbidden.",
                    ));
                }
            }
            _ => {}
        }
    }
}

impl NoUnassignedImportConfig {
    fn is_match_allow_globs(&self, source: &str) -> bool {
        self.globs.iter().any(|glob| fast_glob::glob_match(glob.as_str(), source))
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("import _ from 'foo'", None),
        ("import foo from 'foo'", None),
        ("import foo, { bar } from 'foo'", None),
        ("import * as _ from 'foo'", None),
        ("require('lodash')()", None),
        ("require('lodash').foo", None),
        ("require('lodash').foo()", None),
        ("const _ = require('./')", None),
        ("bar(require('foo'))", None),
        ("const [a, b] = require('lodash')", None),
        ("import 'app.css'", Some(json!([{ "allow": ["**/*.css"]}]))),
        ("import 'app.css'", Some(json!([{ "allow": ["*.css"]}]))),
        ("import './app.css'", Some(json!([{ "allow": ["**/*.css"]}]))),
        ("import '../dist/app.css'", Some(json!([{ "allow": ["**/*.css"]}]))),
        ("import '../dist/app.js'", Some(json!([{ "allow": ["**/dist/**"]}]))),
        ("import 'foo/bar'", Some(json!([{ "allow": ["foo/**"]}]))),
        ("import 'foo/bar'", Some(json!([{ "allow": ["foo/bar"]}]))),
        ("import 'babel-register'", Some(json!([{ "allow": ["babel-register"]}]))),
        ("require('./app.css')", Some(json!([{ "allow": ["**/*.css"]}]))),
        ("import './styles/app.css'", Some(json!([{ "allow": ["**/styles/*.css"]}]))),
    ];

    let fail = vec![
        ("require('should')", None),
        ("import 'foo'", None),
        ("import './styles/app.css'", Some(json!([{ "allow": ["styles/*.css"]}]))),
        ("import './app.css'", Some(json!([{ "allow": ["**/*.js"]}]))),
        ("import './app.css'", Some(json!([{ "allow": ["**/dir/**"]}]))),
        ("import './app.js'", None),
        ("require('./app.css')", Some(json!([{ "allow": ["**/*.js"]}]))),
    ];

    Tester::new(NoUnassignedImport::NAME, NoUnassignedImport::PLUGIN, pass, fail)
        .change_rule_path("no-unassigned-import.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
