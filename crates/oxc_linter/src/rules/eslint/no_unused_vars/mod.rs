mod allowed;
mod binding_pattern;
mod diagnostic;
mod fixers;
mod ignored;
mod options;
mod symbol;
#[cfg(test)]
mod tests;
mod usage;

use std::ops::Deref;

use options::{IgnorePattern, NoUnusedVarsOptions};
use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, ScopeFlags, SymbolFlags, SymbolId};
use oxc_span::GetSpan;
use symbol::Symbol;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
};

#[derive(Debug, Default, Clone)]
pub struct NoUnusedVars(Box<NoUnusedVarsOptions>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows variable declarations, imports, or type declarations that are
    /// not used in code.
    ///
    /// ### Why is this bad?
    ///
    /// Variables that are declared and not used anywhere in the code are most
    /// likely an error due to incomplete refactoring. Such variables take up
    /// space in the code and can lead to confusion by readers.
    ///
    /// ```ts
    /// // `b` is unused; this indicates a bug.
    /// function add(a: number, b: number) {
    ///     return a;
    /// }
    /// console.log(add(1, 2));
    /// ```
    ///
    /// A variable `foo` is considered to be used if any of the following are
    /// true:
    ///
    /// * It is called (`foo()`) or constructed (`new foo()`)
    /// * It is read (`var bar = foo`)
    /// * It is passed into a function or constructor as an argument (`doSomething(foo)`)
    /// * It is read inside of a function that is passed to another function
    ///   (`doSomething(function() { foo(); })`)
    /// * It is exported (`export const foo = 42`)
    /// * It is used as an operand to TypeScript's `typeof` operator (`const bar:
    ///   typeof foo = 4`)
    ///
    /// A variable is _not_ considered to be used if it is only ever declared
    /// (`var foo = 5`) or assigned to (`foo = 7`).
    ///
    /// #### Types
    /// This rule has full support for TypeScript types, interfaces, enums, and
    /// namespaces.
    ///
    /// A type or interface `Foo` is considered to be used if it is used in any
    /// of the following ways:
    /// - It is used in the definition of another type or interface.
    /// - It is used as a type annotation or as part of a function signature.
    /// - It is used in a cast or `satisfies` expression.
    ///
    /// A type or interface is _not_ considered to be used if it is only ever
    /// used in its own definition, e.g. `type Foo = Array<Foo>`.
    ///
    /// Enums and namespaces are treated the same as variables, classes,
    /// functions, etc.
    ///
    /// #### Ignored Files
    /// This rule ignores `.d.ts` files and `.vue` files entirely. Variables,
    /// classes, interfaces, and types declared in `.d.ts` files are generally
    /// used by other files, which are not checked by Oxlint. Since Oxlint does
    /// not support parsing Vue templates, this rule cannot tell if a variable
    /// is used or unused in a Vue file.
    ///
    /// #### Exported
    ///
    /// The original ESLint rule recognizes `/* exported variableName */`
    /// comments as a way to indicate that a variable is used in another script
    /// and should not be considered unused. Since ES6 modules are now a TC39
    /// standard, Oxlint does not support this feature.
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
    /// ```ts
    /// type A = Array<A>;
    ///
    /// enum Color {
    ///     Red,
    ///     Green,
    ///     Blue
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
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
    /// ```ts
    /// export const x = 1;
    /// const y = 1;
    /// export { y };
    ///
    /// type A = Record<string, unknown>;
    /// type B<T> = T extends Record<infer K, any> ? K : never;
    /// const x = 'foo' as B<A>;
    /// console.log(x);
    /// ```
    ///
    /// Examples of **incorrect** code for `/* exported variableName */` operation:
    /// ```js
    /// /* exported global_var */
    ///
    /// // Not respected, use ES6 modules instead.
    /// var global_var = 42;
    /// ```
    NoUnusedVars,
    correctness,
    dangerous_suggestion
);

impl Deref for NoUnusedVars {
    type Target = NoUnusedVarsOptions;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Rule for NoUnusedVars {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(NoUnusedVarsOptions::try_from(value).unwrap()))
    }

    fn run_on_symbol(&self, symbol_id: SymbolId, ctx: &LintContext<'_>) {
        let symbol = Symbol::new(ctx.semantic().as_ref(), symbol_id);
        if Self::should_skip_symbol(&symbol) {
            return;
        }

        self.run_on_symbol_internal(&symbol, ctx);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
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
                    ctx.diagnostic(diagnostic::used_ignored(symbol, &self.vars_ignore_pattern));
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
                let diagnostic = diagnostic::imported(symbol);
                let declaration =
                    symbol.iter_self_and_parents().map(AstNode::kind).find_map(|kind| match kind {
                        AstKind::ImportDeclaration(import) => Some(import),
                        _ => None,
                    });

                if let Some(declaration) = declaration {
                    ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                        self.remove_unused_import_declaration(fixer, symbol, declaration)
                    });
                } else {
                    ctx.diagnostic(diagnostic);
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
                        diagnostic::assign(symbol, span, &self.vars_ignore_pattern)
                    } else {
                        diagnostic::declared(symbol, &self.vars_ignore_pattern)
                    };

                ctx.diagnostic_with_suggestion(report, |fixer| {
                    // NOTE: suggestions produced by this fixer are all flagged
                    // as dangerous
                    self.rename_or_remove_var_declaration(fixer, symbol, decl, declaration.id())
                });
            }
            AstKind::FormalParameter(param) => {
                if self.is_allowed_argument(ctx.semantic().as_ref(), symbol, param) {
                    return;
                }
                ctx.diagnostic(diagnostic::param(symbol, &self.args_ignore_pattern));
            }
            AstKind::BindingRestElement(_) => {
                if NoUnusedVars::is_allowed_binding_rest_element(symbol) {
                    return;
                }
                ctx.diagnostic(diagnostic::declared(symbol, &self.vars_ignore_pattern));
            }
            AstKind::Class(_) | AstKind::Function(_) => {
                if self.is_allowed_class_or_function(symbol) {
                    return;
                }
                ctx.diagnostic(diagnostic::declared(symbol, &IgnorePattern::<&str>::None));
            }
            AstKind::TSModuleDeclaration(namespace) => {
                if self.is_allowed_ts_namespace(symbol, namespace) {
                    return;
                }
                ctx.diagnostic(diagnostic::declared(symbol, &IgnorePattern::<&str>::None));
            }
            AstKind::TSInterfaceDeclaration(_) => {
                if symbol.is_in_declared_module() {
                    return;
                }
                ctx.diagnostic(diagnostic::declared(symbol, &IgnorePattern::<&str>::None));
            }
            AstKind::TSTypeParameter(_) => {
                if self.is_allowed_type_parameter(symbol, declaration.id()) {
                    return;
                }
                ctx.diagnostic(diagnostic::declared(symbol, &self.vars_ignore_pattern));
            }
            AstKind::CatchParameter(_) => {
                ctx.diagnostic(diagnostic::declared(symbol, &self.caught_errors_ignore_pattern));
            }
            _ => ctx.diagnostic(diagnostic::declared(symbol, &IgnorePattern::<&str>::None)),
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
