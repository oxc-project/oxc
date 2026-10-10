use itertools::Itertools;
use oxc_ast::{
    AstKind,
    ast::{
        Class, ClassElement, ClassType, Declaration, ExportDefaultDeclarationKind, FormalParameter,
        FormalParameterRest, Function, FunctionType, MethodDefinition, MethodDefinitionKind,
        PropertyKey, Statement, TSCallSignatureDeclaration, TSConstructSignatureDeclaration,
        TSMethodSignature, TSMethodSignatureKind, TSSignature, TSThisParameter, TSType,
        TSTypeAnnotation, TSTypeParameter, TSTypeParameterDeclaration,
    },
};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::{FxHashMap, FxHashSet};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn unified_signatures_diagnostic<L: Into<LabeledSpan>, T: IntoIterator<Item = L>>(
    message: String,
    labels: T,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(message).with_labels(labels)
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct UnifiedSignaturesOptions {
    /// Whether to ignore parameter name differences when comparing signatures. If `false`, signatures
    /// will not be considered unifiable if they have parameters in the same position with different
    /// names, even if the parameter types are the same.
    ignore_differently_named_parameters: bool,
    /// Whether to ignore JSDoc differences when comparing signatures. If `false`, signatures will not
    /// be considered unifiable if the closest leading block comments for the signatures are different,
    /// even if the signatures themselves are identical.
    #[serde(rename = "ignoreOverloadsWithDifferentJSDoc")]
    ignore_overloads_with_different_jsdoc: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct UnifiedSignatures(UnifiedSignaturesOptions);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow overload signatures that can be unified into one.
    ///
    /// ### Why is this bad?
    ///
    /// Duplicate overload signatures that only differ by a single type, or by an optional/rest
    /// parameter, are harder to maintain and read than a single unified signature.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// function f(a: number): void;
    /// function f(a: string): void;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// function f(a: number | string): void;
    /// ```
    UnifiedSignatures,
    typescript,
    style,
    config = UnifiedSignaturesOptions,
    version = "1.48.0",
    short_description = "Disallow overload signatures that can be unified into one.",
);

impl Rule for UnifiedSignatures {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Program(program) => {
                self.check_statement_scope(&program.body, None, ctx);
            }
            AstKind::TSModuleBlock(module_block) => {
                self.check_statement_scope(&module_block.body, None, ctx);
            }
            AstKind::TSInterfaceDeclaration(interface_decl) => {
                self.check_signature_scope(
                    &interface_decl.body.body,
                    interface_decl.type_parameters.as_deref(),
                    ctx,
                );
            }
            AstKind::TSTypeLiteral(type_literal) => {
                self.check_signature_scope(&type_literal.members, None, ctx);
            }
            AstKind::Class(class) if class.r#type == ClassType::ClassDeclaration => {
                self.check_class_scope(class, ctx);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[derive(Clone, Copy)]
struct OverloadCandidate<'a> {
    signature: SignatureDefinition<'a>,
    comment_target_start: u32,
}

#[derive(Clone, Copy)]
enum SignatureDefinition<'a> {
    Function(&'a Function<'a>),
    MethodDefinition(&'a MethodDefinition<'a>),
    TSMethodSignature(&'a TSMethodSignature<'a>),
    TSCallSignatureDeclaration(&'a TSCallSignatureDeclaration<'a>),
    TSConstructSignatureDeclaration(&'a TSConstructSignatureDeclaration<'a>),
}

impl GetSpan for SignatureDefinition<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Function(function) => function.span,
            Self::MethodDefinition(method) => method.value.span,
            Self::TSMethodSignature(signature) => signature.span,
            Self::TSCallSignatureDeclaration(signature) => signature.span,
            Self::TSConstructSignatureDeclaration(signature) => signature.span,
        }
    }
}

impl<'a> SignatureDefinition<'a> {
    fn type_parameters(self) -> Option<&'a TSTypeParameterDeclaration<'a>> {
        match self {
            Self::Function(function) => function.type_parameters.as_deref(),
            Self::MethodDefinition(method) => method.value.type_parameters.as_deref(),
            Self::TSMethodSignature(method_signature) => {
                method_signature.type_parameters.as_deref()
            }
            Self::TSCallSignatureDeclaration(call_signature) => {
                call_signature.type_parameters.as_deref()
            }
            Self::TSConstructSignatureDeclaration(construct_signature) => {
                construct_signature.type_parameters.as_deref()
            }
        }
    }

    fn this_param(self) -> Option<&'a TSThisParameter<'a>> {
        match self {
            Self::Function(function) => function.this_param.as_deref(),
            Self::MethodDefinition(method) => method.value.this_param.as_deref(),
            Self::TSMethodSignature(method_signature) => method_signature.this_param.as_deref(),
            Self::TSCallSignatureDeclaration(call_signature) => {
                call_signature.this_param.as_deref()
            }
            Self::TSConstructSignatureDeclaration(_) => None,
        }
    }

