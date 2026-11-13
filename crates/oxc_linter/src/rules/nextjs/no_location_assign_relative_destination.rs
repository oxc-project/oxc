use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    rc::Rc,
};

use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::{
    AstKind,
    ast::{AssignmentTarget, Expression, IdentifierReference, MemberExpression, TemplateLiteral},
};
use oxc_cfg::{
    BlockNodeId, EdgeType,
    graph::{Direction, visit::EdgeRef},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{GlobalContext, ToBoolean, ToJsString};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, SymbolId};
use oxc_span::{GetSpan, Span};
use oxc_str::JSStr;
use oxc_syntax::{
    number::ToJsString as _,
    operator::{AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator},
};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_location_assign_relative_destination_diagnostic(
    span: Span,
    expression: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Do not use `{expression}` to navigate to internal Next.js pages."
    ))
        .with_help(
            "Use `redirect()` during rendering, or `useRouter().push()` in a Client Component event handler.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoLocationAssignRelativeDestination;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents assignments to `location.href` and calls to `location.assign()`
    /// with relative URLs.
    ///
    /// ### Why is this bad?
    ///
    /// Next.js cannot apply client-side routing optimizations when browser location
    /// APIs navigate directly to an internal page. Use Next.js navigation APIs to
    /// avoid a full page reload.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// location.href = "/dashboard";
    /// window.location.assign("/profile");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// location.href = "https://example.com";
    /// window.location.assign(externalUrl);
    /// ```
    NoLocationAssignRelativeDestination,
    nextjs,
    correctness,
    version = "next",
    short_description = "Prevents browser location APIs from navigating to internal Next.js pages.",
);

impl Rule for NoLocationAssignRelativeDestination {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call) => {
                let Some(callee) = call.callee.as_member_expression() else {
                    return;
                };
                if !is_named_property(callee, "assign") {
                    return;
                }

                let Some(root) = get_location_root_identifier(callee.object()) else {
                    return;
                };
                if !ctx.is_reference_to_global_variable(root) {
                    return;
                }

                let Some(argument) = call.arguments.first().and_then(|arg| arg.as_expression())
                else {
                    return;
                };
                if !has_relative_url(argument, ctx) {
                    return;
                }

                let expression = format!("{}()", ctx.source_range(callee.span()));
                ctx.diagnostic(no_location_assign_relative_destination_diagnostic(
                    call.span,
                    &expression,
                ));
            }
            AstKind::AssignmentExpression(assignment) => {
                // Compound assignments such as `location.href += '#section'` extend the current
                // absolute URL instead of replacing it.
                if assignment.operator != AssignmentOperator::Assign {
                    return;
                }
                let Some(target) = assignment.left.as_member_expression() else {
                    return;
                };
                if !is_named_property(target, "href") {
                    return;
                }

                let Some(root) = get_location_root_identifier(target.object()) else {
                    return;
                };
                if !ctx.is_reference_to_global_variable(root)
                    || !has_relative_url(&assignment.right, ctx)
                {
                    return;
                }

                ctx.diagnostic(no_location_assign_relative_destination_diagnostic(
                    assignment.span,
                    ctx.source_range(target.span()),
                ));
            }
            _ => {}
        }
    }
}

const GLOBAL_LOCATION_PREFIXES: [&str; 4] = ["window", "globalThis", "document", "self"];

fn is_named_property(member: &MemberExpression<'_>, name: &str) -> bool {
    member.static_property_name().is_some_and(|property| property == name)
}

fn get_location_root_identifier<'a>(
    object: &'a Expression<'a>,
) -> Option<&'a IdentifierReference<'a>> {
    let object = object.get_inner_expression();
    if let Expression::Identifier(identifier) = object
        && identifier.name == "location"
    {
        return Some(identifier);
    }

    let location = object.as_member_expression()?;
    if !is_named_property(location, "location") {
        return None;
    }

    let root = location.object().get_inner_expression().get_identifier_reference()?;
    GLOBAL_LOCATION_PREFIXES.contains(&root.name.as_str()).then_some(root)
}

/// Maximum number of expressions evaluated for one destination.
const MAX_EVALUATIONS: u16 = 256;
/// Maximum number of rounds when evaluating a variable whose value depends on itself in a loop.
const MAX_FIXPOINT_ROUNDS: usize = 8;
/// Maximum number of possible values tracked for one expression.
const MAX_ALTERNATIVES: usize = 32;
/// Strings are cut to this many bytes. Only the beginning of a URL decides whether it is relative.
const MAX_STRING_LENGTH: usize = 2048;

fn has_relative_url<'a>(expression: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    let evaluator = Evaluator {
        ctx,
        budget: Cell::new(MAX_EVALUATIONS),
        reads: RefCell::new(FxHashMap::default()),
        finished_reads: RefCell::new(Vec::new()),
        definitions: RefCell::new(FxHashMap::default()),
    };
    // Values such as `null` or `undefined` are not navigations to a page, so only strings count.
    evaluator.values(expression).is_some_and(|values| {
        !values.is_empty()
            && values.iter().all(|value| {
                matches!(value, Value::String { value, complete, followed }
                    if is_relative_url(value, *complete, *followed))
            })
    })
}

struct RuleGlobalContext<'c, 'a>(&'c LintContext<'a>);

