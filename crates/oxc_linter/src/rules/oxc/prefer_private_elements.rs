use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, Class, ClassElement, ComputedMemberExpression, Expression,
        Function, MethodDefinitionKind, PropertyKey, StaticMemberExpression, TSAccessibility,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_private_elements_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Prefer ECMAScript private elements over TypeScript `private` for `{name}`.",
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferPrivateElements;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer ECMAScript private elements (`#name`) over TypeScript's `private`
    /// accessibility modifier on class members.
    ///
    /// ### Why is this bad?
    ///
    /// TypeScript's `private` modifier is only enforced by the type system.
    /// ECMAScript private elements are enforced by the language itself, which
    /// gives them stronger runtime encapsulation.
    ///
    /// Private elements also use the language's private-name machinery, which
    /// makes them a better fit for production builds that rely on minification.
    /// A regular TypeScript `private foo` still emits a normal property name,
    /// while `#foo` is a private element that can be mangled by minifiers.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// class Counter {
    ///   private value = 0;
    ///
    ///   private get count() {
    ///     return this.value;
    ///   }
    ///
    ///   private set count(next: number) {
    ///     this.value = next;
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// class Counter {
    ///   #value = 0;
    ///
    ///   get #count() {
    ///     return this.#value;
    ///   }
    ///
    ///   set #count(next: number) {
    ///     this.#value = next;
    ///   }
    /// }
    /// ```
    PreferPrivateElements,
    oxc,
    restriction,
    conditional_dangerous_fix,
    version = "next",
);

impl Rule for PreferPrivateElements {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(target) = PrivateMemberTarget::from_node(node.kind()) else {
            return;
        };

        let Some(class) = ctx.nodes().ancestors(node.id()).find_map(|node| node.kind().as_class())
        else {
            return;
        };

        if should_skip_duplicate_accessor_report(target, class) {
            return;
        }

        let report_span = private_keyword_span(target, ctx).unwrap_or(target.report_span);
        let diagnostic = prefer_private_elements_diagnostic(report_span, target.name);
        let fixable_targets = collect_fixable_targets(class, ctx);
        let current_fix_index = fixable_targets
            .iter()
            .position(|(candidate, _)| candidate.declaration_span == target.declaration_span);

        if let Some(current_fix_index) = current_fix_index {
            if current_fix_index > 0 {
                return;
            }

            let fix_plan = if fixable_targets.len() == 1 {
                fixable_targets[0].1.clone()
            } else {
                FixPlan::merge(
                    &fixable_targets.iter().map(|(_, plan)| plan.clone()).collect::<Vec<_>>(),
                )
            };

            ctx.diagnostic_with_dangerous_fix(diagnostic, |fixer| {
                let fixer = fixer.for_multifix();
                let mut fix = fixer.new_fix_with_capacity(
                    fix_plan.declarations.len() * 2 + fix_plan.references.len(),
                );

                for declaration in &fix_plan.declarations {
                    fix.push(fixer.delete_range(declaration.private_modifier_span));
                    fix.push(
                        fixer.replace(declaration.name_span, format!("#{}", declaration.name)),
                    );
                }

                for reference in &fix_plan.references {
                    fix.push(fixer.replace(reference.span, format!("#{}", reference.name)));
                }

                let message = if fix_plan.declarations.len() == 1 {
                    format!(
                        "Convert `{}` to an ECMAScript private element",
                        fix_plan.declarations[0].name
                    )
                } else {
                    "Convert TypeScript `private` members to ECMAScript private elements"
                        .to_string()
                };

                fix.with_message(message)
            });
            return;
        }

        let Some(fix_plan) = FixPlan::build(target, class, ctx) else {
            ctx.diagnostic(diagnostic);
            return;
        };
        ctx.diagnostic_with_dangerous_fix(diagnostic, |fixer| {
            let fixer = fixer.for_multifix();
            let mut fix =
                fixer.new_fix_with_capacity(fix_plan.declarations.len() * 2 + fix_plan.references.len());

            for declaration in &fix_plan.declarations {
                fix.push(fixer.delete_range(declaration.private_modifier_span));
                fix.push(fixer.replace(declaration.name_span, format!("#{}", declaration.name)));
            }

            for reference in &fix_plan.references {
                fix.push(fixer.replace(reference.span, format!("#{}", reference.name)));
            }

            let message = if fix_plan.declarations.len() == 1 {
                format!(
                    "Convert `{}` to an ECMAScript private element",
                    fix_plan.declarations[0].name
                )
            } else {
                "Convert TypeScript `private` members to ECMAScript private elements".to_string()
            };

            fix.with_message(message)
        });
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn private_keyword_span(target: PrivateMemberTarget<'_>, ctx: &LintContext<'_>) -> Option<Span> {
    let start = target.declaration_span.start
        + ctx.find_next_token_within(
            target.declaration_span.start,
            target.name_span.start,
            "private",
        )?;
    Some(Span::new(start, start + 7))
}

#[derive(Clone, Copy)]
struct PrivateMemberTarget<'a> {
    name: &'a str,
    name_span: Span,
    declaration_span: Span,
    report_span: Span,
    is_static: bool,
    accessor_kind: Option<MethodDefinitionKind>,
}

