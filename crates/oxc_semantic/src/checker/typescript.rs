use std::{borrow::Cow, collections::hash_map::Entry};

use itertools::Itertools;
use rustc_hash::FxHashMap;

use oxc_ast::{AstKind, ast::*};
use oxc_ecmascript::BoundNames;
use oxc_span::{GetSpan, Span};
use oxc_str::{Ident, JSStr, Str};
use oxc_syntax::{
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};

use crate::{builder::SemanticBuilder, diagnostics};

pub fn check_ts_type_annotation(annotation: &TSTypeAnnotation<'_>, ctx: &SemanticBuilder<'_>) {
    let (modifier, is_start, span_with_illegal_modifier) = match &annotation.type_annotation {
        TSType::JSDocNonNullableType(ty) => ('!', !ty.postfix, ty.span()),
        TSType::JSDocNullableType(ty) => ('?', !ty.postfix, ty.span()),
        _ => {
            return;
        }
    };

    let valid_type_span = if is_start {
        span_with_illegal_modifier.shrink_left(1)
    } else {
        span_with_illegal_modifier.shrink_right(1)
    };

    let suggestion = &ctx.source_text[valid_type_span];
    let suggestion = if modifier == '?' {
        Cow::Owned(format!("{suggestion} | null | undefined"))
    } else {
        Cow::Borrowed(suggestion)
    };

    ctx.error(diagnostics::jsdoc_type_in_annotation(
        modifier,
        is_start,
        span_with_illegal_modifier,
        &suggestion,
    ));
}

pub fn check_ts_type_predicate(predicate: &TSTypePredicate<'_>, ctx: &SemanticBuilder<'_>) {
    if !is_allowed_type_predicate_position(ctx) {
        ctx.error(diagnostics::type_predicate_only_in_return_type(predicate.span));
    }
}

fn is_allowed_type_predicate_position(ctx: &SemanticBuilder<'_>) -> bool {
    let mut ancestors = ctx.ancestry().ancestor_kinds();

    let Some(AstKind::TSTypeAnnotation(_)) = ancestors.next() else {
        return false;
    };

    match ancestors.next() {
        Some(AstKind::Function(_)) => match ancestors.next() {
            Some(AstKind::MethodDefinition(method)) => method.kind.is_method(),
            Some(AstKind::ObjectProperty(property)) => !property.kind.is_accessor(),
            _ => true,
        },
        Some(
            AstKind::ArrowFunctionExpression(_)
            | AstKind::TSFunctionType(_)
            | AstKind::TSCallSignatureDeclaration(_),
        ) => true,
        Some(AstKind::TSMethodSignature(signature)) => {
            signature.kind == TSMethodSignatureKind::Method
        }
        _ => false,
    }
}

pub fn check_ts_infer_type<'a>(infer_type: &TSInferType<'a>, ctx: &SemanticBuilder<'a>) {
    let is_in_conditional_extends_clause = ctx.ancestry().ancestor_kinds().any(|kind| {
        kind.as_ts_conditional_type().is_some_and(|conditional| {
            conditional.extends_type.span().contains_inclusive(infer_type.span)
        })
    });

    if !is_in_conditional_extends_clause {
        ctx.error(diagnostics::infer_declaration_only_permitted_in_extends_clause(infer_type.span));
    }
}

pub fn check_formal_parameters(params: &FormalParameters, ctx: &SemanticBuilder<'_>) {
    if params.kind == FormalParameterKind::Signature && params.items.len() > 1 {
        check_duplicate_bound_names(params, ctx);
    }
}

fn check_duplicate_bound_names<'a, T: BoundNames<'a>>(bound_names: &T, ctx: &SemanticBuilder<'_>) {
    let mut idents: FxHashMap<Str<'a>, Span> = FxHashMap::default();
    bound_names.bound_names(&mut |ident| {
        if let Some(old_span) = idents.insert(ident.name.into(), ident.span) {
            ctx.error(diagnostics::redeclaration(&ident.name, old_span, ident.span));
        }
    });
}

pub fn check_ts_external_module_declaration<'a>(
    decl: &TSExternalModuleDeclaration<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    check_ambient_module_declaration(decl.span, ctx);
    if let Some(body) = &decl.body {
        check_ts_export_assignment_in_statements(&body.body, decl.scope_id.get().unwrap(), ctx);
    }
}