impl<'a> GlobalContext<'a> for RuleGlobalContext<'_, 'a> {
    fn is_global_reference(&self, reference: &IdentifierReference<'a>) -> bool {
        self.0.is_reference_to_global_variable(reference)
    }
}

/// A possible runtime value of an expression.
#[derive(Debug, Clone, PartialEq)]
enum Value<'a> {
    /// A string. When `complete` is `false`, `value` is only its beginning, and `followed` tells
    /// whether known text follows the unknown part, as in `` `/${locale}/dashboard` ``.
    String {
        value: Cow<'a, str>,
        complete: bool,
        followed: bool,
    },
    Number(f64),
    Boolean(bool),
    Null,
    Undefined,
}

impl<'a> Value<'a> {
    fn string(value: Cow<'a, str>, complete: bool) -> Self {
        if value.len() <= MAX_STRING_LENGTH {
            return Self::String { value, complete, followed: false };
        }
        let end = value.floor_char_boundary(MAX_STRING_LENGTH);
        Self::String {
            value: Cow::Owned(value[..end].to_owned()),
            complete: false,
            followed: false,
        }
    }

    /// Whether this value adds known text when appended to a string.
    fn has_known_text(&self) -> bool {
        match self {
            Self::String { value, followed, .. } => !value.is_empty() || *followed,
            _ => true,
        }
    }

    /// Equality that treats `NaN` as equal to itself, so that loops over `NaN` converge.
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => {
                left.to_bits() == right.to_bits() || (left.is_nan() && right.is_nan())
            }
            _ => self == other,
        }
    }

    /// `ToString` of this value, and whether it is the whole string.
    fn to_string_prefix(&self) -> (Cow<'a, str>, bool) {
        match self {
            Self::String { value, complete, .. } => (value.clone(), *complete),
            Self::Number(number) => {
                let string = if *number == 0.0 {
                    Cow::Borrowed("0")
                } else {
                    Cow::Owned(number.to_js_string())
                };
                (string, true)
            }
            Self::Boolean(boolean) => {
                (Cow::Borrowed(if *boolean { "true" } else { "false" }), true)
            }
            Self::Null => (Cow::Borrowed("null"), true),
            Self::Undefined => (Cow::Borrowed("undefined"), true),
        }
    }

    /// `ToNumber` of a value that is not a string.
    fn to_number(&self) -> Option<f64> {
        match self {
            Self::String { .. } => None,
            Self::Number(number) => Some(*number),
            Self::Boolean(boolean) => Some(if *boolean { 1.0 } else { 0.0 }),
            Self::Null => Some(0.0),
            Self::Undefined => Some(f64::NAN),
        }
    }

    fn truthiness(&self) -> Option<bool> {
        match self {
            Self::String { value, complete, .. } => {
                if value.is_empty() {
                    complete.then_some(false)
                } else {
                    Some(true)
                }
            }
            Self::Number(number) => Some(*number != 0.0 && !number.is_nan()),
            Self::Boolean(boolean) => Some(*boolean),
            Self::Null | Self::Undefined => Some(false),
        }
    }

    fn is_nullish(&self) -> bool {
        matches!(self, Self::Null | Self::Undefined)
    }

    /// The `+` operator. `right` is `None` when the right operand is unknown.
    fn add(&self, right: Option<&Self>) -> Option<Self> {
        match (self, right) {
            // Nothing appended to an incomplete string can change its beginning.
            (Self::String { value, complete: false, followed }, right) => Some(Self::String {
                value: value.clone(),
                complete: false,
                followed: *followed || right.is_some_and(Self::has_known_text),
            }),
            (Self::String { value, .. }, None) => Some(Self::string(value.clone(), false)),
            (_, None) => None,
            (Self::String { .. }, Some(right)) | (_, Some(right @ Self::String { .. })) => {
                let (left, _) = self.to_string_prefix();
                let (right_string, complete) = right.to_string_prefix();
                let mut value = left.into_owned();
                value.push_str(&right_string);
                let mut result = Self::string(Cow::Owned(value), complete);
                if let (
                    Self::String { followed, .. },
                    Self::String { followed: right_followed, .. },
                ) = (&mut result, right)
                {
                    *followed |= *right_followed;
                }
                Some(result)
            }
            (_, Some(right)) => Some(Self::Number(self.to_number()? + right.to_number()?)),
        }
    }
}

/// Possible values of an expression.
type Values<'a> = Vec<Value<'a>>;

fn push_alternatives<'a>(values: &mut Values<'a>, more: Values<'a>) -> Option<()> {
    values.extend(more);
    (values.len() <= MAX_ALTERNATIVES).then_some(())
}