    fn params(self) -> &'a [FormalParameter<'a>] {
        match self {
            Self::Function(function) => &function.params.items,
            Self::MethodDefinition(method) => &method.value.params.items,
            Self::TSMethodSignature(method_signature) => &method_signature.params.items,
            Self::TSCallSignatureDeclaration(call_signature) => &call_signature.params.items,
            Self::TSConstructSignatureDeclaration(construct_signature) => {
                &construct_signature.params.items
            }
        }
    }

    fn rest_param(self) -> Option<&'a FormalParameterRest<'a>> {
        match self {
            Self::Function(function) => function.params.rest.as_deref(),
            Self::MethodDefinition(method) => method.value.params.rest.as_deref(),
            Self::TSMethodSignature(method_signature) => method_signature.params.rest.as_deref(),
            Self::TSCallSignatureDeclaration(call_signature) => {
                call_signature.params.rest.as_deref()
            }
            Self::TSConstructSignatureDeclaration(construct_signature) => {
                construct_signature.params.rest.as_deref()
            }
        }
    }

    fn return_type(self) -> Option<&'a TSTypeAnnotation<'a>> {
        match self {
            Self::Function(function) => function.return_type.as_deref(),
            Self::MethodDefinition(method) => method.value.return_type.as_deref(),
            Self::TSMethodSignature(method_signature) => method_signature.return_type.as_deref(),
            Self::TSCallSignatureDeclaration(call_signature) => {
                call_signature.return_type.as_deref()
            }
            Self::TSConstructSignatureDeclaration(construct_signature) => {
                construct_signature.return_type.as_deref()
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Parameter<'a> {
    This(&'a TSThisParameter<'a>),
    Formal(&'a FormalParameter<'a>),
    Rest(&'a FormalParameterRest<'a>),
}

#[derive(Clone, Copy)]
enum Unify<'a> {
    IdenticalParameters { first: Span, second: Span },
    SingleParameterDifference { p0: Parameter<'a>, p1: Parameter<'a> },
    ExtraParameter { extra_parameter: Parameter<'a>, anchor_parameter: Option<Parameter<'a>> },
}

impl UnifiedSignatures {
    fn check_class_scope<'a>(&self, class: &'a Class<'a>, ctx: &LintContext<'a>) {
        let mut overloads: FxHashMap<String, Vec<OverloadCandidate<'a>>> = FxHashMap::default();
        let source_text = ctx.source_text();

        for element in &class.body.body {
            let ClassElement::MethodDefinition(method) = element else {
                continue;
            };
            if method.value.body.is_some() || is_getter_or_setter_method(method.kind) {
                continue;
            }

            let key = get_overload_key(method.computed, method.r#static, &method.key, source_text);
            overloads.entry(key).or_default().push(OverloadCandidate {
                signature: SignatureDefinition::MethodDefinition(method),
                comment_target_start: method.span.start,
            });
        }

        self.check_overloads(&overloads, class.type_parameters.as_deref(), ctx);
    }

    fn check_signature_scope<'a>(
        &self,
        signatures: &'a [TSSignature<'a>],
        type_parameters: Option<&'a TSTypeParameterDeclaration<'a>>,
        ctx: &LintContext<'a>,
    ) {
        let mut overloads: FxHashMap<String, Vec<OverloadCandidate<'a>>> = FxHashMap::default();
        let source_text = ctx.source_text();

        for signature in signatures {
            match signature {
                TSSignature::TSCallSignatureDeclaration(call_signature) => {
                    let key = format!("{}{}()", overload_key_bit(false), overload_key_bit(false));
                    overloads.entry(key).or_default().push(OverloadCandidate {
                        signature: SignatureDefinition::TSCallSignatureDeclaration(call_signature),
                        comment_target_start: call_signature.span.start,
                    });
                }
                TSSignature::TSConstructSignatureDeclaration(construct_signature) => {
                    let key = format!(
                        "{}{}constructor",
                        overload_key_bit(false),
                        overload_key_bit(false)
                    );
                    overloads.entry(key).or_default().push(OverloadCandidate {
                        signature: SignatureDefinition::TSConstructSignatureDeclaration(
                            construct_signature,
                        ),
                        comment_target_start: construct_signature.span.start,
                    });
                }
                TSSignature::TSMethodSignature(method_signature)
                    if method_signature.kind == TSMethodSignatureKind::Method =>
                {
                    let key = get_overload_key(
                        method_signature.computed,
                        false,
                        &method_signature.key,
                        source_text,
                    );
                    overloads.entry(key).or_default().push(OverloadCandidate {
                        signature: SignatureDefinition::TSMethodSignature(method_signature),
                        comment_target_start: method_signature.span.start,
                    });
                }
                _ => {}
            }
        }

        self.check_overloads(&overloads, type_parameters, ctx);
    }

    fn check_statement_scope<'a>(
        &self,
        statements: &'a [Statement<'a>],
        type_parameters: Option<&'a TSTypeParameterDeclaration<'a>>,
        ctx: &LintContext<'a>,
    ) {
        let mut overloads: FxHashMap<String, Vec<OverloadCandidate<'a>>> = FxHashMap::default();

        for statement in statements {
            let Some((key, candidate)) = get_statement_overload(statement) else {
                continue;
            };
            overloads.entry(key).or_default().push(candidate);
        }

        self.check_overloads(&overloads, type_parameters, ctx);
    }

    fn check_overloads<'a>(
        &self,
        overloads: &FxHashMap<String, Vec<OverloadCandidate<'a>>>,
        type_parameters: Option<&'a TSTypeParameterDeclaration<'a>>,
        ctx: &LintContext<'a>,
    ) {
        let outer_type_parameters: FxHashSet<&'a str> = type_parameters
            .map(|type_parameters| {
                type_parameters
                    .params
                    .iter()
                    .map(|type_parameter| type_parameter.name.name.as_str())
                    .collect()
            })
            .unwrap_or_default();

        for signatures in overloads.values() {
            if signatures.len() < 2 {
                continue;
            }

            for i in 0..signatures.len() {
                for j in (i + 1)..signatures.len() {
                    let Some(unify) = compare_signatures(
                        &signatures[i],
                        &signatures[j],
                        &outer_type_parameters,
                        &self.0,
                        ctx,
                    ) else {
                        continue;
                    };
                    report_failure(unify, signatures.len() == 2, ctx);
                }
            }
        }
    }
}

