use std::{borrow::Cow, cmp::Ordering};

use cow_utils::CowUtils;

use rustc_hash::FxHashSet;

use oxc_ast::{
    AstKind,
    ast::{
        Argument, BindingPattern, Expression, IdentifierReference, ObjectExpression,
        ObjectProperty, ObjectPropertyKind, PropertyKey, PropertyKind, TSLiteral, TSType,
        TSTypeName,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::StringToNumber;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{IsGlobalReference, SymbolId};
use oxc_span::{GetSpan, Span};
use oxc_str::JSStr;
use oxc_syntax::{
    number::ToJsString,
    operator::{BinaryOperator, UnaryOperator},
};

use crate::{AstNode, context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct NoInvalidIntlOptions;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows invalid, unknown, conflicting, and ignored options passed to
    /// `Intl` constructors, `supportedLocalesOf`, and built-in locale methods.
    ///
    /// ### Why is this bad?
    ///
    /// Invalid options can throw at runtime. Misspelled or ignored options silently
    /// produce unexpected output. This rule checks the ECMA-402 option definitions
    /// without relying on the host's ICU data or executing user code.
    ///
    /// Only inline options objects are checked. Objects containing spreads,
    /// accessors, duplicate keys, or unknown computed keys are skipped. Receivers
    /// are recognized from syntax and explicit TypeScript annotations. Results of
    /// arbitrary calls and property reads are not inferred: for example,
    /// `Math.random()`, typed-array `slice()`, or `holder.value`
    /// need an explicit receiver annotation or a directly recognizable expression.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// new Intl.NumberFormat('en', { style: 'currency' });
    /// new Intl.DateTimeFormat('en', { dateStyle: 'short', year: 'numeric' });
    // spellchecker:disable-next-line
    /// new Intl.Collator('en', { sensitvity: 'base' });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// new Intl.NumberFormat('en', { style: 'currency', currency: 'EUR' });
    /// new Intl.DateTimeFormat('en', { dateStyle: 'short' });
    /// new Intl.Collator('en', { sensitivity: 'base' });
    /// ```
    NoInvalidIntlOptions,
    unicorn,
    nursery,
    suggestion,
    version = "next",
    short_description = "Disallow invalid or ignored Intl options.",
);

const CONSTRUCTORS: &[&str] = &[
    "Collator",
    "DateTimeFormat",
    "DisplayNames",
    "DurationFormat",
    "ListFormat",
    "Locale",
    "NumberFormat",
    "PluralRules",
    "RelativeTimeFormat",
    "Segmenter",
];
const TYPED_ARRAYS: &[&str] = &[
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "Float16Array",
    "Float32Array",
    "Float64Array",
    "BigInt64Array",
    "BigUint64Array",
];
const FRACTION: [&str; 2] = ["minimumFractionDigits", "maximumFractionDigits"];
const SIGNIFICANT: [&str; 2] = ["minimumSignificantDigits", "maximumSignificantDigits"];
const DATE_COMPONENTS: &[&str] = &[
    "weekday",
    "era",
    "year",
    "month",
    "day",
    "dayPeriod",
    "hour",
    "minute",
    "second",
    "fractionalSecondDigits",
    "timeZoneName",
];
const TIME_UNITS: &[&str] =
    &["hours", "minutes", "seconds", "milliseconds", "microseconds", "nanoseconds"];
const ROUNDING_INCREMENTS: &[u16] =
    &[1, 2, 5, 10, 20, 25, 50, 100, 200, 250, 500, 1000, 2000, 2500, 5000];

#[derive(Clone, Copy)]
enum Definition {
    Boolean,
    String,
    Number(u16, u16),
    Enum(&'static [&'static str]),
    Identifier(IdentifierKind),
}

#[derive(Clone, Copy)]
enum IdentifierKind {
    UnicodeType,
    Language,
    Script,
    Region,
    Currency,
    Unit,
    Variants,
    FirstDayOfWeek,
}

fn definitions(constructor: &str) -> &'static [(&'static str, Definition)] {
    match constructor {
        "Collator" => COLLATOR_OPTIONS,
        "DateTimeFormat" => DATE_TIME_FORMAT_OPTIONS,
        "DisplayNames" => DISPLAY_NAMES_OPTIONS,
        "DurationFormat" => DURATION_FORMAT_OPTIONS,
        "ListFormat" => LIST_FORMAT_OPTIONS,
        "Locale" => LOCALE_OPTIONS,
        "NumberFormat" => NUMBER_FORMAT_OPTIONS,
        "PluralRules" => PLURAL_RULES_OPTIONS,
        "RelativeTimeFormat" => RELATIVE_TIME_FORMAT_OPTIONS,
        "Segmenter" => SEGMENTER_OPTIONS,
        "supportedLocalesOf" => &[("localeMatcher", Definition::Enum(&["lookup", "best fit"]))],
        _ => &[],
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Value<'a> {
    Unknown,
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(Cow<'a, str>),
    BigInt(Cow<'a, str>),
    Symbol,
}

impl Value<'_> {
    fn string(&self) -> Option<Cow<'_, str>> {
        match self {
            Self::String(s) | Self::BigInt(s) => Some(Cow::Borrowed(s)),
            Self::Number(n) => Some(Cow::Owned(n.to_js_string())),
            Self::Boolean(b) => Some(Cow::Borrowed(if *b { "true" } else { "false" })),
            Self::Null => Some(Cow::Borrowed("null")),
            Self::Undefined => Some(Cow::Borrowed("undefined")),
            _ => None,
        }
    }

    fn number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            Self::String(s) => Some(s.as_ref().string_to_number()),
            Self::Boolean(b) => Some(f64::from(u8::from(*b))),
            Self::Null => Some(0.0),
            Self::Undefined | Self::Symbol | Self::BigInt(_) => Some(f64::NAN),
            Self::Unknown => None,
        }
    }

    fn boolean(&self) -> Option<bool> {
        match self {
            Self::Unknown => None,
            Self::Undefined | Self::Null => Some(false),
            Self::Boolean(b) => Some(*b),
            Self::Number(n) => Some(*n != 0.0 && !n.is_nan()),
            Self::String(s) => Some(!s.is_empty()),
            Self::BigInt(s) => Some(s.as_ref() != "0"),
            Self::Symbol => Some(true),
        }
    }

    fn is(&self, text: &str) -> bool {
        matches!(self, Self::String(s) if s == text)
    }

    fn defined(&self) -> bool {
        !matches!(self, Self::Unknown | Self::Undefined)
    }
}

// Bound work across all branches of one evaluation, not only its recursion
// depth: aliases such as `const b = a + a` otherwise expand exponentially.
const MAX_EVALUATION_NODES: usize = 256;
const MAX_GENERATED_STRING_BYTES: usize = 4096;
const MAX_EVALUATION_STRING_BYTES: usize = 16_384;

struct EvaluationBudget {
    nodes: usize,
    string_bytes: usize,
}

impl EvaluationBudget {
    fn allocate_string(&mut self, length: usize) -> bool {
        if length > MAX_GENERATED_STRING_BYTES || length > self.string_bytes {
            return false;
        }
        self.string_bytes -= length;
        true
    }
}

fn primitive<'a>(expr: &'a Expression<'a>, ctx: &LintContext<'a>) -> Value<'a> {
    let mut budget =
        EvaluationBudget { nodes: MAX_EVALUATION_NODES, string_bytes: MAX_EVALUATION_STRING_BYTES };
    primitive_inner(expr, ctx, 0, &mut budget)
}

fn primitive_inner<'a>(
    expr: &'a Expression<'a>,
    ctx: &LintContext<'a>,
    depth: u8,
    budget: &mut EvaluationBudget,
) -> Value<'a> {
    if depth > 20 || budget.nodes == 0 {
        return Value::Unknown;
    }
    budget.nodes -= 1;
    match expr.get_inner_expression() {
        Expression::StringLiteral(s) => s.value.as_str().map_or(Value::Unknown, |s| Value::String(Cow::Borrowed(s))),
        Expression::NumericLiteral(n) => Value::Number(n.value),
        Expression::BigIntLiteral(n) => Value::BigInt(Cow::Borrowed(n.value.as_str())),
        Expression::BooleanLiteral(b) => Value::Boolean(b.value),
        Expression::NullLiteral(_) => Value::Null,
        Expression::TemplateLiteral(t) => {
            if t.expressions.is_empty() {
                return t.quasis.first().and_then(|quasi| quasi.value.cooked).and_then(JSStr::as_str).map_or(Value::Unknown, |value| Value::String(Cow::Borrowed(value)));
            }
            let mut text = String::new();
            for (index, quasi) in t.quasis.iter().enumerate() {
                let Some(cooked) = quasi.value.cooked.and_then(JSStr::as_str) else { return Value::Unknown; };
                if text.len().saturating_add(cooked.len()) > MAX_GENERATED_STRING_BYTES || !budget.allocate_string(cooked.len()) {
                    return Value::Unknown;
                }
                text.push_str(cooked);
                if let Some(expr) = t.expressions.get(index) {
                    let value = primitive_inner(expr, ctx, depth + 1, budget);
                    let Some(value) = value.string() else { return Value::Unknown; };
                    if text.len().saturating_add(value.len()) > MAX_GENERATED_STRING_BYTES || !budget.allocate_string(value.len()) {
                        return Value::Unknown;
                    }
                    text.push_str(&value);
                }
            }
            Value::String(Cow::Owned(text))
        }
        Expression::Identifier(id) => {
            if id.is_global_reference(ctx.scoping()) {
                return match id.name.as_str() {
                    "undefined" => Value::Undefined,
                    "NaN" => Value::Number(f64::NAN),
                    "Infinity" => Value::Number(f64::INFINITY),
                    _ => Value::Unknown,
                };
            }
            let Some(symbol) = ctx.scoping().get_reference(id.reference_id()).symbol_id() else { return Value::Unknown; };
            // Non-const mutation checks scan all references. Charge that work to
            // the same budget instead of repeatedly scanning an unbounded list.
            if !ctx.scoping().symbol_flags(symbol).is_const_variable() {
                let references = ctx.scoping().get_resolved_reference_ids(symbol).len();
                if references > budget.nodes { return Value::Unknown; }
                budget.nodes -= references;
            }
            let Some(decl) = immutable_declaration(id, ctx) else { return Value::Unknown; };
            if let AstKind::VariableDeclarator(decl) = decl
                && matches!(decl.id, BindingPattern::BindingIdentifier(_))
                && let Some(init) = &decl.init {
                primitive_inner(init, ctx, depth + 1, budget)
            } else { Value::Unknown }
        }
        Expression::UnaryExpression(unary) => {
            let value = primitive_inner(&unary.argument, ctx, depth + 1, budget);
            match unary.operator {
                UnaryOperator::Void if !matches!(value, Value::Unknown) => Value::Undefined,
                UnaryOperator::LogicalNot => value.boolean().map_or(Value::Unknown, |b| Value::Boolean(!b)),
                UnaryOperator::UnaryPlus if !matches!(value, Value::BigInt(_) | Value::Symbol) => value.number().map_or(Value::Unknown, Value::Number),
                UnaryOperator::UnaryNegation => match value {
                    Value::BigInt(s) => {
                        if !budget.allocate_string(s.len().saturating_add(1)) { return Value::Unknown; }
                        Value::BigInt(Cow::Owned(if let Some(stripped) = s.strip_prefix('-') { stripped.to_string() } else if s == "0" { "0".to_string() } else { format!("-{s}") }))
                    },
                    Value::Symbol => Value::Unknown,
                    value => value.number().map_or(Value::Unknown, |n| Value::Number(-n)),
                },
                _ => Value::Unknown,
            }
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let left = primitive_inner(&binary.left, ctx, depth + 1, budget);
            if matches!(left, Value::Unknown) { return Value::Unknown; }
            let right = primitive_inner(&binary.right, ctx, depth + 1, budget);
            if matches!(left, Value::String(_)) || matches!(right, Value::String(_)) {
                match (left.string(), right.string()) {
                    (Some(left), Some(right)) if budget.allocate_string(left.len().saturating_add(right.len())) => Value::String(Cow::Owned(format!("{left}{right}"))),
                    _ => Value::Unknown,
                }
            } else if !matches!(left, Value::BigInt(_) | Value::Symbol) && !matches!(right, Value::BigInt(_) | Value::Symbol) {
                match (left.number(), right.number()) {
                    (Some(left), Some(right)) => Value::Number(left + right),
                    _ => Value::Unknown,
                }
            } else { Value::Unknown }
        }
        expr if expr.get_member_expr().is_some_and(|member| {
            matches!(member.object().get_inner_expression(), Expression::Identifier(id) if id.name == "Symbol" && id.is_global_reference(ctx.scoping()))
                && member.static_property_name().and_then(JSStr::as_str).is_some_and(|s| matches!(s, "iterator" | "asyncIterator" | "hasInstance" | "isConcatSpreadable" | "match" | "matchAll" | "replace" | "search" | "species" | "split" | "toPrimitive" | "toStringTag" | "unscopables" | "dispose" | "asyncDispose"))
        }) => Value::Symbol,
        _ => Value::Unknown,
    }
}

