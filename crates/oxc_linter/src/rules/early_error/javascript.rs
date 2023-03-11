#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, syntax_directed_operations::PropName, AstKind, Atom, Span};
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
            AstKind::WithStatement(stmt) => check_with_statement(stmt, node, ctx),
            AstKind::BreakStatement(stmt) => check_break_statement(stmt, node, ctx),
            AstKind::ContinueStatement(stmt) => check_continue_statement(stmt, node, ctx),
            AstKind::LabeledStatement(stmt) => check_labeled_statement(stmt, node, ctx),
            AstKind::Class(class) => check_class(class, ctx),
            AstKind::Super(sup) => check_super(sup, node, ctx),
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
