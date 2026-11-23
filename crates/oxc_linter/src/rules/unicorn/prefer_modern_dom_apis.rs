use std::borrow::Cow;

use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, Declaration, ExportDefaultDeclarationKind, Expression, Statement,
        TSType, TSTypeName, WhileStatement,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::precedence::Precedence;

use crate::{
    AstNode,
    ast_util::is_method_call,
    context::LintContext,
    rule::Rule,
    utils::{get_precedence, is_same_expression},
};

fn prefer_modern_dom_apis_diagnostic(
    good_method: &str,
    bad_method: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer using `{good_method}` over `{bad_method}`."))
        .with_label(span)
}

fn prefer_replace_children_diagnostic(
    parent_node: &str,
    child_prop: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Prefer `{parent_node}.replaceChildren()` over directly removing `.{child_prop}` in a loop."
    ))
    .with_help(format!("Replace the loop with `{parent_node}.replaceChildren()`."))
    .with_label(span)
}

const NON_PARENT_NODE_TYPE_NAMES: &[&str] = &[
    "Attr",
    "CDATASection",
    "CharacterData",
    "ChildNode",
    "Comment",
    "DocumentType",
    "Node",
    "Text",
    "Window",
    "WindowProxy",
];

const TARGET_PARENT_NODE_TYPE_NAMES: &[&str] = &[
    "Document",
    "DocumentFragment",
    "Element",
    "HTMLDocument",
    "HTMLElement",
    "ParentNode",
    "SVGElement",
    "ShadowRoot",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReceiverTypeKind {
    Target,
    NonTarget,
    Nullish,
    Unknown,
}

#[derive(Debug, Default, Clone)]
pub struct PreferModernDomApis;

fn get_replacement_for_disallowed_method(method: &str) -> Option<&'static str> {
    match method {
        "replaceChild" => Some("replaceWith"),
        "insertBefore" => Some("before"),
        _ => None,
    }
}

fn get_replacement_for_position(position: &str) -> Option<&'static str> {
    match position {
        "beforebegin" => Some("before"),
        "afterbegin" => Some("prepend"),
        "beforeend" => Some("append"),
        "afterend" => Some("after"),
        _ => None,
    }
}

fn is_value_not_usable(node: &AstNode, ctx: &LintContext) -> bool {
    let parent_node = ctx.nodes().parent_node(node.id());
    let grandparent_node = ctx.nodes().parent_node(parent_node.id());
    matches!(
        (parent_node.kind(), grandparent_node.kind()),
        (AstKind::ExpressionStatement(_), _)
            | (AstKind::ChainExpression(_), AstKind::ExpressionStatement(_))
    )
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of:
    /// - `childNode.replaceWith(newNode)` over `parentNode.replaceChild(newNode, oldNode)`
    /// - `referenceNode.before(newNode)` over `parentNode.insertBefore(newNode, referenceNode)`
    /// - `referenceNode.before('text')` over `referenceNode.insertAdjacentText('beforebegin', 'text')`
    /// - `referenceNode.before(newNode)` over `referenceNode.insertAdjacentElement('beforebegin', newNode)`
    /// - `parentNode.replaceChildren()` over `while (parentNode.firstChild) { parentNode.firstChild.remove(); }`
    ///
    /// ### Why is this bad?
    ///
    /// There are some advantages of using the newer DOM APIs, like:
    /// - Traversing to the parent node is not necessary.
    /// - Appending multiple nodes at once.
    /// - Both `DOMString` and DOM node objects can be manipulated.
    /// - Clearing all children with `replaceChildren()` is clearer and avoids removing nodes in a loop.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// parentNode.replaceChild(newChildNode, oldChildNode);
    ///
    /// while (node.firstChild) {
    ///     node.firstChild.remove();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// oldChildNode.replaceWith(newChildNode);
    ///
    /// node.replaceChildren();
    /// ```
    PreferModernDomApis,
    unicorn,
    style,
    recommended,
    conditional_suggestion,
    version = "0.0.20",
    short_description = "Prefer modern DOM APIs like `replaceWith`, `before`, and `replaceChildren` over older methods.",
);

impl Rule for PreferModernDomApis {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call_expr) => Self::check_call_expression(node, call_expr, ctx),
            AstKind::WhileStatement(while_stmt) => {
                Self::check_while_statement(node, while_stmt, ctx);
            }
            _ => {}
        }
    }
}

impl PreferModernDomApis {
    fn check_call_expression<'a>(
        node: &AstNode<'a>,
        call_expr: &CallExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Expression::StaticMemberExpression(member_expr) = &call_expr.callee else {
            return;
        };
        let method = member_expr.property.name.as_str();