/// Replaces strings with their beginnings and drops values covered by others, so that strings
/// that grow on every loop iteration, such as `url = url + '/x'`, converge.
fn widen(values: Values<'_>) -> Values<'_> {
    let mut widened: Values<'_> = Vec::new();
    for value in values {
        let value = match value {
            Value::String { value, followed, .. } => {
                Value::String { value, complete: false, followed }
            }
            value => value,
        };
        if widened.iter().any(|kept| covers(kept, &value)) {
            continue;
        }
        widened.retain(|kept| !covers(&value, kept));
        widened.push(value);
    }
    widened
}

/// Returns `true` if every value `covered` stands for is also one that `value` stands for.
fn covers(value: &Value<'_>, covered: &Value<'_>) -> bool {
    match (value, covered) {
        (
            Value::String { value, complete: false, followed },
            Value::String {
                value: covered,
                complete: covered_complete,
                followed: covered_followed,
            },
        ) => {
            // A beginning known to be followed by more text does not stand for the beginning
            // alone.
            covered.starts_with(value.as_ref())
                && (!followed || *covered_complete || *covered_followed)
        }
        _ => value.same(covered),
    }
}

fn same_values(left: &Values<'_>, right: &Values<'_>) -> bool {
    left.len() == right.len()
        && left.iter().all(|value| right.iter().any(|other| value.same(other)))
}

enum Read<'a> {
    /// The read is being evaluated. Reads that depend on it in a cycle see `provisional`.
    InProgress {
        provisional: Values<'a>,
        used: bool,
    },
    Finished(Option<Values<'a>>),
}

struct Evaluator<'c, 'a> {
    ctx: &'c LintContext<'a>,
    budget: Cell<u16>,
    /// Values of variable reads, by the node of the read.
    reads: RefCell<FxHashMap<NodeId, Read<'a>>>,
    /// Reads in the order they finished, to discard those computed from provisional values.
    finished_reads: RefCell<Vec<NodeId>>,
    /// Definitions of reassigned variables, by symbol.
    definitions: RefCell<FxHashMap<SymbolId, Option<Rc<Definitions<'a>>>>>,
}

impl<'a> Evaluator<'_, 'a> {
    /// Returns the possible values of `expression`, or `None` if any is unknown.
    fn values(&self, expression: &Expression<'a>) -> Option<Values<'a>> {
        let remaining = self.budget.get().checked_sub(1)?;
        self.budget.set(remaining);

        let expression = expression.get_inner_expression();
        match expression {
            Expression::StringLiteral(literal) => {
                Some(vec![Value::string(Cow::Borrowed(literal.value.as_str()?), true)])
            }
            Expression::NumericLiteral(literal) => Some(vec![Value::Number(literal.value)]),
            Expression::BooleanLiteral(literal) => Some(vec![Value::Boolean(literal.value)]),
            Expression::NullLiteral(_) => Some(vec![Value::Null]),
            Expression::Identifier(identifier) => {
                if self.ctx.is_reference_to_global_variable(identifier) {
                    return match identifier.name.as_str() {
                        "undefined" => Some(vec![Value::Undefined]),
                        "NaN" => Some(vec![Value::Number(f64::NAN)]),
                        "Infinity" => Some(vec![Value::Number(f64::INFINITY)]),
                        _ => None,
                    };
                }
                self.variable_values(identifier)
            }
            Expression::TemplateLiteral(template) => self.template_values(template),
            Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
                let left = self.values(&binary.left)?;
                let right = self.values(&binary.right);
                let mut values = Vec::new();
                for left in &left {
                    match &right {
                        Some(right) => {
                            for right in right {
                                values.push(left.add(Some(right))?);
                            }
                        }
                        None => values.push(left.add(None)?),
                    }
                    if values.len() > MAX_ALTERNATIVES {
                        return None;
                    }
                }
                Some(values)
            }
            Expression::ConditionalExpression(conditional) => {
                match self.truthiness(&conditional.test) {
                    Some(true) => self.values(&conditional.consequent),
                    Some(false) => self.values(&conditional.alternate),
                    None => {
                        let mut values = self.values(&conditional.consequent)?;
                        push_alternatives(&mut values, self.values(&conditional.alternate)?)?;
                        Some(values)
                    }
                }
            }
            Expression::LogicalExpression(logical) => {
                let left = self.values(&logical.left)?;
                self.logical_values(logical.operator, left, &logical.right)
            }
            Expression::SequenceExpression(sequence) => self.values(sequence.expressions.last()?),
            Expression::AssignmentExpression(assignment)
                if assignment.operator == AssignmentOperator::Assign =>
            {
                self.values(&assignment.right)
            }
            Expression::UnaryExpression(unary) => match unary.operator {
                UnaryOperator::Void => Some(vec![Value::Undefined]),
                UnaryOperator::LogicalNot => {
                    Some(vec![Value::Boolean(!self.truthiness(&unary.argument)?)])
                }
                _ => None,
            },
            Expression::CallExpression(call) => {
                let callee = call.callee.get_identifier_reference()?;
                if callee.name != "String" || !self.ctx.is_reference_to_global_variable(callee) {
                    return None;
                }
                let Some(argument) = call.arguments.first() else {
                    return Some(vec![Value::string(Cow::Borrowed(""), true)]);
                };
                let values = self.values(argument.as_expression()?)?;
                Some(
                    values
                        .iter()
                        .map(|value| {
                            if let Value::String { .. } = value {
                                return value.clone();
                            }
                            let (string, complete) = value.to_string_prefix();
                            Value::string(string, complete)
                        })
                        .collect(),
                )
            }
            // Arrays, objects and regular expressions convert to strings in `+`.
            _ => expression
                .to_js_string(&RuleGlobalContext(self.ctx))
                .map(|string| vec![Value::string(string, true)]),
        }
    }

    fn template_values(&self, template: &TemplateLiteral<'a>) -> Option<Values<'a>> {
        let first = template.quasis.first()?.value.cooked?.as_str()?;
        let mut values = vec![Value::string(Cow::Borrowed(first), true)];
        for (substitution, quasi) in template.expressions.iter().zip(template.quasis.iter().skip(1))
        {
            let substitutions = self.values(substitution);
            let quasi = quasi.value.cooked.and_then(JSStr::as_str);
            let quasi = quasi.map(|quasi| Value::string(Cow::Borrowed(quasi), true));
            let mut next = Vec::new();
            for value in values {
                if let Value::String { complete: false, .. } = value {
                    next.push(value.add(quasi.as_ref())?);
                    continue;
                }
                let Some(substitutions) = &substitutions else {
                    next.push(value.add(None)?.add(quasi.as_ref())?);
                    continue;
                };
                for substitution in substitutions {
                    next.push(value.add(Some(substitution))?.add(quasi.as_ref())?);
                }
                if next.len() > MAX_ALTERNATIVES {
                    return None;
                }
            }
            values = next;
        }
        Some(values)
    }

    /// Values of `left || right`, `left && right` or `left ?? right`, given the values of `left`.
    fn logical_values(
        &self,
        operator: LogicalOperator,
        left: Values<'a>,
        right: &Expression<'a>,
    ) -> Option<Values<'a>> {
        let mut values = Vec::new();
        let mut right_values = None;
        for left in left {
            // `Some(true)` keeps `left`, `Some(false)` evaluates to `right`, `None` may do either.
            let keeps_left = match operator {
                LogicalOperator::Or => left.truthiness(),
                LogicalOperator::And => left.truthiness().map(|truthy| !truthy),
                LogicalOperator::Coalesce => Some(!left.is_nullish()),
            };
            if keeps_left != Some(false) {
                values.push(left);
            }
            if keeps_left != Some(true) {
                let right = right_values.get_or_insert_with(|| self.values(right)).clone()?;
                push_alternatives(&mut values, right)?;
            }
        }
        (values.len() <= MAX_ALTERNATIVES).then_some(values)
    }

    fn truthiness(&self, expression: &Expression<'a>) -> Option<bool> {
        if let Some(values) = self.values(expression) {
            let mut truthiness = values.iter().map(Value::truthiness);
            let first = truthiness.next()??;
            return truthiness.all(|truthy| truthy == Some(first)).then_some(first);
        }
        expression.to_boolean(&RuleGlobalContext(self.ctx))
    }

    /// Values a variable can hold at the read `identifier`.
    ///
    /// A read whose value depends on itself, such as `url` in `url = url + '/x'` inside a loop, is
    /// evaluated repeatedly, starting from no values, until its values stop changing.
    fn variable_values(&self, identifier: &IdentifierReference<'a>) -> Option<Values<'a>> {
        let reference = self.ctx.scoping().get_reference(identifier.reference_id());
        let read = reference.node_id();
        match self.reads.borrow_mut().get_mut(&read) {
            Some(Read::Finished(values)) => return values.clone(),
            Some(Read::InProgress { provisional, used }) => {
                *used = true;
                return Some(provisional.clone());
            }
            None => {}
        }
        let symbol_id = reference.symbol_id()?;

        self.reads
            .borrow_mut()
            .insert(read, Read::InProgress { provisional: Vec::new(), used: false });
        let first_finished = self.finished_reads.borrow().len();
        // Reads that finished while this one was in progress were computed from its provisional
        // values, so they are discarded unless those values turn out to be final.
        let discard_stale_reads = || {
            let mut reads = self.reads.borrow_mut();
            for stale in self.finished_reads.borrow_mut().drain(first_finished..) {
                reads.remove(&stale);
            }
        };
        let mut result = None;
        for round in 0..MAX_FIXPOINT_ROUNDS {
            let values = self.resolve_variable(identifier, symbol_id, read);
            let Some(Read::InProgress { provisional, used }) =
                self.reads.borrow_mut().remove(&read)
            else {
                break;
            };
            let Some(values) = values else {
                break;
            };
            if !used {
                result = Some(values);
                break;
            }
            let values = if round == 0 { values } else { widen(values) };
            if same_values(&values, &provisional) {
                result = Some(values);
                break;
            }
            discard_stale_reads();
            self.reads
                .borrow_mut()
                .insert(read, Read::InProgress { provisional: values, used: false });
        }
        if result.is_none() {
            discard_stale_reads();
        }

        self.reads.borrow_mut().insert(read, Read::Finished(result.clone()));
        self.finished_reads.borrow_mut().push(read);
        result
    }

    fn resolve_variable(
        &self,
        identifier: &IdentifierReference<'a>,
        symbol_id: SymbolId,
        read: NodeId,
    ) -> Option<Values<'a>> {
        let scoping = self.ctx.scoping();
        // A variable that is declared once and never written holds its initial value everywhere.
        if !scoping.symbol_is_mutated(symbol_id)
            && scoping.symbol_redeclarations(symbol_id).is_empty()
        {
            let declaration = scoping.symbol_declaration(symbol_id);
            return match declarator_definition(declaration, self.ctx)? {
                DefinitionValue::Expression(init) => self.values(init),
                DefinitionValue::Undefined | DefinitionValue::Keep => Some(vec![Value::Undefined]),
                DefinitionValue::Compound { .. } | DefinitionValue::Unknown => None,
            };
        }

        let definitions = self
            .definitions
            .borrow_mut()
            .entry(symbol_id)
            .or_insert_with(|| Definitions::new(symbol_id, self.ctx).map(Rc::new))
            .clone()?;
        let reaching = definitions.reaching(read, identifier.span.start, self.ctx);
        let mut values = Vec::new();
        if reaching.undefined {
            values.push(Value::Undefined);
        }
        for index in reaching.definitions {
            push_alternatives(&mut values, self.definition_values(&definitions.list[index])?)?;
        }
        (!values.is_empty()).then_some(values)
    }

    fn definition_values(&self, definition: &Definition<'a>) -> Option<Values<'a>> {
        match definition.value {
            DefinitionValue::Expression(expression) => self.values(expression),
            DefinitionValue::Undefined => Some(vec![Value::Undefined]),
            DefinitionValue::Compound { target, operator, value } => {
                let previous = self.variable_values(target)?;
                match operator {
                    AssignmentOperator::Addition => {
                        let right = self.values(value);
                        let mut values = Vec::new();
                        for left in &previous {
                            match &right {
                                Some(right) => {
                                    for right in right {
                                        values.push(left.add(Some(right))?);
                                    }
                                }
                                None => values.push(left.add(None)?),
                            }
                        }
                        (values.len() <= MAX_ALTERNATIVES).then_some(values)
                    }
                    AssignmentOperator::LogicalOr => {
                        self.logical_values(LogicalOperator::Or, previous, value)
                    }
                    AssignmentOperator::LogicalAnd => {
                        self.logical_values(LogicalOperator::And, previous, value)
                    }
                    AssignmentOperator::LogicalNullish => {
                        self.logical_values(LogicalOperator::Coalesce, previous, value)
                    }
                    _ => None,
                }
            }
            DefinitionValue::Keep | DefinitionValue::Unknown => None,
        }
    }
}

/// A place where a variable receives a value.
struct Definition<'a> {
    /// Control flow block containing the definition.
    block: BlockNodeId,
    /// Source position after which the variable holds the new value.
    end: u32,
    function: NodeId,
    value: DefinitionValue<'a>,
}

