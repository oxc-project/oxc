use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

const COMMON_JS_GLOBALS: [&str; 5] = ["exports", "require", "module", "__filename", "__dirname"];

fn use_strict_directive_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(r#"Do not use "use strict" directive."#)
        .with_help("ES modules are always strict mode, so this directive is redundant.")
        .with_label(span)
}

fn global_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(r#""return" should be used inside a function."#).with_label(span)
}

fn common_js_global_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(r#"Do not use "{name}"."#))
        .with_help("Prefer ES modules over CommonJS globals.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferModule;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer JavaScript modules (ESM) over CommonJS.
    ///
    /// ### Why is this bad?
    ///
    /// CommonJS globals and patterns (`require`, `module`, `exports`, `__filename`, `__dirname`)
    /// make code harder to migrate and can block ESM-only features.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// "use strict";
    /// const foo = require("foo");
    /// module.exports = foo;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import foo from "foo";
    /// export default foo;
    /// ```
    PreferModule,
    unicorn,
    restriction,
    pending
);

impl Rule for PreferModule {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Directive(directive) => {
                if directive.directive == "use strict" {
                    ctx.diagnostic(use_strict_directive_diagnostic(directive.span));
                }
            }
            AstKind::ReturnStatement(return_statement) => {
                if is_top_level_return(node, ctx) {
                    ctx.diagnostic(global_return_diagnostic(return_statement.span));
                }
            }
            AstKind::IdentifierReference(identifier) => {
                let name = identifier.name.as_str();
                if COMMON_JS_GLOBALS.contains(&name)
                    && ctx.is_reference_to_global_variable(identifier)
                {
                    ctx.diagnostic(common_js_global_diagnostic(identifier.span, name));
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        !ctx.file_extension().is_some_and(|ext| ext.eq_ignore_ascii_case("cjs"))
    }
}

fn is_top_level_return(return_node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    ctx.nodes()
        .ancestors(return_node.id())
        .skip(1)
        .all(|ancestor| !ancestor.kind().is_function_like())
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (r#"const foo = "use strict";"#, None, None, None),
        (r#"("use strict")"#, None, None, None),
        (r#""use strong";"#, None, None, None),
        (r#"eval("'use strict'; var x = 42; x;");"#, None, None, None),
        (r#"new Function("'use strict'; var x = 42; return x;");"#, None, None, None),
        (
            "function a() {
                return;
            }",
            None,
            None,
            None,
        ),
        (
            "const __filename = 1;
            const foo = __filename;",
            None,
            None,
            None,
        ),
        (
            "const __dirname = 1;
            const foo = __dirname;",
            None,
            None,
            None,
        ),
        (r#"import {__filename as filename} from "foo.mjs""#, None, None, None),
        ("const foo = 1;export {foo as __dirname}", None, None, None),
        (
            r#"import {createRequire} from 'module';
            const require = createRequire(import.meta.url);
            const foo = require("foo");"#,
            None,
            None,
            None,
        ),
        (
            "const require = function require(require, id) {
                return require(id);
            }",
            None,
            None,
            None,
        ),
        (
            "const require = class A {
                require(require) {
                    return require(id);
                }
            }",
            None,
            None,
            None,
        ),
        ("function exports(exports) {}", None, None, None),
        ("function module(module) {}", None, None, None),
        (
            "const exports = foo;
            exports.bar = bar;",
            None,
            None,
            None,
        ),
        ("const exports = 1;", None, None, None),
        (
            "const module = foo;
            module.exports = bar;
            module.exports.bar = bar;",
            None,
            None,
            None,
        ),
        ("const module = 1;", None, None, None),
        ("type module = number[];", None, None, None),
        ("type ModuleRegistry = { [module: string]: string };", None, None, None),
        ("const module = 1; type ModuleRegistry = { [module: string]: string };", None, None, None),
        (
            "type module = number[]; type ModuleRegistry = { [module: string]: string };",
            None,
            None,
            None,
        ),
        ("type Data = { [module in keyof string]: number; };", None, None, None),
        ("type ModuleRegistry = { [exports: string]: string };", None, None, None),
        ("__dirname", None, None, Some(PathBuf::from("foo.cjs"))),
        ("__dirname", None, None, Some(PathBuf::from("foo.cjS"))),
    ];

    #[expect(clippy::literal_string_with_formatting_args)]
    let fail = vec![
        (
            "'use strict';
            console.log(1);",
            None,
            None,
            None,
        ),
        (
            r#""use strict";
            console.log(1);"#,
            None,
            None,
            None,
        ),
        (
            r#"function foo () {
                "use strict";
                console.log(1);
            }"#,
            None,
            None,
            None,
        ),
        (
            "'use strict';
            console.log(1);",
            None,
            None,
            Some(PathBuf::from("example.mjs")),
        ),
        (
            "if (foo) {
                return;
            }",
            None,
            None,
            None,
        ), // { "sourceType": "script", "parserOptions": { "ecmaFeatures": { "globalReturn": true, }, }, },
        ("const dirname = __dirname;", None, None, None),
        ("const dirname = __filename;", None, None, None),
        ("const foo = { __dirname};", None, None, None),
        ("const foo = {__filename, };", None, None, None),
        (r#"if (__dirname.startsWith("/project/src/")) {}"#, None, None, None),
        (r#"if (__filename.endsWith(".js")) {}"#, None, None, None),
        (r#"require("foo");"#, None, None, None),
        ("require('foo');", None, None, None),
        (r#"require( (("foo")) );"#, None, None, None),
        (r#"((require))("foo");"#, None, None, None),
        (r#"(( require("foo") ));"#, None, None, None),
        (r#"const foo=require("foo");"#, None, None, None),
        (r#"const foo = require.resolve("foo");"#, None, None, None),
        (
            r#"const foo
                =
                require("foo");"#,
            None,
            None,
            None,
        ),
        (r#"const foo = require("foo");"#, None, None, None),
        (r#"const foo = require( (("foo")) );"#, None, None, None),
        (r#"const foo = ((require))("foo");"#, None, None, None),
        (r#"const foo = (( require("foo") ));"#, None, None, None),
        (r#"const {foo}=require("foo");"#, None, None, None),
        (
            r#"const {foo}
                =
                require("foo");"#,
            None,
            None,
            None,
        ),
        (r#"const {foo} = require("foo");"#, None, None, None),
        (r#"const {foo} = require( (("foo")) );"#, None, None, None),
        (r#"const {foo} = ((require))("foo");"#, None, None, None),
        (r#"const {foo} = (( require("foo") ));"#, None, None, None),
        (r#"const {foo} = (( require("foo") ));"#, None, None, None),
        (r#"const {foo: foo}=require("foo");"#, None, None, None),
        (
            r#"const {foo: foo}
                =
                require("foo");"#,
            None,
            None,
            None,
        ),
        (r#"const {foo: foo} = require("foo");"#, None, None, None),
        (r#"const {foo: foo} = require( (("foo")) );"#, None, None, None),
        (r#"const {foo: foo} = ((require))("foo");"#, None, None, None),
        (r#"const {foo: foo} = (( require("foo") ));"#, None, None, None),
        (r#"const {foo: foo} = (( require("foo") ));"#, None, None, None),
        (r#"const {foo:bar}=require("foo");"#, None, None, None),
        (
            r#"const {foo:bar}
                =
                require("foo");"#,
            None,
            None,
            None,
        ),
        (r#"const {foo:bar} = require("foo");"#, None, None, None),
        (r#"const {foo:bar} = require( (("foo")) );"#, None, None, None),
        (r#"const {foo:bar} = ((require))("foo");"#, None, None, None),
        (r#"const {foo:bar} = (( require("foo") ));"#, None, None, None),
        (r#"const {foo:bar} = (( require("foo") ));"#, None, None, None),
        (r#"const {a   :foo, b:   bar, default   :   baz}=require("foo");"#, None, None, None),
        (
            r#"const {
                a   :foo,
                b:   bar,
                default   :   baz,
            }
                =
                require("foo");"#,
            None,
            None,
            None,
        ),
        (r#"const {a   :foo, b:   bar, default   :   baz} = require("foo");"#, None, None, None),
        (
            r#"const {a   :foo, b:   bar, default   :   baz} = require( (("foo")) );"#,
            None,
            None,
            None,
        ),
        (
            r#"const {a   :foo, b:   bar, default   :   baz} = ((require))("foo");"#,
            None,
            None,
            None,
        ),
        (
            r#"const {a   :foo, b:   bar, default   :   baz} = (( require("foo") ));"#,
            None,
            None,
            None,
        ),
        (
            r#"const {a   :foo, b:   bar, default   :   baz} = (( require("foo") ));"#,
            None,
            None,
            None,
        ),
        (r#"const {} = require("foo");"#, None, None, None),
        (r#"const{   }=require("foo");"#, None, None, None),
        (
            r#"const r = require;
            const foo = r("foo");"#,
            None,
            None,
            None,
        ),
        (r#"new require("foo")"#, None, None, None),
        (r#"require("foo", extraArgument)"#, None, None, None),
        ("const a = require()", None, None, None),
        (r#"require(..."foo")"#, None, None, None),
        (r#"require("../" + "file.js")"#, None, None, None),
        ("require(file)", None, None, None),
        (r#"a = require("foo")"#, None, None, None),
        (r#"function a(a = require("foo")) {}"#, None, None, None),
        (r#"let foo = require("foo");"#, None, None, None),
        (r#"const foo = require("foo"), bar = require("bar");"#, None, None, None),
        (r#"const {[foo]: foo} = require("foo");"#, None, None, None),
        (r#"const {["foo"]: foo} = require("foo");"#, None, None, None),
        (r#"if (foo) require("foo");"#, None, None, None),
        ("const foo = require`foo`;", None, None, None),
        (
            r#"function loadModule() {
                return require("foo");
            }"#,
            None,
            None,
            None,
        ),
        (
            r#"function loadModule() {
                const foo = require("foo");
                return foo;
            }"#,
            None,
            None,
            None,
        ),
        (r#"const foo = require("foo"), bar = 1;"#, None, None, None),
        (r#"const foo = require("foo"), bar = require("bar");"#, None, None, None),
        ("exports = foo;", None, None, None),
        ("module.exports = foo;", None, None, None),
        ("(( ((exports)) = ((foo)) ));", None, None, None),
        ("(( ((module.exports)) = ((foo)) ));", None, None, None),
        ("const foo = 1;exports.foo = foo;", None, None, None),
        ("const foo = 1;module.exports.foo = foo;", None, None, None),
        (r#"exports["foo"] = foo;"#, None, None, None),
        (r#"module.exports["foo"] = foo;"#, None, None, None),
        ("const foo = exports;", None, None, None),
        ("const foo = exports.foo;", None, None, None),
        ("const foo = module.exports;", None, None, None),
        ("const foo = module.exports.foo;", None, None, None),
        (r#"module["exports"] = foo;"#, None, None, None),
        ("module[exports] = foo;", None, None, None),
        ("module.exports.foo.bar = foo;", None, None, None),
        ("const foo = 1;exports.default = foo;", None, None, None),
        ("const foo = 1;module.exports.default = foo;", None, None, None),
        ("exports.foo.bar = foo;", None, None, None),
        ("exports = 1;", None, None, None),
        ("exports.foo = [];", None, None, None),
        ("module.exports = function() {};", None, None, None),
        ("module.exports.foo = foo || bar;", None, None, None),
        ("exports += foo;", None, None, None),
        ("const foo = module.children", None, None, None),
        ("const parentModule = module.parent", None, None, None),
        (
            "function foo() {
                exports.foo = foo;
                module.exports.foo = foo;
            }",
            None,
            None,
            None,
        ),
        ("__filename", None, None, Some(PathBuf::from("foo.mjs"))),
        (r#"require("lodash")"#, None, None, Some(PathBuf::from("foo.js"))),
        (r#"require("lodash")"#, None, None, Some(PathBuf::from("foo.cjs/foo.js"))),
    ];

    Tester::new(PreferModule::NAME, PreferModule::PLUGIN, pass, fail).test_and_snapshot();
}