        if is_method_call(
            call_expr,
            None,
            Some(&["replaceChild", "insertBefore"]),
            Some(2),
            Some(2),
        ) && matches!(member_expr.object, Expression::Identifier(_))
            && !call_expr.optional
            && let Some(preferred_method) = get_replacement_for_disallowed_method(method)
        {
            let diagnostic = prefer_modern_dom_apis_diagnostic(
                preferred_method,
                method,
                member_expr.property.span,
            );

            // Reordering complex arguments can change evaluation semantics.
            let has_safe_arguments = call_expr.arguments.iter().all(|argument| {
                matches!(argument, Argument::Identifier(identifier) if identifier.name != "undefined")
            });

            if has_safe_arguments && is_value_not_usable(node, ctx) {
                ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                    let new_node = ctx.source_range(call_expr.arguments[0].span());
                    let old_node = ctx.source_range(call_expr.arguments[1].span());

                    let replacement = format!("{old_node}.{preferred_method}({new_node})");

                    fixer.replace(call_expr.span, replacement)
                });
            } else {
                ctx.diagnostic(diagnostic);
            }

            return;
        }

        if is_method_call(
            call_expr,
            None,
            Some(&["insertAdjacentText", "insertAdjacentElement"]),
            Some(2),
            Some(2),
        ) && let Argument::StringLiteral(lit) = &call_expr.arguments[0]
            && let Some(preferred_method) =
                lit.value.as_str().and_then(get_replacement_for_position)
        {
            let diagnostic = prefer_modern_dom_apis_diagnostic(
                preferred_method,
                method,
                member_expr.property.span,
            );

            let can_fix = method == "insertAdjacentText" || is_value_not_usable(node, ctx);

            if can_fix {
                ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                    let content = ctx.source_range(call_expr.arguments[1].span());
                    let reference = ctx.source_range(member_expr.object.span());

                    let replacement = format!("{reference}.{preferred_method}({content})");

                    fixer.replace(call_expr.span, replacement)
                });
            } else {
                ctx.diagnostic(diagnostic);
            }
        }
    }

    fn check_while_statement<'a>(
        node: &AstNode<'a>,
        while_stmt: &WhileStatement<'a>,
        ctx: &LintContext<'a>,
    ) {
        let test_expr = while_stmt.test.without_parentheses();
        let Expression::StaticMemberExpression(child_member) = test_expr else {
            return;
        };
        if child_member.optional {
            return;
        }
        let child_prop = child_member.property.name.as_str();
        if !matches!(child_prop, "firstChild" | "lastChild") {
            return;
        }

        let body_stmt = match &while_stmt.body {
            Statement::BlockStatement(block) if block.body.len() == 1 => &block.body[0],
            Statement::BlockStatement(_) => return,
            stmt => stmt,
        };
        let Statement::ExpressionStatement(expr_stmt) = body_stmt else {
            return;
        };
        let Expression::CallExpression(call_expr) = expr_stmt.expression.without_parentheses()
        else {
            return;
        };
        if call_expr.optional
            || !call_expr.arguments.is_empty()
            || call_expr.type_arguments.is_some()
        {
            return;
        }
        let Expression::StaticMemberExpression(remove_member) =
            call_expr.callee.without_parentheses()
        else {
            return;
        };
        if remove_member.optional || remove_member.property.name != "remove" {
            return;
        }

        let remove_object = remove_member.object.without_parentheses();
        if !is_non_optional_reference(&child_member.object)
            || !is_non_optional_reference(remove_object)
            || !is_same_expression(test_expr, remove_object, ctx)
        {
            return;
        }

        let parent_expr = child_member.object.without_parentheses();
        if is_known_non_replace_children_receiver(parent_expr, node, ctx) {
            return;
        }

        let raw_parent = ctx.source_range(parent_expr.span());
        let needs_parens = !matches!(parent_expr, Expression::Super(_))
            && (get_precedence(parent_expr).is_some_and(|p| p < Precedence::Member)
                || starts_with_unparenthesized_let_bracket(parent_expr));
        let parent_node_text: Cow<'_, str> = if needs_parens {
            Cow::Owned(format!("({raw_parent})"))
        } else {
            Cow::Borrowed(raw_parent)
        };

        let diagnostic =
            prefer_replace_children_diagnostic(&parent_node_text, child_prop, while_stmt.span);

        let has_comments = ctx
            .has_comments_between(Span::new(while_stmt.span.start, parent_expr.span().start))
            || ctx.has_comments_between(Span::new(parent_expr.span().end, while_stmt.span.end));

        if has_comments {
            ctx.diagnostic(diagnostic);
            return;
        }

        let leading_semicolon = if parent_node_text.starts_with('(')
            && needs_leading_semicolon_before_while(node, while_stmt.span, ctx)
        {
            ";"
        } else {
            ""
        };
        let replacement = format!("{leading_semicolon}{parent_node_text}.replaceChildren();");

        ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
            fixer.replace(while_stmt.span, replacement)
        });
    }
}

fn classify_ts_type(ty: &TSType) -> ReceiverTypeKind {
    match ty {
        TSType::TSParenthesizedType(paren) => classify_ts_type(&paren.type_annotation),
        TSType::TSNullKeyword(_) | TSType::TSUndefinedKeyword(_) => ReceiverTypeKind::Nullish,
        TSType::TSBigIntKeyword(_)
        | TSType::TSBooleanKeyword(_)
        | TSType::TSNeverKeyword(_)
        | TSType::TSNumberKeyword(_)
        | TSType::TSStringKeyword(_)
        | TSType::TSSymbolKeyword(_)
        | TSType::TSVoidKeyword(_)
        | TSType::TSArrayType(_)
        | TSType::TSLiteralType(_)
        | TSType::TSTupleType(_)
        | TSType::TSTypeLiteral(_)
        | TSType::TSFunctionType(_)
        | TSType::TSConstructorType(_) => ReceiverTypeKind::NonTarget,
        TSType::TSTypeReference(type_ref) => {
            if let TSTypeName::IdentifierReference(ident) = &type_ref.type_name {
                classify_type_name(&ident.name)
            } else {
                ReceiverTypeKind::Unknown
            }
        }
        TSType::TSUnionType(union) => {
            let mut has_non_nullish = false;
            let mut all_target = true;
            let mut any_non_target = false;

            for member in &union.types {
                match classify_ts_type(member) {
                    ReceiverTypeKind::Nullish => {}
                    ReceiverTypeKind::Target => {
                        has_non_nullish = true;
                    }
                    ReceiverTypeKind::NonTarget => {
                        has_non_nullish = true;
                        all_target = false;
                        any_non_target = true;
                    }
                    ReceiverTypeKind::Unknown => {
                        has_non_nullish = true;
                        all_target = false;
                    }
                }
            }

            if !has_non_nullish || any_non_target {
                ReceiverTypeKind::NonTarget
            } else if all_target {
                ReceiverTypeKind::Target
            } else {
                ReceiverTypeKind::Unknown
            }
        }
        TSType::TSIntersectionType(intersection) => {
            let mut has_target = false;
            for member in &intersection.types {
                match classify_ts_type(member) {
                    ReceiverTypeKind::Unknown => return ReceiverTypeKind::Unknown,
                    ReceiverTypeKind::Target => has_target = true,
                    ReceiverTypeKind::NonTarget | ReceiverTypeKind::Nullish => {}
                }
            }
            if has_target { ReceiverTypeKind::Target } else { ReceiverTypeKind::NonTarget }
        }
        _ => ReceiverTypeKind::Unknown,
    }
}

