use crate::{AstNode, context::LintContext, module_record::ImportImportName, rule::Rule};
use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentExpression, AssignmentOperator, AssignmentTarget, AwaitExpression,
        BinaryExpression, BinaryOperator, CallExpression, Class, ComputedMemberExpression,
        Expression, Function, IdentifierReference, ImportExpression, NewExpression,
        ObjectExpression, PrivateInExpression, Statement, StaticMemberExpression,
        TaggedTemplateExpression, TemplateLiteral, TryStatement, UnaryExpression, UnaryOperator,
        UpdateExpression, YieldExpression,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ScopeFlags, SymbolId};
use oxc_span::{GetSpan, Span};

fn prefer_url_can_parse_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Prefer `URL.canParse()` over constructing a `URL` in a try/catch for validation.",
    )
    .with_help("Replace the `try`/`catch` with `URL.canParse()`.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferUrlCanParse;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers [`URL.canParse()`](https://developer.mozilla.org/en-US/docs/Web/API/URL/canParse_static)
    /// over constructing a `URL` inside a `try`/`catch` just to check whether a string is a valid URL.
    ///
    /// This rule only reports simple boolean validation patterns, where the `try` block
    /// consists of `new URL(…)` followed by either a `return` or an assignment of a boolean,
    /// and the `catch` block returns or assigns the opposite boolean. It skips arguments whose
    /// evaluation or string conversion could have side effects or throw, so the rewrite does
    /// not change behavior for ordinary string inputs.
    ///
    /// ### Why is this bad?
    ///
    /// Using exceptions for control flow is verbose and allocates a `URL` object that is
    /// immediately thrown away. `URL.canParse()` says what is meant directly.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function isValidUrl(value) {
    ///   try {
    ///     new URL(value);
    ///     return true;
    ///   } catch {
    ///     return false;
    ///   }
    /// }
    ///
    /// let valid;
    /// try {
    ///   new URL(value, base);
    ///   valid = true;
    /// } catch {
    ///   valid = false;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function isValidUrl(value) {
    ///   return URL.canParse(value);
    /// }
    ///
    /// let valid;
    /// valid = URL.canParse(value, base);
    /// ```
    PreferUrlCanParse,
    unicorn,
    style,
    fix,
    version = "next",
    short_description = "Prefer `URL.canParse()` over constructing a `URL` in a try/catch for validation.",
);

impl Rule for PreferUrlCanParse {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TryStatement(try_statement) = node.kind() else {
            return;
        };

        let Some(problem) = Problem::new(try_statement, ctx) else {
            return;
        };

        let diagnostic = prefer_url_can_parse_diagnostic(problem.new_expression.span);

        // Replacing the whole statement would drop any comment inside it.
        if ctx.has_comments_between(try_statement.span) {
            ctx.diagnostic(diagnostic);
        } else {
            ctx.diagnostic_with_fix(diagnostic, |fixer| {
                fixer.replace(try_statement.span, problem.replacement(ctx))
            });
        }
    }
}

/// How the validity of the URL is reported.
enum Sink<'a> {
    /// `return true` / `return false`
    Return,
    /// `name = true` / `name = false`
    Assignment(&'a IdentifierReference<'a>),
}

struct Problem<'a, 'b> {
    new_expression: &'b NewExpression<'a>,
    sink: Sink<'b>,
    /// `true` if the `try` branch reports invalid, so the result of `canParse()` is negated.
    negate: bool,
}

