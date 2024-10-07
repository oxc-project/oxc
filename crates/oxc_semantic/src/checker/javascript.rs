use phf::{phf_set, Set};
use rustc_hash::FxHashMap;

#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_span::{GetSpan, ModuleKind, Span};
use oxc_syntax::{
    module_record::ExportLocalName,
    number::NumberBase,
    operator::{AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator},
};
use oxc_syntax_operations::{IsSimpleParameterList, PropName};

use crate::{builder::SemanticBuilder, diagnostics::redeclaration, scope::ScopeFlags, AstNode};

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

                is_duplicate = if ctx.source_type.is_typescript() {
                    element.r#static == prev_element.r#static && is_duplicate
                } else {
                    // * It is a Syntax Error if PrivateBoundIdentifiers of ClassElementList contains any duplicate entries,
                    // unless the name is used once for a getter and once for a setter and in no other entries,
                    // and the getter and setter are either both static or both non-static.
                    element.is_private && is_duplicate
                };

                if is_duplicate {
                    ctx.error(redeclaration(&element.name, prev_element.span, element.span));
                }
            }
        }
    });
}

fn undefined_export(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Export '{x0}' is not defined")).with_label(span1)
}

fn duplicate_export(x0: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Duplicated export '{x0}'")).with_labels([
        span1.label("Export has already been declared here"),
        span2.label("It cannot be redeclared here"),
    ])
}

pub fn check_module_record(ctx: &SemanticBuilder<'_>) {
    // Skip checkking for exports in TypeScript for now
    if ctx.source_type.is_typescript() {
        return;
    }

    let module_record = &ctx.module_record;

    // It is a Syntax Error if any element of the ExportedBindings of ModuleItemList
    // does not also occur in either the VarDeclaredNames of ModuleItemList, or the LexicallyDeclaredNames of ModuleItemList.
    module_record
        .local_export_entries
        .iter()
        .filter_map(|export_entry| match &export_entry.local_name {
            ExportLocalName::Name(name_span) => Some(name_span),
            _ => None,
        })
        .filter(|name_span| {
            ctx.scope.get_binding(ctx.current_scope_id, name_span.name().as_ref()).is_none()
        })
        .for_each(|name_span| {
            ctx.error(undefined_export(name_span.name(), name_span.span()));
        });

    // It is a Syntax Error if the ExportedNames of ModuleItemList contains any duplicate entries.
    for name_span in &module_record.exported_bindings_duplicated {
        let old_span = module_record.exported_bindings[name_span.name()];
        ctx.error(duplicate_export(name_span.name(), name_span.span(), old_span));
    }

    for span in &module_record.export_default_duplicated {
        let old_span = module_record.export_default.unwrap();
        ctx.error(duplicate_export("default", *span, old_span));
    }

    // `export default x;`
    // `export { y as default };`
    if let (Some(span), Some(default_span)) =
        (module_record.exported_bindings.get("default"), &module_record.export_default)
    {
        ctx.error(duplicate_export("default", *default_span, *span));
    }
}

fn class_static_block_await(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Cannot use await in class static initialization block").with_label(span)
}

fn reserved_keyword(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("The keyword '{x0}' is reserved")).with_label(span1)
}

pub const STRICT_MODE_NAMES: Set<&'static str> = phf_set! {
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
};

pub fn check_identifier<'a>(name: &str, span: Span, node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) {
    // ts module block allows revered keywords
    if ctx.current_scope_flags().is_ts_module_block() {
        return;
    }
    if name == "await" {
        // It is a Syntax Error if the goal symbol of the syntactic grammar is Module and the StringValue of IdentifierName is "await".
        if ctx.source_type.is_module() {
            return ctx.error(reserved_keyword(name, span));
        }
        // It is a Syntax Error if ClassStaticBlockStatementList Contains await is true.
        if ctx.scope.get_flags(node.scope_id()).is_class_static_block() {
            return ctx.error(class_static_block_await(span));
        }
    }

    // It is a Syntax Error if this phrase is contained in strict mode code and the StringValue of IdentifierName is: "implements", "interface", "let", "package", "private", "protected", "public", "static", or "yield".
    if ctx.strict_mode() && STRICT_MODE_NAMES.contains(name) {
        ctx.error(reserved_keyword(name, span));
    }
}

