#[allow(clippy::wildcard_imports)]
use oxc_ast::{
    ast::*,
    syntax_directed_operations::{BoundNames, IsSimpleParameterList, PropName},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_span::{Atom, GetSpan, ModuleKind, Span};
use oxc_syntax::{
    module_record::ExportLocalName,
    operator::{BinaryOperator, LogicalOperator, UnaryOperator},
    NumberBase,
};
use phf::{phf_set, Set};
use rustc_hash::FxHashMap;

use crate::{builder::SemanticBuilder, diagnostics::Redeclaration, scope::ScopeFlags, AstNode};

pub struct EarlyErrorJavaScript;

impl EarlyErrorJavaScript {
    pub fn run<'a>(node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) {
        let kind = node.kind();
        check_function_declaration(kind, node, ctx);

        match kind {
            AstKind::BindingIdentifier(ident) => {
                check_identifier(&ident.name, ident.span, node, ctx);
                check_binding_identifier(ident, node, ctx);
            }
            AstKind::IdentifierReference(ident) => {
                check_identifier(&ident.name, ident.span, node, ctx);
                check_identifier_reference(ident, node, ctx);
            }
            AstKind::LabelIdentifier(ident) => check_identifier(&ident.name, ident.span, node, ctx),
            AstKind::PrivateIdentifier(ident) => check_private_identifier(ident, node, ctx),

            AstKind::NumberLiteral(lit) => check_number_literal(lit, ctx),
            AstKind::StringLiteral(lit) => check_string_literal(lit, ctx),
            AstKind::RegExpLiteral(lit) => check_regexp_literal(lit, ctx),

            AstKind::Directive(dir) => check_directive(dir, node, ctx),
            AstKind::ModuleDeclaration(decl) => {
                check_module_declaration(decl, node, ctx);
                if let ModuleDeclaration::ImportDeclaration(import_decl) = decl {
                    check_import_declaration(import_decl, ctx);
                }
            }
            AstKind::MetaProperty(prop) => check_meta_property(prop, node, ctx),

            AstKind::WithStatement(stmt) => check_with_statement(stmt, ctx),
            AstKind::SwitchStatement(stmt) => check_switch_statement(stmt, ctx),
            AstKind::BreakStatement(stmt) => check_break_statement(stmt, node, ctx),
            AstKind::ContinueStatement(stmt) => check_continue_statement(stmt, node, ctx),
            AstKind::LabeledStatement(stmt) => check_labeled_statement(stmt, node, ctx),
            AstKind::ForInStatement(stmt) => check_for_statement_left(&stmt.left, true, node, ctx),
            AstKind::ForOfStatement(stmt) => check_for_statement_left(&stmt.left, false, node, ctx),

            AstKind::Class(class) => check_class(class, ctx),
            AstKind::Super(sup) => check_super(sup, node, ctx),
            AstKind::ObjectProperty(prop) => check_object_property(prop, ctx),

            AstKind::FormalParameters(params) => check_formal_parameters(params, node, ctx),
            AstKind::ArrayPattern(pat) => check_array_pattern(pat, ctx),
            AstKind::ObjectExpression(expr) => check_object_expression(expr, ctx),
            AstKind::BinaryExpression(expr) => check_binary_expression(expr, ctx),
            AstKind::LogicalExpression(expr) => check_logical_expression(expr, ctx),
            AstKind::MemberExpression(expr) => check_member_expression(expr, ctx),
            AstKind::UnaryExpression(expr) => check_unary_expression(expr, node, ctx),
            AstKind::AwaitExpression(expr) => check_await_expression(expr, node, ctx),
            AstKind::YieldExpression(expr) => check_yield_expression(expr, node, ctx),
            _ => {}
        }
    }

    pub fn check_module_record(ctx: &SemanticBuilder<'_>) {
        check_module_record(ctx);
    }
}

