mod options;

#[cfg(test)]
mod tests;

use std::path::Path;

use javascript_globals::GLOBALS;
use oxc_allocator::GetAddress;
use oxc_ast::{
    AstKind,
    ast::{BindingPattern, Expression, FunctionType, TSModuleDeclarationName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::Reference;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    node::NodeId,
    symbol::{SymbolFlags, SymbolId},
};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

pub use options::{HoistOption, NoShadowConfig};

pub fn no_shadow_diagnostic(span: Span, name: &str, shadowed_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is already declared in the upper scope."))
        .with_help(format!(
            "Consider renaming '{name}' to avoid shadowing the variable from the outer scope."
        ))
        .with_labels([
            span.label(format!("'{name}' is declared here")),
            shadowed_span.label("shadowed declaration is here"),
        ])
}

pub fn no_shadow_global_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is already a global variable."))
        .with_help(format!("Consider renaming '{name}' to avoid shadowing the global variable."))
        .with_label(span)
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
            let symbol_name = scoping.symbol_ident(symbol_id);
            let symbol_name_str = symbol_name.as_str();
            let symbol_span = scoping.symbol_span(symbol_id);

            if self.should_skip_symbol(ctx, symbol_id, symbol_name_str) {
                continue;
            }

            if let Some(shadowed_symbol_id) = Self::find_shadowed_symbol_id(ctx, symbol_id) {
                if !self.should_ignore_shadowed_symbol(ctx, symbol_id, shadowed_symbol_id) {
                    let shadowed_span = scoping.symbol_span(shadowed_symbol_id);
                    ctx.diagnostic(no_shadow_diagnostic(
                        symbol_span,
                        symbol_name_str,
                        shadowed_span,
                    ));
                }
                continue;
            }

            if let Some(shadowed_span) =
                Self::function_expression_name_shadow_span(ctx, symbol_id, symbol_name_str)
            {
                ctx.diagnostic(no_shadow_diagnostic(symbol_span, symbol_name_str, shadowed_span));
                continue;
            }

            let symbol_flags = scoping.symbol_flags(symbol_id);
            if self.is_builtin_global_shadow(ctx, symbol_id, symbol_name_str, symbol_flags) {
                ctx.diagnostic(no_shadow_global_diagnostic(symbol_span, symbol_name_str));
            }
        }
    }
}

impl NoShadow {
    fn should_skip_symbol(
        &self,
        ctx: &LintContext,
        symbol_id: SymbolId,
        symbol_name: &str,
    ) -> bool {
        self.allow.iter().any(|allowed| allowed.as_str() == symbol_name)
            || Self::is_declare_in_definition_file(ctx, symbol_id)
            || Self::is_in_global_augmentation(ctx, symbol_id)
    }

    fn find_shadowed_symbol_id(ctx: &LintContext, symbol_id: SymbolId) -> Option<SymbolId> {
        let scoping = ctx.scoping();
        let symbol_name = scoping.symbol_ident(symbol_id);
        let symbol_scope = scoping.symbol_scope_id(symbol_id);

        scoping
            .scope_ancestors(symbol_scope)
            .skip(1)
            .find_map(|scope_id| scoping.get_binding(scope_id, symbol_name))
    }

    fn should_ignore_shadowed_symbol(
        &self,
        ctx: &LintContext,
        symbol_id: SymbolId,
        shadowed_symbol_id: SymbolId,
    ) -> bool {
        let scoping = ctx.scoping();

        Self::is_function_name_initializer_exception(ctx, symbol_id, shadowed_symbol_id)
            || (self.ignore_on_initialization
                && Self::is_init_pattern_node(ctx, symbol_id, shadowed_symbol_id))
            || (self.hoist != HoistOption::All
                && self.is_in_tdz(ctx, symbol_id, shadowed_symbol_id))
            || self.should_ignore_shadow(
                ctx,
                symbol_id,
                scoping.symbol_flags(symbol_id),
                shadowed_symbol_id,
                scoping.symbol_flags(shadowed_symbol_id),
            )
            || Self::is_external_declaration_merging(ctx, symbol_id, shadowed_symbol_id)
    }

    fn is_builtin_global_shadow(
        &self,
        ctx: &LintContext,
        symbol_id: SymbolId,
        symbol_name: &str,
        symbol_flags: SymbolFlags,
    ) -> bool {
        self.builtin_globals
            && is_builtin_global_name(ctx, symbol_name)
            && !self.should_ignore_global_shadow(ctx, symbol_id, symbol_flags)
    }

