mod allowed;
mod binding_pattern;
mod diagnostic;
mod ignored;
mod options;
mod symbol;
#[cfg(test)]
mod tests;
mod usage;

use std::ops::Deref;

use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ScopeFlags, SymbolFlags, SymbolId};
use oxc_span::GetSpan;

use crate::{context::LintContext, rule::Rule};
use options::NoUnusedVarsOptions;

use symbol::Symbol;

#[derive(Debug, Default, Clone)]
pub struct NoUnusedVars(Box<NoUnusedVarsOptions>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows variable declarations or imports that are not used in code.
    ///
    /// ### Why is this bad?
    ///
    /// Variables that are declared and not used anywhere in the code are most
    /// likely an error due to incomplete refactoring. Such variables take up
    /// space in the code and can lead to confusion by readers.
    ///
    /// A variable `foo` is considered to be used if any of the following are
    /// true:
    ///
    /// * It is called (`foo()`) or constructed (`new foo()`)
    /// * It is read (`var bar = foo`)
    /// * It is passed into a function as an argument (`doSomething(foo)`)
    /// * It is read inside of a function that is passed to another function
    ///   (`doSomething(function() { foo(); })`)
    ///
    /// A variable is _not_ considered to be used if it is only ever declared
    /// (`var foo = 5`) or assigned to (`foo = 7`).
    ///
    /// #### Exported
    ///
    /// In environments outside of CommonJS or ECMAScript modules, you may use
    /// `var` to create a global variable that may be used by other scripts. You
    /// can use the `/* exported variableName */` comment block to indicate that
    /// this variable is being exported and therefore should not be considered
    /// unused.
    ///
    /// Note that `/* exported */` has no effect for any of the following:
    /// * when the environment is `node` or `commonjs`
    /// * when `parserOptions.sourceType` is `module`
    /// * when `ecmaFeatures.globalReturn` is `true`
    ///
    /// The line comment `//exported variableName` will not work as `exported`
    /// is not line-specific.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```javascript
    /// /*eslint no-unused-vars: "error"*/
    /// /*global some_unused_var*/
    ///
    /// // It checks variables you have defined as global
    /// some_unused_var = 42;
    ///
    /// var x;
    ///
    /// // Write-only variables are not considered as used.
    /// var y = 10;
    /// y = 5;
    ///
    /// // A read for a modification of itself is not considered as used.
    /// var z = 0;
    /// z = z + 1;
    ///
    /// // By default, unused arguments cause warnings.
    /// (function(foo) {
    ///     return 5;
    /// })();
    ///
    /// // Unused recursive functions also cause warnings.
    /// function fact(n) {
    ///     if (n < 2) return 1;
    ///     return n * fact(n - 1);
    /// }
    ///
    /// // When a function definition destructures an array, unused entries from
    /// // the array also cause warnings.
    /// function getY([x, y]) {
    ///     return y;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /*eslint no-unused-vars: "error"*/
    ///
    /// var x = 10;
    /// alert(x);
    ///
    /// // foo is considered used here
    /// myFunc(function foo() {
    ///     // ...
    /// }.bind(this));
    ///
    /// (function(foo) {
    ///     return foo;
    /// })();
    ///
    /// var myFunc;
    /// myFunc = setTimeout(function() {
    ///     // myFunc is considered used
    ///     myFunc();
    /// }, 50);
    ///
    /// // Only the second argument from the destructured array is used.
    /// function getY([, y]) {
    ///     return y;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for `/* exported variableName */` operation:
    /// ```javascript
    /// /* exported global_var */
    ///
    /// var global_var = 42;
    /// ```
    NoUnusedVars,
    correctness
);

impl Deref for NoUnusedVars {
    type Target = NoUnusedVarsOptions;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Rule for NoUnusedVars {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(NoUnusedVarsOptions::from(value)))
    }

    fn run_on_symbol(&self, symbol_id: SymbolId, ctx: &LintContext<'_>) {
        let symbol = Symbol::new(ctx.semantic().as_ref(), symbol_id);
        if Self::should_skip_symbol(&symbol) {
            return;
        }

        self.run_on_symbol_internal(&symbol, ctx);
    }

    fn should_run(&self, ctx: &LintContext) -> bool {
        // ignore .d.ts and vue files.
        // 1. declarations have side effects (they get merged together)
        // 2. vue scripts declare variables that get used in the template, which
        //    we can't detect
        !ctx.source_type().is_typescript_definition()
            && !ctx.file_path().extension().is_some_and(|ext| ext == "vue")
    }
}