fn immutable_declaration<'a>(
    id: &IdentifierReference<'a>,
    ctx: &LintContext<'a>,
) -> Option<AstKind<'a>> {
    let symbol = ctx.scoping().get_reference(id.reference_id()).symbol_id()?;
    if ctx.scoping().symbol_is_mutated(symbol) {
        return None;
    }
    let decl = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol)).kind();
    // Avoid following an initializer before its declaration has run.
    if matches!(decl, AstKind::VariableDeclarator(_)) && decl.span().end >= id.span.start {
        return None;
    }
    Some(decl)
}

#[derive(Clone, Copy)]
enum Global {
    Root,
    Intl,
    Constructor(&'static str),
    Supported(&'static str),
    Builtin(&'static str),
}

fn global_member(global: Global, name: &str) -> Option<Global> {
    match global {
        Global::Root => builtin(name),
        Global::Intl => CONSTRUCTORS
            .iter()
            .copied()
            .find(|&candidate| candidate == name)
            .map(Global::Constructor),
        _ => None,
    }
}

fn builtin(name: &str) -> Option<Global> {
    match name {
        "Intl" => Some(Global::Intl),
        "globalThis" | "window" | "self" | "global" => Some(Global::Root),
        "Date" => Some(Global::Builtin("Date")),
        "String" => Some(Global::Builtin("String")),
        "Number" => Some(Global::Builtin("Number")),
        "BigInt" => Some(Global::Builtin("BigInt")),
        name => TYPED_ARRAYS.iter().copied().find(|&n| n == name).map(Global::Builtin),
    }
}

fn append_global(global: Global, name: &str) -> Option<Global> {
    if let Global::Constructor(constructor) = global {
        return (constructor != "Locale" && name == "supportedLocalesOf")
            .then_some(Global::Supported(constructor));
    }
    global_member(global, name)
}

fn binding_global(
    pattern: &BindingPattern<'_>,
    symbol: SymbolId,
    global: Global,
) -> Option<Global> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => (id.symbol_id() == symbol).then_some(global),
        BindingPattern::AssignmentPattern(pattern) => binding_global(&pattern.left, symbol, global),
        BindingPattern::ObjectPattern(pattern) => pattern.properties.iter().find_map(|property| {
            let key = property.key.static_name()?;
            binding_global(&property.value, symbol, append_global(global, key.as_str()?)?)
        }),
        BindingPattern::ArrayPattern(_) => None,
    }
}

fn global<'a>(expr: &'a Expression<'a>, ctx: &LintContext<'a>, depth: u8) -> Option<Global> {
    if depth > 20 {
        return None;
    }
    let expr = expr.get_inner_expression();
    if let Expression::Identifier(id) = expr {
        if id.is_global_reference(ctx.scoping()) {
            return builtin(id.name.as_str());
        }
        let symbol = ctx.scoping().get_reference(id.reference_id()).symbol_id()?;
        let decl = immutable_declaration(id, ctx)?;
        let AstKind::VariableDeclarator(decl) = decl else {
            return None;
        };
        binding_global(&decl.id, symbol, global(decl.init.as_ref()?, ctx, depth + 1)?)
    } else {
        let member = expr.get_member_expr()?;
        // Like the upstream global tracker, follow literal member names, not
        // computed expressions whose current value merely resembles a global.
        append_global(
            global(member.object(), ctx, depth + 1)?,
            member.static_property_name()?.as_str()?,
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Receiver {
    String,
    Numeric,
    Date,
}

fn receiver_builtin(name: &str) -> Option<Receiver> {
    match name {
        "String" => Some(Receiver::String),
        "Date" => Some(Receiver::Date),
        "Number" | "BigInt" => Some(Receiver::Numeric),
        name if TYPED_ARRAYS.contains(&name) => Some(Receiver::Numeric),
        _ => None,
    }
}

fn annotation(ty: &TSType<'_>, ctx: &LintContext<'_>, depth: u8) -> Option<Receiver> {
    if depth > 20 {
        return None;
    }
    match ty {
        TSType::TSStringKeyword(_) | TSType::TSTemplateLiteralType(_) => Some(Receiver::String),
        TSType::TSNumberKeyword(_) | TSType::TSBigIntKeyword(_) => Some(Receiver::Numeric),
        TSType::TSTypeReference(reference) => {
            let TSTypeName::IdentifierReference(id) = &reference.type_name else {
                return None;
            };
            id.is_global_reference(ctx.scoping())
                .then(|| receiver_builtin(id.name.as_str()))
                .flatten()
        }
        TSType::TSLiteralType(literal) => match &literal.literal {
            TSLiteral::StringLiteral(_) | TSLiteral::TemplateLiteral(_) => Some(Receiver::String),
            TSLiteral::NumericLiteral(_)
            | TSLiteral::BigIntLiteral(_)
            | TSLiteral::UnaryExpression(_) => Some(Receiver::Numeric),
            TSLiteral::BooleanLiteral(_) => None,
        },
        TSType::TSParenthesizedType(ty) => annotation(&ty.type_annotation, ctx, depth + 1),
        TSType::TSUnionType(union) => {
            let mut result = None;
            for ty in &union.types {
                if matches!(ty, TSType::TSNullKeyword(_) | TSType::TSUndefinedKeyword(_)) {
                    continue;
                }
                let current = annotation(ty, ctx, depth + 1)?;
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
            result
        }
        _ => None,
    }
}

// Inferring the return type of String#slice also requires that the receiver's
// method has not been replaced. Reject escaped objects and member writes; a
// bounded reference scan keeps this conservative proof independent of file size.
fn unmodified_slice_receiver<'a>(
    expr: &'a Expression<'a>,
    ctx: &LintContext<'a>,
    depth: u8,
) -> bool {
    if depth > 20 {
        return false;
    }
    let Expression::Identifier(id) = expr.get_inner_expression() else {
        return true;
    };
    let Some(symbol) = ctx.scoping().get_reference(id.reference_id()).symbol_id() else {
        return false;
    };
    let references = ctx.scoping().get_resolved_reference_ids(symbol);
    if references.len() > MAX_EVALUATION_NODES {
        return false;
    }
    for &reference in references {
        let reference = ctx.scoping().get_reference(reference);
        let node = ctx.nodes().get_node(reference.node_id());
        let parent = ctx.nodes().parent_node(node.id());
        let (object, member_span) = match parent.kind() {
            AstKind::StaticMemberExpression(member) => (&member.object, member.span),
            AstKind::ComputedMemberExpression(member) => (&member.object, member.span),
            _ => return false,
        };
        if object.get_inner_expression().span() != node.kind().span() {
            return false;
        }
        match ctx.nodes().parent_kind(parent.id()) {
            AstKind::AssignmentExpression(assignment)
                if assignment.left.span().contains_inclusive(member_span) =>
            {
                return false;
            }
            AstKind::UpdateExpression(_) => return false,
            AstKind::UnaryExpression(unary) if unary.operator == UnaryOperator::Delete => {
                return false;
            }
            _ => {}
        }
    }
    if let Some(AstKind::VariableDeclarator(decl)) = immutable_declaration(id, ctx)
        && let Some(init @ Expression::Identifier(_)) = &decl.init
    {
        return unmodified_slice_receiver(init, ctx, depth + 1);
    }
    true
}

fn receiver<'a>(expr: &'a Expression<'a>, ctx: &LintContext<'a>, depth: u8) -> Option<Receiver> {
    if depth > 20 {
        return None;
    }
    match expr.get_inner_expression() {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => Some(Receiver::String),
        Expression::NumericLiteral(_) | Expression::BigIntLiteral(_) => Some(Receiver::Numeric),
        Expression::UnaryExpression(unary)
            if matches!(
                unary.operator,
                UnaryOperator::UnaryNegation | UnaryOperator::UnaryPlus
            ) =>
        {
            receiver(&unary.argument, ctx, depth + 1).filter(|r| *r == Receiver::Numeric)
        }
        Expression::NewExpression(new) => match global(&new.callee, ctx, 0)? {
            Global::Builtin(name) => receiver_builtin(name),
            _ => None,
        },
        Expression::CallExpression(call) => {
            match global(&call.callee, ctx, 0) {
                Some(Global::Builtin("Number" | "BigInt")) => return Some(Receiver::Numeric),
                Some(Global::Builtin("String")) => return Some(Receiver::String),
                _ => {}
            }
            let member = call.callee.get_member_expr()?;
            let method = member.static_property_name()?.as_str()?;
            // String#slice always returns a string; this does not infer arbitrary
            // call results or assume an unknown receiver has a built-in method.
            if method == "slice"
                && receiver(member.object(), ctx, depth + 1) == Some(Receiver::String)
                && unmodified_slice_receiver(member.object(), ctx, 0)
            {
                return Some(Receiver::String);
            }
            if !matches!(method, "from" | "of") {
                return None;
            }
            match global(member.object(), ctx, 0)? {
                Global::Builtin(name) if TYPED_ARRAYS.contains(&name) => Some(Receiver::Numeric),
                _ => None,
            }
        }
        Expression::Identifier(id) => {
            let decl = immutable_declaration(id, ctx)?;
            match decl {
                AstKind::VariableDeclarator(decl)
                    if matches!(decl.id, BindingPattern::BindingIdentifier(_)) =>
                {
                    if let Some(ty) = &decl.type_annotation {
                        return annotation(&ty.type_annotation, ctx, 0);
                    }
                    receiver(decl.init.as_ref()?, ctx, depth + 1)
                }
                AstKind::FormalParameter(param) => {
                    annotation(&param.type_annotation.as_ref()?.type_annotation, ctx, 0)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn property_name<'a>(
    property: &'a ObjectProperty<'a>,
    ctx: &LintContext<'a>,
) -> Option<Cow<'a, str>> {
    if !property.computed
        && let PropertyKey::StaticIdentifier(id) = &property.key
    {
        return Some(Cow::Borrowed(id.name.as_str()));
    }
    // Use ECMAScript conversion for numeric keys too: e.g. 1e21 and "1e+21"
    // designate the same property and must be detected as duplicates.
    let value = primitive(property.key.as_expression()?, ctx);
    match value {
        Value::String(value) | Value::BigInt(value) => Some(value),
        value => value.string().map(|s| Cow::Owned(s.into_owned())),
    }
}

fn method_name<'a>(expr: &'a Expression<'a>, ctx: &LintContext<'a>) -> Option<Cow<'a, str>> {
    let member = expr.get_member_expr()?;
    match member {
        oxc_ast::ast::MemberExpression::ComputedMemberExpression(member) => {
            primitive(&member.expression, ctx).string().map(|s| Cow::Owned(s.into_owned()))
        }
        _ => member.static_property_name().and_then(JSStr::as_str).map(Cow::Borrowed),
    }
}

impl Rule for NoInvalidIntlOptions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (callee, arguments, construct) = match node.kind() {
            AstKind::CallExpression(call) => (&call.callee, call.arguments.as_slice(), false),
            AstKind::NewExpression(new) => (&new.callee, new.arguments.as_slice(), true),
            _ => return,
        };
        let (constructor, api, index) = match global(callee, ctx, 0) {
            Some(Global::Constructor(name))
                if construct || matches!(name, "Collator" | "DateTimeFormat" | "NumberFormat") =>
            {
                (name, Cow::Owned(format!("Intl.{name}")), 1)
            }
            Some(Global::Supported(name)) if !construct => {
                ("supportedLocalesOf", Cow::Owned(format!("Intl.{name}.supportedLocalesOf")), 1)
            }
            _ if !construct => {
                let Some(member) = callee.get_member_expr() else {
                    return;
                };
                let Some(method) = method_name(callee, ctx) else {
                    return;
                };
                if !matches!(
                    method.as_ref(),
                    "localeCompare"
                        | "toLocaleString"
                        | "toLocaleDateString"
                        | "toLocaleTimeString"
                ) {
                    return;
                }
                match (method.as_ref(), receiver(member.object(), ctx, 0)) {
                    ("localeCompare", Some(Receiver::String)) => {
                        ("Collator", Cow::Borrowed("String.localeCompare"), 2)
                    }
                    ("toLocaleString", Some(Receiver::Numeric)) => {
                        ("NumberFormat", Cow::Borrowed("Number.toLocaleString"), 1)
                    }
                    (method, Some(Receiver::Date)) if method != "localeCompare" => {
                        ("DateTimeFormat", Cow::Owned(format!("Date.{method}")), 1)
                    }
                    _ => return,
                }
            }
            _ => return,
        };
        if arguments.iter().take(index + 1).any(Argument::is_spread) {
            return;
        }
        let Some(Expression::ObjectExpression(options)) = arguments
            .get(index)
            .and_then(Argument::as_expression)
            .map(Expression::get_inner_expression)
        else {
            return;
        };
        check_options(options, constructor, &api, ctx);
    }
}

const COLLATOR_OPTIONS: &[(&str, Definition)] = &[
    ("localeMatcher", Definition::Enum(&["lookup", "best fit"])),
    ("usage", Definition::Enum(&["sort", "search"])),
    ("collation", Definition::Identifier(IdentifierKind::UnicodeType)),
    ("numeric", Definition::Boolean),
    ("caseFirst", Definition::Enum(&["upper", "lower", "false"])),
    ("sensitivity", Definition::Enum(&["base", "accent", "case", "variant"])),
    ("ignorePunctuation", Definition::Boolean),
];

const DATE_TIME_FORMAT_OPTIONS: &[(&str, Definition)] = &[
    ("localeMatcher", Definition::Enum(&["lookup", "best fit"])),
    ("calendar", Definition::Identifier(IdentifierKind::UnicodeType)),
    ("numberingSystem", Definition::Identifier(IdentifierKind::UnicodeType)),
    ("hour12", Definition::Boolean),
    ("hourCycle", Definition::Enum(&["h11", "h12", "h23", "h24"])),
    ("timeZone", Definition::String),
    ("weekday", Definition::Enum(&["long", "short", "narrow"])),
    ("era", Definition::Enum(&["long", "short", "narrow"])),
    ("year", Definition::Enum(&["numeric", "2-digit"])),
    ("month", Definition::Enum(&["numeric", "2-digit", "long", "short", "narrow"])),
    ("day", Definition::Enum(&["numeric", "2-digit"])),
    ("dayPeriod", Definition::Enum(&["long", "short", "narrow"])),
    ("hour", Definition::Enum(&["numeric", "2-digit"])),
    ("minute", Definition::Enum(&["numeric", "2-digit"])),
    ("second", Definition::Enum(&["numeric", "2-digit"])),
    ("fractionalSecondDigits", Definition::Number(1, 3)),
    (
        "timeZoneName",
        Definition::Enum(&[
            "short",
            "long",
            "shortOffset",
            "longOffset",
            "shortGeneric",
            "longGeneric",
        ]),
    ),
    ("formatMatcher", Definition::Enum(&["basic", "best fit"])),
    ("dateStyle", Definition::Enum(&["full", "long", "medium", "short"])),
    ("timeStyle", Definition::Enum(&["full", "long", "medium", "short"])),
];

const DISPLAY_NAMES_OPTIONS: &[(&str, Definition)] = &[
    ("localeMatcher", Definition::Enum(&["lookup", "best fit"])),
    ("style", Definition::Enum(&["long", "short", "narrow"])),
    (
        "type",
        Definition::Enum(&[
            "language",
            "region",
            "script",
            "currency",
            "calendar",
            "dateTimeField",
        ]),
    ),
    ("fallback", Definition::Enum(&["code", "none"])),
    ("languageDisplay", Definition::Enum(&["dialect", "standard"])),
];

const DURATION_FORMAT_OPTIONS: &[(&str, Definition)] = &[
    ("localeMatcher", Definition::Enum(&["lookup", "best fit"])),
    ("numberingSystem", Definition::Identifier(IdentifierKind::UnicodeType)),
    ("style", Definition::Enum(&["long", "short", "narrow", "digital"])),
    ("years", Definition::Enum(&["long", "short", "narrow"])),
    ("months", Definition::Enum(&["long", "short", "narrow"])),
    ("weeks", Definition::Enum(&["long", "short", "narrow"])),
    ("days", Definition::Enum(&["long", "short", "narrow"])),
    ("hours", Definition::Enum(&["long", "short", "narrow", "numeric", "2-digit"])),
    ("minutes", Definition::Enum(&["long", "short", "narrow", "numeric", "2-digit"])),
    ("seconds", Definition::Enum(&["long", "short", "narrow", "numeric", "2-digit"])),
    ("milliseconds", Definition::Enum(&["long", "short", "narrow", "numeric"])),
    ("microseconds", Definition::Enum(&["long", "short", "narrow", "numeric"])),
    ("nanoseconds", Definition::Enum(&["long", "short", "narrow", "numeric"])),
    ("yearsDisplay", Definition::Enum(&["auto", "always"])),
    ("monthsDisplay", Definition::Enum(&["auto", "always"])),
    ("weeksDisplay", Definition::Enum(&["auto", "always"])),
    ("daysDisplay", Definition::Enum(&["auto", "always"])),
    ("hoursDisplay", Definition::Enum(&["auto", "always"])),
    ("minutesDisplay", Definition::Enum(&["auto", "always"])),
    ("secondsDisplay", Definition::Enum(&["auto", "always"])),
    ("millisecondsDisplay", Definition::Enum(&["auto", "always"])),
    ("microsecondsDisplay", Definition::Enum(&["auto", "always"])),
    ("nanosecondsDisplay", Definition::Enum(&["auto", "always"])),
    ("fractionalDigits", Definition::Number(0, 9)),
];

const LIST_FORMAT_OPTIONS: &[(&str, Definition)] = &[
    ("localeMatcher", Definition::Enum(&["lookup", "best fit"])),
    ("type", Definition::Enum(&["conjunction", "disjunction", "unit"])),
    ("style", Definition::Enum(&["long", "short", "narrow"])),
];

const LOCALE_OPTIONS: &[(&str, Definition)] = &[
    ("language", Definition::Identifier(IdentifierKind::Language)),
    ("script", Definition::Identifier(IdentifierKind::Script)),
    ("region", Definition::Identifier(IdentifierKind::Region)),
    ("variants", Definition::Identifier(IdentifierKind::Variants)),
    ("calendar", Definition::Identifier(IdentifierKind::UnicodeType)),
    ("collation", Definition::Identifier(IdentifierKind::UnicodeType)),
    ("firstDayOfWeek", Definition::Identifier(IdentifierKind::FirstDayOfWeek)),
    ("hourCycle", Definition::Enum(&["h11", "h12", "h23", "h24"])),
    ("caseFirst", Definition::Enum(&["upper", "lower", "false"])),
    ("numeric", Definition::Boolean),
    ("numberingSystem", Definition::Identifier(IdentifierKind::UnicodeType)),
];

const NUMBER_FORMAT_OPTIONS: &[(&str, Definition)] = &[
    ("localeMatcher", Definition::Enum(&["lookup", "best fit"])),
    ("numberingSystem", Definition::Identifier(IdentifierKind::UnicodeType)),
    ("style", Definition::Enum(&["decimal", "percent", "currency", "unit"])),
    ("currency", Definition::Identifier(IdentifierKind::Currency)),
    ("currencyDisplay", Definition::Enum(&["code", "symbol", "narrowSymbol", "name"])),
    ("currencySign", Definition::Enum(&["standard", "accounting"])),
    ("unit", Definition::Identifier(IdentifierKind::Unit)),
    ("unitDisplay", Definition::Enum(&["short", "narrow", "long"])),
    ("minimumIntegerDigits", Definition::Number(1, 21)),
    ("minimumFractionDigits", Definition::Number(0, 100)),
    ("maximumFractionDigits", Definition::Number(0, 100)),
    ("minimumSignificantDigits", Definition::Number(1, 21)),
    ("maximumSignificantDigits", Definition::Number(1, 21)),
    ("roundingIncrement", Definition::Number(1, 5000)),
    (
        "roundingMode",
        Definition::Enum(&[
            "ceil",
            "floor",
            "expand",
            "trunc",
            "halfCeil",
            "halfFloor",
            "halfExpand",
            "halfTrunc",
            "halfEven",
        ]),
    ),
    ("roundingPriority", Definition::Enum(&["auto", "morePrecision", "lessPrecision"])),
    ("trailingZeroDisplay", Definition::Enum(&["auto", "stripIfInteger"])),
    ("notation", Definition::Enum(&["standard", "scientific", "engineering", "compact"])),
    ("compactDisplay", Definition::Enum(&["short", "long"])),
    ("useGrouping", Definition::Enum(&["min2", "auto", "always", "true", "false"])),
    ("signDisplay", Definition::Enum(&["auto", "never", "always", "exceptZero", "negative"])),
];

const PLURAL_RULES_OPTIONS: &[(&str, Definition)] = &[
    ("localeMatcher", Definition::Enum(&["lookup", "best fit"])),
    ("type", Definition::Enum(&["cardinal", "ordinal"])),
    ("minimumIntegerDigits", Definition::Number(1, 21)),
    ("minimumFractionDigits", Definition::Number(0, 100)),
    ("maximumFractionDigits", Definition::Number(0, 100)),
    ("minimumSignificantDigits", Definition::Number(1, 21)),
    ("maximumSignificantDigits", Definition::Number(1, 21)),
    ("roundingIncrement", Definition::Number(1, 5000)),
    (
        "roundingMode",
        Definition::Enum(&[
            "ceil",
            "floor",
            "expand",
            "trunc",
            "halfCeil",
            "halfFloor",
            "halfExpand",
            "halfTrunc",
            "halfEven",
        ]),
    ),
    ("roundingPriority", Definition::Enum(&["auto", "morePrecision", "lessPrecision"])),
    ("trailingZeroDisplay", Definition::Enum(&["auto", "stripIfInteger"])),
    ("notation", Definition::Enum(&["standard", "scientific", "engineering", "compact"])),
    ("compactDisplay", Definition::Enum(&["short", "long"])),
];

const RELATIVE_TIME_FORMAT_OPTIONS: &[(&str, Definition)] = &[
    ("localeMatcher", Definition::Enum(&["lookup", "best fit"])),
    ("numberingSystem", Definition::Identifier(IdentifierKind::UnicodeType)),
    ("style", Definition::Enum(&["long", "short", "narrow"])),
    ("numeric", Definition::Enum(&["always", "auto"])),
];

const SEGMENTER_OPTIONS: &[(&str, Definition)] = &[
    ("localeMatcher", Definition::Enum(&["lookup", "best fit"])),
    ("granularity", Definition::Enum(&["grapheme", "word", "sentence"])),
];

// These tables describe ECMA-402, not a particular runtime's supported ICU data.
// https://tc39.es/ecma402/
fn valid_identifier(value: &str, kind: IdentifierKind) -> bool {
    let alpha = |s: &str| s.bytes().all(|b| b.is_ascii_alphabetic());
    let unicode_type = |s: &str| {
        s.split('-').all(|part| {
            (3..=8).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_alphanumeric())
        })
    };
    match kind {
        IdentifierKind::UnicodeType => unicode_type(value),
        IdentifierKind::Language => matches!(value.len(), 2 | 3 | 5..=8) && alpha(value),
        IdentifierKind::Script => value.len() == 4 && alpha(value),
        IdentifierKind::Region => {
            value.len() == 2 && alpha(value)
                || value.len() == 3 && value.bytes().all(|b| b.is_ascii_digit())
        }
        IdentifierKind::Currency => value.len() == 3 && alpha(value),
        IdentifierKind::FirstDayOfWeek => {
            value.len() == 1 && matches!(value.as_bytes()[0], b'0'..=b'7') || unicode_type(value)
        }
        IdentifierKind::Variants => {
            let mut seen: Vec<&str> = Vec::new();
            value.split('-').all(|part| {
                let valid = ((5..=8).contains(&part.len())
                    || part.len() == 4 && part.as_bytes()[0].is_ascii_digit())
                    && part.bytes().all(|b| b.is_ascii_alphanumeric());
                if !valid || seen.iter().any(|previous| previous.eq_ignore_ascii_case(part)) {
                    return false;
                }
                seen.push(part);
                true
            })
        }
        IdentifierKind::Unit => {
            const UNITS: &[&str] = &[
                "acre",
                "bit",
                "byte",
                "celsius",
                "centimeter",
                "day",
                "degree",
                "fahrenheit",
                "fluid-ounce",
                "foot",
                "gallon",
                "gigabit",
                "gigabyte",
                "gram",
                "hectare",
                "hour",
                "inch",
                "kilobit",
                "kilobyte",
                "kilogram",
                "kilometer",
                "liter",
                "megabit",
                "megabyte",
                "meter",
                "microsecond",
                "mile",
                "mile-scandinavian",
                "milliliter",
                "millimeter",
                "millisecond",
                "minute",
                "month",
                "nanosecond",
                "ounce",
                "percent",
                "petabyte",
                "pound",
                "second",
                "stone",
                "terabit",
                "terabyte",
                "week",
                "yard",
                "year",
            ];
            UNITS.contains(&value)
                || value.split_once("-per-").is_some_and(|(numerator, denominator)| {
                    UNITS.contains(&numerator) && UNITS.contains(&denominator)
                })
        }
    }
}