    fn should_ignore_global_shadow(
        &self,
        ctx: &LintContext,
        symbol_id: SymbolId,
        symbol_flags: SymbolFlags,
    ) -> bool {
        (self.ignore_type_value_shadow && is_type_only(symbol_flags))
            || (self.ignore_function_type_parameter_name_value_shadow
                && Self::is_function_type_parameter_name_value_shadow(ctx, symbol_id))
            || Self::is_generic_of_static_method_shadow(ctx, symbol_id)
    }

    /// Check if we should ignore this shadowing based on TypeScript-specific rules.
    fn should_ignore_shadow(
        &self,
        ctx: &LintContext,
        symbol_id: SymbolId,
        symbol_flags: SymbolFlags,
        shadowed_symbol_id: SymbolId,
        shadowed_flags: SymbolFlags,
    ) -> bool {
        // Ignore when one side is type-only and the other side is value-only.
        if self.ignore_type_value_shadow
            && Self::is_type_value_shadow_pair(symbol_flags, shadowed_flags)
        {
            return true;
        }

        if self.ignore_function_type_parameter_name_value_shadow
            && Self::is_function_type_parameter_name_value_shadow(ctx, symbol_id)
        {
            return true;
        }

        if Self::is_generic_of_static_method_shadow(ctx, symbol_id) {
            return true;
        }

        // Value imports that are only used as types are allowed to be shadowed by values.
        Self::is_value_import_used_only_as_type(
            ctx,
            symbol_flags,
            shadowed_symbol_id,
            shadowed_flags,
        )
    }

    fn is_type_value_shadow_pair(symbol_flags: SymbolFlags, shadowed_flags: SymbolFlags) -> bool {
        let symbol_is_value = symbol_flags.can_be_referenced_by_value();
        let shadowed_is_value =
            !shadowed_flags.is_type_import() && shadowed_flags.can_be_referenced_by_value();
        symbol_is_value != shadowed_is_value
    }

    fn is_value_import_used_only_as_type(
        ctx: &LintContext,
        symbol_flags: SymbolFlags,
        shadowed_symbol_id: SymbolId,
        shadowed_flags: SymbolFlags,
    ) -> bool {
        if !shadowed_flags.contains(SymbolFlags::Import) || is_type_only(symbol_flags) {
            return false;
        }

        let mut references = ctx.scoping().get_resolved_references(shadowed_symbol_id).peekable();
        references.peek().is_some() && references.all(Reference::is_type)
    }

    fn is_function_type_parameter_name_value_shadow(
        ctx: &LintContext,
        symbol_id: SymbolId,
    ) -> bool {
        let declaration_id = ctx.scoping().symbol_declaration(symbol_id);
        ctx.nodes().ancestor_kinds(declaration_id).any(|ancestor_kind| {
            matches!(
                ancestor_kind,
                AstKind::TSCallSignatureDeclaration(_)
                    | AstKind::TSFunctionType(_)
                    | AstKind::TSMethodSignature(_)
                    | AstKind::TSConstructSignatureDeclaration(_)
                    | AstKind::TSConstructorType(_)
            ) || matches!(
                ancestor_kind,
                AstKind::Function(func)
                    if matches!(
                        func.r#type,
                        FunctionType::TSDeclareFunction
                            | FunctionType::TSEmptyBodyFunctionExpression
                    )
            )
        })
    }