fn get_statement_overload<'a>(
    statement: &'a Statement<'a>,
) -> Option<(String, OverloadCandidate<'a>)> {
    if let Some(declaration) = statement.as_declaration() {
        let Declaration::FunctionDeclaration(function) = declaration else {
            return None;
        };
        if function.r#type != FunctionType::TSDeclareFunction {
            return None;
        }
        let key = get_function_overload_key(function, None)?;
        return Some((
            key,
            OverloadCandidate {
                signature: SignatureDefinition::Function(function),
                comment_target_start: function.span.start,
            },
        ));
    }

    let module_declaration = statement.as_module_declaration()?;
    match module_declaration {
        oxc_ast::ast::ModuleDeclaration::ExportDeclaration(export_decl) => {
            let Declaration::FunctionDeclaration(function) = &export_decl.declaration else {
                return None;
            };
            if function.r#type != FunctionType::TSDeclareFunction {
                return None;
            }

            let key = get_function_overload_key(function, Some("ExportNamedDeclaration"))?;
            Some((
                key,
                OverloadCandidate {
                    signature: SignatureDefinition::Function(function),
                    comment_target_start: export_decl.span.start,
                },
            ))
        }
        oxc_ast::ast::ModuleDeclaration::ExportDefaultDeclaration(export_default) => {
            let ExportDefaultDeclarationKind::FunctionDeclaration(function) =
                &export_default.declaration
            else {
                return None;
            };
            if function.r#type != FunctionType::TSDeclareFunction {
                return None;
            }

            let key = get_function_overload_key(function, Some("ExportDefaultDeclaration"))?;
            Some((
                key,
                OverloadCandidate {
                    signature: SignatureDefinition::Function(function),
                    comment_target_start: export_default.span.start,
                },
            ))
        }
        _ => None,
    }
}

fn get_function_overload_key(
    function: &Function<'_>,
    exporting_node_kind: Option<&str>,
) -> Option<String> {
    function
        .id
        .as_ref()
        .map(|id| id.name.to_string())
        .or_else(|| exporting_node_kind.map(ToString::to_string))
}

fn overload_key_bit(value: bool) -> char {
    if value { '0' } else { '1' }
}

fn get_overload_key(
    computed: bool,
    r#static: bool,
    key: &PropertyKey<'_>,
    source_text: &str,
) -> String {
    let info = match key {
        PropertyKey::PrivateIdentifier(identifier) => {
            format!("private_identifier_{}", identifier.name)
        }
        PropertyKey::StaticIdentifier(identifier) => format!("identifier_{}", identifier.name),
        _ => key.span().source_text(source_text).to_string(),
    };

    format!("{}{}{}", overload_key_bit(computed), overload_key_bit(r#static), info)
}

fn is_getter_or_setter_method(kind: MethodDefinitionKind) -> bool {
    matches!(kind, MethodDefinitionKind::Get | MethodDefinitionKind::Set)
}

fn compare_signatures<'a>(
    first: &OverloadCandidate<'a>,
    second: &OverloadCandidate<'a>,
    outer_type_parameters: &FxHashSet<&'a str>,
    options: &UnifiedSignaturesOptions,
    ctx: &LintContext<'a>,
) -> Option<Unify<'a>> {
    if !signatures_can_be_unified(first, second, outer_type_parameters, options, ctx) {
        return None;
    }

    let first_parameters = signature_parameters(first.signature);
    let second_parameters = signature_parameters(second.signature);

    if first_parameters.len() == second_parameters.len() {
        return signatures_have_same_amount_of_parameters(
            &first_parameters,
            &second_parameters,
            first.signature.span(),
            second.signature.span(),
            ctx,
        );
    }

    signatures_differ_by_optional_or_rest_parameter(&first_parameters, &second_parameters, ctx)
}

fn signatures_can_be_unified<'a>(
    first: &OverloadCandidate<'a>,
    second: &OverloadCandidate<'a>,
    outer_type_parameters: &FxHashSet<&'a str>,
    options: &UnifiedSignaturesOptions,
    ctx: &LintContext<'a>,
) -> bool {
    let source_text = ctx.source_text();
    let first_signature = first.signature;
    let second_signature = second.signature;

    let first_params = signature_parameters(first_signature);
    let second_params = signature_parameters(second_signature);

    if options.ignore_differently_named_parameters {
        let common_params_length = first_params.len().min(second_params.len());
        for i in 0..common_params_length {
            if parameters_have_same_kind(first_params[i], second_params[i])
                && get_static_parameter_name(first_params[i])
                    != get_static_parameter_name(second_params[i])
            {
                return false;
            }
        }
    }

    if options.ignore_overloads_with_different_jsdoc
        && get_block_comment_for_node(first.comment_target_start, ctx)
            != get_block_comment_for_node(second.comment_target_start, ctx)
    {
        return false;
    }

    types_are_equal(
        first_signature.return_type().map(|annotation| &annotation.type_annotation),
        second_signature.return_type().map(|annotation| &annotation.type_annotation),
        source_text,
    ) && type_parameter_declarations_are_equal(
        first_signature.type_parameters(),
        second_signature.type_parameters(),
        source_text,
    ) && signature_uses_outer_type_parameter(first_signature, outer_type_parameters)
        == signature_uses_outer_type_parameter(second_signature, outer_type_parameters)
}

fn get_block_comment_for_node<'a>(node_start: u32, ctx: &LintContext<'a>) -> Option<&'a str> {
    ctx.comments()
        .iter()
        .rev()
        .find(|comment| {
            comment.attached_to == node_start && comment.is_leading() && comment.is_block()
        })
        .map(|comment| comment.content_span().source_text(ctx.source_text()))
}

fn type_parameter_declarations_are_equal(
    first: Option<&TSTypeParameterDeclaration<'_>>,
    second: Option<&TSTypeParameterDeclaration<'_>>,
    source_text: &str,
) -> bool {
    match (first, second) {
        (None, None) => true,
        (Some(first), Some(second)) => {
            first.params.len() == second.params.len()
                && first
                    .params
                    .iter()
                    .zip(second.params.iter())
                    .all(|(first, second)| type_parameters_are_equal(first, second, source_text))
        }
        _ => false,
    }
}

fn type_parameters_are_equal(
    first: &TSTypeParameter<'_>,
    second: &TSTypeParameter<'_>,
    source_text: &str,
) -> bool {
    first.name.name == second.name.name
        && types_are_equal(first.constraint.as_ref(), second.constraint.as_ref(), source_text)
}

fn signature_uses_outer_type_parameter(
    signature: SignatureDefinition<'_>,
    outer_type_parameters: &FxHashSet<&str>,
) -> bool {
    signature_parameters(signature).into_iter().any(|parameter| {
        parameter_type_annotation(parameter).is_some_and(|type_annotation| {
            type_contains_outer_type_parameter(type_annotation, outer_type_parameters)
        })
    })
}