impl<'a, 'b> Problem<'a, 'b> {
    fn new(try_statement: &'b TryStatement<'a>, ctx: &LintContext<'a>) -> Option<Self> {
        let handler = try_statement.handler.as_ref()?;

        if try_statement.finalizer.is_some()
            || handler.param.as_ref().is_some_and(|param| !param.pattern.is_binding_identifier())
            || try_statement.block.body.len() != 2
            || handler.body.body.len() != 1
        {
            return None;
        }

        let new_expression = get_new_url_expression(&try_statement.block.body[0], ctx)?;

        let try_statement_result = &try_statement.block.body[1];
        let catch_statement_result = &handler.body.body[0];

        if let (Some(try_value), Some(catch_value)) =
            (get_boolean_return(try_statement_result), get_boolean_return(catch_statement_result))
        {
            return (try_value != catch_value).then_some(Self {
                new_expression,
                sink: Sink::Return,
                negate: !try_value,
            });
        }

        let (try_target, try_value) = get_boolean_assignment(try_statement_result)?;
        let (catch_target, catch_value) = get_boolean_assignment(catch_statement_result)?;

        (try_value != catch_value && is_same_assignment_target(try_target, catch_target, ctx))
            .then_some(Self {
                new_expression,
                sink: Sink::Assignment(try_target),
                negate: !try_value,
            })
    }

    fn replacement(&self, ctx: &LintContext<'a>) -> String {
        let callee = self.new_expression.callee.without_parentheses().span();
        let arguments = self
            .new_expression
            .arguments
            .iter()
            .map(|argument| ctx.source_range(argument_span(argument)))
            .collect::<Vec<_>>()
            .join(", ");
        let negation = if self.negate { "!" } else { "" };
        let can_parse = format!("{negation}{}.canParse({arguments})", ctx.source_range(callee));

        match self.sink {
            Sink::Return => format!("return {can_parse};"),
            Sink::Assignment(target) => format!("{} = {can_parse};", target.name),
        }
    }
}

/// The span of an argument without its own parentheses, unless they are needed to keep a
/// sequence expression from being read as two arguments.
fn argument_span(argument: &Argument<'_>) -> Span {
    match argument.as_expression() {
        Some(expression) => match expression.without_parentheses() {
            Expression::SequenceExpression(_) => expression.span(),
            inner => inner.span(),
        },
        None => argument.span(),
    }
}

/// `new URL(url)` or `new URL(url, base)` as an expression statement, where `URL` is the
/// global (or an imported, from `url` / `node:url`) constructor and the arguments are safe
/// to evaluate outside of the `try` block.
fn get_new_url_expression<'a, 'b>(
    statement: &'b Statement<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'b NewExpression<'a>> {
    let Statement::ExpressionStatement(statement) = statement else {
        return None;
    };
    let Expression::NewExpression(new_expression) = statement.expression.without_parentheses()
    else {
        return None;
    };

    if !(1..=2).contains(&new_expression.arguments.len())
        || new_expression.type_arguments.is_some()
        || new_expression.arguments.iter().any(Argument::is_spread)
    {
        return None;
    }

    let Expression::Identifier(callee) = new_expression.callee.without_parentheses() else {
        return None;
    };

    if !is_url_constructor(callee, ctx) || has_unsafe_argument(new_expression, ctx) {
        return None;
    }

    Some(new_expression)
}

fn get_boolean_return(statement: &Statement<'_>) -> Option<bool> {
    let Statement::ReturnStatement(statement) = statement else {
        return None;
    };
    boolean_value(statement.argument.as_ref()?)
}

fn get_boolean_assignment<'a, 'b>(
    statement: &'b Statement<'a>,
) -> Option<(&'b IdentifierReference<'a>, bool)> {
    let Statement::ExpressionStatement(statement) = statement else {
        return None;
    };
    let Expression::AssignmentExpression(assignment) = statement.expression.without_parentheses()
    else {
        return None;
    };
    let AssignmentExpression { operator: AssignmentOperator::Assign, left, right, .. } =
        &**assignment
    else {
        return None;
    };
    let AssignmentTarget::AssignmentTargetIdentifier(target) = left else {
        return None;
    };
    Some((target, boolean_value(right)?))
}

fn boolean_value(expression: &Expression<'_>) -> Option<bool> {
    match expression.without_parentheses() {
        Expression::BooleanLiteral(literal) => Some(literal.value),
        _ => None,
    }
}

/// Both assignments must write to the same variable. For example, `catch (valid) { valid = false }`
/// assigns to the catch parameter, not to the `valid` assigned in the `try` block.
fn is_same_assignment_target(
    a: &IdentifierReference<'_>,
    b: &IdentifierReference<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    a.name == b.name && reference_symbol(a, ctx) == reference_symbol(b, ctx)
}

