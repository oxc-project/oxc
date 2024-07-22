mod diagnostic;
mod options;
mod symbol;
#[cfg(test)]
mod tests;
mod usage;

use std::ops::Deref;

use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;

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
    nursery
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
        let is_ignored = self.is_ignored(&symbol);

        if is_ignored && !self.report_unused_ignore_pattern {
            return;
        }

        let is_used = self.is_used(&symbol);
        if is_used {
            if is_ignored {
                // TODO: report unused ignore pattern
            }
            return;
        }

        match symbol.declaration().kind() {
            // NOTE: match_module_declaration(AstKind) does not work here
            AstKind::ImportDeclaration(_)
            | AstKind::ImportSpecifier(_)
            | AstKind::ImportExpression(_)
            | AstKind::ImportDefaultSpecifier(_)
            | AstKind::ImportNamespaceSpecifier(_) => {
                if !is_ignored {
                    ctx.diagnostic(diagnostic::imported(&symbol));
                }
                return;
            }
            AstKind::VariableDeclarator(decl) => {
                if decl.kind.is_var() && self.vars.is_local() && symbol.is_root() {
                    return;
                }
                let report =
                    if let Some(last_write) = symbol.references().rev().find(|r| r.is_write()) {
                        diagnostic::assign(&symbol, last_write.span())
                    } else {
                        diagnostic::declared(&symbol)
                    };
                ctx.diagnostic(report);
                return;
            }
            _ => {}
        }
        match (is_ignored, is_used) {
            (true, true) => {
                // TODO: report unused ignore pattern
            }
            (false, false) => {
                // TODO: choose correct diagnostic
                ctx.diagnostic(diagnostic::declared(&symbol));
                // self.report_unused(&symbol, ctx);
            }
            _ => { /* no violation */ }
        }
    }

    fn should_run(&self, ctx: &LintContext) -> bool {
        // ignore .d.ts and vue files.
        // 1. declarations have side effects (they get merged together)
        // 2. vue scripts delare variables that get used in the template, which
        //    we can't detect
        !ctx.source_type().is_typescript_definition() && !ctx.file_path().ends_with(".vue")
    }
}