pub fn check_ts_namespace_declaration<'a>(
    decl: &TSNamespaceDeclaration<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    check_ts_module_or_global_declaration(decl.span, ctx);
    check_ts_export_assignment_in_namespace_decl(decl, ctx);
}

pub fn check_ts_global_declaration<'a>(decl: &TSGlobalDeclaration<'a>, ctx: &SemanticBuilder<'a>) {
    check_ts_module_or_global_declaration(decl.span, ctx);

    if !decl.declare && !ctx.in_ambient_context() {
        ctx.error(diagnostics::global_scope_augmentation_should_have_declare_modifier(
            decl.global_span,
        ));
    }
}

fn check_ts_module_or_global_declaration(span: Span, ctx: &SemanticBuilder<'_>) {
    // skip current node
    for kind in ctx.ancestry().ancestor_kinds() {
        match kind {
            AstKind::Program(_)
            | AstKind::TSModuleBlock(_)
            | AstKind::TSExternalModuleDeclaration(_)
            | AstKind::TSNamespaceDeclaration(_)
            | AstKind::TSGlobalDeclaration(_) => {
                break;
            }
            m if m.is_module_declaration() => {
                // We need to check the parent of the parent
            }
            _ => {
                ctx.error(diagnostics::not_allowed_namespace_declaration(span));
                break;
            }
        }
    }
}

fn check_ambient_module_declaration(span: Span, ctx: &SemanticBuilder<'_>) {
    let mut ancestors =
        ctx.ancestry().ancestor_kinds().filter(|kind| !kind.is_module_declaration());

    match ancestors.next() {
        Some(AstKind::Program(_)) => {}
        Some(AstKind::TSModuleBlock(_)) => match ancestors.next() {
            Some(AstKind::TSExternalModuleDeclaration(_))
                if ctx.source_type.is_script()
                    && matches!(ancestors.next(), Some(AstKind::Program(_))) =>
            {
                // Module augmentations may be nested directly in a top-level ambient module.
            }
            Some(
                AstKind::TSExternalModuleDeclaration(_)
                | AstKind::TSNamespaceDeclaration(_)
                | AstKind::TSGlobalDeclaration(_),
            ) => {
                ctx.error(diagnostics::nested_ambient_module_declaration(span));
            }
            _ => ctx.error(diagnostics::not_allowed_ambient_module_declaration(span)),
        },
        Some(
            AstKind::TSExternalModuleDeclaration(_)
            | AstKind::TSNamespaceDeclaration(_)
            | AstKind::TSGlobalDeclaration(_),
        ) => {
            ctx.error(diagnostics::nested_ambient_module_declaration(span));
        }
        _ => ctx.error(diagnostics::not_allowed_ambient_module_declaration(span)),
    }
}

pub fn check_ts_enum_declaration<'a>(decl: &TSEnumDeclaration<'a>, ctx: &SemanticBuilder<'a>) {
    let mut need_initializer = false;
    // Enum member symbols store UTF-8 names, so the binder declares no symbol for a name with a
    // lone surrogate and cannot report its redeclaration.
    let mut lone_surrogate_names = FxHashMap::<JSStr<'a>, Span>::default();

    decl.body.members.iter().for_each(|member| {
        let name = match &member.id {
            TSEnumMemberName::Identifier(_) => None,
            TSEnumMemberName::String(lit) | TSEnumMemberName::ComputedString(lit) => {
                Some(lit.value)
            }
            TSEnumMemberName::ComputedTemplateString(template) => template.single_quasi(),
        };
        if let Some(name) = name.filter(|name| name.as_str().is_none()) {
            let span = member.id.span();
            match lone_surrogate_names.entry(name) {
                Entry::Occupied(first) => ctx.error(diagnostics::redeclaration(
                    &name.display().to_string(),
                    *first.get(),
                    span,
                )),
                Entry::Vacant(entry) => {
                    entry.insert(span);
                }
            }
        }

        #[expect(clippy::unnested_or_patterns)]
        if let Some(initializer) = &member.initializer {
            need_initializer = !matches!(
                initializer.without_parentheses(),
                // A = 1
                Expression::NumericLiteral(_)
                    // B = A
                    | Expression::Identifier(_)
                    // C = E.D
                    | match_member_expression!(Expression)
                    // D = 1 + 2
                    | Expression::BinaryExpression(_)
                    // E = -1
                    | Expression::UnaryExpression(_)
            );
        } else if need_initializer {
            ctx.error(diagnostics::enum_member_must_have_initializer(member.span));
        }
    });
}