impl<'a> PrivateMemberTarget<'a> {
    fn from_node(node: AstKind<'a>) -> Option<Self> {
        match node {
            AstKind::PropertyDefinition(prop) => {
                if prop.accessibility != Some(TSAccessibility::Private) || prop.computed {
                    return None;
                }
                let PropertyKey::StaticIdentifier(ident) = &prop.key else {
                    return None;
                };
                Some(Self {
                    name: ident.name.as_str(),
                    name_span: ident.span,
                    declaration_span: prop.span,
                    report_span: prop.span,
                    is_static: prop.r#static,
                    accessor_kind: None,
                })
            }
            AstKind::AccessorProperty(prop) => {
                if prop.accessibility != Some(TSAccessibility::Private) || prop.computed {
                    return None;
                }
                let PropertyKey::StaticIdentifier(ident) = &prop.key else {
                    return None;
                };
                Some(Self {
                    name: ident.name.as_str(),
                    name_span: ident.span,
                    declaration_span: prop.span,
                    report_span: prop.span,
                    is_static: prop.r#static,
                    accessor_kind: None,
                })
            }
            AstKind::MethodDefinition(method) => {
                if method.accessibility != Some(TSAccessibility::Private)
                    || method.computed
                    || method.kind == MethodDefinitionKind::Constructor
                {
                    return None;
                }
                let PropertyKey::StaticIdentifier(ident) = &method.key else {
                    return None;
                };
                Some(Self {
                    name: ident.name.as_str(),
                    name_span: ident.span,
                    declaration_span: method.span,
                    report_span: method.span,
                    is_static: method.r#static,
                    accessor_kind: match method.kind {
                        MethodDefinitionKind::Get | MethodDefinitionKind::Set => Some(method.kind),
                        MethodDefinitionKind::Method | MethodDefinitionKind::Constructor => None,
                    },
                })
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
struct DeclarationEdit {
    name: String,
    private_modifier_span: Span,
    name_span: Span,
}

#[derive(Clone, Copy)]
struct ReferenceEdit<'a> {
    name: &'a str,
    span: Span,
}

#[derive(Clone)]
struct FixPlan<'a> {
    declarations: Vec<DeclarationEdit>,
    references: Vec<ReferenceEdit<'a>>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> FixPlan<'a> {
    fn build(
        target: PrivateMemberTarget<'a>,
        class: &'a Class<'a>,
        ctx: &LintContext<'a>,
    ) -> Option<Self> {
        let declarations = collect_declaration_edits(target, class, ctx)?;

        let mut collector =
            ReferenceCollector::new(target, class.id.as_ref().map(|id| id.name.as_str()));
        collector.visit_class(class);
        if collector.unsupported {
            return None;
        }

        Some(Self {
            declarations,
            references: collector
                .reference_spans
                .into_iter()
                .map(|span| ReferenceEdit { name: target.name, span })
                .collect(),
            _marker: std::marker::PhantomData,
        })
    }

    fn merge(plans: &[Self]) -> Self {
        let mut declarations = vec![];
        let mut references = vec![];

        for plan in plans {
            for declaration in &plan.declarations {
                if declarations.iter().any(|existing: &DeclarationEdit| {
                    existing.private_modifier_span == declaration.private_modifier_span
                        && existing.name_span == declaration.name_span
                }) {
                    continue;
                }
                declarations.push(declaration.clone());
            }

            for reference in &plan.references {
                if references
                    .iter()
                    .any(|existing: &ReferenceEdit<'a>| existing.span == reference.span)
                {
                    continue;
                }
                references.push(*reference);
            }
        }

        Self { declarations, references, _marker: std::marker::PhantomData }
    }
}

fn collect_declaration_edits<'a>(
    target: PrivateMemberTarget<'a>,
    class: &'a Class<'a>,
    ctx: &LintContext<'a>,
) -> Option<Vec<DeclarationEdit>> {
    let mut edits = vec![DeclarationEdit {
        name: target.name.to_string(),
        private_modifier_span: private_modifier_span(
            target.declaration_span,
            target.name_span.start,
            ctx,
        )?,
        name_span: target.name_span,
    }];

    let mut found_accessor_pair = false;

    for element in &class.body.body {
        match element {
            ClassElement::MethodDefinition(method)
                if same_static_identifier_name(&method.key, target.name)
                    && method.r#static == target.is_static =>
            {
                if method.span == target.declaration_span {
                    continue;
                }

                if method.kind == MethodDefinitionKind::Constructor {
                    return None;
                }

                match (target.accessor_kind, method.kind) {
                    (Some(MethodDefinitionKind::Get), MethodDefinitionKind::Set)
                    | (Some(MethodDefinitionKind::Set), MethodDefinitionKind::Get)
                        if method.accessibility == Some(TSAccessibility::Private) =>
                    {
                        let PropertyKey::StaticIdentifier(ident) = &method.key else {
                            return None;
                        };
                        edits.push(DeclarationEdit {
                            name: target.name.to_string(),
                            private_modifier_span: private_modifier_span(
                                method.span,
                                ident.span.start,
                                ctx,
                            )?,
                            name_span: ident.span,
                        });
                        found_accessor_pair = true;
                    }
                    _ => return None,
                }
            }
            ClassElement::PropertyDefinition(prop)
                if same_static_identifier_name(&prop.key, target.name)
                    && prop.r#static == target.is_static
                    && prop.span != target.declaration_span =>
            {
                return None;
            }
            ClassElement::AccessorProperty(prop)
                if same_static_identifier_name(&prop.key, target.name)
                    && prop.r#static == target.is_static
                    && prop.span != target.declaration_span =>
            {
                return None;
            }
            _ => {}
        }
    }

    if target.accessor_kind.is_some() {
        for element in &class.body.body {
            match element {
                ClassElement::PropertyDefinition(prop)
                    if same_name_any_identifier(&prop.key, target.name)
                        && prop.span != target.declaration_span =>
                {
                    return None;
                }
                ClassElement::AccessorProperty(prop)
                    if same_name_any_identifier(&prop.key, target.name)
                        && prop.span != target.declaration_span =>
                {
                    return None;
                }
                _ => {}
            }
        }

        if let Some(kind) = target.accessor_kind
            && has_matching_opposite_accessor(kind)
            && !found_accessor_pair
        {
            // A lone getter or setter is still fixable, but if the class already
            // contains another same-named declaration we bail out above.
        }
    }

    Some(edits)
}

fn has_matching_opposite_accessor(kind: MethodDefinitionKind) -> bool {
    matches!(kind, MethodDefinitionKind::Get | MethodDefinitionKind::Set)
}

fn should_skip_duplicate_accessor_report(
    target: PrivateMemberTarget<'_>,
    class: &Class<'_>,
) -> bool {
    let Some(kind) = target.accessor_kind else {
        return false;
    };

    class.body.body.iter().any(|element| {
        let ClassElement::MethodDefinition(method) = element else {
            return false;
        };

        same_static_identifier_name(&method.key, target.name)
            && method.r#static == target.is_static
            && method.accessibility == Some(TSAccessibility::Private)
            && matches!(
                (kind, method.kind),
                (MethodDefinitionKind::Get, MethodDefinitionKind::Set)
                    | (MethodDefinitionKind::Set, MethodDefinitionKind::Get)
            )
            && method.span.start < target.declaration_span.start
    })
}

fn private_modifier_span(
    declaration_span: Span,
    name_start: u32,
    ctx: &LintContext<'_>,
) -> Option<Span> {
    let start = declaration_span.start;
    let offset = ctx.find_next_token_within(start, name_start, "private")?;
    let private_start = start + offset;
    let private_end = private_start + 7;

    let trailing = ctx
        .source_range(Span::new(private_end, name_start))
        .chars()
        .take_while(|ch| ch.is_whitespace())
        .map(|ch| u32::try_from(ch.len_utf8()).unwrap())
        .sum::<u32>();

    Some(Span::new(private_start, private_end + trailing))
}

fn same_static_identifier_name(key: &PropertyKey<'_>, expected: &str) -> bool {
    matches!(key, PropertyKey::StaticIdentifier(ident) if ident.name == expected)
}

fn same_name_any_identifier(key: &PropertyKey<'_>, expected: &str) -> bool {
    key.name().is_some_and(|name| name == expected)
}

fn collect_fixable_targets<'a>(
    class: &'a Class<'a>,
    ctx: &LintContext<'a>,
) -> Vec<(PrivateMemberTarget<'a>, FixPlan<'a>)> {
    let mut fixable_targets = vec![];

    for element in &class.body.body {
        let target = match element {
            ClassElement::PropertyDefinition(prop) => {
                PrivateMemberTarget::from_node(AstKind::PropertyDefinition(prop))
            }
            ClassElement::AccessorProperty(prop) => {
                PrivateMemberTarget::from_node(AstKind::AccessorProperty(prop))
            }
            ClassElement::MethodDefinition(method) => {
                PrivateMemberTarget::from_node(AstKind::MethodDefinition(method))
            }
            _ => None,
        };

        let Some(target) = target else {
            continue;
        };

        if should_skip_duplicate_accessor_report(target, class) {
            continue;
        }

        if let Some(fix_plan) = FixPlan::build(target, class, ctx) {
            fixable_targets.push((target, fix_plan));
        }
    }

    fixable_targets.sort_by_key(|(target, _)| target.declaration_span.start);
    fixable_targets
}

struct ReferenceCollector<'a> {
    target: PrivateMemberTarget<'a>,
    class_name: Option<&'a str>,
    class_depth: u8,
    callable_depth: u8,
    reference_spans: Vec<Span>,
    unsupported: bool,
}

impl<'a> ReferenceCollector<'a> {
    fn new(target: PrivateMemberTarget<'a>, class_name: Option<&'a str>) -> Self {
        Self {
            target,
            class_name,
            class_depth: 0,
            callable_depth: 0,
            reference_spans: vec![],
            unsupported: false,
        }
    }