fn unexpected_identifier_assign(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Cannot assign to '{x0}' in strict mode")).with_label(span1)
}

fn invalid_let_declaration(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "`let` cannot be declared as a variable name inside of a `{x0}` declaration"
    ))
    .with_label(span1)
}

pub fn check_binding_identifier<'a>(
    ident: &BindingIdentifier,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    let strict_mode = ctx.strict_mode();
    // It is a Diagnostic if the StringValue of a BindingIdentifier is "eval" or "arguments" within strict mode code.
    if strict_mode && matches!(ident.name.as_str(), "eval" | "arguments") {
        return ctx.error(unexpected_identifier_assign(&ident.name, ident.span));
    }

    // LexicalDeclaration : LetOrConst BindingList ;
    // * It is a Syntax Error if the BoundNames of BindingList contains "let".
    if !strict_mode && ident.name == "let" {
        for node_id in ctx.nodes.ancestors(node.id()).skip(1) {
            match ctx.nodes.kind(node_id) {
                AstKind::VariableDeclaration(decl) if decl.kind.is_lexical() => {
                    return ctx.error(invalid_let_declaration(decl.kind.as_str(), ident.span));
                }
                AstKind::VariableDeclaration(_) | AstKind::Function(_) | AstKind::Program(_) => {
                    break;
                }
                _ => {}
            }
        }
    }
}

fn unexpected_arguments(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("'arguments' is not allowed in {x0}")).with_label(span1)
}

pub fn check_identifier_reference<'a>(
    ident: &IdentifierReference,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    //  Static Semantics: AssignmentTargetType
    //  1. If this IdentifierReference is contained in strict mode code and StringValue of Identifier is "eval" or "arguments", return invalid.
    if ctx.strict_mode() && matches!(ident.name.as_str(), "arguments" | "eval") {
        for node_id in ctx.nodes.ancestors(node.id()).skip(1) {
            match ctx.nodes.kind(node_id) {
                AstKind::AssignmentTarget(_) | AstKind::SimpleAssignmentTarget(_) => {
                    return ctx.error(unexpected_identifier_assign(&ident.name, ident.span));
                }
                AstKind::MemberExpression(_) => break,
                _ => {}
            }
        }
    }

    // FieldDefinition : ClassElementName Initializeropt
    //   It is a Syntax Error if Initializer is present and ContainsArguments of Initializer is true.
    // ClassStaticBlockBody : ClassStaticBlockStatementList
    //   It is a Syntax Error if ContainsArguments of ClassStaticBlockStatementList is true.

    if ident.name == "arguments" {
        for node_id in ctx.nodes.ancestors(node.id()).skip(1) {
            match ctx.nodes.kind(node_id) {
                AstKind::Function(_) => break,
                AstKind::PropertyDefinition(_) => {
                    return ctx.error(unexpected_arguments("class field initializer", ident.span));
                }
                AstKind::StaticBlock(_) => {
                    return ctx
                        .error(unexpected_arguments("static initialization block", ident.span));
                }
                _ => {}
            }
        }
    }
}

fn private_not_in_class(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Private identifier '#{x0}' is not allowed outside class bodies"))
        .with_label(span1)
}

pub fn check_private_identifier_outside_class(
    ident: &PrivateIdentifier,
    ctx: &SemanticBuilder<'_>,
) {
    if ctx.class_table_builder.current_class_id.is_none() {
        ctx.error(private_not_in_class(&ident.name, ident.span));
    }
}

fn private_field_undeclared(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Private field '{x0}' must be declared in an enclosing class"))
        .with_label(span1)
}

fn check_private_identifier(ctx: &SemanticBuilder<'_>) {
    if let Some(class_id) = ctx.class_table_builder.current_class_id {
        ctx.class_table_builder.classes.iter_private_identifiers(class_id).for_each(|reference| {
            if reference.element_ids.is_empty()
                && !ctx.class_table_builder.classes.ancestors(class_id).skip(1).any(|class_id| {
                    ctx.class_table_builder
                        .classes
                        .has_private_definition(class_id, &reference.name)
                })
            {
                ctx.error(private_field_undeclared(&reference.name, reference.span));
            }
        });
    }
}