pub fn check_class<'a>(class: &Class<'a>, ctx: &SemanticBuilder<'a>) {
    if !class.r#abstract {
        for elem in &class.body.body {
            if elem.is_abstract() {
                let span = elem.property_key().map_or_else(|| elem.span(), GetSpan::span);
                ctx.error(diagnostics::abstract_elem_in_concrete_class(elem.is_property(), span));
            }
        }
    }

    if !ctx.in_ambient_context() {
        let mut is_in_overload_group = false;
        for (a, b) in class.body.body.iter().map(Some).chain(std::iter::once(None)).tuple_windows()
        {
            if let Some(ClassElement::MethodDefinition(a)) = a
                && !a.r#type.is_abstract()
                && !a.optional
                && a.value.r#type == FunctionType::TSEmptyBodyFunctionExpression
            {
                let next_is_same = b.is_some_and(|b| {
                    matches!(b,
                        ClassElement::MethodDefinition(b)
                            if b.key.static_name() == a.key.static_name()
                    )
                });
                if next_is_same {
                    is_in_overload_group = true;
                } else if a.key.static_name().is_some() || is_in_overload_group {
                    // Report error for:
                    // 1. Methods with static names that are not followed by an implementation
                    // 2. The last overload in a computed-name overload group (e.g. [Symbol.iterator])
                    if a.kind.is_constructor() {
                        ctx.error(diagnostics::constructor_implementation_missing(a.key.span()));
                    } else {
                        ctx.error(diagnostics::function_implementation_missing(a.key.span()));
                    }
                    is_in_overload_group = false;
                } else {
                    is_in_overload_group = false;
                }
            } else {
                is_in_overload_group = false;
            }
        }
    }
}

pub fn check_method_definition<'a>(method: &MethodDefinition<'a>, ctx: &SemanticBuilder<'a>) {
    let is_abstract = method.r#type.is_abstract();

    let is_empty_body = method.value.r#type == FunctionType::TSEmptyBodyFunctionExpression;
    // Illegal to have `constructor(public foo);`
    if method.kind.is_constructor() && is_empty_body {
        for param in &method.value.params.items {
            if param.has_modifier() {
                ctx.error(diagnostics::parameter_property_only_in_constructor_impl(param.span));
            }
        }
    }

    // Illegal to have `get foo();` or `set foo(a)`
    if method.kind.is_accessor() && is_empty_body && !is_abstract && !ctx.in_ambient_context() {
        ctx.error(diagnostics::accessor_without_body(method.key.span()));
    }
}

pub fn check_for_statement_left(left: &ForStatementLeft, is_for_in: bool, ctx: &SemanticBuilder) {
    let ForStatementLeft::VariableDeclaration(decls) = left else {
        return;
    };

    for decl in &decls.declarations {
        if decl.type_annotation.is_some() {
            let span = decl.id.span();
            ctx.error(diagnostics::type_annotation_in_for_left(span, is_for_in));
        }
    }
}

pub fn check_ts_export_assignment_in_program<'a>(program: &Program<'a>, ctx: &SemanticBuilder<'a>) {
    if !ctx.source_type.is_typescript() {
        return;
    }
    check_ts_export_assignment_in_statements(&program.body, program.scope_id.get().unwrap(), ctx);
}

fn check_ts_export_assignment_in_namespace_decl<'a>(
    namespace_decl: &TSNamespaceDeclaration<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    match &namespace_decl.body {
        TSNamespaceDeclarationBody::TSNamespaceDeclaration(nested) => {
            check_ts_export_assignment_in_namespace_decl(nested, ctx);
        }
        TSNamespaceDeclarationBody::TSModuleBlock(block) => {
            check_ts_export_assignment_in_statements(
                &block.body,
                namespace_decl.scope_id.get().unwrap(),
                ctx,
            );
        }
    }
}

