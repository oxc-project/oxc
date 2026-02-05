mod diagnostic;
mod options;

#[cfg(test)]
mod tests;

use oxc_ast::AstKind;

use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::symbol::SymbolFlags;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

pub use options::{HoistOption, NoShadowConfig};



#[derive(Debug, Default, Clone)]
pub struct NoShadow(Box<NoShadowConfig>);

impl std::ops::Deref for NoShadow {
    type Target = NoShadowConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows variable declarations from shadowing variables declared in the outer scope.
    ///
    /// ### Why is this bad?
    ///
    /// Shadowing is the process by which a local variable shares the same name as a variable
    /// in its containing scope. This can cause confusion, as it may be unclear which variable
    /// is being referenced, and can lead to bugs that are difficult to diagnose.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var x = 1;
    /// function foo() {
    ///     var x = 2; // x shadows the outer x
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var x = 1;
    /// function foo() {
    ///     var y = 2; // different name, no shadowing
    /// }
    /// ```
    ///
    /// ### TypeScript
    ///
    /// This rule supports TypeScript-specific options:
    /// - `ignoreTypeValueShadow`: When `true` (default), ignores cases where a type and a value
    ///   have the same name (e.g., `type Foo = string; const Foo = 'bar';`).
    /// - `ignoreFunctionTypeParameterNameValueShadow`: When `true` (default), ignores cases where
    ///   a function type parameter shadows a value (e.g., `const T = 1; function foo<T>() {}`).
    NoShadow,
    eslint,
    suspicious,
    config = NoShadowConfig
);

impl Rule for NoShadow {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<NoShadowConfig>>(value)
            .map(|c| Self(Box::new(c.into_inner())))
    }

    fn run_once(&self, ctx: &LintContext) {
        let scoping = ctx.scoping();

        for symbol_id in scoping.symbol_ids() {
            let symbol_name = scoping.symbol_name(symbol_id);

            // Skip if in allow list
            if self.allow.iter().any(|allowed| allowed.as_str() == symbol_name) {
                continue;
            }

            let symbol_scope = scoping.symbol_scope_id(symbol_id);
            let symbol_flags = scoping.symbol_flags(symbol_id);
            let symbol_span = scoping.symbol_span(symbol_id);

            // Skip enum members - they don't shadow outer variables
            if symbol_flags.contains(SymbolFlags::EnumMember) {
                continue;
            }

            // Walk parent scopes looking for shadowed variables
            for parent_scope in scoping.scope_ancestors(symbol_scope).skip(1) {
                if let Some(shadowed_symbol_id) = scoping.get_binding(parent_scope, symbol_name) {
                    let shadowed_flags = scoping.symbol_flags(shadowed_symbol_id);
                    let shadowed_span = scoping.symbol_span(shadowed_symbol_id);

                    // Check if we should ignore this shadowing based on TypeScript rules
                    if self.should_ignore_shadow(
                        ctx,
                        symbol_id,
                        symbol_flags,
                        shadowed_symbol_id,
                        shadowed_flags,
                    ) {
                        break;
                    }

                    // Check hoisting rules
                    if !self.check_hoisting(symbol_span, shadowed_span, shadowed_flags) {
                        continue;
                    }

                    // Report the shadowing
                    // Report the shadowing
                    ctx.diagnostic(diagnostic::no_shadow(symbol_span, symbol_name, shadowed_span));
                    break;
                }
            }
        }
    }
}

impl NoShadow {
    /// Check if we should ignore this shadowing based on TypeScript-specific rules.
    fn should_ignore_shadow(
        &self,
        ctx: &LintContext,
        symbol_id: oxc_syntax::symbol::SymbolId,
        symbol_flags: SymbolFlags,
        shadowed_symbol_id: oxc_syntax::symbol::SymbolId,
        shadowed_flags: SymbolFlags,
    ) -> bool {
        // Check type vs value shadowing
        if self.ignore_type_value_shadow {
            let symbol_is_type = is_type_only(symbol_flags);
            let shadowed_is_type = is_type_only(shadowed_flags);

            // If one is a type and the other is a value, ignore
            if symbol_is_type != shadowed_is_type {
                return true;
            }
        }

        // Check function type parameter shadowing value
        if self.ignore_function_type_parameter_name_value_shadow
            && symbol_flags.contains(SymbolFlags::TypeParameter)
        {
            // Check if the type parameter is in a function context
            let declaration_node_id = ctx.scoping().symbol_declaration(symbol_id);
            let declaration_node = ctx.nodes().get_node(declaration_node_id);

            // Walk up to find if we're in a function
            for ancestor in ctx.nodes().ancestor_ids(declaration_node.id()) {
                let ancestor_node = ctx.nodes().get_node(ancestor);
                if matches!(
                    ancestor_node.kind(),
                    AstKind::Function(_)
                        | AstKind::ArrowFunctionExpression(_)
                        | AstKind::TSMethodSignature(_)
                        | AstKind::TSCallSignatureDeclaration(_)
                        | AstKind::TSConstructSignatureDeclaration(_)
                ) {
                    // This is a function type parameter, check if shadowed is a value
                    if !is_type_only(shadowed_flags) {
                        return true;
                    }
                    break;
                }
            }
        }

        // Check if shadowing an import that's only used as a type
        if shadowed_flags.contains(SymbolFlags::Import) {
            // Check if all references to the import are type-only
            let references: Vec<_> =
                ctx.scoping().get_resolved_references(shadowed_symbol_id).collect();
            let has_refs = !references.is_empty();
            let all_type_refs = references.iter().all(|r| r.is_type());

            if has_refs && all_type_refs && !is_type_only(symbol_flags) {
                // The import is only used as a type, and we're declaring a value
                // This is allowed in TypeScript
                return true;
            }
        }

        false
    }

    /// Check if shadowing should be reported based on hoisting rules.
    pub fn check_hoisting(
        &self,
        symbol_span: Span,
        shadowed_span: Span,
        shadowed_flags: SymbolFlags,
    ) -> bool {
        match self.hoist {
            HoistOption::All => true,
            HoistOption::Functions => {
                // Only report if the shadowed variable is a function or if the symbol
                // comes after the shadowed declaration
                shadowed_flags.contains(SymbolFlags::Function)
                    || symbol_span.start >= shadowed_span.start
            }
            HoistOption::Never => {
                // Only report if the symbol comes after the shadowed declaration
                symbol_span.start >= shadowed_span.start
            }
        }
    }
}

/// Check if the symbol is a type-only declaration (not a value).
fn is_type_only(flags: SymbolFlags) -> bool {
    flags.intersects(SymbolFlags::TypeAlias | SymbolFlags::Interface | SymbolFlags::TypeParameter)
        && !flags.intersects(
            SymbolFlags::FunctionScopedVariable
                | SymbolFlags::BlockScopedVariable
                | SymbolFlags::Function
                | SymbolFlags::Class
                | SymbolFlags::Enum
                | SymbolFlags::ConstEnum,
        )
}