#[derive(Clone, Copy)]
enum DefinitionValue<'a> {
    Expression(&'a Expression<'a>),
    Undefined,
    /// `var x;`, which keeps the current value.
    Keep,
    /// `target operator value`, such as `url += '/path'`.
    Compound {
        target: &'a IdentifierReference<'a>,
        operator: AssignmentOperator,
        value: &'a Expression<'a>,
    },
    Unknown,
}

/// What a declaration of a variable assigns.
///
/// Returns `None` if the declaration is not a variable declarator.
fn declarator_definition<'a>(
    declaration: NodeId,
    ctx: &LintContext<'a>,
) -> Option<DefinitionValue<'a>> {
    let nodes = ctx.nodes();
    let AstKind::VariableDeclarator(declarator) = nodes.kind(declaration) else {
        return None;
    };
    let AstKind::VariableDeclaration(declarations) = nodes.parent_kind(declaration) else {
        return None;
    };
    // `declare const url: string` is given its value elsewhere.
    if declarations.declare {
        return Some(DefinitionValue::Unknown);
    }
    // `for (const url of urls)` assigns each element in turn.
    let in_loop_head = match nodes.parent_kind(nodes.parent_id(declaration)) {
        AstKind::ForInStatement(statement) => {
            statement.left.span().contains_inclusive(declarator.span)
        }
        AstKind::ForOfStatement(statement) => {
            statement.left.span().contains_inclusive(declarator.span)
        }
        _ => false,
    };
    Some(if in_loop_head || !declarator.id.is_binding_identifier() {
        DefinitionValue::Unknown
    } else if let Some(init) = &declarator.init {
        DefinitionValue::Expression(init)
    } else if declarations.kind.is_var() {
        DefinitionValue::Keep
    } else {
        DefinitionValue::Undefined
    })
}