fn check_ts_export_assignment_in_statements<'a>(
    statements: &[Statement<'a>],
    scope_id: ScopeId,
    ctx: &SemanticBuilder<'a>,
) {
    if !statements.iter().any(|stmt| matches!(stmt, Statement::TSExportAssignment(_))) {
        return;
    }
    let body = ctx
        .ancestry()
        .ancestor_kinds()
        .find_map(|kind| match kind {
            AstKind::Program(program) => Some(program.body.as_slice()),
            _ => None,
        })
        .unwrap_or(statements);
    let resolver = ExportAssignmentResolver::new(body, ctx);
    let mut export_assignments = vec![];
    let mut has_other_exports = false;
    let mut has_non_alias_exports = false;

    for stmt in statements {
        match stmt {
            Statement::TSExportAssignment(export_assignment) => {
                export_assignments.push(export_assignment);
            }
            Statement::ExportNamedDeclaration(export_decl) => {
                // An unmodified specifier may still name an exclusively type-space symbol.
                has_other_exports |= export_decl.export_kind.is_value()
                    && export_decl.specifiers.iter().any(|specifier| {
                        specifier.export_kind.is_value()
                            && match &specifier.local {
                                ModuleExportName::IdentifierReference(id) => ctx
                                    .scoping
                                    .find_binding(scope_id, id.name)
                                    .is_none_or(|symbol| resolver.symbol_flags(symbol).is_value()),
                                _ => true,
                            }
                    });
            }
            Statement::ExportFromDeclaration(export_decl) => {
                // Preserve the existing treatment of `export {} from "mod"`.
                has_other_exports |= export_decl.export_kind.is_value()
                    && export_decl
                        .specifiers
                        .iter()
                        .any(|specifier| specifier.export_kind.is_value());
            }
            Statement::ExportDeclaration(export_decl) => {
                // Aliases do not augment the namespace associated with `export =`.
                has_non_alias_exports |=
                    !matches!(export_decl.declaration, Declaration::TSImportEqualsDeclaration(_));
                has_other_exports |= match &export_decl.declaration {
                    Declaration::TSTypeAliasDeclaration(_)
                    | Declaration::TSInterfaceDeclaration(_) => false,
                    Declaration::TSNamespaceDeclaration(namespace) => {
                        resolver.symbol_flags(namespace.id.symbol_id.get().unwrap()).is_value()
                    }
                    Declaration::TSImportEqualsDeclaration(alias) => {
                        alias.import_kind.is_value()
                            && resolver.symbol_flags(alias.id.symbol_id.get().unwrap()).is_value()
                    }
                    _ => true,
                };
            }
            Statement::ExportAllDeclaration(export_decl) => {
                has_other_exports |= export_decl.export_kind.is_value();
            }
            Statement::ExportDefaultDeclaration(export_decl) => {
                has_non_alias_exports |=
                    !matches!(export_decl.declaration, ExportDefaultDeclarationKind::Identifier(_));
                has_other_exports |= match &export_decl.declaration {
                    ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => false,
                    ExportDefaultDeclarationKind::Identifier(id) => ctx
                        .scoping
                        .find_binding(scope_id, id.name)
                        .is_none_or(|symbol| resolver.symbol_flags(symbol).is_value()),
                    _ => true,
                };
            }
            _ => {}
        }
    }

    for assignment in export_assignments {
        if has_other_exports
            || (has_non_alias_exports
                && resolver.target_exports_types(&assignment.expression, scope_id))
        {
            ctx.error(diagnostics::ts_export_assignment_cannot_be_used_with_other_exports(
                assignment.span,
            ));
        }
    }
}

/// Resolve local namespaces and import-equals aliases without depending on the optional node store.
struct ExportAssignmentResolver<'s, 'ctx, 'a> {
    namespaces: Vec<(&'s TSNamespaceDeclaration<'a>, bool)>,
    aliases: Vec<&'s TSImportEqualsDeclaration<'a>>,
    ctx: &'ctx SemanticBuilder<'a>,
}