fn legacy_octal(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("'0'-prefixed octal literals and octal escape sequences are deprecated")
        .with_help("for octal literals use the '0o' prefix instead")
        .with_label(span)
}

fn leading_zero_decimal(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Decimals with leading zeros are not allowed in strict mode")
        .with_help("remove the leading zero")
        .with_label(span)
}

pub fn check_number_literal(lit: &NumericLiteral, ctx: &SemanticBuilder<'_>) {
    // NumericLiteral :: legacy_octalIntegerLiteral
    // DecimalIntegerLiteral :: NonOctalDecimalIntegerLiteral
    // * It is a Syntax Error if the source text matched by this production is strict mode code.
    fn leading_zero(s: &str) -> bool {
        let mut chars = s.bytes();
        if let Some(first) = chars.next() {
            if let Some(second) = chars.next() {
                return first == b'0' && second.is_ascii_digit();
            }
        }
        false
    }

    if ctx.strict_mode() {
        match lit.base {
            NumberBase::Octal if leading_zero(lit.raw) => {
                ctx.error(legacy_octal(lit.span));
            }
            NumberBase::Decimal | NumberBase::Float if leading_zero(lit.raw) => {
                ctx.error(leading_zero_decimal(lit.span));
            }
            _ => {}
        }
    }
}

fn non_octal_decimal_escape_sequence(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid escape sequence")
        .with_help("\\8 and \\9 are not allowed in strict mode")
        .with_label(span)
}

pub fn check_string_literal(lit: &StringLiteral, ctx: &SemanticBuilder<'_>) {
    // 12.9.4.1 Static Semantics: Early Errors
    // EscapeSequence ::
    //   legacy_octalEscapeSequence
    //   non_octal_decimal_escape_sequence
    // It is a Syntax Error if the source text matched by this production is strict mode code.
    let raw = lit.span.source_text(ctx.source_text);
    if ctx.strict_mode() && raw.len() != lit.value.len() {
        let mut chars = raw.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('0') => {
                        if chars.peek().is_some_and(|c| ('1'..='9').contains(c)) {
                            return ctx.error(legacy_octal(lit.span));
                        }
                    }
                    Some('1'..='7') => {
                        return ctx.error(legacy_octal(lit.span));
                    }
                    Some('8'..='9') => {
                        return ctx.error(non_octal_decimal_escape_sequence(lit.span));
                    }
                    _ => {}
                }
            }
        }
    }
}

fn illegal_use_strict(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "Illegal 'use strict' directive in function with non-simple parameter list",
    )
    .with_label(span)
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

    if matches!(ctx.nodes.kind(ctx.scope.get_node_id(ctx.current_scope_id)),
        AstKind::Function(Function { params, .. })
        | AstKind::ArrowFunctionExpression(ArrowFunctionExpression { params, .. })
        if !params.is_simple_parameter_list())
    {
        ctx.error(illegal_use_strict(directive.span));
    }
}

fn top_level(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "'{x0}' declaration can only be used at the top level of a module"
    ))
    .with_label(span1)
}

fn module_code(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Cannot use {x0} outside a module")).with_label(span1)
}

pub fn check_module_declaration<'a>(
    decl: &ModuleDeclaration,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    // It is ambiguous between script and module for `TypeScript`, skipping this check for now.
    // Basically we need to "upgrade" from script to module if we see any module syntax inside the
    // semantic builder
    if ctx.source_type.is_typescript() {
        return;
    }

    let text = match decl {
        ModuleDeclaration::ImportDeclaration(_) => "import statement",
        ModuleDeclaration::ExportAllDeclaration(_)
        | ModuleDeclaration::ExportDefaultDeclaration(_)
        | ModuleDeclaration::ExportNamedDeclaration(_)
        | ModuleDeclaration::TSExportAssignment(_)
        | ModuleDeclaration::TSNamespaceExportDeclaration(_) => "export statement",
    };
    let start = decl.span().start;
    let span = Span::new(start, start + 6);
    match ctx.source_type.module_kind() {
        ModuleKind::Unambiguous => {
            #[cfg(debug_assertions)]
            panic!("Technically unreachable, omit to avoid panic.");
        }
        ModuleKind::Script => {
            ctx.error(module_code(text, span));
        }
        ModuleKind::Module => {
            if matches!(ctx.nodes.parent_kind(node.id()), Some(AstKind::Program(_))) {
                return;
            }
            ctx.error(top_level(text, span));
        }
    }
}

