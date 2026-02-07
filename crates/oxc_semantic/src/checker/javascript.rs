use memchr::memchr_iter;
use rustc_hash::FxHashMap;

use oxc_allocator::GetAddress;
use oxc_ast::{AstKind, ModuleDeclarationKind, ast::*};
use oxc_ecmascript::{BoundNames, IsSimpleParameterList, PropName};
use oxc_span::{GetSpan, ModuleKind, Span};
use oxc_syntax::{
    class::ClassId,
    number::NumberBase,
    operator::{AssignmentOperator, UnaryOperator},
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};

use crate::{IsGlobalReference, builder::SemanticBuilder, class::Element, diagnostics};

/// It is a Syntax Error if any element of the ExportedBindings of ModuleItemList
/// does not also occur in either the VarDeclaredNames of ModuleItemList, or the LexicallyDeclaredNames of ModuleItemList.
pub fn check_unresolved_exports(program: &Program<'_>, ctx: &SemanticBuilder<'_>) {
    if ctx.source_type.is_typescript() || !ctx.source_type.is_module() {
        return;
    }

    for stmt in &program.body {
        if let Statement::ExportNamedDeclaration(decl) = stmt {
            for specifier in &decl.specifiers {
                if let ModuleExportName::IdentifierReference(ident) = &specifier.local
                    && ident.is_global_reference(&ctx.scoping)
                {
                    ctx.errors
                        .borrow_mut()
                        .push(diagnostics::undefined_export(&ident.name, ident.span));
                }
            }
        }
    }
}

/// It is a Syntax Error if any element of the BoundNames of ImportDeclaration
/// also occurs in the VarDeclaredNames or LexicallyDeclaredNames of ModuleItemList.
/// <https://tc39.es/ecma262/#sec-imports-static-semantics-early-errors>
pub fn check_import_value_redeclarations(ctx: &SemanticBuilder<'_>) {
    if !ctx.source_type.is_module() || ctx.source_type.is_typescript() {
        return;
    }

    let scope_id = ctx.scoping.root_scope_id();
    for (_, &symbol_id) in ctx.scoping.get_bindings(scope_id) {
        let flags = ctx.scoping.symbol_flags(symbol_id);
        if !flags.contains(SymbolFlags::Import) {
            continue;
        }
        if !flags.intersects(SymbolFlags::Variable | SymbolFlags::Class | SymbolFlags::Function) {
            continue;
        }
        let redeclarations = ctx.scoping.symbol_redeclarations(symbol_id);
        if redeclarations.len() < 2 {
            continue;
        }
        let name = ctx.scoping.symbol_name(symbol_id);
        let first = &redeclarations[0];
        let last = &redeclarations[redeclarations.len() - 1];
        ctx.error(diagnostics::redeclaration(name, first.span, last.span));
    }
}

pub fn check_duplicate_class_elements(ctx: &SemanticBuilder<'_>) {
    let classes = &ctx.class_table_builder.classes;
    classes.iter_enumerated().for_each(|(class_id, _)| {
        let mut defined_elements = FxHashMap::default();
        let elements = &classes.elements[class_id];
        for (element_id, element) in elements.iter_enumerated() {
            if let Some(prev_element_id) = defined_elements.insert(&element.name, element_id) {
                let prev_element = &elements[prev_element_id];

                let mut is_duplicate = element.is_private == prev_element.is_private
                    && if element.kind.is_setter_or_getter()
                        && prev_element.kind.is_setter_or_getter()
                    {
                        element.kind == prev_element.kind
                            || element.r#static != prev_element.r#static
                    } else {
                        true
                    };

                // * It is a Syntax Error if PrivateBoundIdentifiers of ClassElementList contains any duplicate entries,
                // unless the name is used once for a getter and once for a setter and in no other entries,
                // and the getter and setter are either both static or both non-static.
                // For TypeScript, public elements with different static status are allowed (overloads, etc.),
                // but private identifiers follow the same rules as JavaScript - static and instance
                // elements cannot share the same private name.
                is_duplicate = if element.is_private {
                    is_duplicate
                } else if ctx.source_type.is_typescript() {
                    element.r#static == prev_element.r#static && is_duplicate
                } else {
                    false
                };

                if is_duplicate {
                    #[cold]
                    fn report_duplicate_class_element(
                        element: &Element,
                        prev_element: &Element,
                        ctx: &SemanticBuilder<'_>,
                    ) {
                        if element.is_private
                            && element.r#static != prev_element.r#static
                            && ctx.source_type.is_typescript()
                        {
                            ctx.error(diagnostics::static_and_instance_private_identifier(
                                &element.name,
                                prev_element.span,
                                element.span,
                            ));
                        } else {
                            let span = prev_element.span;
                            ctx.error(diagnostics::redeclaration(
                                // `span` includes `#` for private identifiers
                                span.source_text(ctx.source_text),
                                span,
                                element.span,
                            ));
                        }
                    }

                    report_duplicate_class_element(element, prev_element, ctx);
                }
            }
        }
    });
}

pub fn check_identifier(
    name: &str,
    span: Span,
    symbol_id: Option<SymbolId>,
    ctx: &SemanticBuilder<'_>,
) {
    // reserved keywords are allowed in ambient contexts
    if ctx.source_type.is_typescript_definition() || is_current_node_ambient_binding(symbol_id, ctx)
    {
        return;
    }

    match name {
        "await" => {
            // It is a Syntax Error if the goal symbol of the syntactic grammar is Module and the StringValue of IdentifierName is "await".
            if ctx.source_type.is_module() {
                ctx.error(diagnostics::reserved_keyword(name, span));
            }
            // It is a Syntax Error if ClassStaticBlockStatementList Contains await is true.
            else if ctx.scoping.scope_flags(ctx.current_scope_id).is_class_static_block() {
                ctx.error(diagnostics::class_static_block_await(span));
            }
        }
        // TODO: Revisit this match arm when we add `Ident` and pre-hash the identifier names and see if a HashSet
        // becomes better for performance again.
        "implements" | "interface" | "let" | "package" | "private" | "protected" | "public"
        | "static" | "yield"
            if ctx.strict_mode() =>
        {
            // It is a Syntax Error if this phrase is contained in strict mode code and the StringValue of IdentifierName is: "implements", "interface", "let", "package", "private", "protected", "public", "static", or "yield".
            ctx.error(diagnostics::reserved_keyword(name, span));
        }
        _ => {}
    }
}