fn reference_symbol(
    identifier: &IdentifierReference<'_>,
    ctx: &LintContext<'_>,
) -> Option<SymbolId> {
    ctx.scoping().get_reference(identifier.reference_id()).symbol_id()
}

/// `identifier` refers to the global `name`, or to a declaration of it that is erased at
/// runtime (a type-only import, or an ambient `declare` declaration).
fn is_global_or_erased(
    identifier: &IdentifierReference<'_>,
    name: &str,
    ctx: &LintContext<'_>,
) -> bool {
    if identifier.name != name {
        return false;
    }
    if ctx.is_reference_to_global_variable(identifier) {
        return true;
    }
    reference_symbol(identifier, ctx).is_some_and(|symbol_id| {
        let flags = ctx.scoping().symbol_flags(symbol_id);
        flags.is_type_import() || flags.is_ambient()
    })
}

fn is_url_constructor(identifier: &IdentifierReference<'_>, ctx: &LintContext<'_>) -> bool {
    is_global_or_erased(identifier, "URL", ctx) || is_url_import(identifier, ctx)
}

/// `import {URL} from "node:url"` or `import {URL as Alias} from "url"`.
fn is_url_import(identifier: &IdentifierReference<'_>, ctx: &LintContext<'_>) -> bool {
    let Some(symbol_id) = reference_symbol(identifier, ctx) else {
        return false;
    };
    if !ctx.scoping().symbol_flags(symbol_id).is_import() {
        return false;
    }

    ctx.module_record().import_entries.iter().any(|entry| {
        !entry.is_type
            && entry.local_name.name.as_str() == identifier.name.as_str()
            && matches!(&entry.import_name, ImportImportName::Name(name) if name.name.as_str() == "URL")
            && matches!(entry.module_request.name(), "url" | "node:url")
    })
}

fn has_unsafe_argument<'a>(new_expression: &NewExpression<'a>, ctx: &LintContext<'a>) -> bool {
    let mut finder = UnsafeArgumentFinder { ctx, found: false };
    for argument in &new_expression.arguments {
        finder.visit_argument(argument);
    }
    finder.found
}

/// Finds anything in a `new URL(…)` argument that makes it unsafe to move out of the `try`
/// block: side effects, implicit type conversions, and values whose string conversion could
/// throw. Nested functions are not entered, since defining one does not run it.
struct UnsafeArgumentFinder<'a, 'b> {
    ctx: &'b LintContext<'a>,
    found: bool,
}

impl<'a> Visit<'a> for UnsafeArgumentFinder<'a, '_> {
    fn visit_assignment_expression(&mut self, _: &AssignmentExpression<'a>) {
        self.found = true;
    }

    fn visit_await_expression(&mut self, _: &AwaitExpression<'a>) {
        self.found = true;
    }

    fn visit_call_expression(&mut self, _: &CallExpression<'a>) {
        self.found = true;
    }

    fn visit_import_expression(&mut self, _: &ImportExpression<'a>) {
        self.found = true;
    }

    fn visit_new_expression(&mut self, _: &NewExpression<'a>) {
        self.found = true;
    }

    fn visit_update_expression(&mut self, _: &UpdateExpression<'a>) {
        self.found = true;
    }

    fn visit_yield_expression(&mut self, _: &YieldExpression<'a>) {
        self.found = true;
    }

    fn visit_tagged_template_expression(&mut self, _: &TaggedTemplateExpression<'a>) {
        self.found = true;
    }

    // An object's `toString()` can throw.
    fn visit_object_expression(&mut self, _: &ObjectExpression<'a>) {
        self.found = true;
    }

    // A class can run a static block.
    fn visit_class(&mut self, _: &Class<'a>) {
        self.found = true;
    }

    fn visit_template_literal(&mut self, literal: &TemplateLiteral<'a>) {
        if literal.expressions.is_empty() {
            walk::walk_template_literal(self, literal);
        } else {
            self.found = true;
        }
    }