#[expect(
    clippy::float_cmp,
    reason = "ECMA-402 compares normalized integer options exactly, treating both zero signs as equal"
)]
fn same_integer(left: f64, right: f64) -> bool {
    left == right
}

fn normalize<'a>(
    value: &Value<'a>,
    definition: Definition,
    name: &str,
) -> Result<Value<'a>, String> {
    if matches!(value, Value::Undefined | Value::Unknown) {
        return Ok(value.clone());
    }
    if matches!(definition, Definition::Boolean) {
        return Ok(value.boolean().map_or(Value::Unknown, Value::Boolean));
    }
    if let Definition::Number(minimum, maximum) = definition {
        let number = value.number().unwrap_or(f64::NAN);
        if !number.is_finite() || number < f64::from(minimum) || number > f64::from(maximum) {
            return Err(format!("a number between {minimum} and {maximum}"));
        }
        let integer = number.floor();
        if name == "roundingIncrement"
            && !ROUNDING_INCREMENTS.iter().any(|&allowed| same_integer(integer, f64::from(allowed)))
        {
            return Err(format!(
                "one of {}",
                ROUNDING_INCREMENTS.iter().map(u16::to_string).collect::<Vec<_>>().join(", ")
            ));
        }
        return Ok(Value::Number(integer));
    }
    if name == "useGrouping"
        && (matches!(value, Value::Boolean(true)) || value.boolean() == Some(false))
    {
        return Ok(Value::Boolean(value.boolean().unwrap_or(false)));
    }
    if matches!(value, Value::Symbol) {
        return Err("convertible to a string".to_string());
    }
    let Some(string) = value.string() else {
        return Ok(Value::Unknown);
    };
    let valid = match definition {
        Definition::String => true,
        Definition::Enum(allowed) => {
            if !allowed.contains(&string.as_ref()) {
                return Err(format!(
                    "one of {}",
                    allowed.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(", ")
                ));
            }
            true
        }
        Definition::Identifier(kind) => valid_identifier(&string, kind),
        _ => unreachable!(),
    };
    if valid {
        // Borrow literal strings, retaining owned storage for evaluated expressions.
        Ok(match value {
            Value::String(s) => Value::String(s.clone()),
            _ => Value::String(Cow::Owned(string.into_owned())),
        })
    } else {
        Err("a well-formed identifier".to_string())
    }
}