    fn is_generic_of_static_method_shadow(ctx: &LintContext, symbol_id: SymbolId) -> bool {
        if !ctx.scoping().symbol_flags(symbol_id).contains(SymbolFlags::TypeParameter) {
            return false;
        }

        let declaration_id = ctx.scoping().symbol_declaration(symbol_id);
        let Some(type_parameter_decl_id) =
            ctx.nodes().ancestor_ids(declaration_id).find(|&ancestor_id| {
                matches!(ctx.nodes().kind(ancestor_id), AstKind::TSTypeParameterDeclaration(_))
            })
        else {
            return false;
        };

        let function_like_id = ctx.nodes().parent_id(type_parameter_decl_id);
        let method_like_id = ctx.nodes().parent_id(function_like_id);

        matches!(ctx.nodes().kind(method_like_id), AstKind::MethodDefinition(method) if method.r#static)
    }

    fn is_in_tdz(
        &self,
        ctx: &LintContext,
        symbol_id: SymbolId,
        shadowed_symbol_id: SymbolId,
    ) -> bool {
        let inner_span = ctx.scoping().symbol_span(symbol_id);
        let outer_span = ctx.scoping().symbol_span(shadowed_symbol_id);

        if inner_span.end >= outer_span.start {
            return false;
        }

        match self.hoist {
            HoistOption::All => false,
            HoistOption::Never => true,
            HoistOption::Functions => !Self::is_function_declaration(ctx, shadowed_symbol_id),
            HoistOption::Types => !Self::is_hoisted_type_declaration(ctx, shadowed_symbol_id),
            HoistOption::FunctionsAndTypes => {
                !Self::is_function_declaration(ctx, shadowed_symbol_id)
                    && !Self::is_hoisted_type_declaration(ctx, shadowed_symbol_id)
            }
        }
    }

    fn is_function_declaration(ctx: &LintContext, symbol_id: SymbolId) -> bool {
        let declaration_id =
            declaration_or_parent_id(ctx, ctx.scoping().symbol_declaration(symbol_id));
        matches!(
            ctx.nodes().kind(declaration_id),
            AstKind::Function(function) if function.is_function_declaration()
        )
    }

    fn is_hoisted_type_declaration(ctx: &LintContext, symbol_id: SymbolId) -> bool {
        let declaration_id =
            declaration_or_parent_id(ctx, ctx.scoping().symbol_declaration(symbol_id));
        matches!(
            ctx.nodes().kind(declaration_id),
            AstKind::TSInterfaceDeclaration(_) | AstKind::TSTypeAliasDeclaration(_)
        )
    }

    fn is_declare_in_definition_file(ctx: &LintContext, symbol_id: SymbolId) -> bool {
        if !is_definition_file(ctx.file_path()) {
            return false;
        }

        let declaration_id = ctx.scoping().symbol_declaration(symbol_id);
        for ancestor_kind in ctx.nodes().ancestor_kinds(declaration_id) {
            match ancestor_kind {
                AstKind::VariableDeclaration(declaration) if declaration.declare => return true,
                AstKind::Function(function)
                    if function.is_ts_declare_function() || function.declare =>
                {
                    return true;
                }
                AstKind::Class(class) if class.declare => return true,
                AstKind::TSEnumDeclaration(declaration) if declaration.declare => return true,
                AstKind::TSModuleDeclaration(declaration) if declaration.declare => return true,
                AstKind::TSInterfaceDeclaration(declaration) if declaration.declare => return true,
                AstKind::TSTypeAliasDeclaration(declaration) if declaration.declare => return true,
                AstKind::Program(_) => break,
                _ => {}
            }
        }

        false
    }

    fn is_in_global_augmentation(ctx: &LintContext, symbol_id: SymbolId) -> bool {
        let declaration_id = ctx.scoping().symbol_declaration(symbol_id);
        ctx.nodes().ancestor_kinds(declaration_id).any(|ancestor_kind| match ancestor_kind {
            AstKind::TSGlobalDeclaration(_) => true,
            AstKind::TSModuleDeclaration(module) => {
                module.declare
                    && matches!(
                        &module.id,
                        TSModuleDeclarationName::Identifier(identifier)
                            if identifier.name.as_str() == "global"
                    )
            }
            _ => false,
        })
    }

    fn function_expression_name_shadow_span(
        ctx: &LintContext,
        symbol_id: SymbolId,
        symbol_name: &str,
    ) -> Option<Span> {
        let symbol_declaration = ctx.scoping().symbol_declaration(symbol_id);
        let symbol_scope = ctx.scoping().symbol_scope_id(symbol_id);
        let scope_node_id = ctx.scoping().get_node_id(symbol_scope);

        if symbol_declaration == scope_node_id {
            return None;
        }

        match ctx.nodes().kind(scope_node_id) {
            AstKind::Function(function)
                if function.is_expression()
                    && function.id.as_ref().is_some_and(|id| id.name.as_str() == symbol_name) =>
            {
                function.id.as_ref().map(GetSpan::span)
            }
            AstKind::Class(class)
                if class.is_expression()
                    && class.id.as_ref().is_some_and(|id| id.name.as_str() == symbol_name) =>
            {
                class.id.as_ref().map(GetSpan::span)
            }
            _ => None,
        }
    }

    fn is_external_declaration_merging(
        ctx: &LintContext,
        symbol_id: SymbolId,
        shadowed_symbol_id: SymbolId,
    ) -> bool {
        if !Self::is_hoisted_type_declaration(ctx, symbol_id) {
            return false;
        }

        let shadowed_flags = ctx.scoping().symbol_flags(shadowed_symbol_id);
        if !shadowed_flags.is_type_import() {
            return false;
        }

        let shadowed_declaration_id = ctx.scoping().symbol_declaration(shadowed_symbol_id);
        let import_source = ctx.nodes().ancestor_kinds(shadowed_declaration_id).find_map(|kind| {
            if let AstKind::ImportDeclaration(declaration) = kind {
                Some(declaration.source.value.as_str())
            } else {
                None
            }
        });
        let Some(import_source) = import_source else {
            return false;
        };

        let declaration_id = ctx.scoping().symbol_declaration(symbol_id);
        let module_name = ctx.nodes().ancestor_kinds(declaration_id).find_map(|kind| {
            if let AstKind::TSModuleDeclaration(module_decl) = kind
                && module_decl.declare
            {
                match &module_decl.id {
                    TSModuleDeclarationName::StringLiteral(string_literal) => {
                        Some(string_literal.value.as_str())
                    }
                    TSModuleDeclarationName::Identifier(_) => None,
                }
            } else {
                None
            }
        });

        module_name.is_some_and(|module_name| module_name == import_source)
    }

    fn is_function_name_initializer_exception(
        ctx: &LintContext,
        inner_symbol_id: SymbolId,
        outer_symbol_id: SymbolId,
    ) -> bool {
        let inner_expression_id =
            declaration_or_parent_id(ctx, ctx.scoping().symbol_declaration(inner_symbol_id));
        if !matches!(
            ctx.nodes().kind(inner_expression_id),
            AstKind::Function(function) if function.is_expression()
        ) && !matches!(
            ctx.nodes().kind(inner_expression_id),
            AstKind::Class(class) if class.is_expression()
        ) {
            return false;
        }

        let outer_declaration_id = ctx.scoping().symbol_declaration(outer_symbol_id);
        let outer_symbol_span = ctx.scoping().symbol_span(outer_symbol_id);
        let is_outer_formal_parameter = Self::is_formal_parameter_symbol(ctx, outer_declaration_id);

        let Some(initializer) =
            Self::find_initializer_expression(ctx, outer_declaration_id, outer_symbol_span)
        else {
            return false;
        };

        let expression_span = ctx.nodes().kind(inner_expression_id).span();
        let initializer_span = initializer.span();
        if initializer_span.start > expression_span.start
            || expression_span.end > initializer_span.end
        {
            return false;
        }

        if is_outer_formal_parameter {
            let unwrapped_expression_id = Self::unwrap_expression(ctx, inner_expression_id);
            return initializer.address() == ctx.nodes().kind(unwrapped_expression_id).address();
        }

        let inner_scope_id = ctx.scoping().symbol_scope_id(inner_symbol_id);
        let outer_scope_id = ctx.scoping().symbol_scope_id(outer_symbol_id);
        ctx.scoping()
            .scope_parent_id(inner_scope_id)
            .is_some_and(|parent_scope_id| parent_scope_id == outer_scope_id)
    }

    fn is_init_pattern_node(
        ctx: &LintContext,
        symbol_id: SymbolId,
        shadowed_symbol_id: SymbolId,
    ) -> bool {
        let scoping = ctx.scoping();

        let variable_scope_id = scoping.symbol_scope_id(symbol_id);
        let variable_scope_node_id = scoping.get_node_id(variable_scope_id);
        let variable_scope_node_kind = ctx.nodes().kind(variable_scope_node_id);

        if !matches!(
            variable_scope_node_kind,
            AstKind::Function(function) if function.is_expression()
        ) && !matches!(variable_scope_node_kind, AstKind::ArrowFunctionExpression(_))
        {
            return false;
        }

        let Some(outer_scope_id) = scoping.scope_parent_id(variable_scope_id) else {
            return false;
        };

        if outer_scope_id != scoping.symbol_scope_id(shadowed_symbol_id) {
            return false;
        }

        let call_expression_end =
            ctx.nodes().ancestor_ids(variable_scope_node_id).find_map(|ancestor_id| {
                match ctx.nodes().kind(ancestor_id) {
                    AstKind::CallExpression(call_expression) => Some(call_expression.span.end),
                    _ => None,
                }
            });
        let Some(call_expression_end) = call_expression_end else {
            return false;
        };

        let outer_declaration_id = scoping.symbol_declaration(shadowed_symbol_id);
        let shadowed_symbol_span = scoping.symbol_span(shadowed_symbol_id);
        for node_id in std::iter::once(outer_declaration_id)
            .chain(ctx.nodes().ancestor_ids(outer_declaration_id))
        {
            match ctx.nodes().kind(node_id) {
                AstKind::VariableDeclarator(declarator) => {
                    if declarator
                        .init
                        .as_ref()
                        .is_some_and(|init| is_in_range(init.span(), call_expression_end))
                    {
                        return true;
                    }

                    if pattern_initializer_contains_location(
                        &declarator.id,
                        shadowed_symbol_span,
                        call_expression_end,
                    ) {
                        return true;
                    }

                    let variable_declaration_id = ctx.nodes().parent_id(node_id);
                    match ctx.nodes().parent_kind(variable_declaration_id) {
                        AstKind::ForInStatement(statement)
                            if is_in_range(statement.right.span(), call_expression_end) =>
                        {
                            return true;
                        }
                        AstKind::ForOfStatement(statement)
                            if is_in_range(statement.right.span(), call_expression_end) =>
                        {
                            return true;
                        }
                        _ => {}
                    }

                    break;
                }
                AstKind::FormalParameter(parameter) => {
                    if binding_pattern_contains_symbol(&parameter.pattern, shadowed_symbol_span)
                        && parameter
                            .initializer
                            .as_ref()
                            .is_some_and(|init| is_in_range(init.span(), call_expression_end))
                    {
                        return true;
                    }
                }
                AstKind::AssignmentPattern(pattern) => {
                    if binding_pattern_contains_symbol(&pattern.left, shadowed_symbol_span)
                        && is_in_range(pattern.right.span(), call_expression_end)
                    {
                        return true;
                    }
                }
                kind if is_initializer_sentinel(kind) => break,
                _ => {}
            }
        }

        false
    }

    fn find_initializer_expression<'a>(
        ctx: &LintContext<'a>,
        declaration_id: NodeId,
        symbol_span: Span,
    ) -> Option<&'a Expression<'a>> {
        for node_id in
            std::iter::once(declaration_id).chain(ctx.nodes().ancestor_ids(declaration_id))
        {
            match ctx.nodes().kind(node_id) {
                AstKind::FormalParameter(parameter) => {
                    if binding_pattern_contains_symbol(&parameter.pattern, symbol_span)
                        && let Some(initializer) = parameter.initializer.as_ref()
                    {
                        return Some(initializer);
                    }
                }
                AstKind::AssignmentPattern(pattern) => {
                    if binding_pattern_contains_symbol(&pattern.left, symbol_span) {
                        return Some(&pattern.right);
                    }
                }
                AstKind::VariableDeclarator(declarator) => {
                    if let Some(initializer) =
                        find_pattern_initializer_for_symbol(&declarator.id, symbol_span)
                    {
                        return Some(initializer);
                    }
                    return declarator.init.as_ref();
                }
                kind if is_initializer_sentinel(kind) => break,
                _ => {}
            }
        }
        None
    }

