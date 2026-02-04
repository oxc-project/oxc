use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use oxc_syntax::symbol::SymbolFlags;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_shadow_diagnostic(span: Span, name: &str, shadowed_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is already declared in the upper scope."))
        .with_help(format!(
            "Consider renaming '{name}' to avoid shadowing the variable from the outer scope."
        ))
        .with_labels([
            span.label(format!("'{name}' is declared here")),
            shadowed_span.label("shadowed declaration is here"),
        ])
}

/// Controls how hoisting is handled when checking for shadowing.
#[derive(Debug, Clone, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HoistOption {
    /// Report shadowing even before the outer variable is declared (due to hoisting).
    All,
    /// Only report shadowing for function declarations that are hoisted.
    #[default]
    Functions,
    /// Never report shadowing before the outer variable is declared.
    Never,
}

#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoShadowConfig {
    /// Controls how hoisting is handled.
    #[serde(default)]
    pub hoist: HoistOption,

    /// List of variable names that are allowed to shadow.
    #[serde(default)]
    pub allow: Vec<CompactStr>,

    /// If `true`, ignore when a type and a value have the same name.
    /// This is common in TypeScript: `type Foo = ...; const Foo = ...;`
    #[serde(default = "default_true")]
    pub ignore_type_value_shadow: bool,

    /// If `true`, ignore when a function type parameter shadows a value.
    /// Example: `const T = 1; function foo<T>() {}`
    #[serde(default = "default_true")]
    pub ignore_function_type_parameter_name_value_shadow: bool,
}

fn default_true() -> bool {
    true
}

impl Default for NoShadowConfig {
    fn default() -> Self {
        Self {
            hoist: HoistOption::default(),
            allow: Vec::new(),
            ignore_type_value_shadow: true,
            ignore_function_type_parameter_name_value_shadow: true,
        }
    }
}

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
    /// ```ts
    /// const x = 1;
    /// function foo() {
    ///     const x = 2; // x shadows the outer x
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const x = 1;
    /// function foo() {
    ///     const y = 2; // different name, no shadowing
    /// }
    /// ```
    NoShadow,
    typescript,
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
                    ctx.diagnostic(no_shadow_diagnostic(symbol_span, symbol_name, shadowed_span));
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
    fn check_hoisting(
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

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Different names - no shadowing
        ("const x = 1; function foo() { const y = 2; }", None),
        // Same name in different functions - no shadowing
        ("function foo(x) { } function bar(x) { }", None),
        // Type vs value with same name (TypeScript) - default ignored
        ("type Foo = string; const Foo = 'bar';", None),
        // Interface vs value with same name
        ("interface Foo { x: number } const Foo = { x: 1 };", None),
        // Allowed names
        (
            "const x = 1; function foo() { const x = 2; }",
            Some(serde_json::json!([{ "allow": ["x"] }])),
        ),
        // Type parameter shadowing value (default ignored)
        ("const T = 1; function foo<T>() { }", None),
        // Enum member doesn't shadow
        ("const Red = 1; enum Color { Red, Green, Blue }", None),
        // Class with same name as type (declaration merging)
        ("interface Foo { x: number } class Foo { x = 1; }", None),
        // Namespace with same name as class
        ("class Foo { } namespace Foo { export const x = 1; }", None),
        // Import used only as type, value with same name
        (
            r#"
            import { Foo } from './foo';
            type X = Foo;
            const Foo = 1;
            "#,
            None,
        ),
    ];

    let fail = vec![
        // Basic shadowing
        ("const x = 1; function foo() { const x = 2; }", None),
        // Block scope shadowing
        ("const x = 1; { const x = 2; }", None),
        // Parameter shadowing outer variable
        ("const x = 1; function foo(x) { }", None),
        // Nested function shadowing
        ("function foo() { const x = 1; function bar() { const x = 2; } }", None),
        // Arrow function shadowing
        ("const x = 1; const foo = () => { const x = 2; };", None),
        // Class method shadowing
        ("const x = 1; class Foo { method() { const x = 2; } }", None),
        // Let shadowing
        ("let x = 1; { let x = 2; }", None),
        // Var shadowing (hoisted)
        ("var x = 1; function foo() { var x = 2; }", None),
        // Type shadowing type (not ignored)
        ("type Foo = string; { type Foo = number; }", None),
        // Interface shadowing interface
        ("interface Foo { x: number } { interface Foo { y: string } }", None),
        // Catch clause shadowing
        ("const e = 1; try { } catch (e) { }", None),
        // For loop variable shadowing
        ("const i = 1; for (let i = 0; i < 10; i++) { }", None),
        // Destructuring shadowing in nested scope
        ("const x = 1; { const { x } = { x: 2 }; }", None),
        // Array destructuring shadowing in nested scope
        ("const x = 1; { const [x] = [2]; }", None),
    ];

    Tester::new(NoShadow::NAME, NoShadow::PLUGIN, pass, fail).test_and_snapshot();
}