struct Entry<'a> {
    name: Cow<'a, str>,
    property: &'a ObjectProperty<'a>,
    value: Value<'a>,
    definition: Option<Definition>,
}

struct Problem<'a> {
    name: Cow<'a, str>,
    span: Span,
    message: String,
    suggestion: Option<(&'static str, &'a ObjectProperty<'a>)>,
}

struct Options<'a> {
    entries: Vec<Entry<'a>>,
    problems: Vec<Problem<'a>>,
    span: Span,
}

impl<'a> Options<'a> {
    fn value(&self, name: &str) -> &Value<'a> {
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .map_or(&Value::Undefined, |entry| &entry.value)
    }

    fn default(&self, name: &str, fallback: Value<'a>) -> Value<'a> {
        match self.value(name) {
            Value::Undefined => fallback,
            value => value.clone(),
        }
    }

    fn problem(&mut self, name: &str, message: String) {
        if self.problems.iter().any(|problem| problem.name == name) {
            return;
        }
        let entry = self.entries.iter().find(|entry| entry.name == name);
        self.problems.push(Problem {
            name: entry.map_or_else(|| Cow::Owned(name.to_string()), |entry| entry.name.clone()),
            span: entry.map_or(self.span, |entry| entry.property.key.span()),
            message,
            suggestion: None,
        });
    }

    fn conflicting(&mut self, name: &str, reason: &str) {
        self.problem(name, format!("`{name}` {reason}."));
    }

    fn ignored(&mut self, name: &str, reason: &str) {
        self.problem(name, format!("`{name}` is ignored {reason}."));
    }

    fn validate(&mut self, name: &str) {
        let Some(entry) = self.entries.iter_mut().find(|entry| entry.name == name) else {
            return;
        };
        let Some(definition) = entry.definition else {
            return;
        };
        match normalize(&entry.value, definition, name) {
            Ok(value) => entry.value = value,
            Err(expected) => {
                entry.value = Value::Unknown;
                self.problem(name, format!("`{name}` must be {expected}."));
            }
        }
    }

    fn digit_usage(&self) -> (Option<bool>, Option<bool>) {
        let priority = self.default("roundingPriority", Value::String(Cow::Borrowed("auto")));
        if matches!(priority, Value::Unknown) {
            return (None, None);
        }
        if !priority.is("auto") {
            return (Some(true), Some(true));
        }
        let significant = SIGNIFICANT.iter().any(|name| self.value(name).defined());
        if !significant && SIGNIFICANT.iter().any(|name| matches!(self.value(name), Value::Unknown))
        {
            return (None, None);
        }
        let fraction = FRACTION.iter().any(|name| self.value(name).defined());
        let notation = self.default("notation", Value::String(Cow::Borrowed("standard")));
        if !significant
            && !fraction
            && (matches!(notation, Value::Unknown)
                || notation.is("compact")
                    && FRACTION.iter().any(|name| matches!(self.value(name), Value::Unknown)))
        {
            return (None, Some(false));
        }
        (
            Some(
                !significant
                    && (fraction || !matches!(notation, Value::Unknown) && !notation.is("compact")),
            ),
            Some(significant),
        )
    }

    fn digits(&mut self) {
        if self.digit_usage().1 == Some(true) {
            for name in SIGNIFICANT {
                self.validate(name);
            }
        }
        if self.digit_usage().0 == Some(true) {
            for name in FRACTION {
                self.validate(name);
            }
        }
        let (fraction, significant) = self.digit_usage();
        for ([minimum, maximum], usage) in [(FRACTION, fraction), (SIGNIFICANT, significant)] {
            if usage == Some(true)
                && let (Value::Number(min), Value::Number(max)) =
                    (self.value(minimum), self.value(maximum))
                && min > max
            {
                self.conflicting(maximum, &format!("must be greater than or equal to `{minimum}`"));
            }
        }
        let priority = self.default("roundingPriority", Value::String(Cow::Borrowed("auto")));
        let increment = self.default("roundingIncrement", Value::Number(1.0));
        if !matches!(increment, Value::Unknown | Value::Number(1.0)) {
            let reason = if !matches!(priority, Value::Unknown) && !priority.is("auto") {
                Some("requires `roundingPriority` to be \"auto\"")
            } else if significant == Some(true) {
                Some("cannot be combined with significant-digit rounding")
            } else if fraction == Some(false) {
                Some("requires fraction-digit rounding instead of the default compact rounding")
            } else if fraction == Some(true)
                && let (Value::Number(minimum), Value::Number(maximum)) =
                    (self.value(FRACTION[0]), self.value(FRACTION[1]))
                && !same_integer(*minimum, *maximum)
            {
                Some("requires equal `minimumFractionDigits` and `maximumFractionDigits`")
            } else {
                None
            };
            if let Some(reason) = reason {
                self.conflicting("roundingIncrement", reason);
            }
        }
        if significant == Some(true) && priority.is("auto") {
            for name in FRACTION {
                if self.value(name).defined() {
                    self.ignored(name, "when significant-digit rounding takes precedence");
                }
            }
        }
        let notation = self.default("notation", Value::String(Cow::Borrowed("standard")));
        if !matches!(notation, Value::Unknown)
            && !notation.is("compact")
            && self.value("compactDisplay").defined()
        {
            self.ignored("compactDisplay", "unless `notation` is \"compact\"");
        }
    }

    fn number_format(&mut self) {
        let style = self.default("style", Value::String(Cow::Borrowed("decimal")));
        for (required, companions) in [
            ("currency", &["currency", "currencyDisplay", "currencySign"][..]),
            ("unit", &["unit", "unitDisplay"][..]),
        ] {
            if style.is(required) && matches!(self.value(required), Value::Undefined) {
                self.problem(
                    "style",
                    format!("The `{required}` option is required when `style` is \"{required}\"."),
                );
            }
            if !matches!(style, Value::Unknown) && !style.is(required) {
                for name in companions {
                    if self.value(name).defined() {
                        self.ignored(name, &format!("unless `style` is \"{required}\""));
                    }
                }
            }
        }
    }

    fn date_time(&mut self, api: &str) {
        for (method, forbidden) in
            [("toLocaleDateString", "timeStyle"), ("toLocaleTimeString", "dateStyle")]
        {
            if api.ends_with(method) && self.value(forbidden).defined() {
                self.conflicting(forbidden, &format!("cannot be used with `{method}()`"));
            }
        }
        if let Some(style) =
            ["dateStyle", "timeStyle"].iter().find(|name| self.value(name).defined())
        {
            for name in DATE_COMPONENTS {
                if self.value(name).defined() {
                    self.conflicting(name, &format!("cannot be combined with `{style}`"));
                }
            }
            if self.value("formatMatcher").defined() {
                self.ignored("formatMatcher", "when `dateStyle` or `timeStyle` is used");
            }
        }
        if self.value("hour12").defined() && self.value("hourCycle").defined() {
            self.ignored("hourCycle", "when `hour12` is provided");
        }
    }

    fn duration(&mut self) {
        let base = self.default("style", Value::String(Cow::Borrowed("short")));
        let mut previous = Value::String(Cow::Borrowed(""));
        for (index, unit) in TIME_UNITS.iter().enumerate() {
            let mut style = self.value(unit).clone();
            if matches!(style, Value::Undefined) {
                style = if base.is("digital")
                    || previous.is("numeric")
                    || previous.is("2-digit")
                    || previous.is("fractional")
                {
                    Value::String(Cow::Borrowed("numeric"))
                } else if matches!(base, Value::Unknown) || matches!(previous, Value::Unknown) {
                    Value::Unknown
                } else {
                    base.clone()
                };
            }
            if index >= 3 && style.is("numeric") {
                style = Value::String(Cow::Borrowed("fractional"));
            }
            if !matches!(style, Value::Unknown)
                && !matches!(previous, Value::Unknown)
                && (previous.is("fractional") && !style.is("fractional")
                    || (previous.is("numeric") || previous.is("2-digit"))
                        && !style.is("numeric")
                        && !style.is("2-digit")
                        && !style.is("fractional"))
            {
                self.conflicting(
                    unit,
                    "must continue the preceding numeric or fractional unit style",
                );
            }
            let display = format!("{unit}Display");
            if style.is("fractional") && self.value(&display).is("always") {
                self.conflicting(&display, "cannot be \"always\" for a fractional unit");
            }
            previous = style;
        }
    }
}

fn similar(left: &str, right: &str) -> bool {
    let left: Vec<char> = left.cow_to_lowercase().chars().collect();
    let right: Vec<char> = right.cow_to_lowercase().chars().collect();
    if left == right {
        return true;
    }
    if left.len().abs_diff(right.len()) > 1 {
        return false;
    }
    let index = left.iter().zip(&right).take_while(|(left, right)| left == right).count();
    match left.len().cmp(&right.len()) {
        Ordering::Equal => {
            left[index + 1..] == right[index + 1..]
                || index + 1 < left.len()
                    && left[index] == right[index + 1]
                    && left[index + 1] == right[index]
                    && left[index + 2..] == right[index + 2..]
        }
        Ordering::Greater => left[index + 1..] == right[index..],
        Ordering::Less => left[index..] == right[index + 1..],
    }
}