fn new_target(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected new.target expression")
.with_help("new.target is only allowed in constructors and functions invoked using thew `new` operator")
.with_label(span)
}

fn new_target_property(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("The only valid meta property for new is new.target").with_label(span)
}

fn import_meta(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected import.meta expression")
        .with_help("import.meta is only allowed in module code")
        .with_label(span)
}

fn import_meta_property(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("The only valid meta property for import is import.meta").with_label(span)
}

pub fn check_meta_property<'a>(prop: &MetaProperty, node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) {
    match prop.meta.name.as_str() {
        "import" => {
            if prop.property.name == "meta" {
                if ctx.source_type.is_script() {
                    return ctx.error(import_meta(prop.span));
                }
                return;
            }
            ctx.error(import_meta_property(prop.span));
        }
        "new" => {
            if prop.property.name == "target" {
                let mut in_function_scope = false;
                for scope_id in ctx.scope.ancestors(node.scope_id()) {
                    let flags = ctx.scope.get_flags(scope_id);
                    // In arrow functions, new.target is inherited from the surrounding scope.
                    if flags.contains(ScopeFlags::Arrow) {
                        continue;
                    }
                    if flags.intersects(ScopeFlags::Function | ScopeFlags::ClassStaticBlock) {
                        in_function_scope = true;
                        break;
                    }
                }
                if !in_function_scope {
                    return ctx.error(new_target(prop.span));
                }
                return;
            }
            ctx.error(new_target_property(prop.span));
        }
        _ => {}
    }
}

fn function_declaration_strict(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid function declaration")
        .with_help(
            "In strict mode code, functions can only be declared at top level or inside a block",
        )
        .with_label(span)
}

fn function_declaration_non_strict(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid function declaration")
.with_help("In non-strict mode code, functions can only be declared at top level, inside a block, or as the body of an if statement")
.with_label(span)
}

pub fn check_function_declaration<'a>(
    stmt: &Statement<'a>,
    is_if_stmt_or_labeled_stmt: bool,
    ctx: &SemanticBuilder<'a>,
) {
    // Function declaration not allowed in statement position
    if let Statement::FunctionDeclaration(decl) = stmt {
        if ctx.strict_mode() {
            ctx.error(function_declaration_strict(decl.span));
        } else if !is_if_stmt_or_labeled_stmt {
            ctx.error(function_declaration_non_strict(decl.span));
        }
    };
}

fn reg_exp_flag_u_and_v(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "The 'u' and 'v' regular expression flags cannot be enabled at the same time",
    )
    .with_label(span)
}

pub fn check_regexp_literal(lit: &RegExpLiteral, ctx: &SemanticBuilder<'_>) {
    let flags = lit.regex.flags;
    if flags.contains(RegExpFlags::U | RegExpFlags::V) {
        ctx.error(reg_exp_flag_u_and_v(lit.span));
    }
}

fn with_statement(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("'with' statements are not allowed").with_label(span)
}

pub fn check_with_statement(stmt: &WithStatement, ctx: &SemanticBuilder<'_>) {
    if ctx.strict_mode() || ctx.source_type.is_typescript() {
        ctx.error(with_statement(Span::new(stmt.span.start, stmt.span.start + 4)));
    }
}

pub fn check_switch_statement<'a>(stmt: &SwitchStatement<'a>, ctx: &SemanticBuilder<'a>) {
    let mut previous_default: Option<Span> = None;
    for case in &stmt.cases {
        if case.test.is_none() {
            if let Some(previous_span) = previous_default {
                ctx.error(redeclaration("default", previous_span, case.span));
                break;
            }
            previous_default.replace(case.span);
        }
    }
}

fn invalid_label_jump_target(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Jump target cannot cross function boundary.").with_label(span)
}

fn invalid_label_target(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Use of undefined label").with_label(span)
}

fn invalid_label_non_iteration(x0: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("A `{x0}` statement can only jump to a label of an enclosing `for`, `while` or `do while` statement."))
        .with_labels([
            span1.label("This is an non-iteration statement"),
            span2.label("for this label")
        ])
}