impl<'s, 'ctx, 'a> ExportAssignmentResolver<'s, 'ctx, 'a> {
    fn new(statements: &'s [Statement<'a>], ctx: &'ctx SemanticBuilder<'a>) -> Self {
        let mut resolver = Self { namespaces: vec![], aliases: vec![], ctx };
        resolver.collect(statements, ctx.source_type.is_typescript_definition());
        resolver
    }

    fn collect(&mut self, statements: &'s [Statement<'a>], ambient: bool) {
        for statement in statements {
            match statement {
                Statement::TSNamespaceDeclaration(namespace) => {
                    self.collect_namespace(namespace, ambient);
                }
                Statement::TSImportEqualsDeclaration(alias) => self.aliases.push(alias),
                Statement::ExportDeclaration(export) => match &export.declaration {
                    Declaration::TSNamespaceDeclaration(namespace) => {
                        self.collect_namespace(namespace, ambient);
                    }
                    Declaration::TSImportEqualsDeclaration(alias) => self.aliases.push(alias),
                    Declaration::TSExternalModuleDeclaration(module) => {
                        if let Some(body) = &module.body {
                            self.collect(&body.body, ambient || module.declare);
                        }
                    }
                    _ => {}
                },
                Statement::TSExternalModuleDeclaration(module) => {
                    if let Some(body) = &module.body {
                        self.collect(&body.body, ambient || module.declare);
                    }
                }
                Statement::TSGlobalDeclaration(global) => {
                    self.collect(&global.body.body, ambient || global.declare);
                }
                _ => {}
            }
        }
    }

    fn collect_namespace(&mut self, namespace: &'s TSNamespaceDeclaration<'a>, ambient: bool) {
        let ambient = ambient || namespace.declare;
        self.namespaces.push((namespace, ambient));
        match &namespace.body {
            TSNamespaceDeclarationBody::TSNamespaceDeclaration(nested) => {
                self.collect_namespace(nested, ambient);
            }
            TSNamespaceDeclarationBody::TSModuleBlock(block) => self.collect(&block.body, ambient),
        }
    }

    fn symbol_flags(&self, symbol: SymbolId) -> SymbolFlags {
        let flags = self.ctx.scoping.symbol_flags(symbol);
        if !flags.is_import() {
            return flags;
        }
        let targets = self.resolve_path(symbol, &[], &mut vec![]);
        if targets.is_empty() {
            // External imports are outside this syntax checker's local symbol resolution.
            self.ctx.scoping.symbol_flags(symbol)
        } else {
            targets.into_iter().fold(SymbolFlags::empty(), |flags, target| {
                flags | self.ctx.scoping.symbol_flags(target)
            })
        }
    }

    fn resolve_alias(&self, symbol: SymbolId, visited: &mut Vec<SymbolId>) -> Vec<SymbolId> {
        let Some(alias) =
            self.aliases.iter().find(|alias| alias.id.symbol_id.get() == Some(symbol))
        else {
            return vec![symbol];
        };
        if visited.contains(&symbol) {
            return vec![];
        }
        let mut members = vec![];
        let name = match &alias.module_reference {
            TSModuleReference::IdentifierReference(id) => id.name,
            TSModuleReference::QualifiedName(qualified) => {
                members.push(qualified.right.name);
                let mut left = &qualified.left;
                while let TSTypeName::QualifiedName(qualified) = left {
                    members.push(qualified.right.name);
                    left = &qualified.left;
                }
                let TSTypeName::IdentifierReference(id) = left else { return vec![] };
                id.name
            }
            TSModuleReference::ExternalModuleReference(_) => return vec![symbol],
        };
        let scope = self.ctx.scoping.symbol_scope_id(symbol);
        let Some(target) = self.ctx.scoping.find_binding(scope, name) else { return vec![] };
        visited.push(symbol);
        let targets = self.resolve_path(target, &members, visited);
        visited.pop();
        targets
    }

    /// Members are stored from the last segment to the first, matching expression traversal.
    fn resolve_path(
        &self,
        symbol: SymbolId,
        members: &[Ident<'a>],
        visited: &mut Vec<SymbolId>,
    ) -> Vec<SymbolId> {
        let roots = self.resolve_alias(symbol, visited);
        let Some((name, remaining)) = members.split_last() else { return roots };
        let mut targets = vec![];
        for root in roots {
            for (namespace, ambient) in &self.namespaces {
                if namespace.id.symbol_id.get() != Some(root) {
                    continue;
                }
                if let Some(member) = self.namespace_member(namespace, *name, *ambient) {
                    targets.extend(self.resolve_path(member, remaining, visited));
                }
            }
        }
        targets
    }

    fn namespace_member(
        &self,
        namespace: &TSNamespaceDeclaration<'a>,
        name: Ident<'a>,
        ambient: bool,
    ) -> Option<SymbolId> {
        let scope = namespace.scope_id.get()?;
        let TSNamespaceDeclarationBody::TSModuleBlock(block) = &namespace.body else {
            return self.ctx.scoping.get_binding(scope, name);
        };
        for statement in &block.body {
            match statement {
                Statement::ExportDeclaration(export) => {
                    if let Some(symbol) = Self::declaration_symbol(&export.declaration, name) {
                        return Some(symbol);
                    }
                }
                Statement::ExportNamedDeclaration(export) => {
                    for specifier in &export.specifiers {
                        if specifier.exported.name() == name.as_str()
                            && let ModuleExportName::IdentifierReference(id) = &specifier.local
                        {
                            return self.ctx.scoping.find_binding(scope, id.name);
                        }
                    }
                }
                match_declaration!(Statement) if ambient => {
                    let declaration = statement.to_declaration();
                    if !matches!(declaration, Declaration::TSImportEqualsDeclaration(_))
                        && let Some(symbol) = Self::declaration_symbol(declaration, name)
                    {
                        return Some(symbol);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn declaration_symbol(declaration: &Declaration<'a>, name: Ident<'a>) -> Option<SymbolId> {
        if let Declaration::VariableDeclaration(variable) = declaration {
            let mut symbol = None;
            variable.bound_names(&mut |id| {
                if id.name == name {
                    symbol = id.symbol_id.get();
                }
            });
            symbol
        } else {
            declaration.id().filter(|id| id.name == name).and_then(|id| id.symbol_id.get())
        }
    }

    fn target_exports_types(&self, expression: &Expression<'a>, scope: ScopeId) -> bool {
        // Only direct entity-name expressions bind the assignment as an alias.
        // Wrappers bind a property instead, so they do not shadow the target's namespace.
        let mut target = expression;
        let mut members = vec![];
        loop {
            let member = match target {
                Expression::StaticMemberExpression(member) => member,
                Expression::ChainExpression(chain) => match &chain.expression {
                    ChainElement::StaticMemberExpression(member) => member,
                    _ => return false,
                },
                _ => break,
            };
            members.push(member.property.name);
            target = &member.object;
        }
        let Expression::Identifier(id) = target else { return false };
        let Some(symbol) = self.ctx.scoping.find_binding(scope, id.name) else { return false };
        self.resolve_path(symbol, &members, &mut vec![]).iter().any(|target| {
            self.namespaces.iter().any(|(namespace, ambient)| {
                namespace.id.symbol_id.get() == Some(*target)
                    && self.namespace_exports_types(namespace, *ambient)
            })
        })
    }

    fn namespace_exports_types(
        &self,
        namespace: &TSNamespaceDeclaration<'a>,
        ambient: bool,
    ) -> bool {
        let TSNamespaceDeclarationBody::TSModuleBlock(block) = &namespace.body else {
            return true; // A dotted namespace exports its nested namespace.
        };
        block.body.iter().any(|statement| match statement {
            Statement::ExportDeclaration(export) => export.declaration.id().is_some_and(|id| {
                self.symbol_flags(id.symbol_id.get().unwrap())
                    .intersects(SymbolFlags::Type | SymbolFlags::Namespace)
            }),
            Statement::ExportNamedDeclaration(export) => {
                export.specifiers.iter().any(|specifier| match &specifier.local {
                    ModuleExportName::IdentifierReference(id) => self
                        .ctx
                        .scoping
                        .find_binding(namespace.scope_id.get().unwrap(), id.name)
                        .is_some_and(|symbol| {
                            self.symbol_flags(symbol)
                                .intersects(SymbolFlags::Type | SymbolFlags::Namespace)
                        }),
                    _ => false,
                })
            }
            match_declaration!(Statement) if ambient => {
                let declaration = statement.to_declaration();
                !matches!(declaration, Declaration::TSImportEqualsDeclaration(_))
                    && declaration.id().is_some_and(|id| {
                        self.symbol_flags(id.symbol_id.get().unwrap())
                            .intersects(SymbolFlags::Type | SymbolFlags::Namespace)
                    })
            }
            _ => false,
        })
    }
}
