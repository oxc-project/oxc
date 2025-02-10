use regex::Regex;

use oxc_ast::{
    ast::{Argument, TSModuleReference},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{CompactStr, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_require_imports_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected \"import\" statement instead of \"require\" call")
        .with_help("Do not use CommonJS `require` calls")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRequireImports(Box<NoRequireImportsConfig>);

#[derive(Debug, Default, Clone)]
pub struct NoRequireImportsConfig {
    allow: Vec<CompactStr>,
    allow_as_import: bool,
}

impl std::ops::Deref for NoRequireImports {
    type Target = NoRequireImportsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids the use of CommonJS `require` calls.
    ///
    /// ### Why is this bad?
    ///
    /// `require` imports, while functional in Node.js and older JavaScript environments, are generally
    /// considered less desirable than ES modules (`import`) for several key reasons in modern JavaScript development:
    ///
    /// 1. **Static vs. Dynamic**: `require` is a __runtime__ function. It executes when the code runs, which means errors related to missing modules or incorrect paths are only discovered during runtime. ES modules (`import`) are static imports. Their resolution and potential errors are checked during the compilation or bundling process, making them easier to catch during development.
    ///
    /// 2. **Code Organization and Readability**: `require` statements are scattered throughout the code, potentially making it harder to quickly identify the dependencies of a given module. `import` statements are typically grouped at the top of a file, improving code organization and readability.
    ///
    /// 3. **Tree Shaking and Optimization**: Modern bundlers like Webpack and Rollup use tree-shaking to remove unused code from the final bundle. Tree-shaking works significantly better with ES modules because their dependencies are declared statically and explicitly. `require` makes it harder for bundlers to accurately identify and remove unused code, resulting in larger bundle sizes and slower load times.
    ///
    /// 4. **Cyclic Dependencies**: Handling cyclic dependencies (where module A imports B, and B imports A) is significantly more challenging with `require`. ES modules, through their declarative nature and the use of dynamic imports (`import()`), provide better mechanisms to handle cyclic imports and manage asynchronous loading.
    ///
    /// 5. **Maintainability and Refactoring**: Changing module names or paths is simpler with ES modules because the changes are declared directly and the compiler or bundler catches any errors. With `require`, you might have to track down all instances of a specific `require` statement for a particular module, making refactoring more error-prone.
    ///
    /// 6. Modern JavaScript Standards: import is the standard way to import modules in modern JavaScript, aligned with current best practices and language specifications. Using require necessitates additional build steps or tools to translate it to a format that the browser or modern JavaScript environments can understand.
    ///
    /// 7. Error Handling: ES modules provide a more structured way to handle errors during module loading using `try...catch` blocks with dynamic imports, enhancing error management. `require` errors can be less predictable.
    ///
    /// In summary, while `require` works, the benefits of ES modules in terms of static analysis, better bundling, improved code organization, and easier maintainability make it the preferred method for importing modules in modern JavaScript projects.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const lib1 = require('lib1');
    /// const { lib2 } = require('lib2');
    /// import lib3 = require('lib3');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// import * as lib1 from 'lib1';
    /// import { lib2 } from 'lib2';
    /// import * as lib3 from 'lib3';
    /// ```
    ///
    /// ### Options
    ///
    /// #### `allow`
    ///
    /// array of strings
    ///
    /// These strings will be compiled into regular expressions with the u flag and be used to test against the imported path.
    /// A common use case is to allow importing `package.json`. This is because `package.json` commonly lives outside of the TS root directory,
    /// so statically importing it would lead to root directory conflicts, especially with `resolveJsonModule` enabled.
    /// You can also use it to allow importing any JSON if your environment doesn't support JSON modules, or use it for other cases where `import` statements cannot work.
    ///
    /// With { allow: ['/package\\.json$'] }:
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// console.log(require('../package.json').version);
    /// ```
    ///
    /// #### `allowAsImport`
    ///
    /// When set to `true`, `import ... = require(...)` declarations won't be reported.
    /// This is useful if you use certain module options that require strict CommonJS interop semantics.
    ///
    /// With `{ allowAsImport: true }`:
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// var foo = require('foo');
    /// const foo = require('foo');
    /// let foo = require('foo');
    /// ```
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// import foo = require('foo');
    /// import foo from 'foo';
    /// ```
    ///
    NoRequireImports,
    typescript,
    restriction,
    pending  // TODO: fixer (change require to import)
);

fn match_argument_value_with_regex(allow: &[CompactStr], argument_value: &str) -> bool {
    allow
        .iter()
        .map(|pattern| Regex::new(pattern).unwrap())
        .any(|regex| regex.is_match(argument_value))
}

impl Rule for NoRequireImports {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);
        Self(Box::new(NoRequireImportsConfig {
            allow: obj
                .and_then(|v| v.get("allow"))
                .and_then(serde_json::Value::as_array)
                .map(|v| {
                    v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect()
                })
                .unwrap_or_default(),
            allow_as_import: obj
                .and_then(|v| v.get("allowAsImport"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call_expr) => {
                if node.scope_id() != ctx.scopes().root_scope_id() {
                    if let Some(id) = call_expr.callee.get_identifier_reference() {
                        if !id.is_global_reference_name("require", ctx.symbols()) {
                            return;
                        }
                    }
                }

                if !call_expr.is_require_call() {
                    return;
                }

                if !self.allow.is_empty() {
                    let Some(argument) = call_expr.arguments.first() else {
                        return;
                    };

                    match argument {
                        Argument::TemplateLiteral(template_literal) => {
                            let Some(quasi) = template_literal.quasis.first() else {
                                return;
                            };

                            if match_argument_value_with_regex(&self.allow, &quasi.value.raw) {
                                return;
                            }

                            ctx.diagnostic(no_require_imports_diagnostic(quasi.span));
                        }
                        Argument::StringLiteral(string_literal) => {
                            if match_argument_value_with_regex(&self.allow, &string_literal.value) {
                                return;
                            }

                            ctx.diagnostic(no_require_imports_diagnostic(string_literal.span));
                        }
                        _ => {}
                    }
                }

                if ctx.scopes().find_binding(ctx.scopes().root_scope_id(), "require").is_some() {
                    return;
                }

                ctx.diagnostic(no_require_imports_diagnostic(call_expr.span));
            }
            AstKind::TSImportEqualsDeclaration(decl) => match &decl.module_reference {
                TSModuleReference::ExternalModuleReference(mod_ref) => {
                    if self.allow_as_import {
                        return;
                    }

                    if !self.allow.is_empty() {
                        if match_argument_value_with_regex(&self.allow, &mod_ref.expression.value) {
                            return;
                        }

                        ctx.diagnostic(no_require_imports_diagnostic(mod_ref.span));
                    }

                    ctx.diagnostic(no_require_imports_diagnostic(decl.span));
                }
                TSModuleReference::IdentifierReference(_) | TSModuleReference::QualifiedName(_) => {
                }
            },
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("import { l } from 'lib';", None),
        ("var lib3 = load('not_an_import');", None),
        ("var lib4 = lib2.subImport;", None),
        ("var lib7 = 700;", None),
        ("import lib9 = lib2.anotherSubImport;", None),
        ("import lib10 from 'lib10';", None),
        ("var lib3 = load?.('not_an_import');", None),
        (
            "
			import { createRequire } from 'module';
			const require = createRequire();
			require('remark-preset-prettier');
			    ",
            None,
        ),
        (
            "const pkg = require('./package.json');",
            Some(serde_json::json!([{ "allow": ["/package\\.json$"] }])),
        ),
        (
            "const pkg = require('../package.json');",
            Some(serde_json::json!([{ "allow": ["/package\\.json$"] }])),
        ),
        (
            "const pkg = require(`./package.json`);",
            Some(serde_json::json!([{ "allow": ["/package\\.json$"] }])),
        ),
        (
            "const pkg = require('../packages/package.json');",
            Some(serde_json::json!([{ "allow": ["/package\\.json$"] }])),
        ),
        (
            "import pkg = require('../packages/package.json');",
            Some(serde_json::json!([{ "allow": ["/package\\.json$"] }])),
        ),
        (
            "import pkg = require('data.json');",
            Some(serde_json::json!([{ "allow": ["\\.json$"] }])),
        ),
        (
            "import pkg = require('some-package');",
            Some(serde_json::json!([{ "allow": ["^some-package$"] }])),
        ),
        ("import foo = require('foo');", Some(serde_json::json!([{ "allowAsImport": true }]))),
        (
            "
			let require = bazz;
			trick(require('foo'));
			      ",
            Some(serde_json::json!([{ "allowAsImport": true }])),
        ),
        (
            "
			let require = bazz;
			const foo = require('./foo.json') as Foo;
			      ",
            Some(serde_json::json!([{ "allowAsImport": true }])),
        ),
        (
            "
			let require = bazz;
			const foo: Foo = require('./foo.json').default;
			      ",
            Some(serde_json::json!([{ "allowAsImport": true }])),
        ),
        (
            "
			let require = bazz;
			const foo = <Foo>require('./foo.json');
			      ",
            Some(serde_json::json!([{ "allowAsImport": true }])),
        ),
        (
            "
			let require = bazz;
			const configValidator = new Validator(require('./a.json'));
			configValidator.addSchema(require('./a.json'));
			      ",
            Some(serde_json::json!([{ "allowAsImport": true }])),
        ),
        (
            "
			let require = bazz;
			require('foo');
			      ",
            Some(serde_json::json!([{ "allowAsImport": true }])),
        ),
        (
            "
			let require = bazz;
			require?.('foo');
			      ",
            Some(serde_json::json!([{ "allowAsImport": true }])),
        ),
        (
            "
			import { createRequire } from 'module';
			const require = createRequire();
			require('remark-preset-prettier');
			      ",
            Some(serde_json::json!([{ "allowAsImport": true }])),
        ),
    ];

    let fail = vec![
        ("var lib = require('lib');", None),
        ("let lib2 = require('lib2');", None),
        (
            "
			var lib5 = require('lib5'),
			  lib6 = require('lib6');
			      ",
            None,
        ),
        ("import lib8 = require('lib8');", None),
        ("var lib = require?.('lib');", None),
        ("let lib2 = require?.('lib2');", None),
        (
            "
			var lib5 = require?.('lib5'),
			  lib6 = require?.('lib6');
			      ",
            None,
        ),
        ("const pkg = require('./package.json');", None),
        (
            "const pkg = require('./package.jsonc');",
            Some(serde_json::json!([{ "allow": ["/package\\.json$"] }])),
        ),
        (
            "const pkg = require(`./package.jsonc`);",
            Some(serde_json::json!([{ "allow": ["/package\\.json$"] }])),
        ),
        ("import pkg = require('./package.json');", None),
        (
            "import pkg = require('./package.jsonc');",
            Some(serde_json::json!([{ "allow": ["/package\\.json$"] }])),
        ),
        (
            "import pkg = require('./package.json');",
            Some(serde_json::json!([{ "allow": ["^some-package$"] }])),
        ),
        ("var foo = require?.('foo');", Some(serde_json::json!([{ "allowAsImport": true }]))),
        (
            "let foo = trick(require?.('foo'));",
            Some(serde_json::json!([{ "allowAsImport": true }])),
        ),
        ("trick(require('foo'));", Some(serde_json::json!([{ "allowAsImport": true }]))),
        (
            "const foo = require('./foo.json') as Foo;",
            Some(serde_json::json!([{ "allowAsImport": true }])),
        ),
        (
            "const foo: Foo = require('./foo.json').default;",
            Some(serde_json::json!([{ "allowAsImport": true }])),
        ),
        (
            "const foo = <Foo>require('./foo.json');",
            Some(serde_json::json!([{ "allowAsImport": true }])),
        ),
        (
            "
			const configValidator = new Validator(require('./a.json'));
			configValidator.addSchema(require('./a.json'));
			      ",
            Some(serde_json::json!([{ "allowAsImport": true }])),
        ),
        ("require('foo');", Some(serde_json::json!([{ "allowAsImport": true }]))),
        ("require?.('foo');", Some(serde_json::json!([{ "allowAsImport": true }]))),
        // covers global require in scope
        (
            r"function foo() { 
            require('foo')
            }",
            None,
        ),
    ];

    Tester::new(NoRequireImports::NAME, NoRequireImports::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}