fn classify_type_name(name: &str) -> ReceiverTypeKind {
    if NON_PARENT_NODE_TYPE_NAMES.contains(&name) {
        ReceiverTypeKind::NonTarget
    } else if TARGET_PARENT_NODE_TYPE_NAMES.contains(&name) {
        ReceiverTypeKind::Target
    } else {
        ReceiverTypeKind::Unknown
    }
}

fn is_any_or_unknown_ts_type(ty: &TSType) -> bool {
    matches!(ty.without_parenthesized(), TSType::TSAnyKeyword(_) | TSType::TSUnknownKeyword(_))
}

fn unwrap_parens_and_ts_assertions<'a>(mut expr: &'a Expression<'a>) -> &'a Expression<'a> {
    loop {
        match expr {
            Expression::ParenthesizedExpression(e) => expr = &e.expression,
            Expression::TSAsExpression(e) => expr = &e.expression,
            Expression::TSTypeAssertion(e) => expr = &e.expression,
            Expression::TSNonNullExpression(e) => expr = &e.expression,
            Expression::TSSatisfiesExpression(e) => expr = &e.expression,
            _ => return expr,
        }
    }
}

fn is_non_optional_reference(expr: &Expression) -> bool {
    match unwrap_parens_and_ts_assertions(expr) {
        Expression::Identifier(_) | Expression::ThisExpression(_) | Expression::Super(_) => true,
        Expression::StaticMemberExpression(member) => {
            !member.optional && is_non_optional_reference(&member.object)
        }
        Expression::PrivateFieldExpression(member) => {
            !member.optional && is_non_optional_reference(&member.object)
        }
        Expression::ComputedMemberExpression(member) => {
            !member.optional
                && is_non_optional_reference(&member.object)
                && is_reference_key(&member.expression)
        }
        _ => false,
    }
}

fn is_reference_key(expr: &Expression) -> bool {
    let expr = unwrap_parens_and_ts_assertions(expr);
    expr.is_literal() || expr.is_no_substitution_template() || is_non_optional_reference(expr)
}

fn classify_class_super(super_class: Option<&Expression>) -> ReceiverTypeKind {
    let Some(super_class) = super_class else {
        return ReceiverTypeKind::NonTarget;
    };
    if let Expression::Identifier(ident) = super_class.get_inner_expression() {
        classify_type_name(&ident.name)
    } else {
        ReceiverTypeKind::Unknown
    }
}

fn classify_this_receiver(while_node: &AstNode, ctx: &LintContext) -> ReceiverTypeKind {
    for ancestor in ctx.nodes().ancestors(while_node.id()) {
        match ancestor.kind() {
            AstKind::StaticBlock(_) => return ReceiverTypeKind::NonTarget,
            AstKind::PropertyDefinition(prop) if prop.r#static => {
                return ReceiverTypeKind::NonTarget;
            }
            AstKind::AccessorProperty(prop) if prop.r#static => {
                return ReceiverTypeKind::NonTarget;
            }
            AstKind::Function(func) => {
                if let Some(this_param) = &func.this_param
                    && let Some(annotation) = &this_param.type_annotation
                {
                    match classify_ts_type(&annotation.type_annotation) {
                        ReceiverTypeKind::Unknown
                            if is_any_or_unknown_ts_type(&annotation.type_annotation) => {}
                        kind => return kind,
                    }
                }
                match ctx.nodes().parent_kind(ancestor.id()) {
                    AstKind::ObjectProperty(_) => return ReceiverTypeKind::NonTarget,
                    AstKind::MethodDefinition(method) => {
                        if method.r#static {
                            return ReceiverTypeKind::NonTarget;
                        }
                    }
                    _ => return ReceiverTypeKind::Unknown,
                }
            }
            AstKind::Class(class) => return classify_class_super(class.heritage_expression()),
            _ => {}
        }
    }
    ReceiverTypeKind::Unknown
}

fn classify_super_receiver(while_node: &AstNode, ctx: &LintContext) -> ReceiverTypeKind {
    for ancestor in ctx.nodes().ancestors(while_node.id()) {
        match ancestor.kind() {
            AstKind::StaticBlock(_) => return ReceiverTypeKind::NonTarget,
            AstKind::PropertyDefinition(prop) if prop.r#static => {
                return ReceiverTypeKind::NonTarget;
            }
            AstKind::AccessorProperty(prop) if prop.r#static => {
                return ReceiverTypeKind::NonTarget;
            }
            AstKind::MethodDefinition(method) if method.r#static => {
                return ReceiverTypeKind::NonTarget;
            }
            AstKind::Function(_)
                if matches!(ctx.nodes().parent_kind(ancestor.id()), AstKind::ObjectProperty(_)) =>
            {
                return ReceiverTypeKind::NonTarget;
            }
            AstKind::Class(class) => return classify_class_super(class.heritage_expression()),
            _ => {}
        }
    }
    ReceiverTypeKind::Unknown
}

