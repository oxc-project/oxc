#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, syntax_directed_operations::PropName, AstKind, Atom, ModuleKind, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
    Redeclaration,
};

use crate::{ast_util::STRICT_MODE_NAMES, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct EarlyErrorJavaScript;

impl Rule for EarlyErrorJavaScript {
    #[allow(clippy::single_match)]
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.get().kind() {
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

            AstKind::NumberLiteral(lit) => check_number_literal(lit, node, ctx),
            AstKind::StringLiteral(lit) => check_string_literal(lit, node, ctx),
            AstKind::RegExpLiteral(lit) => check_regexp_literal(lit, ctx),

            AstKind::ModuleDeclaration(decl) => check_module_declaration(decl, node, ctx),

            AstKind::WithStatement(stmt) => check_with_statement(stmt, node, ctx),
            AstKind::SwitchStatement(stmt) => check_switch_statement(stmt, ctx),
            AstKind::BreakStatement(stmt) => check_break_statement(stmt, node, ctx),
            AstKind::ContinueStatement(stmt) => check_continue_statement(stmt, node, ctx),
            AstKind::LabeledStatement(stmt) => check_labeled_statement(stmt, node, ctx),
            AstKind::ForInStatement(stmt) => check_for_statement_left(&stmt.left, true, node, ctx),
            AstKind::ForOfStatement(stmt) => check_for_statement_left(&stmt.left, false, node, ctx),

            AstKind::Class(class) => check_class(class, ctx),
            AstKind::Super(sup) => check_super(sup, node, ctx),
            AstKind::Property(prop) => check_property(prop, ctx),

            AstKind::ObjectExpression(expr) => check_object_expression(expr, ctx),
            AstKind::BinaryExpression(expr) => check_binary_expression(expr, ctx),
            AstKind::LogicalExpression(expr) => check_logical_expression(expr, ctx),
            AstKind::MemberExpression(expr) => check_member_expression(expr, ctx),
            AstKind::UnaryExpression(expr) => check_unary_expression(expr, node, ctx),
            _ => {}
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("The keyword '{0}' is reserved")]
#[diagnostic()]
struct ReservedKeyword(Atom, #[label] Span);

fn check_identifier<'a>(name: &Atom, span: Span, node: &AstNode<'a>, ctx: &LintContext<'a>) {
    // if span.ctx.has_ambient() {
    // return None;
    // }

    // It is a Syntax Error if this production has an [Await] parameter.
    // if *name == "await" && span.ctx.has_await() {
    // return Some(Diagnostic::IdentifierAsync("await", span.range()));
    // }

    // It is a Syntax Error if the goal symbol of the syntactic grammar is Module and the StringValue of IdentifierName is "await".
    if *name == "await" && ctx.source_type().is_module() {
        return ctx.diagnostic(ReservedKeyword(name.clone(), span));
    }

    // It is a Syntax Error if this production has a [Yield] parameter.
    // if *name == "yield" && span.ctx.has_yield() {
    // return Some(Diagnostic::IdentifierGenerator("yield", span.range()));
    // }

    // It is a Syntax Error if this phrase is contained in strict mode code and the StringValue of IdentifierName is: "implements", "interface", "let", "package", "private", "protected", "public", "static", or "yield".
    if ctx.strict_mode(node) && STRICT_MODE_NAMES.contains(name.as_str()) {
        ctx.diagnostic(ReservedKeyword(name.clone(), span));
    }
}

fn check_binding_identifier<'a>(
    ident: &BindingIdentifier,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) {
    let strict_mode = ctx.strict_mode(node);
    // It is a Diagnostic if the StringValue of a BindingIdentifier is "eval" or "arguments" within strict mode code.
    // if strict_mode && !span.ctx.has_ambient() && matches!(name.as_str(), "eval" | "arguments") {
    // return Some(Diagnostic::UnexpectedIdentifierAssign(name.clone(), span.range()));
    // }

    // LexicalDeclaration : LetOrConst BindingList ;
    // * It is a Syntax Error if the BoundNames of BindingList contains "let".
    if !strict_mode && ident.name == "let" {
        for node_id in ctx.ancestors(node).skip(1) {
            match ctx.kind(node_id) {
                AstKind::VariableDeclaration(decl) if decl.kind.is_lexical() => {
                    #[derive(Debug, Error, Diagnostic)]
                    #[error(
                        "`let` cannot be declared as a variable name inside of a `{0}` declaration"
                    )]
                    #[diagnostic()]
                    struct InvalidLetDeclaration(String, #[label] Span);
                    return ctx
                        .diagnostic(InvalidLetDeclaration(decl.kind.to_string(), ident.span));
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
    ctx: &LintContext<'a>,
) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Cannot assign to '{0}' in strict mode")]
    #[diagnostic()]
    struct UnexpectedIdentifierAssign(Atom, #[label] Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error("'arguments' is not allowed in {0}")]
    #[diagnostic()]
    struct UnexpectedArguments(&'static str, #[label] Span);

    //  Static Semantics: AssignmentTargetType
    //  1. If this IdentifierReference is contained in strict mode code and StringValue of Identifier is "eval" or "arguments", return invalid.
    if ctx.strict_mode(node) && matches!(ident.name.as_str(), "arguments" | "eval") {
        for node_id in ctx.ancestors(node).skip(1) {
            match ctx.kind(node_id) {
                AstKind::AssignmentTarget(_) | AstKind::SimpleAssignmentTarget(_) => {
                    return ctx
                        .diagnostic(UnexpectedIdentifierAssign(ident.name.clone(), ident.span));
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
        for node_id in ctx.ancestors(node).skip(1) {
            match ctx.kind(node_id) {
                AstKind::Function(_) => break,
                AstKind::PropertyDefinition(_) => {
                    return ctx
                        .diagnostic(UnexpectedArguments("class field initializer", ident.span));
                }
                AstKind::StaticBlock(_) => {
                    return ctx.diagnostic(UnexpectedArguments(
                        "static initialization block",
                        ident.span,
                    ));
                }
                _ => {}
            }
        }
    }
}

fn check_private_identifier<'a>(
    ident: &PrivateIdentifier,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) {
    // Ignore private identifier declaration inside class
    if matches!(ctx.parent_kind(node), AstKind::PropertyKey(_)) {
        return;
    }

    // Find enclosing classes
    let mut classes = vec![];
    for node_id in ctx.ancestors(node).skip(1) {
        let kind = ctx.kind(node_id);
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
        return ctx.diagnostic(PrivateNotInClass(ident.name.clone(), ident.span));
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
            if let Some(key) = def.property_key()
                && let PropertyKey::PrivateIdentifier(prop_ident) = key {
                return prop_ident.name == ident.name;
            }
            false
        })
    });

    if !found_private_ident {
        #[derive(Debug, Error, Diagnostic)]
        #[error("Private field '{0}' must be declared in an enclosing class")]
        #[diagnostic()]
        struct PrivateFieldUndeclared(Atom, #[label] Span);
        ctx.diagnostic(PrivateFieldUndeclared(ident.name.clone(), ident.span));
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("'0'-prefixed octal literals and octal escape sequences are deprecated")]
#[diagnostic(help("for octal literals use the '0o' prefix instead"))]
struct LegacyOctal(#[label] Span);

fn check_number_literal(lit: &NumberLiteral, node: &AstNode, ctx: &LintContext) {
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

    if ctx.strict_mode(node) {
        match lit.base {
            NumberBase::Octal if leading_zero(lit.raw) => {
                ctx.diagnostic(LegacyOctal(lit.span));
            }
            NumberBase::Decimal if leading_zero(lit.raw) => {
                #[derive(Debug, Error, Diagnostic)]
                #[error("Decimals with leading zeros are not allowed in strict mode")]
                #[diagnostic(help("remove the leading zero"))]
                struct LeadingZeroDecimal(#[label] Span);
                ctx.diagnostic(LeadingZeroDecimal(lit.span));
            }
            _ => {}
        }
    }
}

fn check_string_literal<'a>(lit: &StringLiteral, node: &AstNode<'a>, ctx: &LintContext<'a>) {
    // 12.9.4.1 Static Semantics: Early Errors
    // EscapeSequence ::
    //   LegacyOctalEscapeSequence
    //   NonOctalDecimalEscapeSequence
    // It is a Syntax Error if the source text matched by this production is strict mode code.
    let raw = lit.span.source_text(ctx.source_text());
    if ctx.strict_mode(node) && raw.len() != lit.value.len() {
        let mut chars = raw.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('0') => {
                        if chars.peek().is_some_and(|c| ('1'..='9').contains(c)) {
                            return ctx.diagnostic(LegacyOctal(lit.span));
                        }
                    }
                    Some('1'..='7') => {
                        return ctx.diagnostic(LegacyOctal(lit.span));
                    }
                    Some('8'..='9') => {
                        #[derive(Debug, Error, Diagnostic)]
                        #[error("Invalid escape sequence")]
                        #[diagnostic(help("\\8 and \\9 are not allowed in strict mode"))]
                        struct NonOctalDecimalEscapeSequence(#[label] Span);
                        return ctx.diagnostic(NonOctalDecimalEscapeSequence(lit.span));
                    }
                    _ => {}
                }
            }
        }
    }
}

fn check_module_declaration<'a>(
    decl: &ModuleDeclaration,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("'{0}' declaration can only be used at the top level of a module")]
    #[diagnostic()]
    struct TopLevel(&'static str, #[label] Span);

    #[derive(Debug, Error, Diagnostic)]
    #[error("Cannot use {0} outside a module")]
    #[diagnostic()]
    struct ModuleCode(&'static str, #[label] Span);

    if ctx.source_type().is_typescript_definition()
        || ctx.scope(node).is_ts_module()
        || matches!(ctx.parent_kind(node), AstKind::Program(_))
    {
        return;
    }
    let text = match decl.kind {
        ModuleDeclarationKind::ImportDeclaration(_) => "import statement",
        ModuleDeclarationKind::ExportAllDeclaration(_)
        | ModuleDeclarationKind::ExportDefaultDeclaration(_)
        | ModuleDeclarationKind::ExportNamedDeclaration(_)
        | ModuleDeclarationKind::TSExportAssignment(_)
        | ModuleDeclarationKind::TSNamespaceExportDeclaration(_) => "export statement",
    };
    let span = Span::new(decl.span.start, decl.span.start + 6);
    match ctx.source_type().module_kind() {
        ModuleKind::Script => {
            ctx.diagnostic(ModuleCode(text, span));
        }
        ModuleKind::Module => {
            ctx.diagnostic(TopLevel(text, span));
        }
    }
}

fn check_regexp_literal(lit: &RegExpLiteral, ctx: &LintContext) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("The 'u' and 'v' regular expression flags cannot be enabled at the same time")]
    #[diagnostic()]
    struct RegExpFlagUAndV(#[label] Span);

    let flags = lit.regex.flags;
    if flags.contains(RegExpFlags::U | RegExpFlags::V) {
        ctx.diagnostic(RegExpFlagUAndV(lit.span));
    }
}

fn check_with_statement<'a>(stmt: &WithStatement, node: &AstNode<'a>, ctx: &LintContext<'a>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("'with' statements are not allowed")]
    #[diagnostic()]
    struct WithStatement(#[label] Span);

    if ctx.strict_mode(node) || ctx.source_type().is_typescript() {
        ctx.diagnostic(WithStatement(Span::new(stmt.span.start, stmt.span.start + 4)));
    }
}

fn check_switch_statement<'a>(stmt: &SwitchStatement<'a>, ctx: &LintContext<'a>) {
    let mut previous_default: Option<Span> = None;
    for case in &stmt.cases {
        if case.test.is_none() {
            if let Some(previous_span) = previous_default {
                ctx.diagnostic(Redeclaration("default".into(), previous_span, case.span));
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

fn check_break_statement<'a>(stmt: &BreakStatement, node: &AstNode<'a>, ctx: &LintContext<'a>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Illegal break statement")]
    #[diagnostic(help(
        "A `break` statement can only be used within an enclosing iteration or switch statement."
    ))]
    struct InvalidBreak(#[label] Span);

    // It is a Syntax Error if this BreakStatement is not nested, directly or indirectly (but not crossing function or static initialization block boundaries), within an IterationStatement or a SwitchStatement.
    for node_id in ctx.ancestors(node).skip(1) {
        match ctx.kind(node_id) {
            AstKind::Program(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.diagnostic(InvalidBreak(stmt.span)),
                    |label| ctx.diagnostic(InvalidLabelTarget(label.span)),
                );
            }
            AstKind::Function(_) | AstKind::StaticBlock(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.diagnostic(InvalidBreak(stmt.span)),
                    |label| ctx.diagnostic(InvalidLabelJumpTarget(label.span)),
                );
            }
            AstKind::LabeledStatement(labeled_statement) => {
                if let Some(label) = &stmt.label
                    && label.name == labeled_statement.label.name {
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
    ctx: &LintContext<'a>,
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
    for node_id in ctx.ancestors(node).skip(1) {
        match ctx.kind(node_id) {
            AstKind::Program(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.diagnostic(InvalidContinue(stmt.span)),
                    |label| ctx.diagnostic(InvalidLabelTarget(label.span)),
                );
            }
            AstKind::Function(_) | AstKind::StaticBlock(_) => {
                return stmt.label.as_ref().map_or_else(
                    || ctx.diagnostic(InvalidContinue(stmt.span)),
                    |label| ctx.diagnostic(InvalidLabelJumpTarget(label.span)),
                );
            }
            AstKind::LabeledStatement(labeled_statement) => {
                if let Some(label) = &stmt.label
                    && label.name == labeled_statement.label.name {
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
                    return ctx.diagnostic(InvalidLabelNonIteration(
                            "continue",
                            labeled_statement.label.span,
                            label.span,
                    ));
                }
            }
            kind if kind.is_iteration_statement() && stmt.label.is_none() => break,
            _ => {}
        }
    }
}

fn check_labeled_statement<'a>(stmt: &LabeledStatement, node: &AstNode<'a>, ctx: &LintContext<'a>) {
    for node_id in ctx.ancestors(node).skip(1) {
        match ctx.kind(node_id) {
            // label cannot cross boundary on function or static block
            AstKind::Function(_) | AstKind::StaticBlock(_) | AstKind::Program(_) => break,
            // check label name redeclaration
            AstKind::LabeledStatement(label_stmt) if stmt.label.name == label_stmt.label.name => {
                return ctx.diagnostic(Redeclaration(
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
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
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
        return ctx.diagnostic(MultipleDeclarationInForLoopHead(
            if is_for_in { "in" } else { "of" },
            decl.span,
        ));
    }

    let strict_mode = ctx.strict_mode(node);
    for declarator in &decl.declarations {
        if declarator.init.is_some()
            && (strict_mode
                || !is_for_in
                || decl.kind.is_lexical()
                || !matches!(declarator.id.kind, BindingPatternKind::BindingIdentifier(_)))
        {
            return ctx.diagnostic(UnexpectedInitializerInForLoopHead(
                if is_for_in { "for-in" } else { "for-of" },
                decl.span,
            ));
        }
    }
}

fn check_class(class: &Class, ctx: &LintContext) {
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
            return ctx.diagnostic(DuplicateConstructor(prev_span, new_span));
        }
        prev_constructor = Some(new_span);
    }
}

fn check_super<'a>(sup: &Super, node: &AstNode<'a>, ctx: &LintContext<'a>) {
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

    let super_call_span = match ctx.parent_kind(node) {
        AstKind::CallExpression(expr) => Some(expr.span),
        AstKind::NewExpression(expr) => Some(expr.span),
        _ => None,
    };

    // skip(1) is the self `Super`
    // skip(2) is the parent `CallExpression` or `NewExpression`
    for node_id in ctx.ancestors(node).skip(2) {
        match ctx.kind(node_id) {
            AstKind::Class(class) => {
                // ClassTail : ClassHeritageopt { ClassBody }
                // It is a Syntax Error if ClassHeritage is not present and the following algorithm returns true:
                // 1. Let constructor be ConstructorMethod of ClassBody.
                // 2. If constructor is empty, return false.
                // 3. Return HasDirectSuper of constructor.
                if class.super_class.is_none() {
                    return ctx.diagnostic(SuperWithoutDerivedClass(sup.span, class.span));
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
                        return ctx.diagnostic(UnexpectedSuperCall(super_call_span));
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
                    return ctx.diagnostic(UnexpectedSuperCall(super_call_span));
                }
                break;
            }
            AstKind::PropertyValue(value) => {
                if let PropertyValue::Expression(
                    Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_),
                ) = value
                {
                    if let Some(super_call_span) = super_call_span {
                        return ctx.diagnostic(UnexpectedSuperCall(super_call_span));
                    }
                    break;
                }
            }
            // ClassStaticBlockBody : ClassStaticBlockStatementList
            // * It is a Syntax Error if ClassStaticBlockStatementList Contains SuperCall is true.
            AstKind::StaticBlock(_) => {
                if let Some(super_call_span) = super_call_span {
                    return ctx.diagnostic(UnexpectedSuperCall(super_call_span));
                }
            }
            // ModuleBody : ModuleItemList
            // * It is a Syntax Error if ModuleItemList Contains super.
            // ScriptBody : StatementList
            // * It is a Syntax Error if StatementList Contains super
            AstKind::Program(_) => {
                return super_call_span.map_or_else(
                    || ctx.diagnostic(UnexpectedSuperReference(sup.span)),
                    |super_call_span| ctx.diagnostic(UnexpectedSuperCall(super_call_span)),
                );
            }
            _ => {}
        }
    }
}

fn check_property(prop: &Property, ctx: &LintContext) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Invalid assignment in object literal")]
    #[diagnostic(help(
        "Did you mean to use a ':'? An '=' can only follow a property name when the containing object literal is part of a destructuring pattern."
    ))]
    struct CoverInitializedName(#[label] Span);

    // PropertyDefinition : CoverInitializedName
    // It is a Syntax Error if any source text is matched by this production.
    if prop.shorthand {
        if let PropertyValue::Expression(Expression::AssignmentExpression(expr)) = &prop.value {
            ctx.diagnostic(CoverInitializedName(expr.span));
        }
    }
}

fn check_object_expression(obj_expr: &ObjectExpression, ctx: &LintContext) {
    // ObjectLiteral : { PropertyDefinitionList }
    // It is a Syntax Error if PropertyNameList of PropertyDefinitionList contains any duplicate entries for "__proto__"
    // and at least two of those entries were obtained from productions of the form PropertyDefinition : PropertyName : AssignmentExpression
    let mut prev_proto: Option<Span> = None;
    let prop_names = obj_expr.properties.iter().filter_map(PropName::prop_name);
    for prop_name in prop_names {
        if prop_name.0 == "__proto__" {
            if let Some(prev_span) = prev_proto {
                ctx.diagnostic(Redeclaration("__proto__".into(), prev_span, prop_name.1));
            }
            prev_proto = Some(prop_name.1);
        }
    }
}

fn check_binary_expression(binary_expr: &BinaryExpression, ctx: &LintContext) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Unexpected exponentiation expression")]
    #[diagnostic(help("Wrap {0} expression in parentheses to enforce operator precedence"))]
    struct UnexpectedExponential(&'static str, #[label] Span);

    if binary_expr.operator == BinaryOperator::Exponential {
        match binary_expr.left {
            // async () => await 5 ** 6
            // async () => await -5 ** 6
            Expression::AwaitExpression(_) => {
                ctx.diagnostic(UnexpectedExponential("await", binary_expr.span));
            }
            // -5 ** 6
            Expression::UnaryExpression(_) => {
                ctx.diagnostic(UnexpectedExponential("unary", binary_expr.span));
            }
            _ => {}
        }
    }
}

fn check_logical_expression(logical_expr: &LogicalExpression, ctx: &LintContext) {
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
                ctx.diagnostic(MixedCoalesce(logical_expr.span));
            }
        }
    }
}

fn check_member_expression(member_expr: &MemberExpression, ctx: &LintContext) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Private fields cannot be accessed on super")]
    #[diagnostic()]
    struct SuperPrivate(#[label] Span);

    if let MemberExpression::PrivateFieldExpression(private_expr) = member_expr {
        // super.#m
        if let Expression::Super(_) = &private_expr.object {
            ctx.diagnostic(SuperPrivate(private_expr.span));
        }
    }
}

fn check_unary_expression<'a>(
    unary_expr: &'a UnaryExpression,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
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
            Expression::Identifier(ident) if ctx.strict_mode(node) => {
                ctx.diagnostic(DeleteOfUnqualified(ident.span));
            }
            Expression::MemberExpression(expr) => {
                if let MemberExpression::PrivateFieldExpression(expr) = &**expr {
                    ctx.diagnostic(DeletePrivateField(expr.span));
                }
            }
            _ => {}
        }
    }
}