fn check_options<'a>(
    options: &'a ObjectExpression<'a>,
    constructor: &str,
    api: &str,
    ctx: &LintContext<'a>,
) {
    let definitions = definitions(constructor);
    let mut state = Options {
        entries: Vec::with_capacity(options.properties.len()),
        problems: Vec::new(),
        span: options.span,
    };
    let mut names = FxHashSet::default();
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return;
        };
        if property.kind != PropertyKind::Init || property.method {
            return;
        }
        let Some(name) = property_name(property, ctx) else {
            return;
        };
        if !names.insert(name.clone())
            || name == "__proto__" && !property.computed && !property.shorthand
        {
            return;
        }
        let definition = definitions
            .iter()
            .find(|(candidate, _)| *candidate == name)
            .map(|(_, definition)| *definition);
        state.entries.push(Entry {
            name,
            property,
            definition,
            value: if definition.is_some() {
                primitive(&property.value, ctx)
            } else {
                Value::Unknown
            },
        });
    }
    // Record unknown keys before other validation, preserving one diagnostic per option.
    for index in 0..state.entries.len() {
        let entry = &state.entries[index];
        if entry.definition.is_none() {
            let name = entry.name.clone();
            let span = entry.property.key.span();
            let mut matches = definitions.iter().filter(|(candidate, _)| similar(&name, candidate));
            let replacement =
                matches.next().filter(|_| matches.next().is_none()).map(|(name, _)| *name);
            let suggestion = replacement.filter(|replacement| {
                !names.contains(*replacement) && !ctx.has_comments_between(span)
            });
            state.problems.push(Problem {
                message: format!("`{name}` is not a recognized {api} option."),
                name,
                span,
                suggestion: suggestion.map(|replacement| (replacement, entry.property)),
            });
        }
    }
    for index in 0..state.entries.len() {
        if state.entries[index].definition.is_none() {
            continue;
        }
        let name = state.entries[index].name.clone();
        if !FRACTION.contains(&name.as_ref()) && !SIGNIFICANT.contains(&name.as_ref()) {
            state.validate(&name);
        }
    }
    if matches!(constructor, "NumberFormat" | "PluralRules") {
        state.digits();
    }
    match constructor {
        "NumberFormat" => state.number_format(),
        "DateTimeFormat" => state.date_time(api),
        "DurationFormat" => state.duration(),
        "DisplayNames" => {
            if matches!(state.value("type"), Value::Undefined) {
                state.problem(
                    "type",
                    "The `type` option is required for Intl.DisplayNames.".to_string(),
                );
            } else if !matches!(state.value("type"), Value::Unknown)
                && !state.value("type").is("language")
                && state.value("languageDisplay").defined()
            {
                state.ignored("languageDisplay", "unless `type` is \"language\"");
            }
        }
        _ => {}
    }
    for problem in state.problems {
        let diagnostic = OxcDiagnostic::warn(problem.message).with_label(problem.span);
        if let Some((replacement, property)) = problem.suggestion {
            let key = ctx.source_range(problem.span);
            let text = if property.shorthand {
                format!("{replacement}: {key}")
            } else if property.computed || !matches!(property.key, PropertyKey::StaticIdentifier(_))
            {
                format!("'{replacement}'")
            } else {
                replacement.to_string()
            };
            ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                fixer
                    .replace(problem.span, text)
                    .with_message(format!("Rename `{}` to `{replacement}`.", problem.name))
            });
        } else {
            ctx.diagnostic(diagnostic);
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // Expanded from eslint-plugin-unicorn v79.0.0's upstream suite, including
    // every constructor, accepted enum value, coercion, and combination check.
    let mut pass = vec![
        r"new Intl.Collator('en', {})",
        r"new Intl.DateTimeFormat('en', {})",
        r"new Intl.DurationFormat('en', {})",
        r"new Intl.ListFormat('en', {})",
        r"new Intl.Locale('en', {})",
        r"new Intl.NumberFormat('en', {})",
        r"new Intl.PluralRules('en', {})",
        r"new Intl.RelativeTimeFormat('en', {})",
        r"new Intl.Segmenter('en', {})",
        r#"new Intl.DisplayNames('en', {type: "language"})"#,
        r#"new Intl.NumberFormat('en', {style: "currency", currency: "ZZZ", currencyDisplay: "narrowSymbol", currencySign: "accounting"})"#,
        r#"new Intl.NumberFormat('en', {style: "unit", unit: "kilometer-per-hour", unitDisplay: "long"})"#,
        r#"new Intl.NumberFormat('en', {notation: "compact", compactDisplay: "long"})"#,
        r#"new Intl.NumberFormat('en', {maximumFractionDigits: "2.9", minimumFractionDigits: 0.9})"#,
        r"new Intl.NumberFormat('en', {minimumSignificantDigits: 1.9, maximumSignificantDigits: 1.1})",
        r#"new Intl.NumberFormat('en', {roundingIncrement: "2.9", minimumFractionDigits: 3, maximumFractionDigits: 3})"#,
        r"new Intl.NumberFormat('en', {maximumFractionDigits: 0})",
        r"new Intl.NumberFormat('en', {minimumFractionDigits: 100})",
        r#"new Intl.NumberFormat('en', {roundingPriority: "morePrecision", minimumFractionDigits: 2, maximumSignificantDigits: 3})"#,
        r#"new Intl.NumberFormat('en', {style: dynamicStyle, currencyDisplay: "code"})"#,
        r#"new Intl.NumberFormat('en', {style: "currency", currency: dynamicCurrency})"#,
        r#"new Intl.DateTimeFormat('en', {dateStyle: "full", timeStyle: "long", timeZone: "Mars/Olympus"})"#,
        r#"new Intl.DateTimeFormat('en', {dateStyle: "short", year: undefined})"#,
        r#"new Intl.DateTimeFormat('en', {hour: "numeric", hour12: "false"})"#,
        r#"new Intl.DateTimeFormat('en', {hour: "numeric", hourCycle: "h23", hour12: undefined})"#,
        r#"new Intl.DateTimeFormat('en', {calendar: "unknown-calendar", numberingSystem: "unknown"})"#,
        r#"new Intl.Collator('en', {numeric: 1, ignorePunctuation: "false", caseFirst: false})"#,
        r#"new Intl.Locale('en', {language: "abcde", script: "Abcd", region: "123", variants: "1901-fonipa", calendar: "unknown", collation: "unknown", numberingSystem: "unknown"})"#,
        r#"new Intl.Locale('en', {firstDayOfWeek: "foobar"})"#,
        r"new Intl.Locale('en', {firstDayOfWeek: 0})",
        r"new Intl.Locale('en', {firstDayOfWeek: true})",
        r#"new Intl.DurationFormat('en', {style: "digital"})"#,
        r#"new Intl.DurationFormat('en', {hours: "numeric", seconds: "2-digit", milliseconds: "numeric"})"#,
        r#"new Intl.DurationFormat('en', {milliseconds: "numeric", microseconds: "numeric", fractionalDigits: 9})"#,
        r#"new Intl.DurationFormat('en', {hours: dynamicStyle, minutes: "long"})"#,
        r#"new Intl.DurationFormat('en', {milliseconds: dynamicStyle, millisecondsDisplay: "always"})"#,
        r#"new Intl.PluralRules('en', {notation: "compact", compactDisplay: "long"})"#,
        r#"new Intl.DisplayNames('en', {type: dynamicType, languageDisplay: "dialect"})"#,
        r"new Intl.NumberFormat('en', {useGrouping: true})",
        r"new Intl.NumberFormat('en', {useGrouping: false})",
        r#"new Intl.NumberFormat('en', {useGrouping: "true"})"#,
        r#"new Intl.NumberFormat('en', {useGrouping: "false"})"#,
        r#"new Intl.NumberFormat('en', {useGrouping: "auto"})"#,
        r#"new Intl.NumberFormat('en', {useGrouping: "min2"})"#,
        r#"new Intl.NumberFormat('en', {useGrouping: "always"})"#,
        r"new Intl.NumberFormat('en', {useGrouping: 0})",
        r#"new Intl.NumberFormat('en', {useGrouping: ""})"#,
        r"new Intl.NumberFormat('en', {useGrouping: null})",
        r"new Intl.NumberFormat('en', {useGrouping: NaN})",
        r"new Intl.NumberFormat('en', {useGrouping: undefined})",
        r#"new Intl.NumberFormat("en", options)"#,
        r#"const options = {style: "currency"}; new Intl.NumberFormat("en", options)"#,
        r#"new Intl.NumberFormat("en", {...options, style: "currency"})"#,
        r#"new Intl.NumberFormat("en", {style: "currency", ...options})"#,
        r#"new Intl.NumberFormat("en", {[key]: value, style: "currency"})"#,
        r#"new Intl.NumberFormat("en", {style: "currency", style: "decimal"})"#,
        r#"new Intl.NumberFormat("en", {get style() { return "currency"; }})"#,
        r#"new Intl.NumberFormat("en", {style() {}, typo: true})"#,
        r#"new Intl.NumberFormat("en", {__proto__: options, style: "currency"})"#,
        r#"new Intl.NumberFormat(...locales, {style: "currency"})"#,
        r#"Intl.NumberFormat(...locales, {style: "currency"})"#,
        r#"new Intl.NumberFormat("en", {style: getStyle()})"#,
        r#"let style = "currency"; style = "decimal"; new Intl.NumberFormat("en", {style})"#,
        r#"const object = {style: "currency"}; object.style = "decimal"; new Intl.NumberFormat("en", {style: object.style})"#,
        r#"new Intl.NumberFormat("en", {style: {toString() { throw new Error(); }}})"#,
        r#"new Intl.DisplayNames("en")"#,
        r#"Intl.DisplayNames("en", {})"#,
        r#"new Custom.NumberFormat("en", {style: "wrong"})"#,
        r#"function run(Intl) { new Intl.NumberFormat("en", {style: "wrong"}); }"#,
        r#"unknown.toLocaleString("en", {style: "wrong"})"#,
        r#"unknown.localeCompare("other", "en", {sensitivity: "wrong"})"#,
        r#"({toLocaleString() {}}).toLocaleString("en", {style: "wrong"})"#,
        r#"[1, 2].toLocaleString("en", {style: "wrong"})"#,
        r#"new Temporal.PlainDate(2026, 1, 1).toLocaleString("en", {dateStyle: "wrong"})"#,
        r#""text".toLocaleString("en", {style: "wrong"})"#,
        r#"new Date().toLocaleDateString("en", {dateStyle: "short"})"#,
        r#"new Date().toLocaleTimeString("en", {timeStyle: "short"})"#,
        r#"class Custom extends Date {} new Custom().toLocaleString("en", {dateStyle: "wrong"})"#,
        r"Intl.Collator.supportedLocalesOf('en', {localeMatcher: 'lookup'})",
        r"Intl.DateTimeFormat.supportedLocalesOf('en', {localeMatcher: 'lookup'})",
        r"Intl.DisplayNames.supportedLocalesOf('en', {localeMatcher: 'lookup'})",
        r"Intl.DurationFormat.supportedLocalesOf('en', {localeMatcher: 'lookup'})",
        r"Intl.ListFormat.supportedLocalesOf('en', {localeMatcher: 'lookup'})",
        r"Intl.NumberFormat.supportedLocalesOf('en', {localeMatcher: 'lookup'})",
        r"Intl.PluralRules.supportedLocalesOf('en', {localeMatcher: 'lookup'})",
        r"Intl.RelativeTimeFormat.supportedLocalesOf('en', {localeMatcher: 'lookup'})",
        r"Intl.Segmenter.supportedLocalesOf('en', {localeMatcher: 'lookup'})",
        r#"new Intl.Collator('en', {usage: "sort"});"#,
        r#"new Intl.Collator('en', {usage: "search"});"#,
        r#"new Intl.Collator('en', {sensitivity: "base"});"#,
        r#"new Intl.Collator('en', {sensitivity: "accent"});"#,
        r#"new Intl.Collator('en', {sensitivity: "case"});"#,
        r#"new Intl.Collator('en', {sensitivity: "variant"});"#,
        r#"new Intl.Collator('en', {caseFirst: "upper"});"#,
        r#"new Intl.Collator('en', {caseFirst: "lower"});"#,
        r#"new Intl.Collator('en', {caseFirst: "false"});"#,
        r#"new Intl.DateTimeFormat('en', {hourCycle: "h11"});"#,
        r#"new Intl.DateTimeFormat('en', {hourCycle: "h12"});"#,
        r#"new Intl.DateTimeFormat('en', {hourCycle: "h23"});"#,
        r#"new Intl.DateTimeFormat('en', {hourCycle: "h24"});"#,
        r#"new Intl.DateTimeFormat('en', {timeZoneName: "short"});"#,
        r#"new Intl.DateTimeFormat('en', {timeZoneName: "long"});"#,
        r#"new Intl.DateTimeFormat('en', {timeZoneName: "shortOffset"});"#,
        r#"new Intl.DateTimeFormat('en', {timeZoneName: "longOffset"});"#,
        r#"new Intl.DateTimeFormat('en', {timeZoneName: "shortGeneric"});"#,
        r#"new Intl.DateTimeFormat('en', {timeZoneName: "longGeneric"});"#,
        r#"new Intl.DateTimeFormat('en', {month: "numeric"});"#,
        r#"new Intl.DateTimeFormat('en', {month: "2-digit"});"#,
        r#"new Intl.DateTimeFormat('en', {month: "long"});"#,
        r#"new Intl.DateTimeFormat('en', {month: "short"});"#,
        r#"new Intl.DateTimeFormat('en', {month: "narrow"});"#,
        r#"new Intl.DateTimeFormat('en', {dateStyle: "full"});"#,
        r#"new Intl.DateTimeFormat('en', {dateStyle: "long"});"#,
        r#"new Intl.DateTimeFormat('en', {dateStyle: "medium"});"#,
        r#"new Intl.DateTimeFormat('en', {dateStyle: "short"});"#,
        r#"new Intl.DateTimeFormat('en', {formatMatcher: "basic"});"#,
        r#"new Intl.DateTimeFormat('en', {formatMatcher: "best fit"});"#,
        r#"new Intl.ListFormat('en', {type: "conjunction"});"#,
        r#"new Intl.ListFormat('en', {type: "disjunction"});"#,
        r#"new Intl.ListFormat('en', {type: "unit"});"#,
        r#"new Intl.ListFormat('en', {style: "long"});"#,
        r#"new Intl.ListFormat('en', {style: "short"});"#,
        r#"new Intl.ListFormat('en', {style: "narrow"});"#,
        r#"new Intl.Locale('en', {hourCycle: "h11"});"#,
        r#"new Intl.Locale('en', {hourCycle: "h12"});"#,
        r#"new Intl.Locale('en', {hourCycle: "h23"});"#,
        r#"new Intl.Locale('en', {hourCycle: "h24"});"#,
        r#"new Intl.Locale('en', {caseFirst: "upper"});"#,
        r#"new Intl.Locale('en', {caseFirst: "lower"});"#,
        r#"new Intl.Locale('en', {caseFirst: "false"});"#,
        r#"new Intl.PluralRules('en', {type: "cardinal"});"#,
        r#"new Intl.PluralRules('en', {type: "ordinal"});"#,
        r#"new Intl.RelativeTimeFormat('en', {numeric: "always"});"#,
        r#"new Intl.RelativeTimeFormat('en', {numeric: "auto"});"#,
        r#"new Intl.RelativeTimeFormat('en', {style: "long"});"#,
        r#"new Intl.RelativeTimeFormat('en', {style: "short"});"#,
        r#"new Intl.RelativeTimeFormat('en', {style: "narrow"});"#,
        r#"new Intl.Segmenter('en', {granularity: "grapheme"});"#,
        r#"new Intl.Segmenter('en', {granularity: "word"});"#,
        r#"new Intl.Segmenter('en', {granularity: "sentence"});"#,
        r#"new Intl.NumberFormat('en', {roundingMode: "ceil"})"#,
        r#"new Intl.NumberFormat('en', {roundingMode: "floor"})"#,
        r#"new Intl.NumberFormat('en', {roundingMode: "expand"})"#,
        r#"new Intl.NumberFormat('en', {roundingMode: "trunc"})"#,
        r#"new Intl.NumberFormat('en', {roundingMode: "halfCeil"})"#,
        r#"new Intl.NumberFormat('en', {roundingMode: "halfFloor"})"#,
        r#"new Intl.NumberFormat('en', {roundingMode: "halfExpand"})"#,
        r#"new Intl.NumberFormat('en', {roundingMode: "halfTrunc"})"#,
        r#"new Intl.NumberFormat('en', {roundingMode: "halfEven"})"#,
        r#"new Intl.NumberFormat('en', {roundingPriority: "auto"})"#,
        r#"new Intl.NumberFormat('en', {roundingPriority: "morePrecision"})"#,
        r#"new Intl.NumberFormat('en', {roundingPriority: "lessPrecision"})"#,
        r#"new Intl.NumberFormat('en', {trailingZeroDisplay: "auto"})"#,
        r#"new Intl.NumberFormat('en', {trailingZeroDisplay: "stripIfInteger"})"#,
        r#"new Intl.NumberFormat('en', {notation: "standard"})"#,
        r#"new Intl.NumberFormat('en', {notation: "scientific"})"#,
        r#"new Intl.NumberFormat('en', {notation: "engineering"})"#,
        r#"new Intl.NumberFormat('en', {notation: "compact"})"#,
        r"new Intl.NumberFormat('en', {roundingIncrement: 1, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 2, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 5, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 10, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 20, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 25, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 50, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 100, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 200, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 250, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 500, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 1000, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 2000, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 2500, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 5000, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {minimumIntegerDigits: 21, minimumFractionDigits: 0, maximumFractionDigits: 100})",
        r"new Intl.NumberFormat('en', {minimumSignificantDigits: 1, maximumSignificantDigits: 21})",
        r#"new Intl.PluralRules('en', {roundingMode: "ceil"})"#,
        r#"new Intl.PluralRules('en', {roundingMode: "floor"})"#,
        r#"new Intl.PluralRules('en', {roundingMode: "expand"})"#,
        r#"new Intl.PluralRules('en', {roundingMode: "trunc"})"#,
        r#"new Intl.PluralRules('en', {roundingMode: "halfCeil"})"#,
        r#"new Intl.PluralRules('en', {roundingMode: "halfFloor"})"#,
        r#"new Intl.PluralRules('en', {roundingMode: "halfExpand"})"#,
        r#"new Intl.PluralRules('en', {roundingMode: "halfTrunc"})"#,
        r#"new Intl.PluralRules('en', {roundingMode: "halfEven"})"#,
        r#"new Intl.PluralRules('en', {roundingPriority: "auto"})"#,
        r#"new Intl.PluralRules('en', {roundingPriority: "morePrecision"})"#,
        r#"new Intl.PluralRules('en', {roundingPriority: "lessPrecision"})"#,
        r#"new Intl.PluralRules('en', {trailingZeroDisplay: "auto"})"#,
        r#"new Intl.PluralRules('en', {trailingZeroDisplay: "stripIfInteger"})"#,
        r#"new Intl.PluralRules('en', {notation: "standard"})"#,
        r#"new Intl.PluralRules('en', {notation: "scientific"})"#,
        r#"new Intl.PluralRules('en', {notation: "engineering"})"#,
        r#"new Intl.PluralRules('en', {notation: "compact"})"#,
        r"new Intl.PluralRules('en', {roundingIncrement: 1, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {roundingIncrement: 2, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {roundingIncrement: 5, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {roundingIncrement: 10, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {roundingIncrement: 20, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {roundingIncrement: 25, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {roundingIncrement: 50, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {roundingIncrement: 100, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {roundingIncrement: 200, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {roundingIncrement: 250, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {roundingIncrement: 500, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {roundingIncrement: 1000, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {roundingIncrement: 2000, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {roundingIncrement: 2500, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {roundingIncrement: 5000, minimumFractionDigits: 2, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {minimumIntegerDigits: 21, minimumFractionDigits: 0, maximumFractionDigits: 100})",
        r"new Intl.PluralRules('en', {minimumSignificantDigits: 1, maximumSignificantDigits: 21})",
        r#"new Intl.NumberFormat('en', {style: "currency", currency: "usd", currencyDisplay: "code"})"#,
        r#"new Intl.NumberFormat('en', {style: "currency", currency: "usd", currencyDisplay: "symbol"})"#,
        r#"new Intl.NumberFormat('en', {style: "currency", currency: "usd", currencyDisplay: "narrowSymbol"})"#,
        r#"new Intl.NumberFormat('en', {style: "currency", currency: "usd", currencyDisplay: "name"})"#,
        r#"new Intl.NumberFormat('en', {style: "unit", unit: "meter", unitDisplay: "short"})"#,
        r#"new Intl.NumberFormat('en', {style: "unit", unit: "meter", unitDisplay: "narrow"})"#,
        r#"new Intl.NumberFormat('en', {style: "unit", unit: "meter", unitDisplay: "long"})"#,
        r#"new Intl.NumberFormat('en', {signDisplay: "auto"})"#,
        r#"new Intl.NumberFormat('en', {signDisplay: "never"})"#,
        r#"new Intl.NumberFormat('en', {signDisplay: "always"})"#,
        r#"new Intl.NumberFormat('en', {signDisplay: "exceptZero"})"#,
        r#"new Intl.NumberFormat('en', {signDisplay: "negative"})"#,
        r#"new Intl.DisplayNames('en', {type: "language"});"#,
        r#"new Intl.DisplayNames('en', {type: "region"});"#,
        r#"new Intl.DisplayNames('en', {type: "script"});"#,
        r#"new Intl.DisplayNames('en', {type: "currency"});"#,
        r#"new Intl.DisplayNames('en', {type: "calendar"});"#,
        r#"new Intl.DisplayNames('en', {type: "dateTimeField"});"#,
        r#"new Intl.DisplayNames('en', {type: "language", style: "long"})"#,
        r#"new Intl.DisplayNames('en', {type: "language", style: "short"})"#,
        r#"new Intl.DisplayNames('en', {type: "language", style: "narrow"})"#,
        r#"new Intl.DisplayNames('en', {type: "language", fallback: "code"})"#,
        r#"new Intl.DisplayNames('en', {type: "language", fallback: "none"})"#,
        r#"new Intl.DisplayNames('en', {type: "language", languageDisplay: "dialect"})"#,
        r#"new Intl.DisplayNames('en', {type: "language", languageDisplay: "standard"})"#,
        r#"new Intl.DurationFormat('en', {style: "long"});"#,
        r#"new Intl.DurationFormat('en', {style: "short"});"#,
        r#"new Intl.DurationFormat('en', {style: "narrow"});"#,
        r#"new Intl.DurationFormat('en', {style: "digital"});"#,
        r"new Intl.DurationFormat('en', {fractionalDigits: 0})",
        r"new Intl.DateTimeFormat('en', {fractionalSecondDigits: 1})",
        r"new Intl.NumberFormat('en', {minimumIntegerDigits: true, maximumFractionDigits: null})",
        r"new Intl.NumberFormat('en', {style: undefined, currency: undefined, compactDisplay: undefined})",
        r"new Intl.NumberFormat('en', {roundingPriority: dynamicPriority, maximumFractionDigits: -1, maximumSignificantDigits: 0})",
        r"new Intl.NumberFormat('en', {maximumSignificantDigits: dynamicDigits, maximumFractionDigits: -1})",
        r#"new Intl.NumberFormat('en', {notation: dynamicNotation, compactDisplay: "long"})"#,
        r#"new Intl.NumberFormat('en', {notation: "compact", roundingIncrement: 2, maximumFractionDigits: dynamicDigits})"#,
        r#"let key = "style"; key = "numeric"; new Intl.NumberFormat("en", {[key]: "wrong"})"#,
        r#"let method = "toLocaleString"; method = "custom"; (1)[method]("en", {style: "wrong"})"#,
        r#"const name = "NumberFormat"; new Intl[name]("en", {style: "wrong"})"#,
        r#"class Custom extends String {} new Custom("a").localeCompare("b", "en", {sensitivity: "wrong"})"#,
        r#"new Intl.DurationFormat('en', {style: dynamicStyle, millisecondsDisplay: "always"})"#,
        r#"Intl.NumberFormat.supportedLocalesOf("en", {...options, localeMatcher: "wrong"})"#,
        r#""a".localeCompare(...arguments_, "en", {sensitivity: "wrong"})"#,
        r#"new Date().toLocaleDateString("en", ...options)"#,
        r#"new Intl.NumberFormat("en", {[Symbol.iterator]: true, style: "currency"})"#,
        r#"new Intl.NumberFormat("en", {set style(value) {}, typo: true})"#,
        r#"new Intl.NumberFormat("en", {style: "currency", ["style"]: "decimal"})"#,
        r#"const style = (() => { throw new Error(); })(); new Intl.NumberFormat("en", {style})"#,
        r#"function run(value) { return value.toLocaleDateString("en", {timeStyle: "short"}); }"#,
        r"new Intl.Collator('en', {numeric: Symbol.iterator})",
        r#"new Intl.DurationFormat('en', {style: dynamicStyle, hours: "numeric", seconds: "numeric", milliseconds: "numeric"})"#,
        r"function run(value: string | number) { value?.localeCompare('other', 'en', {sensitivity: 'wrong'}); }",
        r"function run(value: string | number | undefined) { value?.localeCompare('other', 'en', {sensitivity: 'wrong'}); }",
        r#"function run(value: string | number | undefined) { value?.localeCompare("other", "en", {sensitivity: "wrong"}); }"#,
        r#"function run(value: Date | number) { value.toLocaleString("en", {style: "wrong"}); }"#,
        r#"class Foo { #toLocaleString() {} bar() { this.#toLocaleString("en", {style: "wrong"}); } }"#,
        r#"function run(globalThis) { new globalThis.Intl.NumberFormat("en", {style: "wrong"}); }"#,
        r#"function run(Date) { new Date().toLocaleString("en", {dateStyle: "wrong"}); }"#,
        r#"const NF = Intl.NumberFormat; let Formatter = NF; Formatter = custom; new Formatter("en", {style: "wrong"})"#,
        r#"const options = {get style() {throw new Error()}}; new Intl.NumberFormat("en", options)"#,
        r#"function run(value: string | number | undefined) { value?.localeCompare("x", "en", {style: "wrong"}); }"#,
        r#"new Intl.NumberFormat("en", {style: Symbol[dynamic]})"#,
        r#"new Intl.NumberFormat("en", {1e21: 0, "1e+21": 1, style: "currency"})"#,
        r"const options = new Intl.NumberFormat('en', {1: true, ['1']: false, style: 'currency'})",
    ];
    // These three upstream failures need call-result or property-type inference
    // from its TypeScript type service, beyond this rule's syntax analysis.
    // An unknown receiver must remain conservative without that information.
    pass.extend([
        r"const s = new String('a'); s.slice = () => ({localeCompare() {}}); s.slice().localeCompare('x', 'en', {sensitivity: 'wrong'});",
        r"const s = new String('a'); const alias = s; s['slice'] = custom; alias.slice().localeCompare('x', 'en', {sensitivity: 'wrong'});",
        r"const s = new String('a'); replaceMethods(s); s.slice().localeCompare('x', 'en', {sensitivity: 'wrong'});",
        r"const {[name]: Formatter} = Intl; new Formatter('en', {style: 'wrong'})",
        r"unknown.slice(0).localeCompare('x', 'en', {sensitivity: 'wrong'})",
        r"({slice() { return 1; }}).slice().localeCompare('x', 'en', {sensitivity: 'wrong'})",
        r"function run(String) { new String('a').slice().localeCompare('x', 'en', {sensitivity: 'wrong'}); }",
        r#"const value = Math.random(); value.toLocaleString("en", {style: "wrong"});"#,
        r#"const value = new Uint8Array(2).slice(0); value.toLocaleString("en", {style: "wrong"});"#,
        r#"declare const holder: {value: 1n}; holder.value.toLocaleString("en", {style: "wrong"});"#,
    ]);
    let fail = vec![
        r#"new Intl.NumberFormat("en", {maximumFractionDigits: 0, maximumSignificantDigits: Symbol.iterator})"#,
        r"const {['NumberFormat']: Formatter} = Intl; new Formatter('en', {style: 'wrong'})",
        r"const {NumberFormat: Formatter = Custom} = Intl; new Formatter('en', {style: 'wrong'})",
        r"function run(text: string) { text.slice(1).localeCompare('x', 'en', {sensitivity: 'wrong'}); }",
        r#"const value = "a".slice(0); value.localeCompare("other", "en", {sensitivity: "wrong"});"#,
        r"new Intl.Collator('en', {typo: true})",
        r"new Intl.DateTimeFormat('en', {typo: true})",
        r"new Intl.DisplayNames('en', {typo: true})",
        r"new Intl.DurationFormat('en', {typo: true})",
        r"new Intl.ListFormat('en', {typo: true})",
        r"new Intl.Locale('en', {typo: true})",
        r"new Intl.NumberFormat('en', {typo: true})",
        r"new Intl.PluralRules('en', {typo: true})",
        r"new Intl.RelativeTimeFormat('en', {typo: true})",
        r"new Intl.Segmenter('en', {typo: true})",
        r"Intl.Collator.supportedLocalesOf('en', {style: 'short'})",
        r"Intl.DateTimeFormat.supportedLocalesOf('en', {style: 'short'})",
        r"Intl.DisplayNames.supportedLocalesOf('en', {style: 'short'})",
        r"Intl.DurationFormat.supportedLocalesOf('en', {style: 'short'})",
        r"Intl.ListFormat.supportedLocalesOf('en', {style: 'short'})",
        r"Intl.NumberFormat.supportedLocalesOf('en', {style: 'short'})",
        r"Intl.PluralRules.supportedLocalesOf('en', {style: 'short'})",
        r"Intl.RelativeTimeFormat.supportedLocalesOf('en', {style: 'short'})",
        r"Intl.Segmenter.supportedLocalesOf('en', {style: 'short'})",
        r"new Intl.NumberFormat('en', {maximumFractionDigit: 0})",
        r#"new Intl.NumberFormat('en', {Style: "decimal"})"#,
        r#"new Intl.NumberFormat('en', {currencyDispaly: "code"})"#, // spellchecker:disable-line
        r#"new Intl.NumberFormat('en', {style: "curreny"})"#,        // spellchecker:disable-line
        r#"new Intl.NumberFormat('en', {style: "currency"})"#,
        r#"new Intl.NumberFormat('en', {style: "currency", currency: undefined})"#,
        r#"new Intl.NumberFormat('en', {style: "unit"})"#,
        r#"new Intl.NumberFormat('en', {style: "unit", unit: "MILE"})"#,
        r#"new Intl.NumberFormat('en', {style: "unit", unit: "meter-per-second-per-hour"})"#,
        r#"new Intl.NumberFormat('en', {style: "currency", currency: "US"})"#,
        r#"new Intl.NumberFormat('en', {style: "currency", currency: 123})"#,
        r#"new Intl.NumberFormat('en', {currency: "USD", currencyDisplay: "code", currencySign: "accounting"})"#,
        r#"new Intl.NumberFormat('en', {unit: "meter", unitDisplay: "long"})"#,
        r#"new Intl.NumberFormat('en', {compactDisplay: "long"})"#,
        r#"new Intl.NumberFormat('en', {currencyDisplay: "wrong"})"#,
        r"new Intl.NumberFormat('en', {useGrouping: 1})",
        r#"new Intl.NumberFormat('en', {useGrouping: "sometimes"})"#,
        r"new Intl.NumberFormat('en', {minimumIntegerDigits: 0})",
        r"new Intl.NumberFormat('en', {maximumFractionDigits: 100.1})",
        r"new Intl.NumberFormat('en', {maximumFractionDigits: NaN})",
        r"new Intl.NumberFormat('en', {maximumFractionDigits: Infinity})",
        r"new Intl.NumberFormat('en', {maximumFractionDigits: 1n})",
        r"new Intl.NumberFormat('en', {minimumFractionDigits: 3, maximumFractionDigits: 2})",
        r"new Intl.NumberFormat('en', {minimumSignificantDigits: 3, maximumSignificantDigits: 2})",
        r"new Intl.NumberFormat('en', {maximumSignificantDigits: 3, maximumFractionDigits: -1})",
        r"new Intl.NumberFormat('en', {maximumSignificantDigits: 3, minimumFractionDigits: 2, maximumFractionDigits: 1})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 3})",
        r#"new Intl.NumberFormat('en', {roundingIncrement: 2, roundingPriority: "morePrecision"})"#,
        r"new Intl.NumberFormat('en', {roundingIncrement: 2, maximumSignificantDigits: 3})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 2, minimumFractionDigits: 1, maximumFractionDigits: 2})",
        r"new Intl.PluralRules('en', {minimumIntegerDigits: 22})",
        r#"new Intl.PluralRules('en', {type: "plural"})"#,
        r#"new Intl.PluralRules('en', {compactDisplay: "long"})"#,
        r#"new Intl.DateTimeFormat('en', {dateStyle: "short", year: "numeric"})"#,
        r#"new Intl.DateTimeFormat('en', {timeStyle: "short", fractionalSecondDigits: 3})"#,
        r#"new Intl.DateTimeFormat('en', {dateStyle: "short", timeZoneName: "long"})"#,
        r#"new Intl.DateTimeFormat('en', {dateStyle: "short", formatMatcher: "basic"})"#,
        r#"new Intl.DateTimeFormat('en', {hour: "numeric", hour12: false, hourCycle: "h23"})"#,
        r#"new Intl.DateTimeFormat('en', {year: "long"})"#,
        r"new Intl.DateTimeFormat('en', {fractionalSecondDigits: 0})",
        r#"new Intl.DateTimeFormat('en', {calendar: "ab"})"#,
        r#"new Intl.Collator('en', {sensitivity: "wrong"})"#,
        r#"new Intl.Collator('en', {collation: "ab"})"#,
        r#"new Intl.Locale('en', {language: "abcd", script: "Latin", region: "USA", variants: "fonipa-FONIPA"})"#,
        r#"new Intl.Locale('en', {firstDayOfWeek: "8"})"#,
        r#"new Intl.Locale('en', {localeMatcher: "lookup"})"#,
        r"new Intl.DisplayNames('en', {})",
        r"new Intl.DisplayNames('en', {type: undefined})",
        r#"new Intl.DisplayNames('en', {type: "languages"})"#,
        r#"new Intl.DisplayNames('en', {type: "region", languageDisplay: "standard"})"#,
        r#"new Intl.ListFormat('en', {type: "and", style: "full"})"#,
        r#"new Intl.RelativeTimeFormat('en', {numeric: true, style: "full"})"#,
        r#"new Intl.Segmenter('en', {granularity: "character"})"#,
        r#"new Intl.DurationFormat('en', {style: "full", fractionalDigits: 10})"#,
        r#"new Intl.DurationFormat('en', {years: "numeric"})"#,
        r#"new Intl.DurationFormat('en', {hours: "numeric", seconds: "long"})"#,
        r#"new Intl.DurationFormat('en', {style: "digital", minutes: "long"})"#,
        r#"new Intl.DurationFormat('en', {milliseconds: "numeric", microseconds: "long"})"#,
        r#"new Intl.DurationFormat('en', {milliseconds: "numeric", millisecondsDisplay: "always"})"#,
        r#"Intl.NumberFormat("en", {style: "wrong"})"#,
        r#"Intl.DateTimeFormat("en", {dateStyle: "wrong"})"#,
        r#"Intl.Collator("en", {usage: "wrong"})"#,
        r#"const {NumberFormat: Formatter} = Intl; new Formatter("en", {style: "wrong"})"#,
        r#"globalThis.Intl.NumberFormat("en", {style: "wrong"})"#,
        r#"new Intl["NumberFormat"]("en", {style: "wrong"})"#,
        r#"Intl?.NumberFormat?.("en", {style: "wrong"})"#,
        r#"new Intl.NumberFormat("en", {["maximumFractionDigit"]: 0})"#,
        r#"new Intl.NumberFormat("en", {[`maximumFractionDigit`]: 0})"#,
        r#"const maximumFractionDigit = 0; new Intl.NumberFormat("en", {maximumFractionDigit})"#,
        r#"new Intl.NumberFormat("en", {maximumFractionDigit: 0, maximumFractionDigits: 2})"#,
        r#"new Intl.DateTimeFormat("en", {ear: "numeric"})"#,
        r#"new Intl.NumberFormat("en", {[/* keep */ "maximumFractionDigit"]: 0})"#,
        r#"new Intl.NumberFormat("en", {maximumFractionDigit /* keep */: 0})"#,
        r#"const style = "wrong"; new Intl.NumberFormat("en", {style})"#,
        r#"new Intl.NumberFormat("en", {style: "wrong"}, ...extra)"#,
        r#""text".localeCompare("other", "en", {sensitivity: "wrong"})"#,
        r#""text"?.localeCompare?.("other", "en", {sensitivity: "wrong"})"#,
        r#"(1).toLocaleString("en", {style: "currency"})"#,
        r#"1n.toLocaleString("en", {maximumFractionDigits: -1})"#,
        r#"new Number(1).toLocaleString("en", {style: "wrong"})"#,
        r#"new Uint8Array(2).toLocaleString("en", {style: "wrong"})"#,
        r#"BigInt64Array.of(1n).toLocaleString("en", {style: "wrong"})"#,
        r#"new Date().toLocaleString("en", {dateStyle: "wrong"})"#,
        r#"new Date().toLocaleDateString("en", {timeStyle: "short"})"#,
        r#"new Date().toLocaleTimeString("en", {dateStyle: "short"})"#,
        r#"const date = new Date(); date.toLocaleString("en", {dateStyle: "wrong"})"#,
        r"new Intl.NumberFormat('en', {maximumFractionDigit: 0} as Intl.NumberFormatOptions)",
        r"new Intl.NumberFormat('en', {maximumFractionDigit: 0} satisfies Intl.NumberFormatOptions)",
        r"new Intl.NumberFormat('en', {maximumFractionDigit: 0} !)",
        r#"function run(value: number) { value.toLocaleString("en", {style: "wrong"}); }"#,
        r#"function run(value: bigint) { value.toLocaleString("en", {style: "wrong"}); }"#,
        r#"function run(value: Date) { value.toLocaleDateString("en", {timeStyle: "short"}); }"#,
        r#"function run(value: Uint8Array) { value.toLocaleString("en", {style: "wrong"}); }"#,
        r#"function run(value: Date | undefined) { value?.toLocaleDateString("en", {timeStyle: "short"}); }"#,
        r"new Intl.NumberFormat('en', {minimumIntegerDigits: 21.1})",
        r"new Intl.NumberFormat('en', {minimumFractionDigits: -0.1})",
        r"new Intl.NumberFormat('en', {maximumSignificantDigits: 21.1})",
        r"new Intl.NumberFormat('en', {minimumSignificantDigits: 0.9})",
        r"new Intl.NumberFormat('en', {roundingIncrement: 5000.1})",
        r#"new Intl.NumberFormat('en', {roundingMode: "up"})"#,
        r#"new Intl.NumberFormat('en', {roundingPriority: "both"})"#,
        r#"new Intl.NumberFormat('en', {trailingZeroDisplay: "never"})"#,
        r#"new Intl.NumberFormat('en', {notation: "decimal"})"#,
        r#"new Intl.NumberFormat('en', {roundingPriority: "lessPrecision", minimumFractionDigits: 2, maximumFractionDigits: 1})"#,
        r"new Intl.PluralRules('en', {minimumIntegerDigits: 21.1})",
        r"new Intl.PluralRules('en', {minimumFractionDigits: -0.1})",
        r"new Intl.PluralRules('en', {maximumSignificantDigits: 21.1})",
        r"new Intl.PluralRules('en', {minimumSignificantDigits: 0.9})",
        r"new Intl.PluralRules('en', {roundingIncrement: 5000.1})",
        r#"new Intl.PluralRules('en', {roundingMode: "up"})"#,
        r#"new Intl.PluralRules('en', {roundingPriority: "both"})"#,
        r#"new Intl.PluralRules('en', {trailingZeroDisplay: "never"})"#,
        r#"new Intl.PluralRules('en', {notation: "decimal"})"#,
        r#"new Intl.PluralRules('en', {roundingPriority: "lessPrecision", minimumFractionDigits: 2, maximumFractionDigits: 1})"#,
        r"new Intl.DateTimeFormat('en', {fractionalSecondDigits: 3.1})",
        r"new Intl.DurationFormat('en', {fractionalDigits: -0.1})",
        r#"new Intl.DurationFormat('en', {hours: "2-digit", minutes: "short"})"#,
        r#"new Intl.DurationFormat('en', {style: "digital", microsecondsDisplay: "always"})"#,
        r#"new Intl.DurationFormat('en', {seconds: "numeric", nanoseconds: "long"})"#,
        r#"new Intl.DurationFormat('en', {microseconds: "numeric", nanoseconds: "short"})"#,
        r#"new Intl.NumberFormat('en', {currency: "US", unitDisplay: "wide"})"#,
        r#"new Intl.DisplayNames('en', {type: "region", languageDisplay: "wrong"})"#,
        r#"new Intl.DateTimeFormat('en', {hour12: true, hourCycle: "wrong"})"#,
        r#"new Intl.Locale('en', {variants: "", calendar: "too-long-identifier"})"#,
        r#"new Intl.NumberFormat('en', {notation: "compact", roundingIncrement: 2})"#,
        r#"new Intl.PluralRules('en', {notation: "compact", roundingIncrement: 2})"#,
        "new Intl.Locale('en', {variants: \"fonip\u{212a}\"})",
        r#"const key = "style"; new Intl.NumberFormat("en", {[key]: "wrong"})"#,
        r#"const key = "maximumFractionDigit"; new Intl.NumberFormat("en", {[key]: 2})"#,
        r#"const method = "toLocaleString"; (1)[method]("en", {style: "wrong"})"#,
        r#"new String("a").localeCompare("b", "en", {sensitivity: "wrong"})"#,
        r#"function run(value: String) { value.localeCompare("other", "en", {sensitivity: "wrong"}); }"#,
        r#"const {supportedLocalesOf: supported} = Intl.Collator; supported("en", {localeMathcer: "lookup"})"#, // spellchecker:disable-line
        r#"new Intl.NumberFormat("en", {stylee: "decimal", styl: "decimal", stxle: "decimal"})"#, // spellchecker:disable-line
        r#"new Intl.NumberFormat("en", {["maximum" + /* keep */ "FractionDigit"]: 0})"#,
        r#"new Intl.NumberFormat("en", (<Intl.NumberFormatOptions>{maximumFractionDigit: 0}))"#,
        r#"function run(value: string) { value.localeCompare("other", "en", {sensitivity: "wrong"}); }"#,
        r"function run(value: 1) { value.toLocaleString('en', {style: 'wrong'}); }",
        r"function run(value: 1n) { value.toLocaleString('en', {style: 'wrong'}); }",
        r"function run(value: -1) { value.toLocaleString('en', {style: 'wrong'}); }",
        r"function run(value: -1n) { value.toLocaleString('en', {style: 'wrong'}); }",
        r#"function run(value: number) { (value as 1).toLocaleString("en", {style: "wrong"}); }"#,
        r"new Intl.NumberFormat('en', {style: Symbol.iterator})",
        r"new Intl.NumberFormat('en', {maximumFractionDigits: Symbol.iterator})",
        r"new Intl.NumberFormat('en', {useGrouping: Symbol.iterator})",
        r"new Intl.DateTimeFormat('en', {timeZone: Symbol.iterator})",
        r#"new Intl.DurationFormat('en', {style: dynamicStyle, hours: "numeric", seconds: "long"})"#,
        r#"new Intl.DurationFormat('en', {style: dynamicStyle, seconds: "numeric", millisecondsDisplay: "always"})"#,
        r#"new Intl.DurationFormat('en', {style: dynamicStyle, milliseconds: "numeric", nanoseconds: "long"})"#,
        r"function run(value: string | undefined) { value?.localeCompare('other', 'en', {sensitivity: 'wrong'}); }",
        r#"function run(value: "a" | null) { value?.localeCompare('other', 'en', {sensitivity: 'wrong'}); }"#,
        r"function run(value: String | undefined) { value?.localeCompare('other', 'en', {sensitivity: 'wrong'}); }",
        r#"function run(value: string | undefined) { value?.localeCompare("other", "en", {sensitivity: "wrong"}); }"#,
        r"new (Intl.NumberFormat as typeof Intl.NumberFormat)('en', {style: 'wrong'})",
        r"new (Intl.NumberFormat !)('en', {style: 'wrong'})",
        r"new (Intl.NumberFormat satisfies typeof Intl.NumberFormat)('en', {style: 'wrong'})",
        r#"const Formatter = Intl.NumberFormat as typeof Intl.NumberFormat; new Formatter("en", {style: "wrong"})"#,
        r#"const maximumFractionDigit = 2; new Intl.NumberFormat("en", {maximumFractionDigit /* keep */})"#,
        r#"function run(value: number | bigint | Uint8Array | undefined) { value?.toLocaleString("en", {style: "wrong"}); }"#,
        r"new Intl.NumberFormat('en', {maximumSignificantDigits: 3, maximumFractionDigits: Symbol.iterator})",
        r#"function check(text: `abc`) { text.localeCompare("x", "en", {usage: "invalid"}); }"#,
        r#"const Formatter = Intl.NumberFormat; new Formatter("en", {style: "wrong"})"#,
        r#"const {Intl: {NumberFormat: Formatter}} = globalThis; new Formatter("en", {style: "wrong"})"#,
        r#"const {NumberFormat: {supportedLocalesOf: supported}} = Intl; supported("en", {localeMatcher: "wrong"})"#,
        r#"const internationalization = Intl; const Formatter = internationalization.NumberFormat; Formatter("en", {style: "wrong"})"#,
        r#"new window.Intl.NumberFormat("en", {style: "wrong"})"#,
        r#"new Intl.NumberFormat("en", {style: `curr${"ency"}`})"#,
        r#"new Intl.NumberFormat("en", {minimumIntegerDigits: "0x16"})"#,
        r#"new Intl.NumberFormat("en", {minimumIntegerDigits: "\u00a0 22 \u00a0"})"#,
        r#"new Intl.Locale("en", {calendar: "foo--bar"})"#,
        r#"new Intl.Locale("en", {variants: "1901-1901"})"#,
        r#"new Intl.DurationFormat("en", {style: "digital", millisecondsDisplay: "always"})"#,
        r#"new Intl.NumberFormat("en", {maximumSignificantDigits: 3, maximumFractionDigits: Symbol.iterator})"#,
        r#"function run(value: (number | bigint)) { value.toLocaleString("en", {style: "wrong"}); }"#,
    ];
    let fix = vec![
        (
            r"new Intl.NumberFormat('en', {maximumFractionDigit: 0})",
            r"new Intl.NumberFormat('en', {maximumFractionDigits: 0})",
        ),
        (
            r"new Intl.NumberFormat('en', {Style: 'decimal'})",
            r"new Intl.NumberFormat('en', {style: 'decimal'})",
        ),
        (
            r"new Intl.NumberFormat('en', {currencyDispaly: 'code'})", // spellchecker:disable-line
            r"new Intl.NumberFormat('en', {currencyDisplay: 'code'})",
        ),
        (
            r"new Intl.NumberFormat('en', {['maximumFractionDigit']: 0})",
            r"new Intl.NumberFormat('en', {['maximumFractionDigits']: 0})",
        ),
        (
            r"const maximumFractionDigit = 2; new Intl.NumberFormat('en', {maximumFractionDigit})",
            r"const maximumFractionDigit = 2; new Intl.NumberFormat('en', {maximumFractionDigits: maximumFractionDigit})",
        ),
        (
            r"new Intl.NumberFormat('en', {maximumFractionDigit /* keep */: 0})",
            r"new Intl.NumberFormat('en', {maximumFractionDigits /* keep */: 0})",
        ),
        (
            r"new Intl.NumberFormat('en', {[/* keep */ 'maximumFractionDigit']: 0})",
            r"new Intl.NumberFormat('en', {[/* keep */ 'maximumFractionDigits']: 0})",
        ),
        (
            r"new Intl.NumberFormat('en', {maximumFractionDigit: 0, maximumFractionDigits: 2})",
            r"new Intl.NumberFormat('en', {maximumFractionDigit: 0, maximumFractionDigits: 2})",
        ),
        (
            r"new Intl.DateTimeFormat('en', {ear: 'numeric'})",
            r"new Intl.DateTimeFormat('en', {ear: 'numeric'})",
        ),
        (
            r"new Intl.NumberFormat('en', {['maximum' + /* keep */ 'FractionDigit']: 0})",
            r"new Intl.NumberFormat('en', {['maximum' + /* keep */ 'FractionDigit']: 0})",
        ),
    ];
    Tester::new(NoInvalidIntlOptions::NAME, NoInvalidIntlOptions::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_evaluation_limits() {
    use std::fmt::Write;

    use crate::tester::Tester;

    // The small source expands into hundreds of repeated alias evaluations.
    // Stop at the shared work budget rather than exploring every branch.
    let mut aliases = String::from("const piece0 = 'x';");
    let mut numbers = String::from("const count0 = 1;");
    for index in 1..=8 {
        let previous = index - 1;
        write!(aliases, "const piece{index} = piece{previous} + piece{previous};").unwrap();
        write!(numbers, "const count{index} = count{previous} + count{previous};").unwrap();
    }
    aliases.push_str("new Intl.NumberFormat('en', {style: piece8});");
    numbers.push_str("new Intl.NumberFormat('en', {minimumIntegerDigits: count8});");
    let large = "x".repeat(MAX_GENERATED_STRING_BYTES + 1);
    let pass = vec![
        aliases,
        numbers,
        format!("new Intl.NumberFormat('en', {{style: '{large}' + ''}});"),
        format!("new Intl.NumberFormat('en', {{style: `{large}${{''}}`}});"),
    ];
    // A borrowed literal does not consume the generated-string budget.
    let fail = vec![format!("new Intl.NumberFormat('en', {{style: '{large}'}});")];
    Tester::new(NoInvalidIntlOptions::NAME, NoInvalidIntlOptions::PLUGIN, pass, fail)
        .intentionally_allow_no_fix_tests()
        .test();
}