fn classify_receiver_expression(
    expr: &Expression,
    while_node: &AstNode,
    ctx: &LintContext,
) -> ReceiverTypeKind {
    match expr.without_parentheses() {
        Expression::TSAsExpression(ts_as) => {
            classify_asserted_receiver(&ts_as.type_annotation, &ts_as.expression, while_node, ctx)
        }
        Expression::TSTypeAssertion(ts_assert) => classify_asserted_receiver(
            &ts_assert.type_annotation,
            &ts_assert.expression,
            while_node,
            ctx,
        ),
        Expression::TSNonNullExpression(non_null) => {
            classify_receiver_expression(&non_null.expression, while_node, ctx)
        }
        Expression::TSSatisfiesExpression(sat) => {
            classify_receiver_expression(&sat.expression, while_node, ctx)
        }
        Expression::Identifier(ident) => {
            if matches!(ident.name.as_str(), "undefined" | "window" | "self" | "globalThis")
                && ctx.is_reference_to_global_variable(ident)
            {
                return ReceiverTypeKind::NonTarget;
            }
            let Some(symbol_id) = ctx.scoping().get_reference(ident.reference_id()).symbol_id()
            else {
                return ReceiverTypeKind::Unknown;
            };
            let annotation = match ctx.symbol_declaration(symbol_id).kind() {
                AstKind::FormalParameter(param) if param.pattern.is_binding_identifier() => {
                    param.type_annotation.as_deref()
                }
                AstKind::VariableDeclarator(decl) if decl.id.is_binding_identifier() => {
                    decl.type_annotation.as_deref()
                }
                _ => None,
            };
            annotation
                .map_or(ReceiverTypeKind::Unknown, |ann| classify_ts_type(&ann.type_annotation))
        }
        Expression::ThisExpression(_) => classify_this_receiver(while_node, ctx),
        Expression::Super(_) => classify_super_receiver(while_node, ctx),
        _ => ReceiverTypeKind::Unknown,
    }
}

fn classify_asserted_receiver(
    ty: &TSType,
    expr: &Expression,
    while_node: &AstNode,
    ctx: &LintContext,
) -> ReceiverTypeKind {
    match classify_ts_type(ty) {
        ReceiverTypeKind::Unknown if is_any_or_unknown_ts_type(ty) => {
            classify_receiver_expression(expr, while_node, ctx)
        }
        kind => kind,
    }
}

fn is_known_non_replace_children_receiver(
    expr: &Expression,
    while_node: &AstNode,
    ctx: &LintContext,
) -> bool {
    matches!(
        classify_receiver_expression(expr, while_node, ctx),
        ReceiverTypeKind::NonTarget | ReceiverTypeKind::Nullish
    )
}

fn starts_with_unparenthesized_let_bracket(mut expr: &Expression) -> bool {
    loop {
        match expr {
            Expression::ComputedMemberExpression(member) => {
                if matches!(&member.object, Expression::Identifier(ident) if ident.name == "let") {
                    return true;
                }
                expr = &member.object;
            }
            Expression::StaticMemberExpression(member) => expr = &member.object,
            Expression::PrivateFieldExpression(member) => expr = &member.object,
            Expression::TSNonNullExpression(non_null) => expr = &non_null.expression,
            _ => return false,
        }
    }
}

fn statement_ends_without_semicolon_expression(stmt: &Statement, ctx: &LintContext) -> bool {
    if ctx.source_range(stmt.span()).trim_end().ends_with(';') {
        return false;
    }
    match stmt {
        Statement::ExpressionStatement(_)
        | Statement::VariableDeclaration(_)
        | Statement::ReturnStatement(_)
        | Statement::ThrowStatement(_)
        | Statement::TSExportAssignment(_) => true,
        Statement::ExportDeclaration(decl) => {
            matches!(decl.declaration, Declaration::VariableDeclaration(_))
        }
        Statement::ExportDefaultDeclaration(decl) => !matches!(
            decl.declaration,
            ExportDefaultDeclarationKind::FunctionDeclaration(_)
                | ExportDefaultDeclarationKind::ClassDeclaration(_)
                | ExportDefaultDeclarationKind::TSInterfaceDeclaration(_)
        ),
        Statement::IfStatement(if_stmt) => {
            let trailing = if_stmt.alternate.as_ref().unwrap_or(&if_stmt.consequent);
            statement_ends_without_semicolon_expression(trailing, ctx)
        }
        Statement::LabeledStatement(labeled) => {
            statement_ends_without_semicolon_expression(&labeled.body, ctx)
        }
        Statement::WhileStatement(while_stmt) => {
            statement_ends_without_semicolon_expression(&while_stmt.body, ctx)
        }
        Statement::ForStatement(for_stmt) => {
            statement_ends_without_semicolon_expression(&for_stmt.body, ctx)
        }
        Statement::ForInStatement(for_in) => {
            statement_ends_without_semicolon_expression(&for_in.body, ctx)
        }
        Statement::ForOfStatement(for_of) => {
            statement_ends_without_semicolon_expression(&for_of.body, ctx)
        }
        Statement::WithStatement(with_stmt) => {
            statement_ends_without_semicolon_expression(&with_stmt.body, ctx)
        }
        _ => false,
    }
}