    fn visit_unary_expression(&mut self, expression: &UnaryExpression<'a>) {
        let converts = matches!(
            expression.operator,
            UnaryOperator::UnaryNegation
                | UnaryOperator::UnaryPlus
                | UnaryOperator::LogicalNot
                | UnaryOperator::BitwiseNot
        ) && !expression.argument.without_parentheses().is_literal();

        if converts || expression.operator == UnaryOperator::Delete {
            self.found = true;
        } else {
            walk::walk_unary_expression(self, expression);
        }
    }

    fn visit_binary_expression(&mut self, expression: &BinaryExpression<'a>) {
        let is_strict_comparison = matches!(
            expression.operator,
            BinaryOperator::StrictEquality
                | BinaryOperator::StrictInequality
                | BinaryOperator::Instanceof
        );
        let both_literals = expression.left.without_parentheses().is_literal()
            && expression.right.without_parentheses().is_literal();

        // Every other operator converts its operands, which can call `toString()` / `valueOf()`.
        if is_strict_comparison || both_literals {
            walk::walk_binary_expression(self, expression);
        } else {
            self.found = true;
        }
    }

    // `#field in object`
    fn visit_private_in_expression(&mut self, _: &PrivateInExpression<'a>) {
        self.found = true;
    }

    fn visit_computed_member_expression(&mut self, expression: &ComputedMemberExpression<'a>) {
        if !expression.expression.without_parentheses().is_literal()
            || self.is_symbol(&expression.object)
        {
            self.found = true;
        } else {
            walk::walk_computed_member_expression(self, expression);
        }
    }

    // `Symbol.iterator` and friends are symbols, and converting a symbol to a string throws.
    fn visit_static_member_expression(&mut self, expression: &StaticMemberExpression<'a>) {
        if self.is_symbol(&expression.object) {
            self.found = true;
        } else {
            walk::walk_static_member_expression(self, expression);
        }
    }

    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _: &oxc_ast::ast::ArrowFunctionExpression<'a>) {}
}