fn is_current_node_ambient_binding(symbol_id: Option<SymbolId>, ctx: &SemanticBuilder<'_>) -> bool {
    if ctx.current_scope_flags().is_ts_module_block() {
        return true;
    }

    if let Some(symbol_id) = symbol_id
        && ctx.scoping.symbol_flags(symbol_id).contains(SymbolFlags::Ambient)
    {
        true
    } else if let AstKind::BindingIdentifier(id) = ctx.nodes.kind(ctx.current_node_id)
        && let Some(symbol_id) = id.symbol_id.get()
    {
        ctx.scoping.symbol_flags(symbol_id).contains(SymbolFlags::Ambient)
    } else {
        false
    }
}

pub fn check_binding_identifier(ident: &BindingIdentifier, ctx: &SemanticBuilder<'_>) {
    // `.d.ts` files are allowed to use `eval` and `arguments` as binding identifiers
    if ctx.source_type.is_typescript_definition() {
        return;
    }

    if ctx.strict_mode() {
        // In strict mode, `eval` and `arguments` are banned as identifiers.
        if matches!(ident.name.as_str(), "eval" | "arguments") {
            // `eval` and `arguments` are allowed as the names of declare functions as well as their arguments.
            //
            // declare function eval(): void; // OK
            // declare function arguments(): void; // OK
            // declare function f(eval: number, arguments: number): number; // OK
            // declare function f(...eval): number; // OK
            // declare function f(...arguments): number; // OK
            // type K = (arguments: any[]) => void; // OK
            // interface Foo { bar(arguments: any[]): void; baz(...arguments: any[]): void; } // OK
            // declare function g({eval, arguments}: {eval: number, arguments: number}): number; // Error
            // declare function h([eval, arguments]: [number, number]): number; // Error
            let is_declare_function = |kind: &AstKind| {
                kind.as_function()
                    .is_some_and(|func| matches!(func.r#type, FunctionType::TSDeclareFunction))
            };

            let parent = ctx.nodes.parent_node(ctx.current_node_id);
            let is_ok = match parent.kind() {
                AstKind::Function(func) => matches!(func.r#type, FunctionType::TSDeclareFunction),
                AstKind::FormalParameter(_) | AstKind::FormalParameterRest(_) => {
                    is_declare_function(&ctx.nodes.parent_kind(parent.id()))
                }
                AstKind::BindingRestElement(_) => {
                    let grand_parent = ctx.nodes.parent_node(parent.id());
                    is_declare_function(&ctx.nodes.parent_kind(grand_parent.id()))
                }
                _ => false,
            };

            if !is_ok {
                ctx.error(diagnostics::unexpected_identifier_assign(&ident.name, ident.span));
            }
        }
    } else {
        // LexicalDeclaration : LetOrConst BindingList ;
        // * It is a Syntax Error if the BoundNames of BindingList contains "let".
        if ident.name == "let" {
            for node_kind in ctx.nodes.ancestor_kinds(ctx.current_node_id) {
                match node_kind {
                    AstKind::VariableDeclarator(decl) => {
                        if decl.kind.is_lexical() {
                            ctx.error(diagnostics::invalid_let_declaration(
                                decl.kind.as_str(),
                                ident.span,
                            ));
                        }
                        break;
                    }
                    AstKind::Function(_) => break,
                    _ => {}
                }
            }
        }
    }
}

pub fn check_identifier_reference(ident: &IdentifierReference, ctx: &SemanticBuilder<'_>) {
    // `.d.ts` files are allowed to use `eval` and `arguments` as identifier references
    if ctx.source_type.is_typescript_definition() {
        return;
    }

    //  Static Semantics: AssignmentTargetType
    //  1. If this IdentifierReference is contained in strict mode code and StringValue of Identifier is "eval" or "arguments", return invalid.
    if ctx.strict_mode() && matches!(ident.name.as_str(), "arguments" | "eval") {
        for node_kind in ctx.nodes.ancestor_kinds(ctx.current_node_id) {
            match node_kind {
                // Only check for actual assignment contexts, not member expression access
                AstKind::ObjectAssignmentTarget(_)
                | AstKind::AssignmentTargetPropertyIdentifier(_)
                | AstKind::UpdateExpression(_)
                | AstKind::ArrayAssignmentTarget(_) => {
                    return ctx
                        .error(diagnostics::unexpected_identifier_assign(&ident.name, ident.span));
                }
                AstKind::AssignmentExpression(assign_expr) => {
                    // only throw error if arguments or eval are being assigned to
                    if let AssignmentTarget::AssignmentTargetIdentifier(target_ident) =
                        &assign_expr.left
                        && target_ident.name == ident.name
                    {
                        return ctx.error(diagnostics::unexpected_identifier_assign(
                            &ident.name,
                            ident.span,
                        ));
                    }
                }
                m if m.is_member_expression_kind() => {
                    break;
                }
                _ => {}
            }
        }
    }

    // FieldDefinition : ClassElementName Initializeropt
    //   It is a Syntax Error if Initializer is present and ContainsArguments of Initializer is true.
    // ClassStaticBlockBody : ClassStaticBlockStatementList
    //   It is a Syntax Error if ContainsArguments of ClassStaticBlockStatementList is true.

    if ident.name == "arguments" {
        let mut previous_node_address = ctx.nodes.get_node(ctx.current_node_id).address();
        for node_kind in ctx.nodes.ancestor_kinds(ctx.current_node_id) {
            match node_kind {
                AstKind::Function(_) => break,
                AstKind::PropertyDefinition(prop) => {
                    if prop
                        .value
                        .as_ref()
                        .is_some_and(|value| value.address() == previous_node_address)
                    {
                        return ctx.error(diagnostics::unexpected_arguments(
                            "class field initializer",
                            ident.span,
                        ));
                    }
                }
                AstKind::StaticBlock(_) => {
                    return ctx.error(diagnostics::unexpected_arguments(
                        "static initialization block",
                        ident.span,
                    ));
                }
                _ => {}
            }
            previous_node_address = node_kind.address();
        }
    }
}

pub fn check_private_identifier_outside_class(
    ident: &PrivateIdentifier,
    ctx: &SemanticBuilder<'_>,
) {
    if ctx.class_table_builder.current_class_id.is_none() {
        ctx.error(diagnostics::private_not_in_class(&ident.name, ident.span));
    }
}

fn check_private_identifier(ctx: &SemanticBuilder<'_>) {
    if let Some(class_id) = ctx.class_table_builder.current_class_id {
        for reference in ctx.class_table_builder.classes.iter_private_identifiers(class_id) {
            if !ctx.class_table_builder.classes.ancestors(class_id).any(|class_id| {
                ctx.class_table_builder.classes.has_private_definition(class_id, reference.name)
            }) {
                ctx.error(diagnostics::private_field_undeclared(&reference.name, reference.span));
            }
        }
    }
}

pub fn check_number_literal(lit: &NumericLiteral, ctx: &SemanticBuilder<'_>) {
    // NumericLiteral :: legacy_octalIntegerLiteral
    // DecimalIntegerLiteral :: NonOctalDecimalIntegerLiteral
    // * It is a Syntax Error if the source text matched by this production is strict mode code.
    fn leading_zero(s: Option<Atom>) -> bool {
        if let Some(s) = s {
            let mut chars = s.bytes();
            if let Some(first) = chars.next()
                && let Some(second) = chars.next()
            {
                return first == b'0' && second.is_ascii_digit();
            }
        }
        false
    }

    if ctx.strict_mode() {
        match lit.base {
            NumberBase::Octal if leading_zero(lit.raw) => {
                ctx.error(diagnostics::legacy_octal(lit.span));
            }
            NumberBase::Decimal | NumberBase::Float if leading_zero(lit.raw) => {
                ctx.error(diagnostics::leading_zero_decimal(lit.span));
            }
            _ => {}
        }
    }
}

const MIN_STRING_SIZE_FOR_BATCH_CHECK: usize = 16;

pub fn check_string_literal(lit: &StringLiteral, ctx: &SemanticBuilder<'_>) {
    // 12.9.4.1 Static Semantics: Early Errors
    // EscapeSequence ::
    //   legacy_octalEscapeSequence
    //   non_octal_decimal_escape_sequence
    // It is a Syntax Error if the source text matched by this production is strict mode code.
    if !ctx.strict_mode() {
        return;
    }
    let raw = lit.span.source_text(ctx.source_text);
    let raw_len = raw.len();
    if raw_len != lit.value.len() {
        if raw_len >= MIN_STRING_SIZE_FOR_BATCH_CHECK {
            let raw_bytes = raw.as_bytes();
            // Exclude the last byte (closing quote) from the search haystack.
            // This ensures any backslash found has at least one byte following it.
            let haystack = &raw_bytes[..raw_len - 1];
            let mut skip_next_backslash = false;
            for backslash_index in memchr_iter(b'\\', haystack) {
                if skip_next_backslash {
                    skip_next_backslash = false;
                    continue;
                }
                debug_assert!(
                    backslash_index + 1 < raw_bytes.len(),
                    "backslash at index {} has no following byte in string of length {}",
                    backslash_index,
                    raw_bytes.len()
                );
                // SAFETY: We search `haystack` which excludes the last byte, so any backslash
                // found is at index < raw_len - 1, meaning backslash_index + 1 < raw_len.
                let next_byte = unsafe { *raw_bytes.get_unchecked(backslash_index + 1) };
                match next_byte {
                    b'\\' => {
                        // Escaped backslash - skip the next backslash in memchr results
                        skip_next_backslash = true;
                    }
                    b'0' => {
                        let following_byte = raw_bytes.get(backslash_index + 2);
                        if following_byte.is_some_and(u8::is_ascii_digit) {
                            return ctx.error(diagnostics::legacy_octal(lit.span));
                        }
                    }
                    b'1'..=b'7' => {
                        return ctx.error(diagnostics::legacy_octal(lit.span));
                    }
                    b'8'..=b'9' => {
                        return ctx.error(diagnostics::non_octal_decimal_escape_sequence(lit.span));
                    }
                    _ => {}
                }
            }
        } else {
            let mut bytes = raw.bytes().peekable();
            while let Some(b) = bytes.next() {
                if b == b'\\' {
                    match bytes.next() {
                        Some(b'0') => {
                            if bytes.peek().is_some_and(u8::is_ascii_digit) {
                                return ctx.error(diagnostics::legacy_octal(lit.span));
                            }
                        }
                        Some(b'1'..=b'7') => {
                            return ctx.error(diagnostics::legacy_octal(lit.span));
                        }
                        Some(b'8'..=b'9') => {
                            return ctx
                                .error(diagnostics::non_octal_decimal_escape_sequence(lit.span));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

// It is a Syntax Error if FunctionBodyContainsUseStrict of AsyncFunctionBody is true and IsSimpleParameterList of FormalParameters is false.
// background: https://humanwhocodes.com/blog/2016/10/the-ecmascript-2016-change-you-probably-dont-know/
pub fn check_directive(directive: &Directive, ctx: &SemanticBuilder<'_>) {
    if directive.directive != "use strict" {
        return;
    }

    if !ctx.current_scope_flags().is_function() {
        return;
    }

    if matches!(ctx.nodes.kind(ctx.scoping.get_node_id(ctx.current_scope_id)),
        AstKind::Function(Function { params, .. })
        | AstKind::ArrowFunctionExpression(ArrowFunctionExpression { params, .. })
        if !params.is_simple_parameter_list())
    {
        ctx.error(diagnostics::illegal_use_strict(directive.span));
    }
}

pub fn check_module_declaration(decl: &ModuleDeclarationKind, ctx: &SemanticBuilder<'_>) {
    // It is ambiguous between script and module for `TypeScript`, skipping this check for now.
    // Basically we need to "upgrade" from script to module if we see any module syntax inside the
    // semantic builder
    if ctx.source_type.is_typescript() {
        return;
    }

    let text = match decl {
        ModuleDeclarationKind::Import(_) => "import statement",
        ModuleDeclarationKind::ExportAll(_)
        | ModuleDeclarationKind::ExportDefault(_)
        | ModuleDeclarationKind::ExportNamed(_)
        | ModuleDeclarationKind::TSExportAssignment(_)
        | ModuleDeclarationKind::TSNamespaceExport(_) => "export statement",
    };
    let start = decl.span().start;
    let span = Span::sized(start, 6);
    match ctx.source_type.module_kind() {
        ModuleKind::Unambiguous => {
            #[cfg(debug_assertions)]
            panic!("Technically unreachable, omit to avoid panic.");
        }
        // CommonJS uses require/module.exports, not import/export statements
        ModuleKind::Script | ModuleKind::CommonJS => {
            ctx.error(diagnostics::module_code(text, span));
        }
        ModuleKind::Module => {
            if matches!(ctx.nodes.parent_kind(ctx.current_node_id), AstKind::Program(_)) {
                return;
            }
            ctx.error(diagnostics::top_level(text, span));
        }
    }
}

/// Check that `using` declarations are not at the top level in script mode.
/// `using` is allowed:
/// - At the top level of ES modules
/// - At the top level of CommonJS modules (wrapped in function scope)
/// - Inside any block scope in scripts
///
/// But NOT at the top level of scripts.
pub fn check_variable_declaration(decl: &VariableDeclaration, ctx: &SemanticBuilder<'_>) {
    if decl.kind.is_using()
        && ctx.source_type.is_script()
        && ctx.current_scope_flags().contains(ScopeFlags::Top)
    {
        ctx.error(diagnostics::using_declaration_not_allowed_in_script(decl.span));
    }
}

pub fn check_meta_property(prop: &MetaProperty, ctx: &SemanticBuilder<'_>) {
    match prop.meta.name.as_str() {
        "import" => {
            // import.meta is only allowed in ES modules, not in scripts or CommonJS
            if prop.property.name == "meta" && !ctx.source_type.is_module() {
                ctx.error(diagnostics::import_meta(prop.span));
            }
        }
        "new" => {
            if prop.property.name == "target" {
                // In CommonJS, the file is wrapped in a function, so new.target is always valid
                if ctx.source_type.is_commonjs() {
                    return;
                }

                // Check if we're in a valid context for new.target:
                // 1. Inside a function (including constructor)
                // 2. Inside a class static block
                // 3. Inside a class field initializer (new.target evaluates to undefined)
                //
                // Arrow functions inherit new.target from their surrounding scope,
                // so we skip them and continue checking the enclosing context.

                let mut in_valid_context = false;

                // First, check AST ancestors for class field initializers.
                // We need to do this because class fields don't have their own scope.
                for node_kind in ctx.nodes.ancestor_kinds(ctx.current_node_id) {
                    match node_kind {
                        // Regular functions have their own new.target binding.
                        // Use scope-based check from here.
                        AstKind::Function(_) => break,
                        // Class field initializers allow new.target (evaluates to undefined).
                        // This includes arrow functions nested inside the initializer.
                        AstKind::PropertyDefinition(_) | AstKind::AccessorProperty(_) => {
                            in_valid_context = true;
                            break;
                        }
                        _ => {}
                    }
                }

                // If not in a class field, fall back to scope-based check
                if !in_valid_context {
                    for scope_id in ctx.scoping.scope_ancestors(ctx.current_scope_id) {
                        let flags = ctx.scoping.scope_flags(scope_id);
                        // In arrow functions, new.target is inherited from the surrounding scope.
                        if flags.contains(ScopeFlags::Arrow) {
                            continue;
                        }
                        if flags.intersects(ScopeFlags::Function | ScopeFlags::ClassStaticBlock) {
                            in_valid_context = true;
                            break;
                        }
                    }
                }

                if !in_valid_context {
                    ctx.error(diagnostics::new_target(prop.span));
                }
            }
        }
        _ => {}
    }
}

pub fn check_function_declaration<'a>(
    stmt: &Statement<'a>,
    is_if_stmt_or_labeled_stmt: bool,
    ctx: &SemanticBuilder<'a>,
) {
    // Function declaration not allowed in statement position
    if let Statement::FunctionDeclaration(decl) = stmt {
        if ctx.strict_mode() {
            ctx.error(diagnostics::function_declaration_strict(decl.span));
        } else if !is_if_stmt_or_labeled_stmt {
            ctx.error(diagnostics::function_declaration_non_strict(decl.span));
        }
    }
}

// It is a Syntax Error if IsLabelledFunction(Statement) is true.
pub fn check_function_declaration_in_labeled_statement<'a>(
    body: &Statement<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    if let Statement::FunctionDeclaration(decl) = body {
        if ctx.strict_mode() {
            ctx.error(diagnostics::function_declaration_strict(decl.span));
        } else {
            // skip(1) for `LabeledStatement`
            for kind in ctx.nodes.ancestor_kinds(ctx.current_node_id) {
                match kind {
                    // Nested labeled statement
                    AstKind::LabeledStatement(_) => {}
                    AstKind::ForOfStatement(_)
                    | AstKind::ForInStatement(_)
                    | AstKind::ForStatement(_)
                    | AstKind::WhileStatement(_)
                    | AstKind::DoWhileStatement(_)
                    | AstKind::WithStatement(_)
                    | AstKind::IfStatement(_) => break,
                    _ => return,
                }
            }
            ctx.error(diagnostics::function_declaration_non_strict(decl.span));
        }
    }
}

// It is a Syntax Error if any element of the LexicallyDeclaredNames of
// StatementList also occurs in the VarDeclaredNames of StatementList.
pub fn check_variable_declarator_redeclaration(
    decl: &VariableDeclarator,
    ctx: &SemanticBuilder<'_>,
) {
    if decl.kind != VariableDeclarationKind::Var {
        return;
    }

    let scope_flags = ctx.current_scope_flags();
    // `function a() {}; var a;` and `function b() { function a() {}; var a; }` are valid
    // in script mode, but in module mode at the top level, function declarations are
    // lexically scoped, so `function a() {}; var a;` is a redeclaration error.
    if scope_flags.intersects(ScopeFlags::Top | ScopeFlags::Function)
        && !(ctx.source_type.is_module() && scope_flags.is_top())
    {
        return;
    }

    decl.id.bound_names(&mut |ident| {
        let redeclarations = ctx.scoping.symbol_redeclarations(ident.symbol_id());
        let Some(rd) = redeclarations.iter().nth_back(1) else { return };

        // `{ function f() {}; var f; }` is invalid in both strict and non-strict mode.
        // In module mode at the top level, `function f() {}; var f;` is also invalid.
        if rd.flags.is_function() {
            ctx.error(diagnostics::redeclaration(&ident.name, rd.span, decl.span));
        }
    });
}

/// Check for Annex B `if (foo) function a() {} else function b() {}`
pub fn is_function_decl_part_of_if_statement(
    function: &Function,
    builder: &SemanticBuilder,
) -> bool {
    debug_assert!(function.is_declaration());

    // Function declarations cannot be `consequent` or `alternate` of an `IfStatement` in strict mode.
    // This check is redundant - parent kind lookup below will always return `false` in strict mode.
    // But this check is cheaper, and strict mode code is more common than sloppy mode, so we do this cheap check first.
    if builder.current_scope_flags().is_strict_mode() {
        return false;
    }

    // A function declaration whose parent is an `IfStatement` can only be
    // either that `IfStatement`'s `consequent` or `alternate`
    // (can't be `test` because that's an expression)
    matches!(builder.nodes.parent_kind(builder.current_node_id), AstKind::IfStatement(_))
}

// It is a Syntax Error if the LexicallyDeclaredNames of StatementList contains any duplicate entries,
// unless the source text matched by this production is not strict mode code
// and the duplicate entries are only bound by FunctionDeclarations.
// https://tc39.es/ecma262/#sec-block-level-function-declarations-web-legacy-compatibility-semantics
pub fn check_function_redeclaration(func: &Function, ctx: &SemanticBuilder<'_>) {
    if !func.is_declaration() {
        return;
    }

    // Function declarations always have an identifier, except for `export default function () {}`.
    // Skip that case.
    let Some(id) = &func.id else { return };

    if is_function_decl_part_of_if_statement(func, ctx) {
        return;
    }

    let symbol_id = id.symbol_id();

    let redeclarations = ctx.scoping.symbol_redeclarations(symbol_id);
    let Some(prev) = redeclarations.iter().nth_back(1) else {
        // No redeclarations
        return;
    };

    // Already checked in `check_redeclaration`, because it is also not allowed in TypeScript.
    // `let a; function a() {}` is invalid in both strict and non-strict mode.
    if prev.flags.contains(SymbolFlags::BlockScopedVariable) {
        return;
    }

    let current_scope_flags = ctx.current_scope_flags();
    if prev.flags.intersects(SymbolFlags::FunctionScopedVariable | SymbolFlags::Function)
        && (current_scope_flags.is_function()
            || current_scope_flags.is_class_static_block()
            || (!ctx.source_type.is_module() && current_scope_flags.is_top()))
    {
        // https://tc39.github.io/ecma262/#sec-scripts-static-semantics-lexicallydeclarednames
        // `function a() {}; function a() {}` and `var a; function a() {}` are
        // still valid in script code, and should not be valid for module code.
        //
        // `function a() { var b; function b() { } }` valid in any mode.
        return;
    } else if !(current_scope_flags.is_strict_mode() || func.r#async || func.generator) {
        // `class a {}; function a() {}` and `async function a() {} function a () {}` are
        // invalid in both strict and non-strict mode.
        let prev_function = ctx.nodes.kind(prev.declaration).as_function();
        if prev_function.is_some_and(|func| !(func.r#async || func.generator)) {
            return;
        }
    }

    ctx.error(diagnostics::redeclaration(&id.name, prev.span, id.span));
}

pub fn check_class_redeclaration(class: &Class, ctx: &SemanticBuilder<'_>) {
    let Some(id) = &class.id else { return };
    let symbol_id = id.symbol_id();

    let redeclarations = ctx.scoping.symbol_redeclarations(symbol_id);
    let Some(prev) = redeclarations.iter().nth_back(1) else {
        // No redeclarations
        return;
    };

    if prev.flags.contains(SymbolFlags::Function) {
        ctx.error(diagnostics::redeclaration(&id.name, prev.span, id.span));
    }
}

pub fn check_with_statement(stmt: &WithStatement, ctx: &SemanticBuilder<'_>) {
    if ctx.strict_mode() || ctx.source_type.is_typescript() {
        ctx.error(diagnostics::with_statement(Span::sized(stmt.span.start, 4)));
    }
}

pub fn check_switch_statement<'a>(stmt: &SwitchStatement<'a>, ctx: &SemanticBuilder<'a>) {
    let mut previous_default: Option<Span> = None;
    for case in &stmt.cases {
        if case.test.is_none() {
            if let Some(previous_span) = previous_default {
                ctx.error(diagnostics::switch_stmt_cannot_have_multiple_default_case(
                    previous_span,
                    case.span,
                ));
                break;
            }
            previous_default.replace(case.span);
        }
    }
}

pub fn check_break_statement(stmt: &BreakStatement, ctx: &SemanticBuilder<'_>) {
    // It is a Syntax Error if this BreakStatement is not nested, directly or indirectly (but not crossing function or static initialization block boundaries), within an IterationStatement or a SwitchStatement.
    for node_kind in ctx.nodes.ancestor_kinds(ctx.current_node_id) {
        match node_kind {
            AstKind::Program(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.error(diagnostics::invalid_break(stmt.span)),
                    |label| ctx.error(diagnostics::invalid_label_target(label.span)),
                );
            }
            AstKind::Function(_) | AstKind::StaticBlock(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.error(diagnostics::invalid_break(stmt.span)),
                    |label| ctx.error(diagnostics::invalid_label_jump_target(label.span)),
                );
            }
            AstKind::LabeledStatement(labeled_statement) => {
                if stmt
                    .label
                    .as_ref()
                    .is_some_and(|label| label.name == labeled_statement.label.name)
                {
                    break;
                }
            }
            kind if (kind.is_iteration_statement()
                || matches!(kind, AstKind::SwitchStatement(_)))
                && stmt.label.is_none() =>
            {
                break;
            }
            _ => {}
        }
    }
}

pub fn check_continue_statement(stmt: &ContinueStatement, ctx: &SemanticBuilder<'_>) {
    // It is a Syntax Error if this ContinueStatement is not nested, directly or indirectly (but not crossing function or static initialization block boundaries), within an IterationStatement.
    for node_kind in ctx.nodes.ancestor_kinds(ctx.current_node_id) {
        match node_kind {
            AstKind::Program(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.error(diagnostics::invalid_continue(stmt.span)),
                    |label| ctx.error(diagnostics::invalid_label_target(label.span)),
                );
            }
            AstKind::Function(_) | AstKind::StaticBlock(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.error(diagnostics::invalid_continue(stmt.span)),
                    |label| ctx.error(diagnostics::invalid_label_jump_target(label.span)),
                );
            }
            AstKind::LabeledStatement(labeled_statement) => match &stmt.label {
                Some(label) if label.name == labeled_statement.label.name => {
                    if matches!(
                        labeled_statement.body,
                        Statement::LabeledStatement(_)
                            | Statement::DoWhileStatement(_)
                            | Statement::WhileStatement(_)
                            | Statement::ForStatement(_)
                            | Statement::ForInStatement(_)
                            | Statement::ForOfStatement(_)
                    ) {
                        break;
                    }
                    return ctx.error(diagnostics::invalid_label_non_iteration(
                        "continue",
                        labeled_statement.label.span,
                        label.span,
                    ));
                }
                _ => {}
            },
            kind if kind.is_iteration_statement() && stmt.label.is_none() => break,
            _ => {}
        }
    }
}

pub fn check_labeled_statement(stmt: &LabeledStatement, ctx: &SemanticBuilder<'_>) {
    for node_kind in ctx.nodes.ancestor_kinds(ctx.current_node_id) {
        match node_kind {
            // label cannot cross boundary on function or static block
            AstKind::Function(_)
            | AstKind::ArrowFunctionExpression(_)
            | AstKind::StaticBlock(_) => break,
            // check label name redeclaration
            AstKind::LabeledStatement(label_stmt) if stmt.label.name == label_stmt.label.name => {
                return ctx.error(diagnostics::label_redeclaration(
                    stmt.label.name.as_str(),
                    label_stmt.label.span,
                    stmt.label.span,
                ));
            }
            _ => {}
        }
    }
}

pub fn check_for_statement_left(
    left: &ForStatementLeft,
    is_for_in: bool,
    ctx: &SemanticBuilder<'_>,
) {
    let ForStatementLeft::VariableDeclaration(decl) = left else { return };

    // initializer is not allowed for for-in / for-of
    if decl.declarations.len() > 1 {
        return ctx.error(diagnostics::multiple_declaration_in_for_loop_head(
            if is_for_in { "in" } else { "of" },
            decl.span,
        ));
    }

    let strict_mode = ctx.strict_mode();
    for declarator in &decl.declarations {
        if declarator.init.is_some()
            && (strict_mode
                || !is_for_in
                || decl.kind.is_lexical()
                || !matches!(declarator.id, BindingPattern::BindingIdentifier(_)))
        {
            ctx.error(diagnostics::unexpected_initializer_in_for_loop_head(
                if is_for_in { "for-in" } else { "for-of" },
                decl.span,
            ));
        }
    }
}

pub fn check_class(class: &Class, ctx: &SemanticBuilder<'_>) {
    check_private_identifier(ctx);

    if class.is_declaration()
        && class.id.is_none()
        && !matches!(
            ctx.nodes.parent_kind(ctx.current_node_id),
            AstKind::ExportDefaultDeclaration(_)
        )
    {
        let start = class.span.start;
        ctx.error(diagnostics::require_class_name(Span::sized(start, 5)));
    }

    // ClassBody : ClassElementList
    // It is a Syntax Error if PrototypePropertyNameList of ClassElementList contains more than one occurrence of "constructor".
    let mut prev_constructor: Option<Span> = None;
    let constructors = class.body.body.iter().filter_map(|e| {
        if let ClassElement::MethodDefinition(def) = e {
            // is declaration
            def.value.body.as_ref()?;
            if def.kind == MethodDefinitionKind::Constructor {
                return def.key.prop_name().map_or(Some(def.span), |(_, node)| Some(node));
            }
        }
        None
    });
    for new_span in constructors {
        if let Some(prev_span) = prev_constructor {
            return ctx.error(diagnostics::duplicate_constructor(prev_span, new_span));
        }
        prev_constructor = Some(new_span);
    }
}

pub fn check_super(sup: &Super, ctx: &SemanticBuilder<'_>) {
    // `Some` for `super()`, `None` for `super.foo` / `super.bar()` etc
    let super_call_span = match ctx.nodes.parent_kind(ctx.current_node_id) {
        AstKind::CallExpression(expr) => Some(expr.span),
        AstKind::NewExpression(expr) => Some(expr.span),
        _ => None,
    };

    let (mut class_scope_id, mut class_id) =
        get_class_details(ctx.class_table_builder.current_class_id, ctx);

    let mut previous_scope_id = None;

    // In this loop, we `return` if `super` is legal, or `break` if it's illegal.
    //
    // We also fall through to an error if `super` is not inside a function or class.
    // > ModuleBody : ModuleItemList
    // > * It is a Syntax Error if ModuleItemList Contains super.
    // > ScriptBody : StatementList
    // > * It is a Syntax Error if StatementList Contains super
    'scopes: for scope_id in ctx.scoping.scope_ancestors(ctx.current_scope_id) {
        if Some(scope_id) == class_scope_id {
            // Reached the class scope.
            //
            // We already exited if inside a method, or static block (see below).
            // Therefore we're in one of:
            // 1. Class property value.
            // 2. Class accessor value.
            // 3. Computed key of a class method / property / accessor.
            // 4. Decorators on a class method / property / accessor.
            // 5. `TSIndexSignature`.
            // Find out which.
            //
            // Note: In terms of scopes, we could also be in a class's `super_class`,
            // but `ClassTableBuilder` does not enter the class until entering class body.
            // So when visiting `super` in `class Outer { method() { class Inner extends super.foo {} } }`,
            // `ctx.class_table_builder.current_class_id` is `Outer` class, not `Inner`.

            let search_start_node_id = if let Some(previous_scope_id) = previous_scope_id {
                ctx.scoping.get_node_id(previous_scope_id)
            } else {
                ctx.current_node_id
            };
            let mut previous_node_address = ctx.nodes.kind(search_start_node_id).address();

            for ancestor_kind in ctx.nodes.ancestor_kinds(search_start_node_id) {
                match ancestor_kind {
                    AstKind::PropertyDefinition(prop) => {
                        if prop
                            .value
                            .as_ref()
                            .is_some_and(|value| value.address() == previous_node_address)
                        {
                            // In property's value - `super.foo` is legal here, `super()` is not.
                            // > FieldDefinition : ClassElementName Initializer opt
                            // > * It is a Syntax Error if Initializer is present and Initializer Contains SuperCall is true.
                            if super_call_span.is_some() {
                                break 'scopes;
                            }
                            return;
                        }
                        // In computed key or decorators
                    }
                    AstKind::AccessorProperty(prop) => {
                        if prop
                            .value
                            .as_ref()
                            .is_some_and(|value| value.address() == previous_node_address)
                        {
                            // In accessor's value - `super.foo` is legal here, `super()` is not
                            if super_call_span.is_some() {
                                break 'scopes;
                            }
                            return;
                        }
                        // In computed key or decorators
                    }
                    AstKind::MethodDefinition(_) => {
                        // In computed key or decorators.
                        // If we were in the value, we would have exited loop already,
                        // because `value` is a function - which is handled below.
                    }
                    AstKind::TSIndexSignature(sig) => {
                        // I (@overlookmotel) don't think `Super` should appear in a type annotation.
                        // e.g. `super` is parsed as an `IdentifierReference`, not `Super` in:
                        // `class C { [keys: typeof super.foo]: typeof super.foo }`
                        // But I did find one weird case where `super` *is* currently parsed as `Super`:
                        // `class C { [keys: string]: typeof import('x', { with: super.foo }).y; }`
                        //
                        // So probably this branch is unreachable in practice. But handle it just in case,
                        // to avoid falling through to `unreachable!()` below.
                        //
                        // If it *is* possible, I'm also not sure what correct behavior should be.
                        // As best guess, treating it like class properties:
                        // Treat `parameters` like computed key, `type_annotation` like initializer value.
                        if sig.type_annotation.address() == previous_node_address {
                            // In signature's `type_annotation` - `super.foo` is legal here, `super()` is not
                            if super_call_span.is_some() {
                                break 'scopes;
                            }
                            return;
                        }
                        // In `parameters` - treat like computed key
                    }
                    _ => {
                        previous_node_address = ancestor_kind.address();
                        continue;
                    }
                }

                // `super` is in a computed key, decorator, or `TSIndexSignature`'s `parameters`.
                //
                // Whether it's legal or not depends on external context
                // (whether this class is nested in another class or object method).
                //
                // Illegal:
                // * `class C { [super.foo] = 1 }`
                // * `class C { @super.foo method() {} }`
                // * `class C extends super.foo {}`
                //
                // Legal:
                // * `class Outer { method() { class Inner { [super.foo] = 1 } } }`
                // * `class Outer { method() { class Inner { @super.foo method() {} } } }`
                // * `class Outer { method() { class Inner extends super.foo {} } }`
                // * `obj = { method() { class Inner { [super.foo] = 1 } } }`
                // * `obj = { method() { class Inner { @super.foo method() {} } } }`
                // * `obj = { method() { class Inner extends super.foo {} } }`
                //
                // So continue searching up the scope tree.

                // Set `previous_scope_id` to the class. On next ancestor search, start from this class.
                previous_scope_id = Some(scope_id);

                // We're now in the parent class
                let parent_class_id =
                    ctx.class_table_builder.classes.parent_ids.get(&class_id).copied();
                (class_scope_id, class_id) = get_class_details(parent_class_id, ctx);

                continue 'scopes;
            }

            // See comment above. The `for` loop above cannot complete without exiting early
            // with `return`, `break 'scopes`, or `continue 'scopes`.
            unreachable!();
        }

        let scope_flags = ctx.scoping.scope_flags(scope_id);

        // `super.foo` is legal in static blocks, `super()` is not.
        // > ClassStaticBlockBody : ClassStaticBlockStatementList
        // > * It is a Syntax Error if ClassStaticBlockStatementList Contains SuperCall is true.
        if scope_flags.is_class_static_block() {
            if super_call_span.is_some() {
                break;
            }
            return;
        }

        // Skip over non-function scopes and arrow functions
        if !scope_flags.is_function() || scope_flags.is_arrow() {
            // If we reach class scope in a later iteration, we can search for class element containing
            // `super` starting from this scope's node, instead of starting from `super`,
            // which saves iterations over ancestor nodes
            previous_scope_id = Some(scope_id);

            continue;
        }

        // We're in a function.
        // If function is a class or object method/getter/setter/constructor, then `super.foo` is legal.
        // `super()` is only legal if in a class constructor.
        // If function is anywhere else, both `super()` and `super.foo` are illegal.
        let func_node_id = ctx.scoping.get_node_id(scope_id);
        let func_address = ctx.nodes.kind(func_node_id).address();

        match ctx.nodes.parent_kind(func_node_id) {
            AstKind::ObjectProperty(prop) => {
                // Function's parent is an `ObjectProperty`.
                // Check the function is a method/getter/setter, not a normal property.
                // Valid: `obj = { method() { super.foo } }`
                // Invalid: `obj = { x: function() { super.foo } }`
                let is_method_or_getter_or_setter = prop.method || prop.kind != PropertyKind::Init;
                if is_method_or_getter_or_setter {
                    // Function's parent is an `ObjectProperty` representing a method/getter/setter.
                    // Check the function is the value of the property, not computed key.
                    // Valid: `obj = { method() { super.foo } }`
                    // Invalid: `obj = { [ function() { super.foo } ]() {} }`
                    if func_address == prop.value.address() {
                        // `super.foo` is legal here, `super()` is not.
                        // > PropertyDefinition : MethodDefinition
                        // > * It is a Syntax Error if HasDirectSuper of MethodDefinition is true.
                        if super_call_span.is_some() {
                            break;
                        }
                        return;
                    }
                }
                // Function is value of a normal property, or computed key - illegal
                break;
            }
            AstKind::MethodDefinition(method) => {
                // Function's parent is a `MethodDefinition` representing a class method/getter/setter/constructor.
                // Check the function is the method itself, not computed key or decorator.
                // Valid: `class C { method() { super.foo } }`
                // Invalid: `class C { [ function() { super.foo } ]() {} }`
                // Invalid: `class C { @(function() { super.foo }) method() {} }`
                if func_address == method.value.address() {
                    // `super.foo` is legal here.
                    // `super()` is only legal if method is class constructor, and class has a super-class.
                    //
                    // > ClassElement : MethodDefinition
                    // > * It is a Syntax Error if PropName of MethodDefinition is not "constructor" and
                    // >   HasDirectSuper of MethodDefinition is true.
                    // > * It is a Syntax Error if SuperCall in nested set/get function.
                    // >
                    // > ClassTail : ClassHeritageopt { ClassBody }
                    // > * It is a Syntax Error if ClassHeritage is not present and the following algorithm returns true:
                    // >   1. Let constructor be ConstructorMethod of ClassBody.
                    // >   2. If constructor is empty, return false.
                    // >   3. Return HasDirectSuper of constructor.
                    if super_call_span.is_some() {
                        if method.kind != MethodDefinitionKind::Constructor {
                            break;
                        }

                        let class_node_id = ctx.class_table_builder.classes.get_node_id(class_id);
                        let class = ctx.nodes.kind(class_node_id).as_class().unwrap();
                        if class.super_class.is_none() {
                            ctx.error(diagnostics::super_without_derived_class(
                                sup.span, class.span,
                            ));
                        }
                    }
                    return;
                }
                // Function is computed key or decorator - illegal
                break;
            }
            // Function is not a class or object method/getter/setter/constructor - illegal.
            // > * It is a Syntax Error if FunctionBody Contains SuperProperty is true.
            _ => break,
        }
    }

    // `super` is in illegal position
    if let Some(super_call_span) = super_call_span {
        ctx.error(diagnostics::unexpected_super_call(super_call_span));
    } else {
        ctx.error(diagnostics::unexpected_super_reference(sup.span));
    }
}

fn get_class_details(
    maybe_class_id: Option<ClassId>,
    ctx: &SemanticBuilder<'_>,
) -> (Option<ScopeId>, ClassId) {
    let Some(class_id) = maybe_class_id else {
        return (None, ClassId::new(0)); // Dummy class ID
    };
    let node_id = ctx.class_table_builder.classes.get_node_id(class_id);
    let class = ctx.nodes.kind(node_id).as_class().unwrap();
    let scope_id = class.scope_id();
    (Some(scope_id), class_id)
}

pub fn check_assignment_expression(assign_expr: &AssignmentExpression, ctx: &SemanticBuilder<'_>) {
    // AssignmentExpression :
    //     LeftHandSideExpression AssignmentOperator AssignmentExpression
    //     LeftHandSideExpression &&= AssignmentExpression
    //     LeftHandSideExpression ||= AssignmentExpression
    //     LeftHandSideExpression ??= AssignmentExpression
    // It is a Syntax Error if AssignmentTargetType of LeftHandSideExpression is not SIMPLE.
    if assign_expr.operator != AssignmentOperator::Assign
        && !assign_expr.left.is_simple_assignment_target()
    {
        ctx.error(diagnostics::assignment_is_not_simple(assign_expr.left.span()));
    }
}

pub fn check_object_expression(obj_expr: &ObjectExpression, ctx: &SemanticBuilder<'_>) {
    // ObjectLiteral : { PropertyDefinitionList }
    // It is a Syntax Error if PropertyNameList of PropertyDefinitionList contains any duplicate entries for "__proto__"
    // and at least two of those entries were obtained from productions of the form PropertyDefinition : PropertyName : AssignmentExpression
    let mut prev_proto: Option<Span> = None;
    for prop in &obj_expr.properties {
        if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
            // Skip if not a property definition production:
            // PropertyDefinition : PropertyName : AssignmentExpression
            if obj_prop.kind != PropertyKind::Init || obj_prop.method {
                continue;
            }
            if let Some((prop_name, span)) = prop.prop_name()
                && prop_name == "__proto__"
            {
                if let Some(prev_span) = prev_proto {
                    ctx.error(diagnostics::redeclaration("__proto__", prev_span, span));
                }
                prev_proto = Some(span);
            }
        }
    }
}

pub fn check_private_field_expression(
    private_expr: &PrivateFieldExpression,
    ctx: &SemanticBuilder<'_>,
) {
    // `super.#m`
    if private_expr.object.is_super() {
        ctx.error(diagnostics::super_private(private_expr.span));
    }
}

pub fn check_unary_expression(unary_expr: &UnaryExpression, ctx: &SemanticBuilder<'_>) {
    // https://tc39.es/ecma262/#sec-delete-operator-static-semantics-early-errors
    if unary_expr.operator == UnaryOperator::Delete {
        match unary_expr.argument.get_inner_expression() {
            Expression::Identifier(ident) if ctx.strict_mode() => {
                ctx.error(diagnostics::delete_of_unqualified(ident.span));
            }
            Expression::PrivateFieldExpression(expr) => {
                ctx.error(diagnostics::delete_private_field(expr.span));
            }
            Expression::ChainExpression(chain_expr) => {
                if let ChainElement::PrivateFieldExpression(e) = &chain_expr.expression {
                    ctx.error(diagnostics::delete_private_field(e.field.span));
                }
            }
            _ => {}
        }
    }
}

fn is_in_formal_parameters(ctx: &SemanticBuilder<'_>) -> bool {
    for node_kind in ctx.nodes.ancestor_kinds(ctx.current_node_id) {
        match node_kind {
            AstKind::FormalParameter(_) => return true,
            AstKind::Program(_) | AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                break;
            }
            _ => {}
        }
    }
    false
}

pub fn check_await_expression(expr: &AwaitExpression, ctx: &SemanticBuilder<'_>) {
    if is_in_formal_parameters(ctx) {
        ctx.error(diagnostics::await_or_yield_in_parameter("await", expr.span));
    }
    // It is a Syntax Error if ClassStaticBlockStatementList Contains await is true.
    if ctx.scoping.scope_flags(ctx.current_scope_id).is_class_static_block() {
        let start = expr.span.start;
        ctx.error(diagnostics::class_static_block_await(Span::sized(start, 5)));
    }
}

pub fn check_yield_expression(expr: &YieldExpression, ctx: &SemanticBuilder<'_>) {
    if is_in_formal_parameters(ctx) {
        ctx.error(diagnostics::await_or_yield_in_parameter("yield", expr.span));
    }
}