    fn receiver_is_allowed(&self, object: &Expression<'a>) -> bool {
        if self.target.is_static {
            matches!(object.without_parentheses(), Expression::Identifier(ident) if self.class_name.is_some_and(|class_name| ident.name == class_name))
        } else {
            matches!(object.without_parentheses(), Expression::ThisExpression(_))
        }
    }
}

impl<'a> Visit<'a> for ReferenceCollector<'a> {
    fn visit_class(&mut self, class: &Class<'a>) {
        if self.class_depth > 0 {
            return;
        }
        self.class_depth += 1;
        walk::walk_class(self, class);
        self.class_depth -= 1;
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        self.callable_depth += 1;
        walk::walk_function(self, function, flags);
        self.callable_depth -= 1;
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.callable_depth += 1;
        walk::walk_arrow_function_expression(self, arrow);
        self.callable_depth -= 1;
    }

    fn visit_static_member_expression(&mut self, expr: &StaticMemberExpression<'a>) {
        if expr.property.name == self.target.name {
            if self.callable_depth > 1 || !self.receiver_is_allowed(&expr.object) {
                self.unsupported = true;
            } else {
                self.reference_spans.push(expr.property.span);
            }
        }

        walk::walk_static_member_expression(self, expr);
    }

    fn visit_computed_member_expression(&mut self, expr: &ComputedMemberExpression<'a>) {
        if expr.static_property_name().is_some_and(|name| name == self.target.name) {
            self.unsupported = true;
        }

        walk::walk_computed_member_expression(self, expr);
    }
}