impl UnsafeArgumentFinder<'_, '_> {
    fn is_symbol(&self, expression: &Expression<'_>) -> bool {
        matches!(
            expression.get_inner_expression(),
            Expression::Identifier(identifier) if is_global_or_erased(identifier, "Symbol", self.ctx)
        )
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const valid = URL.canParse(value);",
        r"const url = new URL(value);",
        r"try { new URL(value); } catch {}",
        r"function isValidUrl() { try { new URL(value); return true; } catch { return true; } }",
        r"function isValidUrl() { try { new URL(value); return isValid; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(value); return true; } catch { return isValid; } }",
        r"function isValidUrl() { try { new URL(value); return true; } catch { log(error); return false; } }",
        r"function isValidUrl() { try { new URL(value); log(url); return true; } catch { return false; } }",
        r"try { new URL(value); valid = true; } catch { other = false; }",
        r"try { new URL(value); valid += true; } catch { valid = false; }",
        r"try { new URL(value); valid = true; } catch { valid = false; } finally {}",
        r"function isValidUrl() { try { new URL(...arguments_); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(value, ...rest); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(value, base, extra); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new globalThis.URL(value); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(getValue()); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(value, getBase()); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(value++); return true; } catch { return false; } }",
        r#"function isValidUrl() { try { new URL(value + ""); return true; } catch { return false; } }"#,
        r#"function isValidUrl() { try { new URL({toString() { throw new Error("Invalid URL"); }}); return true; } catch { return false; } }"#,
        r#"function isValidUrl() { try { new URL(value, {toString() { throw new Error("Invalid base URL"); }}); return true; } catch { return false; } }"#,
        r#"function isValidUrl() { try { new URL([{toString() { throw new Error("Invalid URL"); }}]); return true; } catch { return false; } }"#,
        r#"function isValidUrl() { try { new URL(class { static { throw new Error("Invalid URL"); } }); return true; } catch { return false; } }"#,
        r"function isValidUrl() { try { new URL(`${value}`); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(tag`value`); return true; } catch { return false; } }",
        r"async function isValidUrl() { try { new URL(await value); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(Symbol()); return true; } catch { return false; } }",
        r#"function isValidUrl() { try { new URL(Symbol("url")); return true; } catch { return false; } }"#,
        r#"function isValidUrl() { try { new URL(Symbol.for("url")); return true; } catch { return false; } }"#,
        r"function isValidUrl() { try { new URL(Symbol.iterator); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(value, Symbol.iterator); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL([Symbol.iterator]); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(value, [Symbol.iterator]); return true; } catch { return false; } }",
        r#"function isValidUrl() { try { new URL(Symbol["for"]("url")); return true; } catch { return false; } }"#,
        r#"function isValidUrl() { try { new URL(Symbol["iterator"]); return true; } catch { return false; } }"#,
        r#"function isValidUrl() { try { new URL(Symbol?.for("url")); return true; } catch { return false; } }"#,
        r"function isValidUrl() { try { new URL(Symbol?.iterator); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(Symbol.iterator as symbol); return true; } catch { return false; } }",
        r#"function isValidUrl() { try { new URL(Symbol.for("url")!); return true; } catch { return false; } }"#,
        r"function isValidUrl() { try { new URL((Symbol as any).iterator); return true; } catch { return false; } }",
        r#"function isValidUrl() { try { new URL((Symbol as any).for("url")); return true; } catch { return false; } }"#,
        r#"import type {Symbol} from "symbol"; function isValidUrl() { try { new URL(Symbol.iterator); return true; } catch { return false; } }"#,
        r"declare const Symbol: SymbolConstructor; function isValidUrl() { try { new URL(Symbol.iterator); return true; } catch { return false; } }",
        r"function isValidUrl() { const URL = class {}; try { new URL(value); return true; } catch { return false; } }",
        r"import {URL} from 'node:url'; function isValidUrl() { { const URL = class {}; try { new URL(value); return true; } catch { return false; } } }",
        r"function isValidUrl() { try { new NotURL(value); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(value); return true; } catch ([error]) { return false; } }",
        r"function isValidUrl() { try { new URL(value); return true; } catch ({[sideEffect()]: error}) { return false; } }",
        r"try { new URL(value); valid = true; } catch ([error]) { valid = false; }",
        r"try { new URL(value); valid = true; } catch ({[sideEffect()]: error}) { valid = false; }",
        r"try { new URL(value); valid = true; } catch (valid) { valid = false; }",
        r"function isValidUrl() { try { new URL(-value); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(value[key]); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(import(value)); return true; } catch { return false; } }",
        r#"import {URL} from "other"; function isValidUrl() { try { new URL(value); return true; } catch { return false; } }"#,
        r#"import {Url as URL} from "node:url"; function isValidUrl() { try { new URL(value); return true; } catch { return false; } }"#,
        r"function isValidUrl() { try { new URL(value); return true; } catch { return false; } finally { cleanup(); } }",
    ];