struct Definitions<'a> {
    list: Vec<Definition<'a>>,
    /// Whether the variable is declared with `var`, so it is `undefined` before any assignment.
    hoisted: bool,
    /// Function containing the declaration.
    function: NodeId,
    /// Definitions in the function of the declaration, by control flow block.
    by_block: FxHashMap<BlockNodeId, Vec<usize>>,
    /// Definitions in other functions, which may run at any time.
    in_closures: Vec<usize>,
}

/// Definitions that can supply the value at a read.
struct Reaching {
    definitions: Vec<usize>,
    /// Whether the read can run before the variable is assigned.
    undefined: bool,
}

impl<'a> Definitions<'a> {
    /// Collects every definition of a variable, or returns `None` if it is not a variable.
    fn new(symbol_id: SymbolId, ctx: &LintContext<'a>) -> Option<Self> {
        let scoping = ctx.scoping();
        let nodes = ctx.nodes();
        let mut list = Vec::new();
        let mut hoisted = false;
        let mut function = None;

        for declaration in scoping.symbol_declarations(symbol_id) {
            let AstKind::VariableDeclarator(declarator) = nodes.kind(declaration) else {
                return None;
            };
            hoisted |= matches!(
                nodes.parent_kind(declaration),
                AstKind::VariableDeclaration(declarations) if declarations.kind.is_var()
            );
            let declaration_function = enclosing_function(declaration, ctx);
            function.get_or_insert(declaration_function);
            let value = declarator_definition(declaration, ctx)?;
            if !matches!(value, DefinitionValue::Keep) {
                list.push(Definition {
                    block: nodes.cfg_id(declaration),
                    end: declarator.span.end,
                    function: declaration_function,
                    value,
                });
            }
        }

        for reference in ctx.symbol_references(symbol_id) {
            if !reference.is_write() {
                continue;
            }
            let write = reference.node_id();
            let (end, value) = match nodes.parent_kind(write) {
                AstKind::AssignmentExpression(assignment) => {
                    let AssignmentTarget::AssignmentTargetIdentifier(target) = &assignment.left
                    else {
                        return None;
                    };
                    let value = match assignment.operator {
                        AssignmentOperator::Assign => {
                            DefinitionValue::Expression(&assignment.right)
                        }
                        operator @ (AssignmentOperator::Addition
                        | AssignmentOperator::LogicalOr
                        | AssignmentOperator::LogicalAnd
                        | AssignmentOperator::LogicalNullish) => {
                            DefinitionValue::Compound { target, operator, value: &assignment.right }
                        }
                        _ => DefinitionValue::Unknown,
                    };
                    (assignment.span.end, value)
                }
                // `for (url of urls)` assigns each element before the body runs.
                AstKind::ForOfStatement(statement) => {
                    (statement.left.span().end, DefinitionValue::Unknown)
                }
                AstKind::ForInStatement(statement) => {
                    (statement.left.span().end, DefinitionValue::Unknown)
                }
                // Updates such as `url++`, and destructuring.
                parent => (parent.span().end, DefinitionValue::Unknown),
            };
            list.push(Definition {
                block: nodes.cfg_id(write),
                end,
                function: enclosing_function(write, ctx),
                value,
            });
        }

        let function = function?;
        let mut by_block: FxHashMap<BlockNodeId, Vec<usize>> = FxHashMap::default();
        let mut in_closures = Vec::new();
        for (index, definition) in list.iter().enumerate() {
            if definition.function == function {
                by_block.entry(definition.block).or_default().push(index);
            } else {
                in_closures.push(index);
            }
        }

        Some(Self { list, hoisted, function, by_block, in_closures })
    }