fn invalid_break(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Illegal break statement")
.with_help("A `break` statement can only be used within an enclosing iteration or switch statement.")
.with_label(span)
}

pub fn check_break_statement<'a>(
    stmt: &BreakStatement,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    // It is a Syntax Error if this BreakStatement is not nested, directly or indirectly (but not crossing function or static initialization block boundaries), within an IterationStatement or a SwitchStatement.
    for node_id in ctx.nodes.ancestors(node.id()).skip(1) {
        match ctx.nodes.kind(node_id) {
            AstKind::Program(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.error(invalid_break(stmt.span)),
                    |label| ctx.error(invalid_label_target(label.span)),
                );
            }
            AstKind::Function(_) | AstKind::StaticBlock(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.error(invalid_break(stmt.span)),
                    |label| ctx.error(invalid_label_jump_target(label.span)),
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

fn invalid_continue(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Illegal continue statement: no surrounding iteration statement")
.with_help("A `continue` statement can only be used within an enclosing `for`, `while` or `do while` ")
.with_label(span)
}

pub fn check_continue_statement<'a>(
    stmt: &ContinueStatement,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    // It is a Syntax Error if this ContinueStatement is not nested, directly or indirectly (but not crossing function or static initialization block boundaries), within an IterationStatement.
    for node_id in ctx.nodes.ancestors(node.id()).skip(1) {
        match ctx.nodes.kind(node_id) {
            AstKind::Program(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.error(invalid_continue(stmt.span)),
                    |label| ctx.error(invalid_label_target(label.span)),
                );
            }
            AstKind::Function(_) | AstKind::StaticBlock(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.error(invalid_continue(stmt.span)),
                    |label| ctx.error(invalid_label_jump_target(label.span)),
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
                    return ctx.error(invalid_label_non_iteration(
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

fn label_redeclaration(x0: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Label `{x0}` has already been declared")).with_labels([
        span1.label(format!("`{x0}` has already been declared here")),
        span2.label("It can not be redeclared here"),
    ])
}

pub fn check_labeled_statement<'a>(
    stmt: &LabeledStatement,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    for node_id in ctx.nodes.ancestors(node.id()).skip(1) {
        match ctx.nodes.kind(node_id) {
            // label cannot cross boundary on function or static block
            AstKind::Function(_) | AstKind::StaticBlock(_) | AstKind::Program(_) => break,
            // check label name redeclaration
            AstKind::LabeledStatement(label_stmt) if stmt.label.name == label_stmt.label.name => {
                return ctx.error(label_redeclaration(
                    stmt.label.name.as_str(),
                    label_stmt.label.span,
                    stmt.label.span,
                ));
            }
            _ => {}
        }
    }
}

fn multiple_declaration_in_for_loop_head(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "Only a single declaration is allowed in a `for...{x0}` statement"
    ))
    .with_label(span1)
}

fn unexpected_initializer_in_for_loop_head(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{x0} loop variable declaration may not have an initializer"))
        .with_label(span1)
}

pub fn check_for_statement_left<'a>(
    left: &ForStatementLeft,
    is_for_in: bool,
    _node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    let ForStatementLeft::VariableDeclaration(decl) = left else { return };

    // initializer is not allowed for for-in / for-of
    if decl.declarations.len() > 1 {
        return ctx.error(multiple_declaration_in_for_loop_head(
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
                || !matches!(declarator.id.kind, BindingPatternKind::BindingIdentifier(_)))
        {
            return ctx.error(unexpected_initializer_in_for_loop_head(
                if is_for_in { "for-in" } else { "for-of" },
                decl.span,
            ));
        }
    }
}

fn duplicate_constructor(span: Span, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Multiple constructor implementations are not allowed.").with_labels([
        LabeledSpan::new_with_span(Some("constructor has already been declared here".into()), span),
        LabeledSpan::new_with_span(Some("it cannot be redeclared here".into()), span1),
    ])
}

fn require_class_name(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A class name is required.").with_label(span)
}

