use oxc_ast::{
    AstKind,
    ast::{
        AssignmentOperator, AssignmentTarget, BindingPattern, Expression, MemberExpression,
        PropertyKey, TSSignature, TSType, VariableDeclarationKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeId;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::{Rule, TupleRuleConfig},
};

fn prefer_object_destructuring(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use Object destructuring.")
        .with_help("Use object destructuring rather than direct member access.")
        .with_label(span)
}

fn prefer_array_destructuring(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use Array destructuring.")
        .with_help("Use array destructuring rather than direct member access.")
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct PreferDestructuringTargetConfig {
    array: bool,
    object: bool,
}

impl Default for PreferDestructuringTargetConfig {
    fn default() -> Self {
        Self { array: true, object: true }
    }
}

impl PreferDestructuringTargetConfig {
    fn disabled() -> Self {
        Self { array: false, object: false }
    }
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PreferDestructuringTargetOption {
    array: Option<bool>,
    object: Option<bool>,
}

impl PreferDestructuringTargetOption {
    fn enabled_by_default() -> Self {
        Self { array: Some(true), object: Some(true) }
    }

    fn into_config(self) -> PreferDestructuringTargetConfig {
        PreferDestructuringTargetConfig {
            array: self.array.unwrap_or(false),
            object: self.object.unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
struct PreferDestructuringAssignmentConfig {
    variable_declarator: Option<PreferDestructuringTargetOption>,
    assignment_expression: Option<PreferDestructuringTargetOption>,
}

impl PreferDestructuringAssignmentConfig {
    fn into_configs(self) -> (PreferDestructuringTargetConfig, PreferDestructuringTargetConfig) {
        let variable_declarator = self.variable_declarator.map_or_else(
            PreferDestructuringTargetConfig::disabled,
            PreferDestructuringTargetOption::into_config,
        );
        let assignment_expression = self.assignment_expression.map_or_else(
            PreferDestructuringTargetConfig::disabled,
            PreferDestructuringTargetOption::into_config,
        );

        (variable_declarator, assignment_expression)
    }
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(untagged)]
enum PreferDestructuringOption {
    Target(PreferDestructuringTargetOption),
    Assignment(PreferDestructuringAssignmentConfig),
}

impl Default for PreferDestructuringOption {
    fn default() -> Self {
        Self::Target(PreferDestructuringTargetOption::enabled_by_default())
    }
}

impl PreferDestructuringOption {
    fn into_configs(self) -> (PreferDestructuringTargetConfig, PreferDestructuringTargetConfig) {
        match self {
            Self::Target(config) => {
                let config = config.into_config();
                (config.clone(), config)
            }
            Self::Assignment(config) => config.into_configs(),
        }
    }
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct PreferDestructuringEnforcementConfig {
    /// Whether to enforce destructuring on variable declarations with type annotations.
    enforce_for_declaration_with_type_annotation: bool,
    /// Whether to enforce destructuring that uses a different variable name than the property name.
    enforce_for_renamed_properties: bool,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(default)]
struct PreferDestructuringConfig(PreferDestructuringOption, PreferDestructuringEnforcementConfig);

impl PreferDestructuringConfig {
    fn into_rule(self) -> PreferDestructuring {
        let (variable_declarator, assignment_expression) = self.0.into_configs();

        PreferDestructuring {
            variable_declarator,
            assignment_expression,
            enforce_for_renamed_properties: self.1.enforce_for_renamed_properties,
            enforce_for_declaration_with_type_annotation: self
                .1
                .enforce_for_declaration_with_type_annotation,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PreferDestructuring {
    variable_declarator: PreferDestructuringTargetConfig,
    assignment_expression: PreferDestructuringTargetConfig,
    enforce_for_renamed_properties: bool,
    enforce_for_declaration_with_type_annotation: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires destructuring from arrays and/or objects. This is the
    /// TypeScript-aware version of the [`prefer-destructuring`](https://eslint.org/docs/latest/rules/prefer-destructuring)
    /// ESLint rule.
    ///
    /// ### Why is this bad?
    ///
    /// The plain ESLint rule treats any numeric member access such as `x[0]`
    /// as array-like, and reports it as an opportunity for array destructuring.
    /// That is incorrect when the object being indexed is not actually an array
    /// or iterable — for example `let x: { 0: unknown }; let y = x[0];`, where
    /// `x[0]` is really an object property access. This rule uses the available
    /// type annotations to distinguish array/iterable access from object
    /// property access, and only reports the appropriate destructuring form.
    ///
    /// It also adds the `enforceForDeclarationWithTypeAnnotation` option, which
    /// controls whether declarations that carry an explicit type annotation are
    /// reported (they are skipped by default, since destructuring them requires
    /// rewriting the annotation).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// let x: { [Symbol.iterator]: unknown };
    /// let y = x[0];
    ///
    /// let obj = { foo: 'bar' };
    /// const foo = obj.foo;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const object: { foo: string };
    /// var foo: string = object.foo;
    ///
    /// let x: { 0: unknown };
    /// let y = x[0];
    /// ```
    PreferDestructuring,
    typescript,
    style,
    conditional_fix,
    config = PreferDestructuringConfig,
    version = "next",
    short_description = "Require destructuring from arrays and/or objects.",
);

impl Rule for PreferDestructuring {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<TupleRuleConfig<PreferDestructuringConfig>>(value)
            .map(|config| config.into_inner().into_rule())
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::AssignmentExpression(assign_expr)
                if assign_expr.operator == AssignmentOperator::Assign =>
            {
                let AssignmentTarget::AssignmentTargetIdentifier(target) = &assign_expr.left else {
                    return;
                };
                let Some(member) = assign_expr.right.without_parentheses().as_member_expression()
                else {
                    return;
                };
                self.check(
                    member,
                    Some(target.name.as_str()),
                    assign_expr.span,
                    None,
                    &self.assignment_expression,
                    node.scope_id(),
                    ctx,
                );
            }
            AstKind::VariableDeclarator(declarator) => {
                // Skip `using` and `await using` declarations - destructuring doesn't apply to them.
                if matches!(
                    declarator.kind,
                    VariableDeclarationKind::Using | VariableDeclarationKind::AwaitUsing
                ) {
                    return;
                }
                let Some(init) = &declarator.init else {
                    return;
                };
                let Some(member) = init.without_parentheses().as_member_expression() else {
                    return;
                };
                let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
                    // Already destructured (array/object pattern) - nothing to report.
                    return;
                };

                // `enforceForDeclarationWithTypeAnnotation`: declarations with an
                // explicit type annotation are skipped unless the option is enabled.
                // When enabled, we still report but suppress the autofix, because
                // destructuring the binding would require rewriting the annotation.
                let has_type_annotation = declarator.type_annotation.is_some();
                if has_type_annotation && !self.enforce_for_declaration_with_type_annotation {
                    return;
                }

                self.check(
                    member,
                    Some(ident.name.as_str()),
                    init.span(),
                    if has_type_annotation { None } else { Some(declarator.span()) },
                    &self.variable_declarator,
                    node.scope_id(),
                    ctx,
                );
            }
            _ => {}
        }
    }
}

impl PreferDestructuring {
    /// `target_name` is the name of the binding/assignment target (e.g. `foo` in
    /// `var foo = obj.foo`), used to decide whether a property access is
    /// "renamed". `fix_span`, when `Some`, is the span to replace when applying
    /// an object-destructuring autofix (only available for plain declarations
    /// without a type annotation).
    #[expect(clippy::too_many_arguments)]
    fn check<'a>(
        &self,
        member: &MemberExpression<'a>,
        target_name: Option<&str>,
        report_span: Span,
        fix_span: Option<Span>,
        config: &PreferDestructuringTargetConfig,
        scope_id: ScopeId,
        ctx: &LintContext<'a>,
    ) {
        // Access on `super` or a private field cannot be destructured.
        if matches!(member, MemberExpression::PrivateFieldExpression(_))
            || matches!(member.object(), Expression::Super(_))
        {
            return;
        }
        // Optional chaining member access is not reported.
        if member.optional() {
            return;
        }

        match member {
            MemberExpression::ComputedMemberExpression(computed) => {
                // Template-literal keys are never reported.
                if matches!(computed.expression, Expression::TemplateLiteral(_)) {
                    return;
                }

                if let Expression::NumericLiteral(_) = &computed.expression {
                    // `x[0]`: use the type of `x` to decide array vs object.
                    if is_iterable_or_any(member.object(), scope_id, ctx) {
                        // Iterable/any -> array destructuring.
                        if config.array {
                            ctx.diagnostic(prefer_array_destructuring(report_span));
                        }
                    } else {
                        // Object-like -> only report as object destructuring when
                        // renamed properties are enforced and object is enabled.
                        if self.enforce_for_renamed_properties && config.object {
                            ctx.diagnostic(prefer_object_destructuring(report_span));
                        }
                    }
                    return;
                }

                // Non-numeric, non-template computed access (`x[i]`, `obj['foo']`, `obj[key]`).
                if !config.object {
                    return;
                }
                if let Expression::StringLiteral(string_literal) = &computed.expression {
                    if self.enforce_for_renamed_properties
                        || target_name.is_some_and(|name| name == string_literal.value.as_str())
                    {
                        ctx.diagnostic(prefer_object_destructuring(report_span));
                    }
                } else if self.enforce_for_renamed_properties {
                    ctx.diagnostic(prefer_object_destructuring(report_span));
                }
            }
            MemberExpression::StaticMemberExpression(static_expr) => {
                if !config.object {
                    return;
                }
                let names_match =
                    target_name.is_some_and(|name| name == static_expr.property.name.as_str());
                if !self.enforce_for_renamed_properties && !names_match {
                    return;
                }

                if names_match && let Some(fix_span) = fix_span {
                    ctx.diagnostic_with_fix(prefer_object_destructuring(report_span), |fixer| {
                        let prop = fixer.source_range(static_expr.property.span);
                        let object_text = fixer.source_range(
                            get_object_span_without_redundant_parentheses(&static_expr.object),
                        );
                        fixer.replace(fix_span, format!("{{{prop}}} = {object_text}"))
                    });
                } else {
                    ctx.diagnostic(prefer_object_destructuring(report_span));
                }
            }
            MemberExpression::PrivateFieldExpression(_) => unreachable!(),
        }
    }
}

/// Syntactically approximate TypeScript's `isTypeAnyOrIterableType`: resolve
/// the object identifier to its declaration and classify the declared type.
/// A call expression whose callee is a generator is treated as iterable.
/// Anything whose type cannot be resolved falls back to `true` (array-like),
/// matching the base ESLint rule's behavior of treating `x[0]` as array access.
fn is_iterable_or_any<'a>(
    object: &Expression<'a>,
    scope_id: ScopeId,
    ctx: &LintContext<'a>,
) -> bool {
    match object.without_parentheses() {
        Expression::Identifier(ident) => {
            let Some(symbol_id) = ctx.scoping().get_binding(scope_id, ident.name) else {
                // Unresolved (e.g. `declare`d elsewhere / global) -> treat as array-like.
                return true;
            };
            let decl = ctx.semantic().symbol_declaration(symbol_id);
            let Some(ts_type) = declared_type_of(decl) else {
                // No annotation available -> treat as array-like.
                return true;
            };
            is_iterable_type(ts_type)
        }
        Expression::CallExpression(call) => {
            // `it()[0]` where `it` is a generator is iterable.
            if let Expression::Identifier(ident) = &call.callee
                && let Some(symbol_id) = ctx.scoping().get_binding(scope_id, ident.name)
            {
                let decl = ctx.semantic().symbol_declaration(symbol_id);
                if let AstKind::Function(func) = decl.kind() {
                    return func.generator;
                }
            }
            true
        }
        // Any other object expression: fall back to array-like handling.
        _ => true,
    }
}

/// Extract the declared `TSType` of a binding declaration node, if present.
fn declared_type_of<'a>(node: &AstNode<'a>) -> Option<&'a TSType<'a>> {
    match node.kind() {
        AstKind::VariableDeclarator(declarator) => {
            declarator.type_annotation.as_ref().map(|annotation| &annotation.type_annotation)
        }
        _ => None,
    }
}

/// Syntactic approximation of "is this type any or iterable?".
fn is_iterable_type(ts_type: &TSType) -> bool {
    match ts_type {
        // `any` is treated as iterable by the reference rule; `T[]` and tuples are iterable too.
        TSType::TSAnyKeyword(_) | TSType::TSArrayType(_) | TSType::TSTupleType(_) => true,
        // A union is iterable only if *every* member is iterable.
        TSType::TSUnionType(union) => union.types.iter().all(is_iterable_type),
        // An intersection is iterable if *any* member is iterable.
        TSType::TSIntersectionType(intersection) => intersection.types.iter().any(is_iterable_type),
        // `{ [Symbol.iterator]: ... }` is iterable; other object literals are not.
        TSType::TSTypeLiteral(literal) => literal.members.iter().any(|member| {
            matches!(member, TSSignature::TSPropertySignature(prop)
                if prop.computed && is_symbol_iterator_key(&prop.key))
        }),
        // `unknown`, `object`, `Record<...>`, and everything else are object-like.
        _ => false,
    }
}

/// Whether a computed property key is `[Symbol.iterator]`.
fn is_symbol_iterator_key(key: &PropertyKey) -> bool {
    if let PropertyKey::StaticMemberExpression(member) = key
        && let Expression::Identifier(object) = &member.object
    {
        return object.name == "Symbol" && member.property.name == "iterator";
    }
    false
}

/// Returns the span of the object expression, stripping redundant parentheses for expressions
/// where they are unnecessary in the `{ ... } = <object>` destructuring context.
///
/// A `SequenceExpression` (`(a, b)`) must keep its parentheses, otherwise
/// `var {foo} = a, b` would parse as two declarators. Other parenthesized
/// expressions such as arrow functions (`(() => null)`) and assignments
/// (`(a = obj)`) do not need the parentheses in this position.
fn get_object_span_without_redundant_parentheses(object: &Expression) -> Span {
    match object.without_parentheses() {
        Expression::SequenceExpression(_) => object.span(),
        inner => inner.span(),
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                  declare const object: { foo: string };
                  var foo: string = object.foo;
                ",
            None,
        ),
        (
            "
                  declare const array: number[];
                  const bar: number = array[0];
                ",
            None,
        ),
        (
            "
                    declare const object: { foo: string };
                    var { foo } = object;
                  ",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    declare const object: { foo: string };
                    var { foo }: { foo: number } = object;
                  ",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    declare const array: number[];
                    var [foo] = array;
                  ",
            Some(
                serde_json::json!([ { "array": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    declare const array: number[];
                    var [foo]: [foo: number] = array;
                  ",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    declare const object: { bar: string };
                    var foo: unknown = object.bar;
                  ",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    declare const object: { foo: string };
                    var { foo: bar } = object;
                  ",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    declare const object: { foo: boolean };
                    var { foo: bar }: { foo: boolean } = object;
                  ",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    declare class Foo {
                      foo: string;
                    }

                    class Bar extends Foo {
                      static foo() {
                        var foo: any = super.foo;
                      }
                    }
                  ",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                  let x: { 0: unknown };
                  let y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: { 0: unknown };
                  y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: unknown;
                  let y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: unknown;
                  y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: { 0: unknown } | unknown[];
                  let y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: { 0: unknown } | unknown[];
                  y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: { 0: unknown } & (() => void);
                  let y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: { 0: unknown } & (() => void);
                  y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: Record<number, unknown>;
                  let y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: Record<number, unknown>;
                  y = x[0];
                ",
            None,
        ),
        (
            "
                    let x: { 0: unknown };
                    let { 0: y } = x;
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: { 0: unknown };
                    ({ 0: y } = x);
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: { 0: unknown };
                    let y = x[0];
                  ",
            Some(serde_json::json!([{ "array": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let x: { 0: unknown };
                    y = x[0];
                  ",
            Some(serde_json::json!([{ "array": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let x: { 0: unknown };
                    let y = x[0];
                  ",
            Some(
                serde_json::json!([ { "AssignmentExpression": { "array": true, "object": true }, "VariableDeclarator": { "array": true, "object": false }, }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: { 0: unknown };
                    y = x[0];
                  ",
            Some(
                serde_json::json!([ { "AssignmentExpression": { "array": true, "object": false }, "VariableDeclarator": { "array": true, "object": true }, }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: Record<number, unknown>;
                    let i: number = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": false }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: Record<number, unknown>;
                    let i: 0 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": false }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: Record<number, unknown>;
                    let i: 0 | 1 | 2 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": false }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: unknown[];
                    let i: number = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": false }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: unknown[];
                    let i: 0 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": false }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: unknown[];
                    let i: 0 | 1 | 2 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": false }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: unknown[];
                    let i: number = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": false }, ]),
            ),
        ),
        (
            "
                    let x: { 0: unknown };
                    y += x[0];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    class Bar {
                      public [0]: unknown;
                    }
                    class Foo extends Bar {
                      static foo() {
                        let y = super[0];
                      }
                    }
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    class Bar {
                      public [0]: unknown;
                    }
                    class Foo extends Bar {
                      static foo() {
                        y = super[0];
                      }
                    }
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                  let xs: unknown[] = [1];
                  let [x] = xs;
                ",
            None,
        ),
        (
            "
                  const obj: { x: unknown } = { x: 1 };
                  const { x } = obj;
                ",
            None,
        ),
        (
            "
                  var obj: { x: unknown } = { x: 1 };
                  var { x: y } = obj;
                ",
            None,
        ),
        (
            "
                  let obj: { x: unknown } = { x: 1 };
                  let key: 'x' = 'x';
                  let { [key]: foo } = obj;
                ",
            None,
        ),
        (
            "
                  const obj: { x: unknown } = { x: 1 };
                  let x: unknown;
                  ({ x } = obj);
                ",
            None,
        ),
        (
            "
                  let obj: { x: unknown } = { x: 1 };
                  let y = obj.x;
                ",
            None,
        ),
        (
            "
                  var obj: { x: unknown } = { x: 1 };
                  var y: unknown;
                  y = obj.x;
                ",
            None,
        ),
        (
            "
                  const obj: { x: unknown } = { x: 1 };
                  const y = obj['x'];
                ",
            None,
        ),
        (
            "
                  let obj: Record<string, unknown> = {};
                  let key = 'abc';
                  var y = obj[key];
                ",
            None,
        ),
        (
            "
                  let obj: { x: number } = { x: 1 };
                  let x = 10;
                  x += obj.x;
                ",
            None,
        ),
        (
            "
                  let obj: { x: boolean } = { x: false };
                  let x = true;
                  x ||= obj.x;
                ",
            None,
        ),
        (
            "
                  const xs: number[] = [1];
                  let x = 3;
                  x *= xs[0];
                ",
            None,
        ),
        (
            "
                  let xs: unknown[] | undefined;
                  let x = xs?.[0];
                ",
            None,
        ),
        (
            "
                  let obj: Record<string, unknown> | undefined;
                  let x = obj?.x;
                ",
            None,
        ),
        (
            "
                  class C {
                    #foo: string;

                    method() {
                      const foo: unknown = this.#foo;
                    }
                  }
                ",
            None,
        ),
        (
            "
                  class C {
                    #foo: string;

                    method() {
                      let foo: unknown;
                      foo = this.#foo;
                    }
                  }
                ",
            None,
        ),
        (
            "
                    class C {
                      #foo: string;

                      method() {
                        const bar: unknown = this.#foo;
                      }
                    }
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    class C {
                      #foo: string;

                      method(another: C) {
                        let bar: unknown;
                        bar: unknown = another.#foo;
                      }
                    }
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    class C {
                      #foo: string;

                      method() {
                        const foo: unknown = this.#foo;
                      }
                    }
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
    ];

    let fail = vec![
        (
            "var foo: string = object.foo;",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "var foo: string = array[0];",
            Some(
                serde_json::json!([ { "array": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "var foo: unknown = object.bar;",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true, "enforceForRenamedProperties": true, }, ]),
            ),
        ),
        (
            "
                    let x: { [Symbol.iterator]: unknown };
                    let y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: { [Symbol.iterator]: unknown };
                    y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: [1, 2, 3];
                    let y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: [1, 2, 3];
                    y = x[0];
                  ",
            None,
        ),
        (
            "
                    function* it() {
                      yield 1;
                    }
                    let y = it()[0];
                  ",
            None,
        ),
        (
            "
                    function* it() {
                      yield 1;
                    }
                    y = it()[0];
                  ",
            None,
        ),
        (
            "
                    let x: any;
                    let y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: any;
                    y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: string[] | { [Symbol.iterator]: unknown };
                    let y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: string[] | { [Symbol.iterator]: unknown };
                    y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: object & unknown[];
                    let y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: object & unknown[];
                    y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: { 0: string };
                    let y = x[0];
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let x: { 0: string };
                    y = x[0];
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let x: { 0: string };
                    let y = x[0];
                  ",
            Some(
                serde_json::json!([ { "AssignmentExpression": { "array": false, "object": false }, "VariableDeclarator": { "array": false, "object": true }, }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: { 0: string };
                    y = x[0];
                  ",
            Some(
                serde_json::json!([ { "AssignmentExpression": { "array": false, "object": true }, "VariableDeclarator": { "array": false, "object": false }, }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: Record<number, unknown>;
                    let i: number = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: Record<number, unknown>;
                    let i: 0 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: Record<number, unknown>;
                    let i: 0 | 1 | 2 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: unknown[];
                    let i: number = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: unknown[];
                    let i: 0 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: unknown[];
                    let i: 0 | 1 | 2 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: { 0: unknown } | unknown[];
                    let y = x[0];
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let x: { 0: unknown } | unknown[];
                    y = x[0];
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let obj = { foo: 'bar' };
                    const foo = obj.foo;
                  ",
            None,
        ),
        (
            "
                    let obj = { foo: 'bar' };
                    var x: null = null;
                    const foo = (x, obj).foo;
                  ",
            None,
        ),
        ("const call = (() => null).call;", None),
        (
            "
                    const obj = { foo: 'bar' };
                    let a: any;
                    var foo = (a = obj).foo;
                  ",
            None,
        ),
        (
            "
                    const obj = { asdf: { qwer: null } };
                    const qwer = obj.asdf.qwer;
                  ",
            None,
        ),
        (
            "
                    const obj = { foo: 100 };
                    const /* comment */ foo = obj.foo;
                  ",
            None,
        ),
        (
            "
                    let obj = { foo: 'bar' };
                    const x = obj.foo;
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let obj = { foo: 'bar' };
                    let x: unknown;
                    x = obj.foo;
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let obj: Record<string, unknown>;
                    let key = 'abc';
                    const x = obj[key];
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let obj: Record<string, unknown>;
                    let key = 'abc';
                    let x: unknown;
                    x = obj[key];
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
    ];

    let fix = vec![
        (
            "
                    let obj = { foo: 'bar' };
                    const foo = obj.foo;
                  ",
            "
                    let obj = { foo: 'bar' };
                    const {foo} = obj;
                  ",
        ),
        (
            "
                    let obj = { foo: 'bar' };
                    var x: null = null;
                    const foo = (x, obj).foo;
                  ",
            "
                    let obj = { foo: 'bar' };
                    var x: null = null;
                    const {foo} = (x, obj);
                  ",
        ),
        ("const call = (() => null).call;", "const {call} = () => null;"),
        (
            "
                    const obj = { foo: 'bar' };
                    let a: any;
                    var foo = (a = obj).foo;
                  ",
            "
                    const obj = { foo: 'bar' };
                    let a: any;
                    var {foo} = a = obj;
                  ",
        ),
        (
            "
                    const obj = { asdf: { qwer: null } };
                    const qwer = obj.asdf.qwer;
                  ",
            "
                    const obj = { asdf: { qwer: null } };
                    const {qwer} = obj.asdf;
                  ",
        ),
        (
            "
                    const obj = { foo: 100 };
                    const /* comment */ foo = obj.foo;
                  ",
            "
                    const obj = { foo: 100 };
                    const /* comment */ {foo} = obj;
                  ",
        ),
    ];

    Tester::new(PreferDestructuring::NAME, PreferDestructuring::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