fn type_contains_outer_type_parameter(
    ty: &TSType<'_>,
    outer_type_parameters: &FxHashSet<&str>,
) -> bool {
    match ty {
        TSType::TSTypeReference(type_reference) => type_reference
            .type_name
            .get_identifier_reference()
            .is_some_and(|identifier_reference| {
                outer_type_parameters.contains(identifier_reference.name.as_str())
            }),
        TSType::TSArrayType(array) => {
            type_contains_outer_type_parameter(&array.element_type, outer_type_parameters)
        }
        _ => false,
    }
}

fn signature_parameters(signature: SignatureDefinition<'_>) -> Vec<Parameter<'_>> {
    let mut parameters = Vec::with_capacity(
        signature.params().len() + usize::from(signature.rest_param().is_some()) + 1,
    );

    if let Some(this_param) = signature.this_param() {
        parameters.push(Parameter::This(this_param));
    }

    parameters.extend(signature.params().iter().map(Parameter::Formal));

    if let Some(rest_param) = signature.rest_param() {
        parameters.push(Parameter::Rest(rest_param));
    }

    parameters
}

fn signatures_have_same_amount_of_parameters<'a>(
    first: &[Parameter<'a>],
    second: &[Parameter<'a>],
    first_signature: Span,
    second_signature: Span,
    ctx: &LintContext<'a>,
) -> Option<Unify<'a>> {
    let first_param = first.first().copied();
    let second_param = second.first().copied();

    if first_param.is_some_and(is_this_param) != second_param.is_some_and(is_this_param) {
        return None;
    }
    if first_param.is_some_and(is_this_void_param) || second_param.is_some_and(is_this_void_param) {
        return None;
    }

    let source_text = ctx.source_text();
    let Some(index) = get_index_of_first_difference(first, second, |first, second| {
        parameters_are_equal(*first, *second, source_text)
    }) else {
        return Some(Unify::IdenticalParameters {
            first: first_signature,
            second: second_signature,
        });
    };

    if !first[index + 1..]
        .iter()
        .zip(second[index + 1..].iter())
        .all(|(first, second)| parameters_are_equal(*first, *second, source_text))
    {
        return None;
    }

    let first = first[index];
    let second = second[index];

    if parameters_have_equal_sigils(first, second) && !matches!(first, Parameter::Rest(_)) {
        return Some(Unify::SingleParameterDifference { p0: first, p1: second });
    }

    None
}

fn signatures_differ_by_optional_or_rest_parameter<'a>(
    first_parameters: &[Parameter<'a>],
    second_parameters: &[Parameter<'a>],
    ctx: &LintContext<'a>,
) -> Option<Unify<'a>> {
    let (longer, shorter) = if first_parameters.len() >= second_parameters.len() {
        (first_parameters, second_parameters)
    } else {
        (second_parameters, first_parameters)
    };

    let first_param = first_parameters.first().copied();
    let second_param = second_parameters.first().copied();

    if first_param.is_some_and(is_this_param) != second_param.is_some_and(is_this_param) {
        return None;
    }
    if first_param.is_some_and(is_this_void_param) || second_param.is_some_and(is_this_void_param) {
        return None;
    }

    let min_length = first_parameters.len().min(second_parameters.len());

    for parameter in longer.iter().skip(min_length + 1) {
        if !parameter_may_be_missing(*parameter) {
            return None;
        }
    }

    for i in 0..min_length {
        if !types_are_equal(
            parameter_type_annotation(first_parameters[i]),
            parameter_type_annotation(second_parameters[i]),
            ctx.source_text(),
        ) {
            return None;
        }
    }

    if min_length > 0 && matches!(shorter[min_length - 1], Parameter::Rest(_)) {
        return None;
    }

    Some(Unify::ExtraParameter {
        extra_parameter: *longer.last().unwrap(),
        anchor_parameter: shorter.last().copied(),
    })
}

fn report_failure(unify: Unify<'_>, only_two_overloads: bool, ctx: &LintContext<'_>) {
    let failure_string_start = if only_two_overloads {
        "These overloads can be combined into one signature"
    } else {
        "This overload can be combined with another overload into one signature"
    };

    match unify {
        Unify::IdenticalParameters { first, second } => {
            ctx.diagnostic(unified_signatures_diagnostic(
                format!("{failure_string_start} with identical parameters."),
                [
                    second.primary_label("this signature can be unified"),
                    first.label("with this overload signature"),
                ],
            ));
        }
        Unify::SingleParameterDifference { p0, p1 } => {
            let types = get_unified_type_text(
                parameter_type_annotation(p0),
                parameter_type_annotation(p1),
                ctx.source_text(),
            );

            ctx.diagnostic(unified_signatures_diagnostic(
                format!("{failure_string_start} taking `{types}`."),
                [
                    parameter_span(p1).primary_label("this parameter can be unified"),
                    parameter_span(p0).label("with this overload parameter"),
                ],
            ));
        }
        Unify::ExtraParameter { extra_parameter, anchor_parameter } => {
            let message = if matches!(extra_parameter, Parameter::Rest(_)) {
                format!("{failure_string_start} with a rest parameter.")
            } else {
                format!("{failure_string_start} with an optional parameter.")
            };

            let mut labels = vec![
                parameter_span(extra_parameter)
                    .primary_label("this parameter only appears in one overload"),
            ];
            if let Some(anchor_parameter) = anchor_parameter {
                labels.push(parameter_span(anchor_parameter).label("matching overload ends here"));
            }

            ctx.diagnostic(unified_signatures_diagnostic(message, labels));
        }
    }
}

fn get_unified_type_text(
    first: Option<&TSType<'_>>,
    second: Option<&TSType<'_>>,
    source_text: &str,
) -> String {
    let (Some(first), Some(second)) = (first, second) else {
        return get_union_member_text(
            first.or(second).expect("one of the differing parameters must have a type annotation"),
            source_text,
        );
    };

    let mut members = Vec::new();
    collect_union_members(first, &mut members);
    collect_union_members(second, &mut members);
    let mut seen = FxHashSet::default();
    members
        .into_iter()
        .filter(|member| seen.insert(member.span().source_text(source_text)))
        .map(|member| get_union_member_text(member, source_text))
        .join(" | ")
}

fn collect_union_members<'t, 'a>(ty: &'t TSType<'a>, members: &mut Vec<&'t TSType<'a>>) {
    // Unlike ESTree, Oxc retains parenthesized type nodes.
    let ty = ty.without_parenthesized();
    if let TSType::TSUnionType(union) = ty {
        for member in &union.types {
            collect_union_members(member, members);
        }
    } else {
        members.push(ty);
    }
}

fn get_union_member_text(ty: &TSType<'_>, source_text: &str) -> String {
    let ty = ty.without_parenthesized();
    let text = ty.span().source_text(source_text);
    if matches!(
        ty,
        TSType::TSConditionalType(_) | TSType::TSConstructorType(_) | TSType::TSFunctionType(_)
    ) {
        format!("({text})")
    } else {
        text.to_string()
    }
}

fn parameter_type_annotation(parameter: Parameter<'_>) -> Option<&TSType<'_>> {
    let annotation = match parameter {
        Parameter::This(this_param) => this_param.type_annotation.as_deref(),
        Parameter::Formal(parameter) => parameter.type_annotation.as_deref(),
        Parameter::Rest(parameter) => parameter.type_annotation.as_deref(),
    }?;
    Some(&annotation.type_annotation)
}

fn parameter_may_be_missing(parameter: Parameter<'_>) -> bool {
    matches!(parameter, Parameter::Rest(_)) || parameter_is_optional(parameter)
}

fn parameter_is_optional(parameter: Parameter<'_>) -> bool {
    match parameter {
        Parameter::Formal(parameter) => parameter.optional,
        Parameter::This(_) | Parameter::Rest(_) => false,
    }
}

fn is_this_param(parameter: Parameter<'_>) -> bool {
    matches!(parameter, Parameter::This(_))
}

fn is_this_void_param(parameter: Parameter<'_>) -> bool {
    matches!(
        parameter_type_annotation(parameter),
        Some(type_annotation)
            if matches!(type_annotation, TSType::TSVoidKeyword(_))
    )
}

fn parameter_span(parameter: Parameter<'_>) -> Span {
    match parameter {
        Parameter::This(this_param) => this_param.span,
        Parameter::Formal(parameter) => parameter.span,
        Parameter::Rest(parameter) => parameter.span,
    }
}

fn get_static_parameter_name(parameter: Parameter<'_>) -> Option<&str> {
    match parameter {
        Parameter::This(_) => Some("this"),
        Parameter::Formal(parameter) => {
            parameter.pattern.get_identifier_name().map(|name| name.as_str())
        }
        Parameter::Rest(parameter) => {
            parameter.rest.argument.get_identifier_name().map(|name| name.as_str())
        }
    }
}

fn parameters_have_same_kind(first: Parameter<'_>, second: Parameter<'_>) -> bool {
    matches!(
        (first, second),
        (Parameter::This(_), Parameter::This(_))
            | (Parameter::Formal(_), Parameter::Formal(_))
            | (Parameter::Rest(_), Parameter::Rest(_))
    )
}

fn parameters_have_equal_sigils(first: Parameter<'_>, second: Parameter<'_>) -> bool {
    matches!(first, Parameter::Rest(_)) == matches!(second, Parameter::Rest(_))
        && parameter_is_optional(first) == parameter_is_optional(second)
}

fn parameters_are_equal(first: Parameter<'_>, second: Parameter<'_>, source_text: &str) -> bool {
    parameters_have_equal_sigils(first, second)
        && types_are_equal(
            parameter_type_annotation(first),
            parameter_type_annotation(second),
            source_text,
        )
}

fn types_are_equal(
    first: Option<&TSType<'_>>,
    second: Option<&TSType<'_>>,
    source_text: &str,
) -> bool {
    match (first, second) {
        (None, None) => true,
        (Some(first), Some(second)) => {
            first.without_parenthesized().span().source_text(source_text)
                == second.without_parenthesized().span().source_text(source_text)
        }
        _ => false,
    }
}

fn get_index_of_first_difference<T>(
    first: &[T],
    second: &[T],
    mut equal: impl FnMut(&T, &T) -> bool,
) -> Option<usize> {
    (0..first.len().min(second.len())).find(|&i| !equal(&first[i], &second[i]))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
            function g(): void;
            function g(a: number, b: number): void;
            function g(a?: number, b?: number): void {}
                ",
            None,
        ),
        (
            "
            function rest(...xs: number[]): void;
            function rest(xs: number[], y: string): void;
            function rest(...args: any[]) {}
                ",
            None,
        ),
        (
            "
            class C {
              constructor();
              constructor(a: number, b: number);
              constructor(a?: number, b?: number) {}

              a(): void;
              a(a: number, b: number): void;
              a(a?: number, b?: number): void {}
            }
                ",
            None,
        ),
        (
            "
            declare class Example {
              privateMethod(a: number): void;
              #privateMethod(a: number, b?: string): void;
            }
                ",
            None,
        ),
        (
            "
            declare class Example {
              #privateMethod1(a: number): void;
              #privateMethod2(a: number, b?: string): void;
            }
                ",
            None,
        ),
        (
            "
            interface I {
              a2(): void;
              a2(x: number, y: number): void;
            }
                ",
            None,
        ),
        (
            "
            interface I {
              a4(): void;
              a4(x: number): number;
            }
                ",
            None,
        ),
        (
            "
            interface I {
              a5<T>(x: T): T;
              a5(x: number): number;
            }
                ",
            None,
        ),
        (
            "
            interface I {
              b2(x: string): void;
              b2(...x: number[]): void;
            }
                ",
            None,
        ),
        (
            "
            interface I {
              b3(...x: number[]): void;
              b3(...x: string[]): void;
            }
                ",
            None,
        ),
        (
            "
            interface I {
              c3(x: number): void;
              c3(x?: string): void;
            }
                ",
            None,
        ),
        (
            "
            interface I {
              d2(x: string, y: number): void;
              d2(x: number, y: string): void;
            }
                ",
            None,
        ),
        (
            "
            declare class D {
              static a();
              a(x: number);
            }
                ",
            None,
        ),
        (
            "
            interface Generic<T> {
              x(): void;
              x(x: T[]): void;
            }
                ",
            None,
        ),
        (
            "
            interface I {
              f(x1: number): void;
              f(x1: boolean, x2?: number): void;
            }
                ",
            None,
        ),
        (
            "
            function f<T extends number>(x: T[]): void;
            function f<T extends string>(x: T): void;
                ",
            None,
        ),
        (
            "
            declare function foo(n: number): number;

            declare module 'hello' {
              function foo(n: number, s: string): number;
            }
                ",
            None,
        ),
        (
            "
            {
              function block(): number;
              function block(n: number): number;
              function block(n?: number): number {
                return 3;
              }
            }
                ",
            None,
        ),
        (
            "
            export interface Foo {
              bar(baz: string): number[];
              bar(): string[];
            }
                ",
            None,
        ),
        (
            "
            declare module 'foo' {
              export default function (foo: number): string[];
            }
                ",
            None,
        ),
        (
            "
            export default function (foo: number): string[];
                ",
            None,
        ),
        (
            "
            function p(key: string): Promise<string | undefined>;
            function p(key: string, defaultValue: string): Promise<string>;
            function p(key: string, defaultValue?: string): Promise<string | undefined> {
              const obj: Record<string, string> = {};
              return obj[key] || defaultValue;
            }
                ",
            None,
        ),
        (
            "
            interface I {
              p<T>(x: T): Promise<T>;
              p(x: number): Promise<number>;
            }
                ",
            None,
        ),
        (
            "
            function rest(...xs: number[]): Promise<number[]>;
            function rest(xs: number[], y: string): Promise<string>;
            async function rest(...args: any[]): Promise<number[] | string> {
              const y = args[1] as string | undefined;
              return y || args;
            }
                ",
            None,
        ),
        (
            "
            declare class Foo {
              get bar();
              set bar(x: number);
            }
                ",
            None,
        ),
        (
            "
            interface Foo {
              get bar();
              set bar(x: number);
            }
                ",
            None,
        ),
        (
            "
            abstract class Foo {
              abstract get bar();
              abstract set bar(a: unknown);
            }
                ",
            None,
        ),
        (
            "
            function f(a: number): void;
            function f(b: string): void;
            function f(a: number | string): void {}
                  ",
            Some(serde_json::json!([{ "ignoreDifferentlyNamedParameters": true }])),
        ),
        (
            "
            function f(m: number): void;
            function f(v: number, u: string): void;
            function f(v: number, u?: string): void {}
                  ",
            Some(serde_json::json!([{ "ignoreDifferentlyNamedParameters": true }])),
        ),
        (
            "
            function f(v: boolean): number;
            function f(): string;
                  ",
            Some(serde_json::json!([{ "ignoreDifferentlyNamedParameters": true }])),
        ),
        (
            "
            function f(v: boolean, u: boolean): number;
            function f(v: boolean): string;
                  ",
            Some(serde_json::json!([{ "ignoreDifferentlyNamedParameters": true }])),
        ),
        (
            "
            function f(v: number, u?: string): void {}
            function f(v: number): void;
            function f(): string;
                  ",
            Some(serde_json::json!([{ "ignoreDifferentlyNamedParameters": true }])),
        ),
        (
            "
            function f(a: boolean, ...c: number[]): void;
            function f(a: boolean, ...d: string[]): void;
            function f(a: boolean, ...c: (number | string)[]): void {}
                  ",
            Some(serde_json::json!([{ "ignoreDifferentlyNamedParameters": true }])),
        ),
        (
            "
            class C {
              constructor();
              constructor(a: number, b: number);
              constructor(c?: number, b?: number) {}

              a(): void;
              a(a: number, b: number): void;
              a(a?: number, d?: number): void {}
            }
                  ",
            Some(serde_json::json!([{ "ignoreDifferentlyNamedParameters": true }])),
        ),
        (
            "
            /** @deprecated */
            declare function f(x: number): unknown;
            declare function f(x: boolean): unknown;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            declare function f(x: number): unknown;
            /** @deprecated */
            declare function f(x: boolean): unknown;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            declare function f(x: number): unknown;
            /** @deprecated */ declare function f(x: boolean): unknown;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            declare function f(x: string): void;
            /**
             * @async
             */
            declare function f(x: boolean): void;
            /**
             * @deprecate
             */
            declare function f(x: number): void;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            /**
             * @deprecate
             */
            declare function f(x: string): void;
            /**
             * @async
             */
            declare function f(x: boolean): void;
            declare function f(x: number): void;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            /**
             * This signature does something.
             */
            declare function f(x: number): void;

            /**
             * This signature does something else.
             */
            declare function f(x: string): void;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            /** @deprecated */
            export function f(x: number): unknown;
            export function f(x: boolean): unknown;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            /**
             * This signature does something.
             */

            // some other comment
            export function f(x: number): void;

            /**
             * This signature does something else.
             */
            export function f(x: string): void;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            interface I {
              /**
               * This signature does something else.
               */
              f(x: number): void;
              f(x: string): void;
            }
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            /* @deprecated */
            declare function f(x: number): unknown;
            declare function f(x: boolean): unknown;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            /*
             * This signature does something.
             */
            declare function f(x: number): unknown;
            declare function f(x: boolean): unknown;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            /**
             * This signature does something.
             **/
            declare function f(x: number): unknown;
            declare function f(x: boolean): unknown;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            class C {
              a(b: string): void;
              /**
               * @deprecate
               */
              a(b: number): void;
            }
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            function f(): void;
            function f(this: {}): void;
            function f(this: void | {}): void {}
                ",
            None,
        ),
        (
            "
            function f(a: boolean): void;
            function f(this: {}, a: boolean): void;
            function f(this: void | {}, a: boolean): void {}
                ",
            None,
        ),
        (
            "
            function f(this: void, a: boolean): void;
            function f(this: {}, a: boolean): void;
            function f(this: void | {}, a: boolean): void {}
                ",
            None,
        ),
        (
            "type A = 1 | 2;
type B = 3 | 4;
function f<T extends A>(x: T, y: string): void;
function f<T extends B>(x: T): void;",
            None,
        ),
        (
            "function f<T extends 1 | 2>(x: T, y: string): void;
function f<T extends 3 | 4>(x: T): void;",
            None,
        ),
        (
            "function f<T extends number>(x: T[]): void;
function f<R extends number>(x: R): void;",
            None,
        ),
        ("function f<T>(x: T): void; function f<T extends A>(x: T): void;", None),
        (
            "function f<T extends { a: string }>(x: T): void; function f<T extends { a: number }>(x: T): void;",
            None,
        ),
        ("function f<T extends A>(x: T): void; function f<T extends B>(x: T[]): void;", None),
        ("function f(this: void): void; function f(this: void): void;", None),
        ("function f(x?: number): void; function f(x: number): void;", None),
        (
            "function f(a: number): void; function f(b: number): void;",
            Some(serde_json::json!([{"ignoreDifferentlyNamedParameters": true}])),
        ),
        (
            "/** first */ function f(x: number): void; /** second */ function f(x: number): void;",
            Some(serde_json::json!([{"ignoreOverloadsWithDifferentJSDoc": true}])),
        ),
        ("declare function f(this: string): void; declare function f(x: string): void;", None),
        ("declare function f(x: string): void; declare function f(this: string): void;", None),
        ("declare function f(this: string): void; declare function f(x: number): void;", None),
        ("declare function f(x: number): void; declare function f(this: string): void;", None),
    ];

    let fail = vec![
        (
            "
            function f(a: number): void;
            function f(b: string): void;
            function f(a: number | string): void {}
                  ",
            None,
        ),
        (
            "
            function f(x: number): void;
            function f(x: string): void;
            function f(x: any): any {
              return x;
            }
                  ",
            None,
        ),
        (
            "
            function f(x: number): void;
            function f(x: string): void;
            function f(x: any): any {
              return x;
            }
                  ",
            Some(serde_json::json!([{ "ignoreDifferentlyNamedParameters": true }])),
        ),
        (
            "
            function opt(xs?: number[]): void;
            function opt(xs: number[], y: string): void;
            function opt(...args: any[]) {}
                  ",
            None,
        ),
        (
            "
            interface I {
              a0(): void;
              a0(x: string): string;
              a0(x: number): void;
            }
                  ",
            None,
        ),
        (
            "
            interface I {
              a0(): void;
              a0(x: string): string;
              a0(x: number): void;
            }
                  ",
            Some(serde_json::json!([{ "ignoreDifferentlyNamedParameters": true }])),
        ),
        (
            "
            interface I {
              a1(): void;
              a1(x: number): void;
            }
                  ",
            None,
        ),
        (
            "
            interface I {
              a3(): void;
              a3(x: number, y?: number, ...z: number[]): void;
            }
                  ",
            None,
        ),
        (
            "
            interface I {
              b(): void;
              b(...x: number[]): void;
            }
                  ",
            None,
        ),
        (
            "
            interface I {
              c(): void;
              c(x?: number): void;
            }
                  ",
            None,
        ),
        (
            "
            interface I {
              c2(x?: number): void;
              c2(x?: string): void;
            }
                  ",
            None,
        ),
        (
            "
            interface I {
              d(x: number): void;
              d(x: string): void;
            }
                  ",
            None,
        ),
        (
            "
            type T = {
              (): void;
              (x: number): void;
            };
                  ",
            None,
        ),
        (
            "
            declare class Example {
              #privateMethod(a: number): void;
              #privateMethod(a: number, b?: string): void;
            }
                  ",
            None,
        ),
        (
            "
            declare class C {
              constructor();
              constructor(x: number);
            }
                  ",
            None,
        ),
        (
            "
            interface I {
              f(x: number);
              f(x: string | boolean);
            }
                  ",
            None,
        ),
        (
            "
            interface I {
              f(x: number);
              f(x: [string, boolean]);
            }
                  ",
            None,
        ),
        (
            "
            interface Generic<T> {
              y(x: T[]): void;
              y(x: T): void;
            }
                  ",
            None,
        ),
        (
            "
            function f<T>(x: T[]): void;
            function f<T>(x: T): void;
                  ",
            None,
        ),
        (
            "
            function f<T extends number>(x: T[]): void;
            function f<T extends number>(x: T): void;
                  ",
            None,
        ),
        (
            "
            abstract class Foo {
              public abstract f(x: number): void;
              public abstract f(x: string): void;
            }
                  ",
            None,
        ),
        (
            "
            abstract class C {
              a(b: string): void;
              /**
               * @deprecate
               */
              a(b: number): void;
            }
                  ",
            None,
        ),
        (
            "
            interface Foo {
              'f'(x: string): void;
              'f'(x: number): void;
            }
                  ",
            None,
        ),
        (
            "
            interface Foo {
              new (x: string): Foo;
              new (x: number): Foo;
            }
                  ",
            None,
        ),
        (
            "
            enum Enum {
              Func = 'function',
            }

            interface IFoo {
              [Enum.Func](x: string): void;
              [Enum.Func](x: number): void;
            }
                  ",
            None,
        ),
        (
            "
            export function foo(line: number): number;
            export function foo(line: number, character?: number): number;
                  ",
            None,
        ),
        (
            "
            declare function foo(line: number): number;
            export function foo(line: number, character?: number): number;
                  ",
            None,
        ),
        (
            "
            declare module 'foo' {
              export default function (foo: number): string[];
              export default function (foo: number, bar?: string): string[];
            }
                  ",
            None,
        ),
        (
            "
            export default function (foo: number): string[];
            export default function (foo: number, bar?: string): string[];
                  ",
            None,
        ),
        (
            "
            /**
             * @deprecate
             */
            declare function f(x: string): void;
            declare function f(x: number): void;
            declare function f(x: boolean): void;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            /**
             * @deprecate
             */
            declare function f(x: string): void;
            /**
             * @deprecate
             */
            declare function f(x: number): void;
            declare function f(x: boolean): void;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            declare function f(x: string): void;
            /**
             * @deprecate
             */
            declare function f(x: number): void;
            /**
             * @deprecate
             */
            declare function f(x: boolean): void;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            export function f(x: string): void;
            /**
             * @deprecate
             */
            export function f(x: number): void;
            /**
             * @deprecate
             */
            export function f(x: boolean): void;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            /**
             * This signature does something.
             */

            /**
             * This signature does something else.
             */
            function f(x: number): void;

            /**
             * This signature does something else.
             */
            function f(x: string): void;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            interface I {
              f(x: string): void;
              /**
               * @deprecate
               */
              f(x: number): void;
              /**
               * @deprecate
               */
              f(x: boolean): void;
            }
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            // a line comment
            declare function f(x: number): unknown;
            declare function f(x: boolean): unknown;
                  ",
            Some(serde_json::json!([{ "ignoreOverloadsWithDifferentJSDoc": true }])),
        ),
        (
            "
            function f(this: {}, a: boolean): void;
            function f(this: {}, a: string): void;
            function f(this: {}, a: boolean | string): void {}
                  ",
            None,
        ),
        (
            "
            function f(this: {}): void;
            function f(this: {}, a: string): void;
            function f(this: {}, a?: string): void {}
                  ",
            None,
        ),
        (
            "
            function f(this: string): void;
            function f(this: number): void;
            function f(this: string | number): void {}
                  ",
            None,
        ),
        (
            "
            function f(this: string, a: boolean): void;
            function f(this: number, a: boolean): void;
            function f(this: string | number, a: boolean): void {}
                  ",
            None,
        ),
        (
            "function f(x: string | number): void;
function f(x: number | boolean): void;",
            None,
        ),
        (
            "function f(value): void;
function f(value: string): void;",
            None,
        ),
        (
            "function f(value: string): void;
function f(value): void;",
            None,
        ),
        (
            "type Alias = string;
function f(x: Alias): void;
function f(x: Alias | number): void;",
            None,
        ),
        (
            "type Name = string;
function f(x: Name): void;
function f(x: string | Name): void;",
            None,
        ),
        (
            "function f<T>(x: T): void;
function f<T>(x: T | string): void;",
            None,
        ),
        (
            "function f(x: 'a|b'): void;
function f(x: 'a|b' | string): void;",
            None,
        ),
        (
            "function f(x: string /* first */ | number): void;
function f(x: number | /* second */ string): void;",
            None,
        ),
        (
            "declare function fn(a: number): void;
declare function fn(a: (/* before */ string /* after */)): void;",
            None,
        ),
        (
            "function f(x: string & { brand: true }): void;
function f(x: number | string & /* brand */ { brand: true }): void;",
            None,
        ),
        (
            "function f(x: string & { brand: true }): void;
function f(x: number | string & { brand: true }): void;",
            None,
        ),
        (
            "function f<T, U>(x: () => void): void;
function f<T, U>(x: T extends U ? string : number): void;",
            None,
        ),
        (
            "interface Value {}
function f(x: new () => Value): void;
function f(x: string): void;",
            None,
        ),
        (
            "interface I {
  f(x: string | number): void;
  f(x: number | boolean): void;
  f(x: symbol): string;
}",
            None,
        ),
        (
            "function f(x: 'a' | 'b'): void;
function f(x: 'b' | 'c'): void;",
            None,
        ),
        (
            "function f(x: Array<string> | boolean): void;
function f(x: Array<number> | boolean): void;",
            None,
        ),
        (
            "function f(x: Promise | boolean): void;
function f(x: Promise<string> | boolean): void;",
            None,
        ),
        (
            "namespace Namespace {
  export type Value = string;
}
function f(x: Namespace.Value): void;
function f(x: Namespace.Value | string): void;",
            None,
        ),
        (
            "declare function f(a: number): void;
declare function f(a: number): void;",
            None,
        ),
        (
            "declare function f(a: number): void;
declare function f(a: number): void;
declare function f(a: string): string;",
            None,
        ),
        (
            "function f(a: number): void;
function f(a: number): void;
function f(a: number): void {}",
            None,
        ),
        (
            "class A {
  f(a: number): void;
  f(a: number): void;
  f(a: number): void {}
}",
            None,
        ),
        ("function f(): void; function f(): void;", None),
        ("function f(value): void; function f(value): void;", None),
        ("function f(...values: string[]): void; function f(...values: string[]): void;", None),
        ("function f(x?: string): void; function f(x?: string): void;", None),
        ("function f(this: string): void; function f(this: string): void;", None),
        ("interface I { (): void; (): void; }", None),
        ("interface I { new (): I; new (): I; }", None),
        ("interface I { f(): void; f(): void; }", None),
        (
            "class C { constructor(x: number); constructor(x: number); constructor(x: number) {} }",
            None,
        ),
        (
            "function f(x: string | (number | boolean)): void; function f(x: (boolean | string)): void;",
            None,
        ),
        ("function f(x: (() => void) | string): void; function f(x: () => void): void;", None),
        ("function f<T extends A>(x: T): void; function f<T extends A>(x: T[]): void;", None),
        (
            "function f<T extends 1 | 2>(x: T): void; function f<T extends 1 | 2>(x: T, y: string): void;",
            None,
        ),
        ("function f<T extends (A)>(x: T): void; function f<T extends A>(x: T[]): void;", None),
        ("function f(a: number): void; function f(b: number): void;", None),
        (
            "/** same */ function f(x: number): void; /** same */ function f(x: number): void;",
            Some(serde_json::json!([{"ignoreOverloadsWithDifferentJSDoc": true}])),
        ),
        ("function f(x): void; function f(x: () => void): void;", None),
    ];

    Tester::new(UnifiedSignatures::NAME, UnifiedSignatures::PLUGIN, pass, fail).test_and_snapshot();
}