pub fn check_class(class: &Class, node: &AstNode<'_>, ctx: &SemanticBuilder<'_>) {
    check_private_identifier(ctx);

    if class.is_declaration()
        && class.id.is_none()
        && !matches!(ctx.nodes.parent_kind(node.id()), Some(AstKind::ExportDefaultDeclaration(_)))
    {
        let start = class.span.start;
        ctx.error(require_class_name(Span::new(start, start + 5)));
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
            return ctx.error(duplicate_constructor(prev_span, new_span));
        }
        prev_constructor = Some(new_span);
    }
}

fn setter_with_parameters(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A 'set' accessor must have exactly one parameter.").with_label(span)
}

fn setter_with_rest_parameter(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A 'set' accessor cannot have rest parameter.").with_label(span)
}

fn check_setter(function: &Function<'_>, ctx: &SemanticBuilder<'_>) {
    function.params.rest.as_ref().map_or_else(
        || {
            if function.params.parameters_count() != 1 {
                ctx.error(setter_with_parameters(function.params.span));
            }
        },
        |rest| {
            ctx.error(setter_with_rest_parameter(rest.span));
        },
    );
}

fn getter_parameters(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A 'get' accessor must not have any formal parameters.").with_label(span)
}

fn check_getter(function: &Function<'_>, ctx: &SemanticBuilder<'_>) {
    if !function.params.items.is_empty() {
        ctx.error(getter_parameters(function.params.span));
    }
}

pub fn check_method_definition(method: &MethodDefinition<'_>, ctx: &SemanticBuilder<'_>) {
    match method.kind {
        MethodDefinitionKind::Set => check_setter(&method.value, ctx),
        MethodDefinitionKind::Get => check_getter(&method.value, ctx),
        _ => {}
    }
}

fn super_without_derived_class(span: Span, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("'super' can only be referenced in a derived class.")
        .with_help("either remove this super, or extend the class")
        .with_labels([
            span.into(),
            LabeledSpan::new_with_span(Some("class does not have `extends`".into()), span1),
        ])
}

fn unexpected_super_call(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Super calls are not permitted outside constructors or in nested functions inside constructors.")
.with_label(span)
}

fn unexpected_super_reference(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("'super' can only be referenced in members of derived classes or object literal expressions.")
.with_label(span)
}

pub fn check_super<'a>(sup: &Super, node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) {
    let super_call_span = match ctx.nodes.parent_kind(node.id()) {
        Some(AstKind::CallExpression(expr)) => Some(expr.span),
        Some(AstKind::NewExpression(expr)) => Some(expr.span),
        _ => None,
    };

    let Some(class_id) = ctx.class_table_builder.current_class_id else {
        for scope_id in ctx.scope.ancestors(ctx.current_scope_id) {
            let flags = ctx.scope.get_flags(scope_id);
            if flags.is_function()
                && matches!(
                    ctx.nodes.parent_kind(ctx.scope.get_node_id(scope_id)),
                    Some(AstKind::ObjectProperty(_))
                )
            {
                if let Some(super_call_span) = super_call_span {
                    ctx.error(unexpected_super_call(super_call_span));
                }
                return;
            };
        }

        // ModuleBody : ModuleItemList
        // * It is a Syntax Error if ModuleItemList Contains super.
        // ScriptBody : StatementList
        // * It is a Syntax Error if StatementList Contains super
        return super_call_span.map_or_else(
            || ctx.error(unexpected_super_reference(sup.span)),
            |super_call_span| ctx.error(unexpected_super_call(super_call_span)),
        );
    };

    // skip(1) is the self `Super`
    // skip(2) is the parent `CallExpression` or `NewExpression`
    for node_id in ctx.nodes.ancestors(node.id()).skip(2) {
        match ctx.nodes.kind(node_id) {
            AstKind::MethodDefinition(def) => {
                // ClassElement : MethodDefinition
                // It is a Syntax Error if PropName of MethodDefinition is not "constructor" and HasDirectSuper of MethodDefinition is true.
                if let Some(super_call_span) = super_call_span {
                    if def.kind == MethodDefinitionKind::Constructor {
                        // It is a Syntax Error if SuperCall in nested set/get function
                        if ctx.scope.get_flags(node.scope_id()).is_set_or_get_accessor() {
                            return ctx.error(unexpected_super_call(super_call_span));
                        }

                        // check ClassHeritage
                        if let AstKind::Class(class) =
                            ctx.nodes.kind(ctx.class_table_builder.classes.get_node_id(class_id))
                        {
                            // ClassTail : ClassHeritageopt { ClassBody }
                            // It is a Syntax Error if ClassHeritage is not present and the following algorithm returns true:
                            // 1. Let constructor be ConstructorMethod of ClassBody.
                            // 2. If constructor is empty, return false.
                            // 3. Return HasDirectSuper of constructor.
                            if class.super_class.is_none() {
                                return ctx.error(super_without_derived_class(sup.span, class.span));
                            }
                        }
                        break;
                    }
                    return ctx.error(unexpected_super_call(super_call_span));
                }
                // super references are allowed in method
                break;
            }
            // FieldDefinition : ClassElementName Initializer opt
            // * It is a Syntax Error if Initializer is present and Initializer Contains SuperCall is true.
            // PropertyDefinition : MethodDefinition
            // * It is a Syntax Error if HasDirectSuper of MethodDefinition is true.
            AstKind::PropertyDefinition(_)
            // ClassStaticBlockBody : ClassStaticBlockStatementList
            // * It is a Syntax Error if ClassStaticBlockStatementList Contains SuperCall is true.
            | AstKind::StaticBlock(_) => {
                if let Some(super_call_span) = super_call_span {
                    return ctx.error(unexpected_super_call(super_call_span));
                }
                break;
            }
            _ => {}
        }
    }
}