    fn is_formal_parameter_symbol(ctx: &LintContext, declaration_id: NodeId) -> bool {
        for node_id in
            std::iter::once(declaration_id).chain(ctx.nodes().ancestor_ids(declaration_id))
        {
            match ctx.nodes().kind(node_id) {
                AstKind::FormalParameter(_) => return true,
                kind if is_initializer_sentinel(kind) => break,
                _ => {}
            }
        }

        false
    }

    fn unwrap_expression(ctx: &LintContext, expression_id: NodeId) -> NodeId {
        let mut current_id = expression_id;

        loop {
            let parent_id = ctx.nodes().parent_id(current_id);
            match ctx.nodes().kind(parent_id) {
                AstKind::ParenthesizedExpression(_) | AstKind::LogicalExpression(_) => {
                    current_id = parent_id;
                }
                AstKind::ConditionalExpression(conditional_expression) => {
                    if conditional_expression.test.address()
                        == ctx.nodes().kind(current_id).address()
                    {
                        break;
                    }
                    current_id = parent_id;
                }
                _ => break,
            }
        }

        current_id
    }
}

/// Check if the symbol is a type-only declaration (not a value).
fn is_type_only(flags: SymbolFlags) -> bool {
    flags.can_be_referenced_by_type() && !flags.can_be_referenced_by_value()
}