#[test]
fn test() {
    use crate::{fixer::FixKind, tester::Tester};

    let pass = vec![
        "class Foo { #value = 1; }",
        "class Foo { #method() {} }",
        "class Foo { get #value() { return 1; } }",
        "class Foo { set #value(v) {} }",
        "class Foo { static #value = 1; }",
        "class Foo { public value = 1; protected other() {} }",
        "class Foo { private constructor() {} }",
        "class Foo { constructor(private readonly value: string) {} }",
        "class Foo { private ['value'] = 1; }",
        "class Foo { private ['value']() {} }",
    ];

    let fail = vec![
        "class Foo { private value: string; }",
        "class Foo { private method() {} }",
        "class Foo { private get value() { return 1; } }",
        "class Foo { private set value(next: number) {} }",
        "class Foo { private accessor value = 1; }",
        "class Foo { private static value = 1; }",
        "class Foo { private value = 1; method() { return other.value; } }",
        "class Foo { private static value = 1; static method() { return this.value; } }",
        "const Foo = class { private static value = 1; static method() { return Foo.value; } }",
    ];

    let fix = vec![
        (
            "class Foo { private value = 1; method() { return this.value + 1; } }",
            "class Foo { #value = 1; method() { return this.#value + 1; } }",
            None,
            FixKind::DangerousFix,
        ),
        (
            "class Foo { private method() { return 1; } run() { return this.method(); } }",
            "class Foo { #method() { return 1; } run() { return this.#method(); } }",
            None,
            FixKind::DangerousFix,
        ),
        (
            "class Foo { private get value() { return 1; } private set value(next: number) {} method() { this.value = 2; return this.value; } }",
            "class Foo { get #value() { return 1; } set #value(next: number) {} method() { this.#value = 2; return this.#value; } }",
            None,
            FixKind::DangerousFix,
        ),
        (
            "class Foo { private static value = 1; static read() { return Foo.value; } }",
            "class Foo { static #value = 1; static read() { return Foo.#value; } }",
            None,
            FixKind::DangerousFix,
        ),
        (
            "class Foo { private value: number = 1; method(): number { this.value = 2; return this.value; } }",
            "class Foo { #value: number = 1; method(): number { this.#value = 2; return this.#value; } }",
            None,
            FixKind::DangerousFix,
        ),
    ];

    Tester::new(PreferPrivateElements::NAME, PreferPrivateElements::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .expect_fix(fix)
        .test_and_snapshot();
}