fn cover_initialized_name(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid assignment in object literal")
.with_help("Did you mean to use a ':'? An '=' can only follow a property name when the containing object literal is part of a destructuring pattern.")
.with_label(span)
}

pub fn check_object_property(prop: &ObjectProperty, ctx: &SemanticBuilder<'_>) {
    // PropertyDefinition : cover_initialized_name
    // It is a Syntax Error if any source text is matched by this production.
    if let Some(expr) = &prop.init {
        ctx.error(cover_initialized_name(expr.span()));
    }

    if let Expression::FunctionExpression(function) = &prop.value {
        match prop.kind {
            PropertyKind::Set => check_setter(function, ctx),
            PropertyKind::Get => check_getter(function, ctx),
            PropertyKind::Init => {}
        }
    }
}

fn a_rest_parameter_cannot_have_an_initializer(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("A rest parameter cannot have an initializer").with_label(span)
}

pub fn check_formal_parameters<'a>(
    params: &FormalParameters,
    _node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    if let Some(rest) = &params.rest {
        if let BindingPatternKind::AssignmentPattern(pat) = &rest.argument.kind {
            ctx.error(a_rest_parameter_cannot_have_an_initializer(pat.span));
        }
    }
}

pub fn check_array_pattern(pattern: &ArrayPattern, ctx: &SemanticBuilder<'_>) {
    // function foo([...x = []]) { }
    //                    ^^^^ A rest element cannot have an initializer
    if let Some(rest) = &pattern.rest {
        if let BindingPatternKind::AssignmentPattern(pat) = &rest.argument.kind {
            ctx.error(a_rest_parameter_cannot_have_an_initializer(pat.span));
        }
    }
}

fn assignment_is_not_simple(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid left-hand side in assignment").with_label(span)
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
        ctx.error(assignment_is_not_simple(assign_expr.left.span()));
    }
}

pub fn check_object_expression(obj_expr: &ObjectExpression, ctx: &SemanticBuilder<'_>) {
    // ObjectLiteral : { PropertyDefinitionList }
    // It is a Syntax Error if PropertyNameList of PropertyDefinitionList contains any duplicate entries for "__proto__"
    // and at least two of those entries were obtained from productions of the form PropertyDefinition : PropertyName : AssignmentExpression
    let mut prev_proto: Option<Span> = None;
    let prop_names = obj_expr.properties.iter().filter_map(PropName::prop_name);
    for prop_name in prop_names {
        if prop_name.0 == "__proto__" {
            if let Some(prev_span) = prev_proto {
                ctx.error(redeclaration("__proto__", prev_span, prop_name.1));
            }
            prev_proto = Some(prop_name.1);
        }
    }
}

fn unexpected_exponential(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected exponentiation expression")
        .with_help(format!("Wrap {x0} expression in parentheses to enforce operator precedence"))
        .with_label(span1)
}