fn is_builtin_global_name(ctx: &LintContext, name: &str) -> bool {
    GLOBALS.values().any(|globals| globals.contains_key(name)) || ctx.globals().is_enabled(name)
}

fn is_definition_file(path: &Path) -> bool {
    let path = path.to_string_lossy();
    path.ends_with(".d.ts") || path.ends_with(".d.cts") || path.ends_with(".d.mts")
}

fn is_in_range(span: Span, location: u32) -> bool {
    span.start <= location && location <= span.end
}

fn is_initializer_sentinel(kind: AstKind) -> bool {
    matches!(
        kind,
        AstKind::Function(_)
            | AstKind::Class(_)
            | AstKind::ArrowFunctionExpression(_)
            | AstKind::CatchClause(_)
            | AstKind::ImportDeclaration(_)
            | AstKind::ExportNamedDeclaration(_)
    )
}

fn declaration_or_parent_id(ctx: &LintContext, declaration_id: NodeId) -> NodeId {
    if matches!(ctx.nodes().kind(declaration_id), AstKind::BindingIdentifier(_)) {
        ctx.nodes().parent_id(declaration_id)
    } else {
        declaration_id
    }
}

fn binding_pattern_contains_symbol(pattern: &BindingPattern, symbol_span: Span) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => identifier.span == symbol_span,
        BindingPattern::AssignmentPattern(pattern) => {
            binding_pattern_contains_symbol(&pattern.left, symbol_span)
        }
        BindingPattern::ObjectPattern(pattern) => {
            pattern
                .properties
                .iter()
                .any(|property| binding_pattern_contains_symbol(&property.value, symbol_span))
                || pattern.rest.as_ref().is_some_and(|rest| {
                    binding_pattern_contains_symbol(&rest.argument, symbol_span)
                })
        }
        BindingPattern::ArrayPattern(pattern) => {
            pattern
                .elements
                .iter()
                .flatten()
                .any(|element| binding_pattern_contains_symbol(element, symbol_span))
                || pattern.rest.as_ref().is_some_and(|rest| {
                    binding_pattern_contains_symbol(&rest.argument, symbol_span)
                })
        }
    }
}