fn needs_leading_semicolon_before_while(
    while_node: &AstNode,
    while_span: Span,
    ctx: &LintContext,
) -> bool {
    let parent = ctx.nodes().parent_node(while_node.id());
    let (statements, last_directive_span): (&[Statement], Option<Span>) = match parent.kind() {
        AstKind::Program(program) => (&program.body, program.directives.last().map(|d| d.span)),
        AstKind::FunctionBody(body) => (&body.statements, body.directives.last().map(|d| d.span)),
        AstKind::BlockStatement(block) => (&block.body, None),
        AstKind::StaticBlock(block) => (&block.body, None),
        AstKind::SwitchCase(case) => (&case.consequent, None),
        AstKind::TSModuleBlock(block) => (&block.body, block.directives.last().map(|d| d.span)),
        _ => return false,
    };

    let Some(idx) = statements.iter().position(|stmt| stmt.span() == while_span) else {
        return false;
    };

    if idx == 0 {
        return last_directive_span
            .is_some_and(|span| !ctx.source_range(span).trim_end().ends_with(';'));
    }

    statement_ends_without_semicolon_expression(&statements[idx - 1], ctx)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "oldChildNode.replaceWith(newChildNode);",
        "referenceNode.before(newNode);",
        r#"referenceNode.before("text");"#,
        "referenceNode.prepend(newNode);",
        r#"referenceNode.prepend("text");"#,
        "referenceNode.append(newNode);",
        r#"referenceNode.append("text");"#,
        "referenceNode.after(newNode);",
        r#"referenceNode.after("text");"#,
        "oldChildNode.replaceWith(undefined, oldNode);",
        "oldChildNode.replaceWith(newNode, undefined);",
        "new parentNode.replaceChild(newNode, oldNode);",
        "new parentNode.insertBefore(newNode, referenceNode);",
        "new referenceNode.insertAdjacentText('beforebegin', 'text');",
        "new referenceNode.insertAdjacentElement('beforebegin', newNode);",
        "replaceChild(newNode, oldNode);",
        "insertBefore(newNode, referenceNode);",
        "insertAdjacentText('beforebegin', 'text');",
        "insertAdjacentElement('beforebegin', newNode);",
        "parentNode['replaceChild'](newNode, oldNode);",
        "parentNode['insertBefore'](newNode, referenceNode);",
        "referenceNode['insertAdjacentText']('beforebegin', 'text');",
        "referenceNode['insertAdjacentElement']('beforebegin', newNode);",
        "parentNode[replaceChild](newNode, oldNode);",
        "parentNode[insertBefore](newNode, referenceNode);",
        "referenceNode[insertAdjacentText]('beforebegin', 'text');",
        "referenceNode[insertAdjacentElement]('beforebegin', newNode);",
        "parent.foo(a, b);",
        "parentNode.replaceChild(newNode);",
        "parentNode.insertBefore(newNode);",
        "referenceNode.insertAdjacentText('beforebegin');",
        "referenceNode.insertAdjacentElement('beforebegin');",
        "parentNode.replaceChild(newNode, oldNode, extra);",
        "parentNode.insertBefore(newNode, referenceNode, extra);",
        "referenceNode.insertAdjacentText('beforebegin', 'text', extra);",
        "referenceNode.insertAdjacentElement('beforebegin', newNode, extra);",
        "referenceNode.insertAdjacentText(...argumentsArray1, ...argumentsArray2);",
        "referenceNode.insertAdjacentElement(...argumentsArray1, ...argumentsArray2);",
        "referenceNode.insertAdjacentText('foo', 'text');",
        "referenceNode.insertAdjacentElement('foo', newNode);",
        "while (node.firstElementChild) { node.firstElementChild.remove(); }",
        "while (node.lastElementChild) { node.lastElementChild.remove(); }",
        "while (node.firstChild) { node.lastChild.remove(); }",
        "while (node.lastChild) { otherNode.lastChild.remove(); }",
        "while (node.firstChild) { child = node.firstChild; child.remove(); }",
        "while (node.firstChild) { node.firstChild.remove(); node.normalize(); }",
        "while (getNode().firstChild) { getNode().firstChild.remove(); }",
        "while (nodes[getIndex()].lastChild) { nodes[getIndex()].lastChild.remove(); }",
        "while ((node?.parent).firstChild) { (node?.parent).firstChild.remove(); }",
        "while (node.firstChild) { node.firstChild?.remove(); }",
        "while (node.firstChild) { (node?.firstChild).remove(); }",
        r#"while (node.firstChild) { node.firstChild["remove"](); }"#,
        "while (node.firstChild) { node.removeChild(node.firstChild); }",
        "while (node.firstChild) { const child = node.firstChild; child.remove(); }",
        "while (window.firstChild) { window.firstChild.remove(); }",
        "while (self.firstChild) { self.firstChild.remove(); }",
        "while (globalThis.firstChild) { globalThis.firstChild.remove(); }",
        "do { node.firstChild.remove(); } while (node.firstChild)",
        "for (; node.firstChild;) { node.firstChild.remove(); }",
        r#"while ("x".firstChild) { "x".firstChild.remove(); }"#,
        "while (undefined.firstChild) { undefined.firstChild.remove(); }",
        "function foo(node: string) { while (node.firstChild) { node.firstChild.remove(); } }",
        "function foo(node: Node) { while (node.firstChild) { node.firstChild.remove(); } }",
        "function foo(node: Element | Node) { while (node.firstChild) { node.firstChild.remove(); } }",
        "function foo(node: Element) { while ((node as Node).firstChild) { (node as Node).firstChild.remove(); } }",
        "function foo(node: Node) { while ((node as any).firstChild) { (node as any).firstChild.remove(); } }",
        "function foo(node: Node) { while ((node as (any)).firstChild) { (node as (any)).firstChild.remove(); } }",
        "function foo(node: Node) { while (node!.firstChild) { node!.firstChild.remove(); } }",
        "const node: Node = getNode(); while (node.firstChild) { node.firstChild.remove(); }",
        "function foo(node: Node & string) { while (node.firstChild) { node.firstChild.remove(); } }",
        "function foo(this: Node) { while (this.firstChild) { this.firstChild.remove(); } }",
        "class PlainTree { method() { while (this.firstChild) { this.firstChild.remove(); } } }",
        "class NodeSubclass extends Node { method() { while (this.firstChild) { this.firstChild.remove(); } } }",
        "class NodeSubclass extends Node { method() { while (super.firstChild) { super.firstChild.remove(); } } }",
        "class ElementSubclass extends Element { static method() { while (super.firstChild) { super.firstChild.remove(); } } }",
        "class ElementSubclass extends Element { static method() { while (this.firstChild) { this.firstChild.remove(); } } }",
        "class ElementSubclass extends Element { static { while (this.firstChild) { this.firstChild.remove(); } } }",
        "class ElementSubclass extends Element { static { while (super.firstChild) { super.firstChild.remove(); } } }",
        "const obj = { method() { while (this.firstChild) { this.firstChild.remove(); } } };",
        "const obj = { clear(this: any) { while (this.firstChild) { this.firstChild.remove(); } } };",
        "const obj = { method() { while (super.firstChild) { super.firstChild.remove(); } } };",
        "class Foo extends Element { method() { const obj = { m() { while (super.firstChild) { super.firstChild.remove(); } } }; } }",
        "while ((foo<number>).firstChild) { (foo<number>).firstChild.remove(); }",
        "while (node.firstChild) { node.firstChild.remove(arg); }",
        "while (node.firstChild) { node.firstChild.remove<void>(); }",
        "while (node?.firstChild) { node.firstChild.remove(); }",
        "while (node.firstChild) { node.firstChild.remove?.(); }",
        "while (node.firstChild) {}",
    ];

    let fail = vec![
        "parentNode.replaceChild(newChildNode, oldChildNode);",
        "parentNode.replaceChild(
                newChildNode,
                oldChildNode
            );",
        "parentNode.replaceChild( // inline comments
                newChildNode, // inline comments
                oldChildNode // inline comments
            );",
        "const foo = parentNode.replaceChild(newChildNode, oldChildNode);",
        "foo = parentNode.replaceChild(newChildNode, oldChildNode);",
        "parentNode.insertBefore(newNode, referenceNode);",
        "parentNode.insertBefore(alfa, beta).insertBefore(charlie, delta);",
        "const foo = parentNode.insertBefore(alfa, beta);",
        "foo = parentNode.insertBefore(alfa, beta);",
        "new Dom(parentNode.insertBefore(alfa, beta))",
        "`${parentNode.insertBefore(alfa, beta)}`",
        // Non-identifier arguments are reported without a suggestion. Rewriting them before
        // `.replaceWith` or `.before` could change precedence or evaluation order.
        "parentNode.replaceChild(newNode, cond ? c : d);",
        "parentNode.insertBefore(newNode, p || q);",
        "parentNode.replaceChild(getNode(), oldNode);",
        "parentNode.replaceChild(...argumentsArray1, ...argumentsArray2);",
        "parentNode.insertBefore(...argumentsArray1, ...argumentsArray2);",
        "parentNode.replaceChild(newNode, undefined);",
        r#"referenceNode.insertAdjacentText("beforebegin", "text");"#,
        r#"referenceNode.insertAdjacentText("afterbegin", "text");"#,
        r#"referenceNode.insertAdjacentText("beforeend", "text");"#,
        r#"referenceNode.insertAdjacentText("afterend", "text");"#,
        r#"const foo = referenceNode.insertAdjacentText("beforebegin", "text");"#,
        r#"foo = referenceNode.insertAdjacentText("beforebegin", "text");"#,
        r#"referenceNode.insertAdjacentElement("beforebegin", newNode);"#,
        r#"referenceNode.insertAdjacentElement("afterbegin", "text");"#,
        r#"referenceNode.insertAdjacentElement("beforeend", "text");"#,
        r#"referenceNode.insertAdjacentElement("afterend", newNode);"#,
        r#"referenceNode.insertAdjacentElement(
                "afterend",
                newNode
            );"#,
        r#"referenceNode.insertAdjacentElement( // inline comments
                "afterend", // inline comments
                newNode  // inline comments
            ); // inline comments"#,
        r#"const foo = referenceNode.insertAdjacentElement("beforebegin", newNode);"#,
        r#"foo = referenceNode.insertAdjacentElement("beforebegin", newNode);"#,
        r#"const foo = [referenceNode.insertAdjacentElement("beforebegin", newNode)]"#,
        r#"foo(bar = referenceNode.insertAdjacentElement("beforebegin", newNode))"#,
        r#"const foo = () => { return referenceNode.insertAdjacentElement("beforebegin", newNode); }"#,
        r#"if (referenceNode.insertAdjacentElement("beforebegin", newNode)) {}"#,
        r#"const foo = { bar: referenceNode.insertAdjacentElement("beforebegin", newNode) }"#,
        "while (node.firstChild) { node.firstChild.remove(); }",
        "while (node.lastChild) { node.lastChild.remove(); }",
        "while (node.firstChild) node.firstChild.remove();",
        "while (node.lastChild) node.lastChild.remove();",
        "while (this.firstChild) { this.firstChild.remove(); }",
        "class Foo extends Element { method() { while (this.firstChild) { this.firstChild.remove(); } } }",
        "class Foo extends Element { method() { while (super.firstChild) { super.firstChild.remove(); } } }",
        "class Foo { #node: Element; method() { while (this.#node.firstChild) { this.#node.firstChild.remove(); } } }",
        "while (parent.node.firstChild) { parent.node.firstChild.remove(); }",
        "while (nodes[0].firstChild) { nodes[0].firstChild.remove(); }",
        "function foo(node: Element) { while (node.firstChild) { node.firstChild.remove(); } }",
        "function foo(node: Element | undefined) { while (node.firstChild) { node.firstChild.remove(); } }",
        "function foo(node: Element & { extra: number }) { while (node.firstChild) { node.firstChild.remove(); } }",
        "const node: Element = getNode(); while (node.firstChild) { node.firstChild.remove(); }",
        "function foo(this: Element) { while (this.firstChild) { this.firstChild.remove(); } }",
        "const obj = { clear(this: HTMLDivElement) { while (this.firstChild) { this.firstChild.remove(); } } };",
        "class Sub extends Node { clear(this: HTMLDivElement) { while (this.firstChild) { this.firstChild.remove(); } } }",
        "function foo(node: Node) { while ((node as Element).firstChild) { (node as Element).firstChild.remove(); } }",
        "function foo(node: Element) { while ((node satisfies Node).firstChild) { (node satisfies Node).firstChild.remove(); } }",
        "function foo(node: Element | null) { while (node!.firstChild) { node!.firstChild.remove(); } }",
        "foo()\nwhile ((node as Element).firstChild) {\n\t(node as Element).firstChild.remove();\n}",
        "foo();\nwhile ((node as Element).firstChild) {\n\t(node as Element).firstChild.remove();\n}",
        "export const x = foo\nwhile ((node as Element).firstChild) {\n\t(node as Element).firstChild.remove();\n}",
        "namespace N {\n\t\"use strict\"\n\twhile ((node as Element).firstChild) {\n\t\t(node as Element).firstChild.remove();\n\t}\n}",
        "function foo() {\n\t\"use strict\"\n\twhile ((node as Element).firstChild) {\n\t\t(node as Element).firstChild.remove();\n\t}\n}",
        "if (cond) {\n\tconst x = bar\n\twhile ((node as Element).firstChild) {\n\t\t(node as Element).firstChild.remove();\n\t}\n}",
        "const array = []\nwhile ((node).firstChild) {\n\t(node).firstChild.remove();\n}",
        "while (node.firstChild) {\n\t// Keep this comment.\n\tnode.firstChild.remove();\n}",
        "while (/* keep */ node.firstChild) { node.firstChild.remove(); }",
        "function f(window) { while (window.firstChild) { window.firstChild.remove(); } }",
        "while ((node satisfies Node).firstChild) { (node satisfies Node).firstChild.remove(); }",
    ];

    let fix = vec![
        (
            "parentNode.replaceChild(newChildNode, oldChildNode);",
            "oldChildNode.replaceWith(newChildNode);",
        ),
        ("parentNode.insertBefore(newNode, referenceNode);", "referenceNode.before(newNode);"),
        (
            r#"referenceNode.insertAdjacentText("beforebegin", "text");"#,
            r#"referenceNode.before("text");"#,
        ),
        (
            r#"referenceNode.insertAdjacentText("afterbegin", "text");"#,
            r#"referenceNode.prepend("text");"#,
        ),
        (
            r#"referenceNode.insertAdjacentText("beforeend", "text");"#,
            r#"referenceNode.append("text");"#,
        ),
        (
            r#"referenceNode.insertAdjacentText("afterend", "text");"#,
            r#"referenceNode.after("text");"#,
        ),
        (
            r#"const foo = referenceNode.insertAdjacentText("beforebegin", "text");"#,
            r#"const foo = referenceNode.before("text");"#,
        ),
        (
            r#"foo = referenceNode.insertAdjacentText("beforebegin", "text");"#,
            r#"foo = referenceNode.before("text");"#,
        ),
        (
            r#"referenceNode.insertAdjacentElement("beforebegin", newNode);"#,
            "referenceNode.before(newNode);",
        ),
        (
            r#"referenceNode.insertAdjacentElement("afterbegin", "text");"#,
            r#"referenceNode.prepend("text");"#,
        ),
        (
            r#"referenceNode.insertAdjacentElement("beforeend", "text");"#,
            r#"referenceNode.append("text");"#,
        ),
        (
            r#"referenceNode.insertAdjacentElement("afterend", newNode);"#,
            "referenceNode.after(newNode);",
        ),
        (
            "parentNode.replaceChild(
                newChildNode,
                oldChildNode
            );",
            "oldChildNode.replaceWith(newChildNode);",
        ),
        (
            "parentNode. replaceChild( // inline comments
                newChildNode, // inline comments
                oldChildNode // inline comments
            );",
            "oldChildNode.replaceWith(newChildNode);",
        ),
        (
            r#"referenceNode.insertAdjacentElement(
                "afterend",
                newNode
            );"#,
            "referenceNode.after(newNode);",
        ),
        (
            r#"referenceNode.insertAdjacentElement( // inline comments
                "afterend", // inline comments
                newNode  // inline comments
            ); // inline comments"#,
            "referenceNode.after(newNode); // inline comments",
        ),
        ("while (node.firstChild) { node.firstChild.remove(); }", "node.replaceChildren();"),
        ("while (node.lastChild) { node.lastChild.remove(); }", "node.replaceChildren();"),
        ("while (node.firstChild) node.firstChild.remove();", "node.replaceChildren();"),
        ("while (node.lastChild) node.lastChild.remove();", "node.replaceChildren();"),
        ("while (this.firstChild) { this.firstChild.remove(); }", "this.replaceChildren();"),
        (
            "class Foo extends Element { method() { while (this.firstChild) { this.firstChild.remove(); } } }",
            "class Foo extends Element { method() { this.replaceChildren(); } }",
        ),
        (
            "class Foo extends Element { method() { while (super.firstChild) { super.firstChild.remove(); } } }",
            "class Foo extends Element { method() { super.replaceChildren(); } }",
        ),
        (
            "class Foo { #node: Element; method() { while (this.#node.firstChild) { this.#node.firstChild.remove(); } } }",
            "class Foo { #node: Element; method() { this.#node.replaceChildren(); } }",
        ),
        (
            "while (parent.node.firstChild) { parent.node.firstChild.remove(); }",
            "parent.node.replaceChildren();",
        ),
        (
            "while (nodes[0].firstChild) { nodes[0].firstChild.remove(); }",
            "nodes[0].replaceChildren();",
        ),
        (
            "function foo(node: Element) { while (node.firstChild) { node.firstChild.remove(); } }",
            "function foo(node: Element) { node.replaceChildren(); }",
        ),
        (
            "function foo(node: Element | undefined) { while (node.firstChild) { node.firstChild.remove(); } }",
            "function foo(node: Element | undefined) { node.replaceChildren(); }",
        ),
        (
            "function foo(node: Element & { extra: number }) { while (node.firstChild) { node.firstChild.remove(); } }",
            "function foo(node: Element & { extra: number }) { node.replaceChildren(); }",
        ),
        (
            "const node: Element = getNode(); while (node.firstChild) { node.firstChild.remove(); }",
            "const node: Element = getNode(); node.replaceChildren();",
        ),
        (
            "function foo(this: Element) { while (this.firstChild) { this.firstChild.remove(); } }",
            "function foo(this: Element) { this.replaceChildren(); }",
        ),
        (
            "const obj = { clear(this: HTMLDivElement) { while (this.firstChild) { this.firstChild.remove(); } } };",
            "const obj = { clear(this: HTMLDivElement) { this.replaceChildren(); } };",
        ),
        (
            "class Sub extends Node { clear(this: HTMLDivElement) { while (this.firstChild) { this.firstChild.remove(); } } }",
            "class Sub extends Node { clear(this: HTMLDivElement) { this.replaceChildren(); } }",
        ),
        (
            "function foo(node: Node) { while ((node as Element).firstChild) { (node as Element).firstChild.remove(); } }",
            "function foo(node: Node) { (node as Element).replaceChildren(); }",
        ),
        (
            "function foo(node: Element) { while ((node satisfies Node).firstChild) { (node satisfies Node).firstChild.remove(); } }",
            "function foo(node: Element) { (node satisfies Node).replaceChildren(); }",
        ),
        (
            "function foo(node: Element | null) { while (node!.firstChild) { node!.firstChild.remove(); } }",
            "function foo(node: Element | null) { node!.replaceChildren(); }",
        ),
        (
            "foo()\nwhile ((node as Element).firstChild) {\n\t(node as Element).firstChild.remove();\n}",
            "foo()\n;(node as Element).replaceChildren();",
        ),
        (
            "foo();\nwhile ((node as Element).firstChild) {\n\t(node as Element).firstChild.remove();\n}",
            "foo();\n(node as Element).replaceChildren();",
        ),
        (
            "export const x = foo\nwhile ((node as Element).firstChild) {\n\t(node as Element).firstChild.remove();\n}",
            "export const x = foo\n;(node as Element).replaceChildren();",
        ),
        (
            "namespace N {\n\t\"use strict\"\n\twhile ((node as Element).firstChild) {\n\t\t(node as Element).firstChild.remove();\n\t}\n}",
            "namespace N {\n\t\"use strict\"\n\t;(node as Element).replaceChildren();\n}",
        ),
        (
            "function foo() {\n\t\"use strict\"\n\twhile ((node as Element).firstChild) {\n\t\t(node as Element).firstChild.remove();\n\t}\n}",
            "function foo() {\n\t\"use strict\"\n\t;(node as Element).replaceChildren();\n}",
        ),
        (
            "if (cond) {\n\tconst x = bar\n\twhile ((node as Element).firstChild) {\n\t\t(node as Element).firstChild.remove();\n\t}\n}",
            "if (cond) {\n\tconst x = bar\n\t;(node as Element).replaceChildren();\n}",
        ),
        (
            "const array = []\nwhile ((node).firstChild) {\n\t(node).firstChild.remove();\n}",
            "const array = []\nnode.replaceChildren();",
        ),
        (
            "while (node.firstChild) {\n\t// Keep this comment.\n\tnode.firstChild.remove();\n}",
            "while (node.firstChild) {\n\t// Keep this comment.\n\tnode.firstChild.remove();\n}",
        ),
        (
            "while (/* keep */ node.firstChild) { node.firstChild.remove(); }",
            "while (/* keep */ node.firstChild) { node.firstChild.remove(); }",
        ),
        (
            "function f(window) { while (window.firstChild) { window.firstChild.remove(); } }",
            "function f(window) { window.replaceChildren(); }",
        ),
        (
            "while ((node satisfies Node).firstChild) { (node satisfies Node).firstChild.remove(); }",
            "(node satisfies Node).replaceChildren();",
        ),
    ];

    Tester::new(PreferModernDomApis::NAME, PreferModernDomApis::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_ts_type_assertion() {
    use crate::tester::Tester;

    let pass = vec![
        "function foo(node: Node) { while ((<Node>node).lastChild) { (<Node>node).lastChild.remove(); } }",
        "function foo(node: Node) { while ((<(any)>node).lastChild) { (<(any)>node).lastChild.remove(); } }",
    ];
    let fail = vec![
        "function foo(node: Element) { while ((<Element>node).lastChild) { (<Element>node).lastChild.remove(); } }",
        "export = foo\nwhile ((<Element>node).lastChild) { (<Element>node).lastChild.remove(); }",
    ];
    let fix = vec![
        (
            "function foo(node: Element) { while ((<Element>node).lastChild) { (<Element>node).lastChild.remove(); } }",
            "function foo(node: Element) { (<Element>node).replaceChildren(); }",
        ),
        (
            "export = foo\nwhile ((<Element>node).lastChild) { (<Element>node).lastChild.remove(); }",
            "export = foo\n;(<Element>node).replaceChildren();",
        ),
    ];

    Tester::new(PreferModernDomApis::NAME, PreferModernDomApis::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .expect_fix(fix)
        .test();
}

#[test]
fn test_sloppy_let_bracket() {
    use crate::tester::Tester;

    let pass: Vec<&str> = vec![];
    let fail = vec![
        "while ((let[0]).firstChild) { (let[0]).firstChild.remove(); }",
        "foo()\nwhile (let[0].node.firstChild) { (let[0].node).firstChild.remove(); }",
    ];
    let fix = vec![
        (
            "while ((let[0]).firstChild) { (let[0]).firstChild.remove(); }",
            "(let[0]).replaceChildren();",
        ),
        (
            "foo()\nwhile (let[0].node.firstChild) { (let[0].node).firstChild.remove(); }",
            "foo()\n;(let[0].node).replaceChildren();",
        ),
    ];

    Tester::new(PreferModernDomApis::NAME, PreferModernDomApis::PLUGIN, pass, fail)
        .change_rule_path_extension("js")
        .expect_fix(fix)
        .test();
}