pub fn check_binary_expression(binary_expr: &BinaryExpression, ctx: &SemanticBuilder<'_>) {
    if binary_expr.operator == BinaryOperator::Exponential {
        match binary_expr.left {
            // async () => await 5 ** 6
            // async () => await -5 ** 6
            Expression::AwaitExpression(_) => {
                ctx.error(unexpected_exponential("await", binary_expr.span));
            }
            // -5 ** 6
            Expression::UnaryExpression(_) => {
                ctx.error(unexpected_exponential("unary", binary_expr.span));
            }
            _ => {}
        }
    }
}

fn mixed_coalesce(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Logical expressions and coalesce expressions cannot be mixed")
        .with_help("Wrap either expression by parentheses")
        .with_label(span)
}

pub fn check_logical_expression(logical_expr: &LogicalExpression, ctx: &SemanticBuilder<'_>) {
    // check mixed coalesce
    // a ?? b || c - a ?? (b || c)
    // a ?? b && c - a ?? (b && c)
    // a || b ?? c - (a || b) ?? c
    // a && b ?? c - (a && b) ?? c
    if logical_expr.operator == LogicalOperator::Coalesce {
        let mut maybe_mixed_coalesce_expr = None;
        if let Expression::LogicalExpression(rhs) = &logical_expr.right {
            maybe_mixed_coalesce_expr = Some(rhs);
        } else if let Expression::LogicalExpression(lhs) = &logical_expr.left {
            maybe_mixed_coalesce_expr = Some(lhs);
        }
        if let Some(expr) = maybe_mixed_coalesce_expr {
            if matches!(expr.operator, LogicalOperator::And | LogicalOperator::Or) {
                ctx.error(mixed_coalesce(logical_expr.span));
            }
        }
    }
}

fn super_private(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Private fields cannot be accessed on super").with_label(span)
}

pub fn check_member_expression(member_expr: &MemberExpression, ctx: &SemanticBuilder<'_>) {
    if let MemberExpression::PrivateFieldExpression(private_expr) = member_expr {
        // super.#m
        if let Expression::Super(_) = &private_expr.object {
            ctx.error(super_private(private_expr.span));
        }
    }
}

fn delete_of_unqualified(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Delete of an unqualified identifier in strict mode.").with_label(span)
}

fn delete_private_field(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Private fields can not be deleted").with_label(span)
}

pub fn check_unary_expression<'a>(
    unary_expr: &'a UnaryExpression,
    _node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    // https://tc39.es/ecma262/#sec-delete-operator-static-semantics-early-errors
    if unary_expr.operator == UnaryOperator::Delete {
        match unary_expr.argument.get_inner_expression() {
            Expression::Identifier(ident) if ctx.strict_mode() => {
                ctx.error(delete_of_unqualified(ident.span));
            }
            Expression::PrivateFieldExpression(expr) => {
                ctx.error(delete_private_field(expr.span));
            }
            _ => {}
        }
    }
}

fn is_in_formal_parameters<'a>(node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) -> bool {
    for node_id in ctx.nodes.ancestors(node.id()).skip(1) {
        match ctx.nodes.kind(node_id) {
            AstKind::FormalParameter(_) => return true,
            AstKind::Program(_) | AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                break;
            }
            _ => {}
        }
    }
    false
}

fn await_or_yield_in_parameter(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("{x0} expression not allowed in formal parameter"))
        .with_label(span1.label(format!("{x0} expression not allowed in formal parameter")))
}

pub fn check_await_expression<'a>(
    expr: &AwaitExpression,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    if is_in_formal_parameters(node, ctx) {
        ctx.error(await_or_yield_in_parameter("await", expr.span));
    }
    // It is a Syntax Error if ClassStaticBlockStatementList Contains await is true.
    if ctx.scope.get_flags(node.scope_id()).is_class_static_block() {
        let start = expr.span.start;
        ctx.error(class_static_block_await(Span::new(start, start + 5)));
    }
}

pub fn check_yield_expression<'a>(
    expr: &YieldExpression,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    if is_in_formal_parameters(node, ctx) {
        ctx.error(await_or_yield_in_parameter("yield", expr.span));
    }
}