fn check_module_record(ctx: &SemanticBuilder<'_>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Export '{0}' is not defined")]
    #[diagnostic()]
    struct UndefinedExport(Atom, #[label] Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error("Duplicated export '{0}'")]
    #[diagnostic()]
    struct DuplicateExport(
        Atom,
        #[label("Export has already been declared here")] Span,
        #[label("It cannot be redeclared here")] Span,
    );

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
        .filter(|name_span| ctx.scope.get_binding(ctx.current_scope_id, name_span.name()).is_none())
        .for_each(|name_span| {
            ctx.error(UndefinedExport(name_span.name().clone(), name_span.span()));
        });

    // It is a Syntax Error if the ExportedNames of ModuleItemList contains any duplicate entries.
    for name_span in &module_record.exported_bindings_duplicated {
        let old_span = module_record.exported_bindings[name_span.name()];
        ctx.error(DuplicateExport(name_span.name().clone(), name_span.span(), old_span));
    }

    for span in &module_record.export_default_duplicated {
        let old_span = module_record.export_default.unwrap();
        ctx.error(DuplicateExport("default".into(), *span, old_span));
    }

    // `export default x;`
    // `export { y as default };`
    if let (Some(span), Some(default_span)) =
        (module_record.exported_bindings.get("default"), &module_record.export_default)
    {
        ctx.error(DuplicateExport("default".into(), *default_span, *span));
    }
}

fn check_duplicate_bound_names<T: BoundNames>(bound_names: &T, ctx: &SemanticBuilder<'_>) {
    let mut idents: FxHashMap<Atom, Span> = FxHashMap::default();
    bound_names.bound_names(&mut |ident| {
        if let Some(old_span) = idents.insert(ident.name.clone(), ident.span) {
            ctx.error(Redeclaration(ident.name.clone(), old_span, ident.span));
        }
    });
}

#[derive(Debug, Error, Diagnostic)]
#[error("Cannot use await in class static initialization block")]
#[diagnostic()]
struct ClassStatickBlockAwait(#[label] Span);

#[derive(Debug, Error, Diagnostic)]
#[error("The keyword '{0}' is reserved")]
#[diagnostic()]
struct ReservedKeyword(Atom, #[label] Span);

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

fn check_identifier<'a>(name: &Atom, span: Span, node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) {
    if *name == "await" {
        // It is a Syntax Error if the goal symbol of the syntactic grammar is Module and the StringValue of IdentifierName is "await".
        if ctx.source_type.is_module() {
            return ctx.error(ReservedKeyword(name.clone(), span));
        }
        // It is a Syntax Error if ClassStaticBlockStatementList Contains await is true.
        if ctx.scope.get_flags(node.scope_id()).is_class_static_block() {
            return ctx.error(ClassStatickBlockAwait(span));
        }
    }

    // It is a Syntax Error if this phrase is contained in strict mode code and the StringValue of IdentifierName is: "implements", "interface", "let", "package", "private", "protected", "public", "static", or "yield".
    if ctx.strict_mode() && STRICT_MODE_NAMES.contains(name.as_str()) {
        ctx.error(ReservedKeyword(name.clone(), span));
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Cannot assign to '{0}' in strict mode")]
#[diagnostic()]
struct UnexpectedIdentifierAssign(Atom, #[label] Span);

fn check_binding_identifier<'a>(
    ident: &BindingIdentifier,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    let strict_mode = ctx.strict_mode();
    // It is a Diagnostic if the StringValue of a BindingIdentifier is "eval" or "arguments" within strict mode code.
    if strict_mode && matches!(ident.name.as_str(), "eval" | "arguments") {
        return ctx.error(UnexpectedIdentifierAssign(ident.name.clone(), ident.span));
    }

    // LexicalDeclaration : LetOrConst BindingList ;
    // * It is a Syntax Error if the BoundNames of BindingList contains "let".
    if !strict_mode && ident.name == "let" {
        for node_id in ctx.nodes.ancestors(node.id()).skip(1) {
            match ctx.nodes.kind(node_id) {
                AstKind::VariableDeclaration(decl) if decl.kind.is_lexical() => {
                    #[derive(Debug, Error, Diagnostic)]
                    #[error(
                        "`let` cannot be declared as a variable name inside of a `{0}` declaration"
                    )]
                    #[diagnostic()]
                    struct InvalidLetDeclaration(String, #[label] Span);
                    return ctx.error(InvalidLetDeclaration(decl.kind.to_string(), ident.span));
                }
                AstKind::VariableDeclaration(_) | AstKind::Function(_) | AstKind::Program(_) => {
                    break;
                }
                _ => {}
            }
        }
    }
}

fn check_identifier_reference<'a>(
    ident: &IdentifierReference,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("'arguments' is not allowed in {0}")]
    #[diagnostic()]
    struct UnexpectedArguments(&'static str, #[label] Span);

    //  Static Semantics: AssignmentTargetType
    //  1. If this IdentifierReference is contained in strict mode code and StringValue of Identifier is "eval" or "arguments", return invalid.
    if ctx.strict_mode() && matches!(ident.name.as_str(), "arguments" | "eval") {
        for node_id in ctx.nodes.ancestors(node.id()).skip(1) {
            match ctx.nodes.kind(node_id) {
                AstKind::AssignmentTarget(_) | AstKind::SimpleAssignmentTarget(_) => {
                    return ctx.error(UnexpectedIdentifierAssign(ident.name.clone(), ident.span));
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
                    return ctx.error(UnexpectedArguments("class field initializer", ident.span));
                }
                AstKind::StaticBlock(_) => {
                    return ctx
                        .error(UnexpectedArguments("static initialization block", ident.span));
                }
                _ => {}
            }
        }
    }
}

fn check_private_identifier<'a>(
    ident: &PrivateIdentifier,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    // Ignore private identifier declaration inside class
    if matches!(ctx.nodes.parent_kind(node.id()), Some(AstKind::PropertyKey(_))) {
        return;
    }

    // Find enclosing classes
    let mut classes = vec![];
    for node_id in ctx.nodes.ancestors(node.id()).skip(1) {
        let kind = ctx.nodes.kind(node_id);
        if let AstKind::Class(class) = kind {
            classes.push(class);
        }
        // stop lookup when the class is a heritage, e.g.
        // `class C extends class extends class { x = this.#foo; } {} { #foo }`
        // `class C extends function() { x = this.#foo; } { #foo }`
        if matches!(kind, AstKind::ClassHeritage(_)) {
            break;
        }
    }

    if classes.is_empty() {
        #[derive(Debug, Error, Diagnostic)]
        #[error("Private identifier '#{0}' is not allowed outside class bodies")]
        #[diagnostic()]
        struct PrivateNotInClass(Atom, #[label] Span);
        return ctx.error(PrivateNotInClass(ident.name.clone(), ident.span));
    };

    // Check private identifier declarations in class.
    // This implementations does a simple lookup for private identifier declarations inside a class.
    // Performance can be improved by storing private identifiers for each class inside a lookup table,
    // but there are not many private identifiers in the wild so we should be good fow now.
    let found_private_ident = classes.iter().any(|class| {
        class.body.body.iter().any(|def| {
            // let key = match def {
            // ClassElement::PropertyDefinition(def) => &def.key,
            // ClassElement::MethodDefinition(def) => &def.key,
            // _ => return false,
            // };
            matches!(def.property_key(), Some(PropertyKey::PrivateIdentifier(prop_ident))
                if prop_ident.name == ident.name)
        })
    });

    if !found_private_ident {
        #[derive(Debug, Error, Diagnostic)]
        #[error("Private field '{0}' must be declared in an enclosing class")]
        #[diagnostic()]
        struct PrivateFieldUndeclared(Atom, #[label] Span);
        ctx.error(PrivateFieldUndeclared(ident.name.clone(), ident.span));
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("'0'-prefixed octal literals and octal escape sequences are deprecated")]
#[diagnostic(help("for octal literals use the '0o' prefix instead"))]
struct LegacyOctal(#[label] Span);

fn check_number_literal(lit: &NumberLiteral, ctx: &SemanticBuilder<'_>) {
    // NumericLiteral :: LegacyOctalIntegerLiteral
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
                ctx.error(LegacyOctal(lit.span));
            }
            NumberBase::Decimal | NumberBase::Float if leading_zero(lit.raw) => {
                #[derive(Debug, Error, Diagnostic)]
                #[error("Decimals with leading zeros are not allowed in strict mode")]
                #[diagnostic(help("remove the leading zero"))]
                struct LeadingZeroDecimal(#[label] Span);
                ctx.error(LeadingZeroDecimal(lit.span));
            }
            _ => {}
        }
    }
}

fn check_string_literal(lit: &StringLiteral, ctx: &SemanticBuilder<'_>) {
    // 12.9.4.1 Static Semantics: Early Errors
    // EscapeSequence ::
    //   LegacyOctalEscapeSequence
    //   NonOctalDecimalEscapeSequence
    // It is a Syntax Error if the source text matched by this production is strict mode code.
    let raw = lit.span.source_text(ctx.source_text);
    if ctx.strict_mode() && raw.len() != lit.value.len() {
        let mut chars = raw.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('0') => {
                        if chars.peek().is_some_and(|c| ('1'..='9').contains(c)) {
                            return ctx.error(LegacyOctal(lit.span));
                        }
                    }
                    Some('1'..='7') => {
                        return ctx.error(LegacyOctal(lit.span));
                    }
                    Some('8'..='9') => {
                        #[derive(Debug, Error, Diagnostic)]
                        #[error("Invalid escape sequence")]
                        #[diagnostic(help("\\8 and \\9 are not allowed in strict mode"))]
                        struct NonOctalDecimalEscapeSequence(#[label] Span);
                        return ctx.error(NonOctalDecimalEscapeSequence(lit.span));
                    }
                    _ => {}
                }
            }
        }
    }
}

// It is a Syntax Error if FunctionBodyContainsUseStrict of AsyncFunctionBody is true and IsSimpleParameterList of FormalParameters is false.
// background: https://humanwhocodes.com/blog/2016/10/the-ecmascript-2016-change-you-probably-dont-know/
fn check_directive<'a>(directive: &Directive, node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Illegal 'use strict' directive in function with non-simple parameter list")]
    #[diagnostic()]
    struct IllegalUseStrict(#[label] Span);

    if directive.expression.value != "use strict" {
        return;
    }

    if !ctx.scope.get_flags(node.scope_id()).is_function() {
        return;
    }

    for node_id in ctx.nodes.ancestors(node.id()) {
        match ctx.nodes.kind(node_id) {
            AstKind::Function(Function { params, .. })
            | AstKind::ArrowExpression(ArrowExpression { params, .. }) => {
                if !params.is_simple_parameter_list() {
                    return ctx.error(IllegalUseStrict(directive.span));
                }
                break;
            }
            _ => {}
        }
    }
}

fn check_module_declaration<'a>(
    decl: &ModuleDeclaration,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("'{0}' declaration can only be used at the top level of a module")]
    #[diagnostic()]
    struct TopLevel(&'static str, #[label] Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error("Cannot use {0} outside a module")]
    #[diagnostic()]
    struct ModuleCode(&'static str, #[label] Span);

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
    let span = Span::new(decl.span().start, decl.span().start + 6);
    match ctx.source_type.module_kind() {
        ModuleKind::Script => {
            ctx.error(ModuleCode(text, span));
        }
        ModuleKind::Module => {
            if matches!(ctx.nodes.parent_kind(node.id()), Some(AstKind::Program(_))) {
                return;
            }
            ctx.error(TopLevel(text, span));
        }
    }
}

fn check_import_declaration(decl: &ImportDeclaration, ctx: &SemanticBuilder<'_>) {
    // ModuleItem : ImportDeclaration
    // It is a Syntax Error if the BoundNames of ImportDeclaration contains any duplicate entries.
    // bound_names are usually small, a simple loop should be more performant checking with a hashmap
    check_duplicate_bound_names(decl, ctx);
}

fn check_meta_property<'a>(prop: &MetaProperty, node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Unexpected new.target expression")]
    #[diagnostic(help(
        "new.target is only allowed in constructors and functions invoked using thew `new` operator"
    ))]
    struct NewTarget(#[label] Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error("The only valid meta property for new is new.target")]
    #[diagnostic()]
    struct NewTargetProperty(#[label] Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error("Unexpected import.meta expression")]
    #[diagnostic(help("import.meta is only allowed in module code"))]
    struct ImportMeta(#[label] Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error("The only valid meta property for import is import.meta")]
    #[diagnostic()]
    struct ImportMetaProperty(#[label] Span);

    match prop.meta.name.as_str() {
        "import" => {
            if prop.property.name == "meta" {
                if ctx.source_type.is_script() {
                    return ctx.error(ImportMeta(prop.span));
                }
                return;
            }
            ctx.error(ImportMetaProperty(prop.span));
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
                    return ctx.error(NewTarget(prop.span));
                }
                return;
            }
            ctx.error(NewTargetProperty(prop.span));
        }
        _ => {}
    }
}

fn check_function_declaration<'a>(
    kind: AstKind<'a>,
    _node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Invalid function declaration")]
    #[diagnostic(help(
        "In strict mode code, functions can only be declared at top level or inside a block"
    ))]
    struct FunctionDeclarationStrict(#[label] Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error("Invalid function declaration")]
    #[diagnostic(help(
        "In non-strict mode code, functions can only be declared at top level, inside a block, or as the body of an if statement"
    ))]
    struct FunctionDeclarationNonStrict(#[label] Span);

    // Function declaration not allowed in statement position
    let check = |stmt: &Statement<'a>| {
        if let Statement::Declaration(Declaration::FunctionDeclaration(decl)) = stmt {
            if ctx.strict_mode() {
                ctx.error(FunctionDeclarationStrict(decl.span));
            } else if !matches!(kind, AstKind::IfStatement(_) | AstKind::LabeledStatement(_)) {
                ctx.error(FunctionDeclarationNonStrict(decl.span));
            }
        }
    };

    match kind {
        AstKind::WithStatement(WithStatement { body, .. })
        | AstKind::WhileStatement(WhileStatement { body, .. })
        | AstKind::DoWhileStatement(DoWhileStatement { body, .. })
        | AstKind::ForStatement(ForStatement { body, .. })
        | AstKind::ForInStatement(ForInStatement { body, .. })
        | AstKind::ForOfStatement(ForOfStatement { body, .. })
        | AstKind::LabeledStatement(LabeledStatement { body, .. }) => {
            check(body);
        }
        AstKind::IfStatement(if_stmt) => {
            check(&if_stmt.consequent);
            if let Some(alternate) = &if_stmt.alternate {
                check(alternate);
            }
        }
        _ => {}
    }
}