    let fail = vec![
        r"function isValidUrl() { try { new URL(value); return true; } catch { return false; } }",
        r"function isValidUrl() {
	try {
		new URL(value, base);
		return true;
	} catch {
		return false;
	}
}",
        r"function isValidUrl() { try { new URL(value); return false; } catch { return true; } }",
        r"function isValidUrl() { try { new URL(value, base); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL((value)); return true; } catch { return false; } }",
        r"function isValidUrl() { const Symbol = {iterator: value}; try { new URL(Symbol.iterator); return true; } catch { return false; } }",
        r"try { new URL(value); valid = true; } catch { valid = false; }",
        r"try { new URL(value); valid = false; } catch { valid = true; }",
        r"let valid; try { new URL(value); valid = true; } catch { valid = false; }",
        r"let valid; try { new URL(value); valid = false; } catch { valid = true; }",
        r"let valid; /* comment */ try { new URL(value); valid = true; } catch { valid = false; }",
        r"let valid; try { new URL(valid); valid = true; } catch { valid = false; }",
        r"let valid; try { new URL(value); valid = true; } catch { valid = false; } valid = maybe;",
        r"const value = {toString() { return valid; }}; let valid; try { new URL(value); valid = true; } catch { valid = false; }",
        r"try { new URL(value); valid = true; } catch (error) { valid = false; }",
        r"import {URL} from 'node:url'; function isValidUrl() { try { new URL(value); return true; } catch { return false; } }",
        r"import {URL as NodeURL} from 'url'; function isValidUrl() { try { new NodeURL(value); return true; } catch { return false; } }",
        r"import {URL as NodeURL} from 'url'; function isValidUrl() { try { new NodeURL(value, base); return true; } catch { return false; } }",
        r#"import type {URL} from "node:url"; function isValidUrl() { try { new URL(value); return true; } catch { return false; } }"#,
        r"declare const URL: URLConstructor; function isValidUrl() { try { new URL(value); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(value); return true; } catch (error) { return false; } }",
        r"function isValidUrl() { const foo = 1
try { new URL(value); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(value as string); return true; } catch { return false; } }",
        r"let valid: boolean; try { new URL(value); valid = true; } catch { valid = false; }",
        r"function isValidUrl() { try { /* comment */ new URL(value); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(value); /* comment */ return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(value); return true; } catch { /* comment */ return false; } }",
        r"try { new URL(value); /* comment */ valid = true; } catch { valid = false; }",
        r"try { new URL(value); valid = true; } catch { /* comment */ valid = false; }",
        r"function isValidUrl() { try { new URL((a, b)); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(value.href, base.href); return true; } catch { return false; } }",
        r#"function isValidUrl() { try { new URL(value, "https://example.com"); return true; } catch { return false; } }"#,
        r"function isValidUrl() { try { new URL(`https://example.com`); return true; } catch { return false; } }",
        r"function isValidUrl() { try { new URL(() => value); return true; } catch { return false; } }",
        r"function isValidUrl() { try { (new URL(value)); return true; } catch { return false; } }",
    ];

    let fix = vec![
        (
            r"function isValidUrl() { try { new URL(value); return true; } catch { return false; } }",
            r"function isValidUrl() { return URL.canParse(value); }",
        ),
        (
            r"function isValidUrl() {
	try {
		new URL(value, base);
		return true;
	} catch {
		return false;
	}
}",
            r"function isValidUrl() {
	return URL.canParse(value, base);
}",
        ),
        (
            r"function isValidUrl() { try { new URL(value); return false; } catch { return true; } }",
            r"function isValidUrl() { return !URL.canParse(value); }",
        ),
        (
            r"function isValidUrl() { try { new URL(value, base); return true; } catch { return false; } }",
            r"function isValidUrl() { return URL.canParse(value, base); }",
        ),
        (
            r"function isValidUrl() { try { new URL((value)); return true; } catch { return false; } }",
            r"function isValidUrl() { return URL.canParse(value); }",
        ),
        (
            r"function isValidUrl() { const Symbol = {iterator: value}; try { new URL(Symbol.iterator); return true; } catch { return false; } }",
            r"function isValidUrl() { const Symbol = {iterator: value}; return URL.canParse(Symbol.iterator); }",
        ),
        (
            r"try { new URL(value); valid = true; } catch { valid = false; }",
            r"valid = URL.canParse(value);",
        ),
        (
            r"try { new URL(value); valid = false; } catch { valid = true; }",
            r"valid = !URL.canParse(value);",
        ),
        (
            r"let valid; try { new URL(value); valid = true; } catch { valid = false; }",
            r"let valid; valid = URL.canParse(value);",
        ),
        (
            r"let valid; try { new URL(value); valid = false; } catch { valid = true; }",
            r"let valid; valid = !URL.canParse(value);",
        ),
        (
            r"let valid; /* comment */ try { new URL(value); valid = true; } catch { valid = false; }",
            r"let valid; /* comment */ valid = URL.canParse(value);",
        ),
        (
            r"let valid; try { new URL(valid); valid = true; } catch { valid = false; }",
            r"let valid; valid = URL.canParse(valid);",
        ),
        (
            r"let valid; try { new URL(value); valid = true; } catch { valid = false; } valid = maybe;",
            r"let valid; valid = URL.canParse(value); valid = maybe;",
        ),
        (
            r"const value = {toString() { return valid; }}; let valid; try { new URL(value); valid = true; } catch { valid = false; }",
            r"const value = {toString() { return valid; }}; let valid; valid = URL.canParse(value);",
        ),
        (
            r"try { new URL(value); valid = true; } catch (error) { valid = false; }",
            r"valid = URL.canParse(value);",
        ),
        (
            r"import {URL} from 'node:url'; function isValidUrl() { try { new URL(value); return true; } catch { return false; } }",
            r"import {URL} from 'node:url'; function isValidUrl() { return URL.canParse(value); }",
        ),
        (
            r"import {URL as NodeURL} from 'url'; function isValidUrl() { try { new NodeURL(value); return true; } catch { return false; } }",
            r"import {URL as NodeURL} from 'url'; function isValidUrl() { return NodeURL.canParse(value); }",
        ),
        (
            r"import {URL as NodeURL} from 'url'; function isValidUrl() { try { new NodeURL(value, base); return true; } catch { return false; } }",
            r"import {URL as NodeURL} from 'url'; function isValidUrl() { return NodeURL.canParse(value, base); }",
        ),
        (
            r#"import type {URL} from "node:url"; function isValidUrl() { try { new URL(value); return true; } catch { return false; } }"#,
            r#"import type {URL} from "node:url"; function isValidUrl() { return URL.canParse(value); }"#,
        ),
        (
            r"declare const URL: URLConstructor; function isValidUrl() { try { new URL(value); return true; } catch { return false; } }",
            r"declare const URL: URLConstructor; function isValidUrl() { return URL.canParse(value); }",
        ),
        (
            r"function isValidUrl() { try { new URL(value); return true; } catch (error) { return false; } }",
            r"function isValidUrl() { return URL.canParse(value); }",
        ),
        (
            r"function isValidUrl() { const foo = 1
try { new URL(value); return true; } catch { return false; } }",
            r"function isValidUrl() { const foo = 1