    /// Finds the definitions whose value the variable can hold at `read`, which starts at
    /// `read_start`.
    ///
    /// Definitions in the function of the read are followed backwards through the control flow
    /// graph. Definitions in other functions may run at any time, so they can always reach.
    fn reaching(&self, read: NodeId, read_start: u32, ctx: &LintContext<'a>) -> Reaching {
        let function = enclosing_function(read, ctx);
        let mut reaching = vec![false; self.list.len()];
        let mut undefined = false;

        if function != self.function {
            // A read in a closure may run after any definition, or before all of them.
            return Reaching {
                definitions: (0..self.list.len()).collect(),
                undefined: self.hoisted,
            };
        }

        for &index in &self.in_closures {
            reaching[index] = true;
        }

        let cfg = ctx.cfg();
        // Blocks to visit, with whether the path crossed a loop back edge, and whether the block
        // can be left at any point because it throws.
        let mut stack = vec![(ctx.nodes().cfg_id(read), false, false)];
        let mut visited = FxHashSet::default();
        while let Some((block, crossed_back_edge, throws)) = stack.pop() {
            if !visited.insert((block, crossed_back_edge, throws)) {
                continue;
            }

            // Without a back edge, a definition that ends after the read has not run yet.
            let candidates = self
                .by_block
                .get(&block)
                .into_iter()
                .flatten()
                .copied()
                .filter(|&index| crossed_back_edge || self.list[index].end <= read_start);
            if throws {
                for index in candidates {
                    reaching[index] = true;
                }
            } else if let Some(last) = candidates.max_by_key(|&index| self.list[index].end) {
                reaching[last] = true;
                continue;
            }

            let mut has_predecessor = false;
            for edge in cfg.graph().edges_directed(block, Direction::Incoming) {
                let edge_type = edge.weight();
                if matches!(edge_type, EdgeType::NewFunction | EdgeType::Unreachable)
                    || cfg.basic_block(edge.source()).is_unreachable()
                {
                    continue;
                }
                has_predecessor = true;
                stack.push((
                    edge.source(),
                    crossed_back_edge || matches!(edge_type, EdgeType::Backedge),
                    // Both edges leave `try` and `catch` blocks at any point: an error edge to
                    // the `catch` block, and a finalize edge to the `finally` block.
                    matches!(edge_type, EdgeType::Error(_) | EdgeType::Finalize),
                ));
            }
            // At the start of the function, only a `var` has a value.
            if !has_predecessor && self.hoisted {
                undefined = true;
            }
        }

        Reaching {
            definitions: reaching
                .iter()
                .enumerate()
                .filter_map(|(index, &reaches)| reaches.then_some(index))
                .collect(),
            undefined,
        }
    }
}

fn enclosing_function(node: NodeId, ctx: &LintContext<'_>) -> NodeId {
    ctx.nodes()
        .ancestors(node)
        .find(|ancestor| {
            matches!(
                ancestor.kind(),
                AstKind::Function(_)
                    | AstKind::ArrowFunctionExpression(_)
                    | AstKind::StaticBlock(_)
                    | AstKind::PropertyDefinition(_)
                    | AstKind::Program(_)
            )
        })
        .map_or(node, AstNode::id)
}