fn pattern_initializer_contains_location(
    pattern: &BindingPattern,
    symbol_span: Span,
    location: u32,
) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(_) => false,
        BindingPattern::AssignmentPattern(pattern) => {
            (binding_pattern_contains_symbol(&pattern.left, symbol_span)
                && is_in_range(pattern.right.span(), location))
                || pattern_initializer_contains_location(&pattern.left, symbol_span, location)
        }
        BindingPattern::ObjectPattern(pattern) => {
            pattern.properties.iter().any(|property| {
                pattern_initializer_contains_location(&property.value, symbol_span, location)
            }) || pattern.rest.as_ref().is_some_and(|rest| {
                pattern_initializer_contains_location(&rest.argument, symbol_span, location)
            })
        }
        BindingPattern::ArrayPattern(pattern) => {
            pattern.elements.iter().flatten().any(|element| {
                pattern_initializer_contains_location(element, symbol_span, location)
            }) || pattern.rest.as_ref().is_some_and(|rest| {
                pattern_initializer_contains_location(&rest.argument, symbol_span, location)
            })
        }
    }
}

fn find_pattern_initializer_for_symbol<'a>(
    pattern: &'a BindingPattern<'a>,
    symbol_span: Span,
) -> Option<&'a Expression<'a>> {
    match pattern {
        BindingPattern::BindingIdentifier(_) => None,
        BindingPattern::AssignmentPattern(pattern) => {
            if binding_pattern_contains_symbol(&pattern.left, symbol_span) {
                return Some(&pattern.right);
            }
            find_pattern_initializer_for_symbol(&pattern.left, symbol_span)
        }
        BindingPattern::ObjectPattern(pattern) => {
            for property in &pattern.properties {
                if let Some(initializer) =
                    find_pattern_initializer_for_symbol(&property.value, symbol_span)
                {
                    return Some(initializer);
                }
            }
            pattern
                .rest
                .as_ref()
                .and_then(|rest| find_pattern_initializer_for_symbol(&rest.argument, symbol_span))
        }
        BindingPattern::ArrayPattern(pattern) => {
            for element in pattern.elements.iter().flatten() {
                if let Some(initializer) = find_pattern_initializer_for_symbol(element, symbol_span)
                {
                    return Some(initializer);
                }
            }
            pattern
                .rest
                .as_ref()
                .and_then(|rest| find_pattern_initializer_for_symbol(&rest.argument, symbol_span))
        }
    }
}