fn check_regexp_literal(lit: &RegExpLiteral, ctx: &SemanticBuilder<'_>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("The 'u' and 'v' regular expression flags cannot be enabled at the same time")]
    #[diagnostic()]
    struct RegExpFlagUAndV(#[label] Span);

    let flags = lit.regex.flags;
    if flags.contains(RegExpFlags::U | RegExpFlags::V) {
        ctx.error(RegExpFlagUAndV(lit.span));
    }
}

fn check_with_statement(stmt: &WithStatement, ctx: &SemanticBuilder<'_>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("'with' statements are not allowed")]
    #[diagnostic()]
    struct WithStatement(#[label] Span);

    if ctx.strict_mode() || ctx.source_type.is_typescript() {
        ctx.error(WithStatement(Span::new(stmt.span.start, stmt.span.start + 4)));
    }
}

fn check_switch_statement<'a>(stmt: &SwitchStatement<'a>, ctx: &SemanticBuilder<'a>) {
    let mut previous_default: Option<Span> = None;
    for case in &stmt.cases {
        if case.test.is_none() {
            if let Some(previous_span) = previous_default {
                ctx.error(Redeclaration("default".into(), previous_span, case.span));
                break;
            }
            previous_default.replace(case.span);
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Jump target cannot cross function boundary.")]
#[diagnostic()]
struct InvalidLabelJumpTarget(#[label] Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Use of undefined label")]
#[diagnostic()]
struct InvalidLabelTarget(#[label("This label is used, but not defined")] Span);

fn check_break_statement<'a>(stmt: &BreakStatement, node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Illegal break statement")]
    #[diagnostic(help(
        "A `break` statement can only be used within an enclosing iteration or switch statement."
    ))]
    struct InvalidBreak(#[label] Span);

    // It is a Syntax Error if this BreakStatement is not nested, directly or indirectly (but not crossing function or static initialization block boundaries), within an IterationStatement or a SwitchStatement.
    for node_id in ctx.nodes.ancestors(node.id()).skip(1) {
        match ctx.nodes.kind(node_id) {
            AstKind::Program(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.error(InvalidBreak(stmt.span)),
                    |label| ctx.error(InvalidLabelTarget(label.span)),
                );
            }
            AstKind::Function(_) | AstKind::StaticBlock(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.error(InvalidBreak(stmt.span)),
                    |label| ctx.error(InvalidLabelJumpTarget(label.span)),
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

fn check_continue_statement<'a>(
    stmt: &ContinueStatement,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Illegal continue statement: no surrounding iteration statement")]
    #[diagnostic(help(
        "A `continue` statement can only be used within an enclosing `for`, `while` or `do while` "
    ))]
    struct InvalidContinue(#[label] Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error(
        "A `{0}` statement can only jump to a label of an enclosing `for`, `while` or `do while` statement."
    )]
    #[diagnostic()]
    struct InvalidLabelNonIteration(
        &'static str,
        #[label("This is an non-iteration statement")] Span,
        #[label("for this label")] Span,
    );

    // It is a Syntax Error if this ContinueStatement is not nested, directly or indirectly (but not crossing function or static initialization block boundaries), within an IterationStatement.
    for node_id in ctx.nodes.ancestors(node.id()).skip(1) {
        match ctx.nodes.kind(node_id) {
            AstKind::Program(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.error(InvalidContinue(stmt.span)),
                    |label| ctx.error(InvalidLabelTarget(label.span)),
                );
            }
            AstKind::Function(_) | AstKind::StaticBlock(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.error(InvalidContinue(stmt.span)),
                    |label| ctx.error(InvalidLabelJumpTarget(label.span)),
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
                    return ctx.error(InvalidLabelNonIteration(
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

fn check_labeled_statement<'a>(
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
                return ctx.error(Redeclaration(
                    stmt.label.name.clone(),
                    label_stmt.label.span,
                    stmt.label.span,
                ));
            }
            _ => {}
        }
    }
}

fn check_for_statement_left<'a>(
    left: &ForStatementLeft,
    is_for_in: bool,
    _node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Only a single declaration is allowed in a `for...{0}` statement")]
    #[diagnostic()]
    struct MultipleDeclarationInForLoopHead(&'static str, #[label] Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error("{0} loop variable declaration may not have an initializer")]
    #[diagnostic()]
    struct UnexpectedInitializerInForLoopHead(&'static str, #[label] Span);

    let ForStatementLeft::VariableDeclaration(decl) = left else { return };

    // initializer is not allowed for for-in / for-of
    if decl.declarations.len() > 1 {
        return ctx.error(MultipleDeclarationInForLoopHead(
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
            return ctx.error(UnexpectedInitializerInForLoopHead(
                if is_for_in { "for-in" } else { "for-of" },
                decl.span,
            ));
        }
    }
}

fn check_class(class: &Class, ctx: &SemanticBuilder<'_>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Multiple constructor implementations are not allowed.")]
    #[diagnostic()]
    struct DuplicateConstructor(
        #[label("constructor has already been declared here")] Span,
        #[label("it cannot be redeclared here")] Span,
    );

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
            return ctx.error(DuplicateConstructor(prev_span, new_span));
        }
        prev_constructor = Some(new_span);
    }
}

fn check_super<'a>(sup: &Super, node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("'super' can only be referenced in a derived class.")]
    #[diagnostic(help("either remove this super, or extend the class"))]
    struct SuperWithoutDerivedClass(#[label] Span, #[label("class does not have `extends`")] Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error("Super calls are not permitted outside constructors or in nested functions inside constructors.
")]
    #[diagnostic()]
    struct UnexpectedSuperCall(#[label] Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error("'super' can only be referenced in members of derived classes or object literal expressions.
")]
    #[diagnostic()]
    struct UnexpectedSuperReference(#[label] Span);

    let super_call_span = match ctx.nodes.parent_kind(node.id()) {
        Some(AstKind::CallExpression(expr)) => Some(expr.span),
        Some(AstKind::NewExpression(expr)) => Some(expr.span),
        _ => None,
    };

    // skip(1) is the self `Super`
    // skip(2) is the parent `CallExpression` or `NewExpression`
    for node_id in ctx.nodes.ancestors(node.id()).skip(2) {
        match ctx.nodes.kind(node_id) {
            AstKind::Class(class) => {
                // ClassTail : ClassHeritageopt { ClassBody }
                // It is a Syntax Error if ClassHeritage is not present and the following algorithm returns true:
                // 1. Let constructor be ConstructorMethod of ClassBody.
                // 2. If constructor is empty, return false.
                // 3. Return HasDirectSuper of constructor.
                if class.super_class.is_none() {
                    return ctx.error(SuperWithoutDerivedClass(sup.span, class.span));
                }
                break;
            }
            AstKind::MethodDefinition(def) => {
                // ClassElement : MethodDefinition
                // It is a Syntax Error if PropName of MethodDefinition is not "constructor" and HasDirectSuper of MethodDefinition is true.
                if let Some(super_call_span) = super_call_span {
                    if def.kind == MethodDefinitionKind::Constructor {
                        // pass through and let AstKind::Class check ClassHeritage
                    } else {
                        return ctx.error(UnexpectedSuperCall(super_call_span));
                    }
                } else {
                    // super references are allowed in method
                    break;
                }
            }
            // FieldDefinition : ClassElementName Initializer opt
            // * It is a Syntax Error if Initializer is present and Initializer Contains SuperCall is true.
            // PropertyDefinition : MethodDefinition
            // * It is a Syntax Error if HasDirectSuper of MethodDefinition is true.
            AstKind::PropertyDefinition(_) => {
                if let Some(super_call_span) = super_call_span {
                    return ctx.error(UnexpectedSuperCall(super_call_span));
                }
                break;
            }
            AstKind::ObjectProperty(prop) => {
                if prop.value.is_function() {
                    match super_call_span {
                        Some(super_call_span) if super_call_span.start > prop.key.span().end => {
                            return ctx.error(UnexpectedSuperCall(super_call_span));
                        }
                        _ => {
                            break;
                        }
                    }
                }
            }
            // ClassStaticBlockBody : ClassStaticBlockStatementList
            // * It is a Syntax Error if ClassStaticBlockStatementList Contains SuperCall is true.
            AstKind::StaticBlock(_) => {
                if let Some(super_call_span) = super_call_span {
                    return ctx.error(UnexpectedSuperCall(super_call_span));
                }
            }
            // ModuleBody : ModuleItemList
            // * It is a Syntax Error if ModuleItemList Contains super.
            // ScriptBody : StatementList
            // * It is a Syntax Error if StatementList Contains super
            AstKind::Program(_) => {
                return super_call_span.map_or_else(
                    || ctx.error(UnexpectedSuperReference(sup.span)),
                    |super_call_span| ctx.error(UnexpectedSuperCall(super_call_span)),
                );
            }
            _ => {}
        }
    }
}

fn check_object_property(prop: &ObjectProperty, ctx: &SemanticBuilder<'_>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Invalid assignment in object literal")]
    #[diagnostic(help(
        "Did you mean to use a ':'? An '=' can only follow a property name when the containing object literal is part of a destructuring pattern."
    ))]
    struct CoverInitializedName(#[label] Span);

    // PropertyDefinition : CoverInitializedName
    // It is a Syntax Error if any source text is matched by this production.
    if let Some(expr) = &prop.init {
        ctx.error(CoverInitializedName(expr.span()));
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("A rest parameter cannot have an initializer")]
#[diagnostic()]
struct ARestParameterCannotHaveAnInitializer(#[label] Span);

fn check_formal_parameters<'a>(
    params: &FormalParameters,
    _node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    if let Some(rest) = &params.rest {
        if let BindingPatternKind::AssignmentPattern(pat) = &rest.argument.kind {
            ctx.error(ARestParameterCannotHaveAnInitializer(pat.span));
        }
    }

    if params.is_empty() {
        return;
    }

    // Note: all other cases forbid duplicate parameter names.
    if params.kind == FormalParameterKind::FormalParameter
        && !ctx.strict_mode()
        && params.is_simple_parameter_list()
    {
        return;
    }

    check_duplicate_bound_names(params, ctx);
}

fn check_array_pattern(pattern: &ArrayPattern, ctx: &SemanticBuilder<'_>) {
    // function foo([...x = []]) { }
    //                    ^^^^ A rest element cannot have an initializer
    if let Some(rest) = &pattern.rest {
        if let BindingPatternKind::AssignmentPattern(pat) = &rest.argument.kind {
            ctx.error(ARestParameterCannotHaveAnInitializer(pat.span));
        }
    }
}

fn check_object_expression(obj_expr: &ObjectExpression, ctx: &SemanticBuilder<'_>) {
    // ObjectLiteral : { PropertyDefinitionList }
    // It is a Syntax Error if PropertyNameList of PropertyDefinitionList contains any duplicate entries for "__proto__"
    // and at least two of those entries were obtained from productions of the form PropertyDefinition : PropertyName : AssignmentExpression
    let mut prev_proto: Option<Span> = None;
    let prop_names = obj_expr.properties.iter().filter_map(PropName::prop_name);
    for prop_name in prop_names {
        if prop_name.0 == "__proto__" {
            if let Some(prev_span) = prev_proto {
                ctx.error(Redeclaration("__proto__".into(), prev_span, prop_name.1));
            }
            prev_proto = Some(prop_name.1);
        }
    }
}

fn check_binary_expression(binary_expr: &BinaryExpression, ctx: &SemanticBuilder<'_>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Unexpected exponentiation expression")]
    #[diagnostic(help("Wrap {0} expression in parentheses to enforce operator precedence"))]
    struct UnexpectedExponential(&'static str, #[label] Span);

    if binary_expr.operator == BinaryOperator::Exponential {
        match binary_expr.left {
            // async () => await 5 ** 6
            // async () => await -5 ** 6
            Expression::AwaitExpression(_) => {
                ctx.error(UnexpectedExponential("await", binary_expr.span));
            }
            // -5 ** 6
            Expression::UnaryExpression(_) => {
                ctx.error(UnexpectedExponential("unary", binary_expr.span));
            }
            _ => {}
        }
    }
}

fn check_logical_expression(logical_expr: &LogicalExpression, ctx: &SemanticBuilder<'_>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Logical expressions and coalesce expressions cannot be mixed")]
    #[diagnostic(help("Wrap either expression by parentheses"))]
    struct MixedCoalesce(#[label] Span);

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
                ctx.error(MixedCoalesce(logical_expr.span));
            }
        }
    }
}

fn check_member_expression(member_expr: &MemberExpression, ctx: &SemanticBuilder<'_>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Private fields cannot be accessed on super")]
    #[diagnostic()]
    struct SuperPrivate(#[label] Span);

    if let MemberExpression::PrivateFieldExpression(private_expr) = member_expr {
        // super.#m
        if let Expression::Super(_) = &private_expr.object {
            ctx.error(SuperPrivate(private_expr.span));
        }
    }
}

fn check_unary_expression<'a>(
    unary_expr: &'a UnaryExpression,
    _node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Delete of an unqualified identifier in strict mode.")]
    #[diagnostic()]
    struct DeleteOfUnqualified(#[label] Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error("Private fields can not be deleted")]
    #[diagnostic()]
    struct DeletePrivateField(#[label] Span);

    // https://tc39.es/ecma262/#sec-delete-operator-static-semantics-early-errors
    if unary_expr.operator == UnaryOperator::Delete {
        match unary_expr.argument.get_inner_expression() {
            Expression::Identifier(ident) if ctx.strict_mode() => {
                ctx.error(DeleteOfUnqualified(ident.span));
            }
            Expression::MemberExpression(expr) => {
                if let MemberExpression::PrivateFieldExpression(expr) = &**expr {
                    ctx.error(DeletePrivateField(expr.span));
                }
            }
            _ => {}
        }
    }
}

fn is_in_formal_parameters<'a>(node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) -> bool {
    for node_id in ctx.nodes.ancestors(node.id()).skip(1) {
        match ctx.nodes.kind(node_id) {
            AstKind::FormalParameters(_) => return true,
            AstKind::Program(_) | AstKind::Function(_) | AstKind::ArrowExpression(_) => break,
            _ => {}
        }
    }
    false
}

#[derive(Debug, Error, Diagnostic)]
#[error("{0} expression not allowed in formal parameter")]
#[diagnostic()]
struct AwaitOrYieldInParameter(
    &'static str,
    #[label("{0} expression not allowed in formal parameter")] Span,
);

fn check_await_expression<'a>(
    expr: &AwaitExpression,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    if is_in_formal_parameters(node, ctx) {
        ctx.error(AwaitOrYieldInParameter("await", expr.span));
    }
    // It is a Syntax Error if ClassStaticBlockStatementList Contains await is true.
    if ctx.scope.get_flags(node.scope_id()).is_class_static_block() {
        let start = expr.span.start;
        ctx.error(ClassStatickBlockAwait(Span::new(start, start + 5)));
    }
}

fn check_yield_expression<'a>(
    expr: &YieldExpression,
    node: &AstNode<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    if is_in_formal_parameters(node, ctx) {
        ctx.error(AwaitOrYieldInParameter("yield", expr.span));
    }
}