/// Returns `true` if `value` is definitely a relative URL.
///
/// When `complete` is `false`, `value` is only the beginning of the URL, and the result is `true`
/// only if no continuation could make it absolute.
///
/// The one exception is a beginning of just `/` followed by known text, as in
/// `` `/${locale}/dashboard` ``. It becomes protocol-relative only if the unknown part starts with
/// `/`, which is not how paths are built in practice.
fn is_relative_url(value: &str, complete: bool, followed: bool) -> bool {
    // The URL parser ignores leading C0 control characters and spaces, and ASCII tabs and
    // newlines anywhere in the input.
    let mut chars = value
        .trim_start_matches(|character: char| character <= ' ')
        .chars()
        .filter(|character| !matches!(character, '\t' | '\n' | '\r'));

    match chars.next() {
        // Browsers treat `\` like `/` in http(s) URLs, so `//host` and `\\host` are
        // protocol-relative.
        Some('/' | '\\') => match chars.next() {
            Some('/' | '\\') => false,
            Some(_) => true,
            None => complete || followed,
        },
        Some(first) if first.is_ascii_alphabetic() => {
            for character in chars {
                if character == ':' {
                    return false;
                }
                if !(character.is_ascii_alphanumeric() || matches!(character, '+' | '.' | '-')) {
                    return true;
                }
            }
            // A scheme-like beginning such as `http` may still be followed by `://`.
            complete
        }
        Some(_) => true,
        None => complete,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // no-location-assign-relative-destination
        "location.href = 'https://example.com'",
        "location.href = 'https://example.com/path?q=1'",
        "window.location.href = 'https://example.com'",
        "globalThis.location.href = 'https://example.com'",
        "location.assign('https://example.com')",
        "window.location.assign('https://example.com')",
        "globalThis.location.assign('https://example.com')",
        "location.href = '//example.com/path'",
        "location.assign('//cdn.example.com/file.js')",
        r"location.href = '\\\\example.com/path'",
        "location.href = ' https://example.com'",
        "location.href = 'ftp://files.example.com'",
        "location.href = 'mailto:user@example.com'",
        "location.href = 'CUSTOM+SCHEME:value'",
        "location.href = 'https' + '://example.com/path'",
        "location.href = `https://example.com/${path}`",
        "location.href = `${'https://example.com'}`",
        "location.href = true ? 'https://example.com' : '/dashboard'",
        "location.href = String('https://example.com')",
        "location.assign(`https://example.com/${path}`)",
        "location.href = someVariable",
        "location.assign(someVariable)",
        "window.location.href = computedUrl()",
        "window.location.assign(computedUrl())",
        "location.assign()",
        "location.assign(...urls)",
        // The destination starts with an unknown value, which may be an absolute URL.
        "location.href = `${baseUrl}/dashboard`",
        "location.assign(`${process.env.NEXT_PUBLIC_SITE_URL}/login`)",
        "location.href = '' + externalUrl",
        "location.href = 'http' + (secure ? 's' : '') + '://example.com'",
        // Compound assignments extend the current absolute URL.
        "location.href += '#section'",
        "location.href += `?tab=${tab}`",
        "
                  const url = 'https://example.com';
                  location.href = url;
                  location.assign(url);
                ",
        "
                  const url = 'https://example.com/' + someVariable;
                  location.href = url;
                  location.assign(url);
                ",
        "
                  const url = `https://example.com/${someVariable}`;
                  location.href = url;
                  location.assign(url);
                ",
        "foo.location.href = '/path'",
        "foo.location.assign('/path')",
        "
                  let url = '/dashboard';
                  url = 'https://example.com';
                  location.href = url;
                ",
        "
                  let url = 'https://example.com';
                  url = url + '/path';
                  location.href = url;
                ",
        // The variable may hold an absolute URL on some path.
        "
                  let url;
                  if (external) {
                    url = 'https://example.com';
                  } else {
                    url = '/dashboard';
                  }
                  location.href = url;
                ",
        "location.href = external ? 'https://example.com' : '/dashboard'",
        "
                  let url = 'https://example.com';
                  try {
                    url = '/dashboard';
                    risky();
                    url = 'https://other.example.com';
                  } catch {
                    location.href = url;
                  }
                ",
        "
                  let url = '/home';
                  try {
                    url = 'https://example.com';
                    risky();
                    url = '/dashboard';
                  } finally {
                    location.href = url;
                  }
                ",
        "for (const url of urls) location.href = url",
        "
                  let url = '/fallback';
                  for (url of candidates) {
                    location.assign(url);
                    break;
                  }
                ",
        "
                  let url = '/fallback';
                  for (url in routes) {
                    location.assign(url);
                    break;
                  }
                ",
        // Declared variables get their value elsewhere, such as at build time.
        "declare const APP_URL: string; location.href = APP_URL",
        // `next` may start with `/`, which makes the URL protocol-relative.
        "location.href = '/' + next",
        "location.href = `/${next}`",
        // Values that are not strings are not navigations to pages.
        "location.assign(undefined)",
        "location.href = null",
        "location.href = 5",
        "location.href = void 0",
        "
                  let url = '/dashboard';
                  if (external) url = 'https://example.com';
                  location.href = url;
                ",
        "
                  let url = '/dashboard';
                  if (external) {
                    url = 'https://example.com';
                  }
                  location.href = url;
                ",
        "
                  let url = 'https://example.com';
                  url += '/path';
                  location.href = url;
                ",
        "
                  let url = 'https://example.com';
                  const reset = () => { url = '/dashboard' };
                  location.href = url;
                ",
        "
                  let url = '/dashboard';
                  function go() { location.href = url }
                  url = 'https://example.com';
                  go();
                ",
        "
                  let url = 'https://example.com';
                  for (const path of paths) {
                    location.href = url;
                    url = '/' + path;
                  }
                ",
        "
                  var url = '/dashboard';
                  var url = 'https://example.com';
                  location.href = url;
                ",
        // Destructured variables are not resolved.
        "const { url } = { url: 'https://example.com' }; location.href = url",
        "const [, url] = ['/dashboard', 'https://example.com']; location.href = url",
        "
                  const location = { href: '' };
                  location.href = '/foo'
                ",
        "function handler(location) { location.href = '/foo'; location.assign('/foo') }",
        "
                  const window = { location: { href: '' } };
                  window.location.href = '/foo'
                ",
        "
                  function handler(globalThis) {
                    globalThis.location.assign('/foo')
                  }
                ",
        "function handler(document) { document.location.href = '/foo' }",
        "function handler(self) { self.location.assign('/foo') }",
        "function handler(String) { location.href = String('/foo') }",
        "const protocol = 'https'; location.href = `${protocol}://example.com`",
        "
                  import { location } from './my-module';
                  location.href = '/foo'
                ",
    ];

    let fail = vec![
        // no-location-assign-relative-destination
        "location.href = '/foo'",
        "location['href'] = '/foo'",
        "location[`href`] = '/foo'",
        "location.href = `/users/${id}`",
        "location.href = `/${locale}/dashboard`",
        "location.href = '/' + locale + '/dashboard'",
        "location.href = ' /dashboard'",
        "window.location.href = '/foo'",
        "window.location.href = '/dashboard'",
        "window.location['href'] = '/foo'",
        "window.location['href'] = '/dashboard'",
        "globalThis.location.href = '/foo'",
        "globalThis.location.href = '/dashboard'",
        "document.location.href = '/foo'",
        "self.location.href = '/foo'",
        "window['location'].href = '/foo'",
        "location.assign('/foo')",
        "location.assign('/dashboard')",
        "location['assign']('/foo')",
        "location[`assign`]('/foo')",
        "location['assign']('/dashboard')",
        "location.assign(`/users/${id}/profile`)",
        "window.location.assign('/foo')",
        "window.location.assign('/dashboard')",
        "window.location['assign']('/foo')",
        "globalThis.location.assign('/foo')",
        "globalThis.location.assign('/dashboard')",
        "document.location.assign('/foo')",
        "self.location.assign('/foo')",
        "globalThis['location']['assign']('/foo')",
        "location.href = './page'",
        "location.href = '../page'",
        "location.assign('?tab=settings')",
        "location.assign('#section')",
        "location.href = true ? '/dashboard' : 'https://example.com'",
        "location.href = (0, '/dashboard')",
        "location.href = String('/dashboard')",
        "location.href = String()",
        "location.href = (true + false) + ':'",
        "const useInternal = true; const url = useInternal ? '/dashboard' : 'https://example.com'; location.href = url",
        "
                      const url = '/dashboard';
                      location.href = url;",
        "
                      const url = '/dashboard/' + someVariable;
                      location.href = url;",
        "
                      const url = `/dashboard/${someVariable}`;
                      location.href = url;",
        "
                      const base = '/dashboard';
                      location.href = `${base}/${id}`;",
        "
                      let url = 'https://example.com';
                      url = '/other-path';
                      location.href = url;",
        "
                      let url = '/dashboard';
                      location.href = url;
                      url = 'https://example.com';",
        "
                      let url = '/base';
                      url = url + '/path';
                      location.href = url;",
        "
                      let url;
                      if (external) url = 'https://example.com';
                      url = '/dashboard';
                      location.href = url;",
        "
                      function handleClick(id) {
                        let url = 'https://example.com';
                        if (id) {
                          url = `/users/${id}`;
                          location.assign(url);
                        }
                      }",
        // Every path assigns a relative URL.
        "location.href = admin ? '/admin' : '/dashboard'",
        "location.href = '/dashboard' + (tab ? `?tab=${tab}` : '')",
        "
                      let url;
                      if (admin) {
                        url = '/admin';
                      } else {
                        url = '/dashboard';
                      }
                      location.href = url;",
        "
                      let url = '/home';
                      switch (role) {
                        case 'admin':
                          url = '/admin';
                          break;
                        case 'user':
                          url = '/dashboard';
                          break;
                      }
                      location.href = url;",
        "
                      let url = '/home';
                      try {
                        url = '/dashboard';
                        risky();
                      } catch {
                        location.href = url;
                      }",
        "
                      let url = '/home';
                      try {
                        url = '/dashboard';
                        risky();
                        url = '/settings';
                      } finally {
                        location.href = url;
                      }",
        "
                      let url = '/home';
                      for (const id of ids) {
                        location.href = url;
                        url = '/dashboard';
                      }",
        "
                      let url = '/home';
                      const go = () => { location.href = url };
                      url = '/dashboard';",
        "
                      let url = '/users';
                      url += `/${id}`;
                      location.href = url;",
        "
                      let url = '/dashboard';
                      url ||= 'https://example.com';
                      location.href = url;",
        "
                      function handleClick() {
                        window.location.href = '/dashboard'
                      }",
        "
                      function handleClick() {
                        location.assign('/dashboard')
                      }",
    ];

    Tester::new(
        NoLocationAssignRelativeDestination::NAME,
        NoLocationAssignRelativeDestination::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