return URL.canParse(value); }",
        ),
        (
            r"function isValidUrl() { try { new URL(value as string); return true; } catch { return false; } }",
            r"function isValidUrl() { return URL.canParse(value as string); }",
        ),
        (
            r"let valid: boolean; try { new URL(value); valid = true; } catch { valid = false; }",
            r"let valid: boolean; valid = URL.canParse(value);",
        ),
        (
            r"function isValidUrl() { try { /* comment */ new URL(value); return true; } catch { return false; } }",
            r"function isValidUrl() { try { /* comment */ new URL(value); return true; } catch { return false; } }",
        ),
        (
            r"function isValidUrl() { try { new URL(value); /* comment */ return true; } catch { return false; } }",
            r"function isValidUrl() { try { new URL(value); /* comment */ return true; } catch { return false; } }",
        ),
        (
            r"function isValidUrl() { try { new URL(value); return true; } catch { /* comment */ return false; } }",
            r"function isValidUrl() { try { new URL(value); return true; } catch { /* comment */ return false; } }",
        ),
        (
            r"try { new URL(value); /* comment */ valid = true; } catch { valid = false; }",
            r"try { new URL(value); /* comment */ valid = true; } catch { valid = false; }",
        ),
        (
            r"try { new URL(value); valid = true; } catch { /* comment */ valid = false; }",
            r"try { new URL(value); valid = true; } catch { /* comment */ valid = false; }",
        ),
        (
            r"function isValidUrl() { try { new URL((a, b)); return true; } catch { return false; } }",
            r"function isValidUrl() { return URL.canParse((a, b)); }",
        ),
        (
            r"function isValidUrl() { try { (new URL(value)); return true; } catch { return false; } }",
            r"function isValidUrl() { return URL.canParse(value); }",
        ),
    ];

    Tester::new(PreferUrlCanParse::NAME, PreferUrlCanParse::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