impl NoUnusedVars {
    fn run_on_symbol_internal<'a>(&self, symbol: &Symbol<'_, 'a>, ctx: &LintContext<'a>) {
        let is_ignored = self.is_ignored(symbol);

        if is_ignored && !self.report_used_ignore_pattern {
            return;
        }

        // Order matters. We want to call cheap/high "yield" functions first.
        let is_exported = symbol.is_exported();
        let is_used = is_exported || symbol.has_usages(self);

        match (is_used, is_ignored) {
            (true, true) => {
                if self.report_used_ignore_pattern {
                    ctx.diagnostic(diagnostic::used_ignored(symbol));
                }
                return;
            },
            // not used but ignored, no violation
            (false, true)
            // used and not ignored, no violation
            | (true, false) => {
                return
            },
            // needs acceptance check and/or reporting
            (false, false) => {}
        }

        let declaration = symbol.declaration();
        match declaration.kind() {
            // NOTE: match_module_declaration(AstKind) does not work here
            AstKind::ImportDeclaration(_)
            | AstKind::ImportSpecifier(_)
            | AstKind::ImportExpression(_)
            | AstKind::ImportDefaultSpecifier(_)
            | AstKind::ImportNamespaceSpecifier(_) => {
                if !is_ignored {
                    ctx.diagnostic(diagnostic::imported(symbol));
                }
            }
            AstKind::VariableDeclarator(decl) => {
                if self.is_allowed_variable_declaration(symbol, decl) {
                    return;
                };
                let report =
                    if let Some(last_write) = symbol.references().rev().find(|r| r.is_write()) {
                        // ahg
                        let span = ctx.nodes().get_node(last_write.node_id()).kind().span();
                        diagnostic::assign(symbol, span)
                    } else {
                        diagnostic::declared(symbol)
                    };
                ctx.diagnostic(report);
            }
            AstKind::FormalParameter(param) => {
                if self.is_allowed_argument(ctx.semantic().as_ref(), symbol, param) {
                    return;
                }
                ctx.diagnostic(diagnostic::param(symbol));
            }
            AstKind::Class(_) | AstKind::Function(_) => {
                if self.is_allowed_class_or_function(symbol) {
                    return;
                }
                ctx.diagnostic(diagnostic::declared(symbol));
            }
            AstKind::TSModuleDeclaration(namespace) => {
                if self.is_allowed_ts_namespace(symbol, namespace) {
                    return;
                }
                ctx.diagnostic(diagnostic::declared(symbol));
            }
            AstKind::TSInterfaceDeclaration(_) => {
                if symbol.is_in_declared_module() {
                    return;
                }
                ctx.diagnostic(diagnostic::declared(symbol));
            }
            AstKind::TSTypeParameter(_) => {
                if self.is_allowed_type_parameter(symbol, declaration.id()) {
                    return;
                }
                ctx.diagnostic(diagnostic::declared(symbol));
            }
            _ => ctx.diagnostic(diagnostic::declared(symbol)),
        };
    }

    fn should_skip_symbol(symbol: &Symbol<'_, '_>) -> bool {
        const AMBIENT_NAMESPACE_FLAGS: SymbolFlags =
            SymbolFlags::NameSpaceModule.union(SymbolFlags::Ambient);
        let flags = symbol.flags();

        // 1. ignore enum members. Only enums get checked
        // 2. ignore all ambient TS declarations, e.g. `declare class Foo {}`
        if flags.intersects(SymbolFlags::EnumMember.union(SymbolFlags::Ambient))
            // ambient namespaces
            || flags == AMBIENT_NAMESPACE_FLAGS
            || (symbol.is_in_ts() && symbol.is_in_declare_global())
        {
            return true;
        }

        // In some cases (e.g. "jsx": "react" in tsconfig.json), React imports
        // get used in generated code. We don't have a way to detect
        // "jsxPragmas" or whether TSX files are using "jsx": "react-jsx", so we
        // just allow all cases.
        if symbol.flags().contains(SymbolFlags::Import)
            && symbol.is_in_jsx()
            && symbol.is_possibly_jsx_factory()
        {
            return true;
        }

        false
    }
}

impl Symbol<'_, '_> {
    #[inline]
    fn is_possibly_jsx_factory(&self) -> bool {
        let name = self.name();
        name == "React" || name == "h"
    }

    fn is_in_declare_global(&self) -> bool {
        self.scopes()
            .ancestors(self.scope_id())
            .filter(|scope_id| {
                let flags = self.scopes().get_flags(*scope_id);
                flags.contains(ScopeFlags::TsModuleBlock)
            })
            .any(|ambient_module_scope_id| {
                let AstKind::TSModuleDeclaration(module) = self
                    .nodes()
                    .get_node(self.scopes().get_node_id(ambient_module_scope_id))
                    .kind()
                else {
                    return false;
                };

                module.kind.is_global()
            })
    }
}
