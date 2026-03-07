use std::cmp::Ordering;

use cow_utils::CowUtils;
use natord::compare;
use oxc_ast::{
    AstKind,
    ast::{
        ClassElement, Expression, MethodDefinitionKind, MethodDefinitionType, PropertyKey,
        TSAccessibility, TSIndexSignature, TSMethodSignatureKind, TSSignature,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn incorrect_group_order_diagnostic(name: &str, rank: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Member `{name}` should be declared before all `{rank}` definitions."
    ))
    .with_label(span)
}

fn incorrect_order_diagnostic(member: &str, before_member: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Member `{member}` should be declared before member `{before_member}`."
    ))
    .with_label(span)
}

fn incorrect_required_members_order_diagnostic(
    member: &str,
    optional_or_required: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Member `{member}` should be declared after all `{optional_or_required}` members."
    ))
    .with_label(span)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MemberKind {
    Accessor,
    CallSignature,
    Constructor,
    Field,
    Get,
    Method,
    Set,
    Signature,
    StaticInitialization,
    ReadonlyField,
    ReadonlySignature,
}

impl MemberKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Accessor => "accessor",
            Self::CallSignature => "call-signature",
            Self::Constructor => "constructor",
            Self::Field => "field",
            Self::Get => "get",
            Self::Method => "method",
            Self::Set => "set",
            Self::Signature => "signature",
            Self::StaticInitialization => "static-initialization",
            Self::ReadonlyField => "readonly-field",
            Self::ReadonlySignature => "readonly-signature",
        }
    }

    fn supports_decorators(self) -> bool {
        matches!(
            self,
            Self::ReadonlyField
                | Self::Field
                | Self::Method
                | Self::Accessor
                | Self::Get
                | Self::Set
        )
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum AlphabeticalOrder {
    Alphabetically,
    AlphabeticallyCaseInsensitive,
    Natural,
    NaturalCaseInsensitive,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum Order {
    AsWritten,
    Alphabetically,
    AlphabeticallyCaseInsensitive,
    Natural,
    NaturalCaseInsensitive,
}

impl Order {
    fn as_alphabetical(self) -> Option<AlphabeticalOrder> {
        match self {
            Self::AsWritten => None,
            Self::Alphabetically => Some(AlphabeticalOrder::Alphabetically),
            Self::AlphabeticallyCaseInsensitive => {
                Some(AlphabeticalOrder::AlphabeticallyCaseInsensitive)
            }
            Self::Natural => Some(AlphabeticalOrder::Natural),
            Self::NaturalCaseInsensitive => Some(AlphabeticalOrder::NaturalCaseInsensitive),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum OptionalityOrder {
    OptionalFirst,
    RequiredFirst,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
enum MemberType {
    Single(String),
    Group(Vec<String>),
}

impl MemberType {
    fn includes(&self, group: &str) -> bool {
        match self {
            Self::Single(member_type) => member_type == group,
            Self::Group(member_types) => {
                member_types.iter().any(|member_type| member_type == group)
            }
        }
    }

    fn display(&self) -> String {
        match self {
            Self::Single(member_type) => member_type.cow_replace('-', " ").into_owned(),
            Self::Group(member_types) => member_types
                .iter()
                .map(|member_type| member_type.cow_replace('-', " ").into_owned())
                .collect::<Vec<_>>()
                .join(", "),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum NeverConfig {
    Never,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
enum MemberTypesConfig {
    Never(NeverConfig),
    Types(Vec<MemberType>),
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
struct SortedOrderConfig {
    member_types: Option<MemberTypesConfig>,
    optionality_order: Option<OptionalityOrder>,
    order: Option<Order>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
enum OrderConfig {
    Never(NeverConfig),
    Types(Vec<MemberType>),
    Sorted(SortedOrderConfig),
}

impl Default for OrderConfig {
    fn default() -> Self {
        Self::Sorted(SortedOrderConfig {
            member_types: Some(MemberTypesConfig::Types(default_member_types())),
            optionality_order: None,
            order: None,
        })
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct MemberOrderingOptions {
    classes: Option<OrderConfig>,
    class_expressions: Option<OrderConfig>,
    default: OrderConfig,
    interfaces: Option<OrderConfig>,
    type_literals: Option<OrderConfig>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct MemberOrdering(Box<MemberOrderingOptions>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires a consistent member declaration order.
    ///
    /// ### Why is this bad?
    ///
    /// Keeping member declarations in a consistent order makes types and classes easier to scan,
    /// and avoids hard-to-find ordering drift across large codebases.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// class Foo {
    ///   method() {}
    ///   constructor() {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// class Foo {
    ///   constructor() {}
    ///   method() {}
    /// }
    /// ```
    MemberOrdering,
    typescript,
    style,
    config = MemberOrderingOptions,
);

impl Rule for MemberOrdering {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        let mut rule = serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .map(DefaultRuleConfig::into_inner)?;
        rule.ensure_default_member_types();
        Ok(rule)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Class(class) => {
                let order_config = if class.is_expression() {
                    self.0.class_expressions.as_ref().unwrap_or(&self.0.default)
                } else {
                    self.0.classes.as_ref().unwrap_or(&self.0.default)
                };
                let members =
                    class.body.body.iter().map(MemberRef::Class).collect::<Vec<MemberRef<'_>>>();
                validate_members_order(&members, order_config, true, ctx);
            }
            AstKind::TSInterfaceDeclaration(interface) => {
                let order_config = self.0.interfaces.as_ref().unwrap_or(&self.0.default);
                let members = interface
                    .body
                    .body
                    .iter()
                    .map(MemberRef::Signature)
                    .collect::<Vec<MemberRef<'_>>>();
                validate_members_order(&members, order_config, false, ctx);
            }
            AstKind::TSTypeLiteral(type_literal) => {
                let order_config = self.0.type_literals.as_ref().unwrap_or(&self.0.default);
                let members = type_literal
                    .members
                    .iter()
                    .map(MemberRef::Signature)
                    .collect::<Vec<MemberRef<'_>>>();
                validate_members_order(&members, order_config, false, ctx);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

impl MemberOrdering {
    fn ensure_default_member_types(&mut self) {
        if let OrderConfig::Sorted(config) = &mut self.0.default
            && config.member_types.is_none()
        {
            config.member_types = Some(MemberTypesConfig::Types(default_member_types()));
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum MemberRef<'a> {
    Class(&'a ClassElement<'a>),
    Signature(&'a TSSignature<'a>),
}

impl<'a> MemberRef<'a> {
    fn span(self) -> Span {
        match self {
            Self::Class(member) => member.span(),
            Self::Signature(member) => member.span(),
        }
    }

    fn key_span(self) -> Option<Span> {
        match self {
            Self::Class(member) => match member {
                ClassElement::MethodDefinition(method) => Some(method.key.span()),
                ClassElement::PropertyDefinition(property) => Some(property.key.span()),
                ClassElement::AccessorProperty(accessor) => Some(accessor.key.span()),
                ClassElement::TSIndexSignature(signature) => Some(signature.span),
                ClassElement::StaticBlock(_) => None,
            },
            Self::Signature(member) => match member {
                TSSignature::TSMethodSignature(signature) => Some(signature.key.span()),
                TSSignature::TSPropertySignature(signature) => Some(signature.key.span()),
                TSSignature::TSIndexSignature(signature) => Some(signature.span),
                TSSignature::TSCallSignatureDeclaration(signature) => Some(signature.span),
                TSSignature::TSConstructSignatureDeclaration(signature) => Some(signature.span),
            },
        }
    }

    fn node_type(self) -> MemberKind {
        match self {
            Self::Class(member) => match member {
                ClassElement::MethodDefinition(method) => match method.kind {
                    MethodDefinitionKind::Method => MemberKind::Method,
                    MethodDefinitionKind::Get => MemberKind::Get,
                    MethodDefinitionKind::Set => MemberKind::Set,
                    MethodDefinitionKind::Constructor => MemberKind::Constructor,
                },
                ClassElement::PropertyDefinition(property) => {
                    if property.value.as_ref().is_some_and(|value| {
                        matches!(
                            value.get_inner_expression(),
                            Expression::FunctionExpression(_)
                                | Expression::ArrowFunctionExpression(_)
                        )
                    }) {
                        MemberKind::Method
                    } else if property.readonly {
                        MemberKind::ReadonlyField
                    } else {
                        MemberKind::Field
                    }
                }
                ClassElement::AccessorProperty(_) => MemberKind::Accessor,
                ClassElement::TSIndexSignature(signature) => {
                    if signature.readonly {
                        MemberKind::ReadonlySignature
                    } else {
                        MemberKind::Signature
                    }
                }
                ClassElement::StaticBlock(_) => MemberKind::StaticInitialization,
            },
            Self::Signature(member) => match member {
                TSSignature::TSMethodSignature(signature) => match signature.kind {
                    TSMethodSignatureKind::Method => MemberKind::Method,
                    TSMethodSignatureKind::Get => MemberKind::Get,
                    TSMethodSignatureKind::Set => MemberKind::Set,
                },
                TSSignature::TSCallSignatureDeclaration(_) => MemberKind::CallSignature,
                TSSignature::TSConstructSignatureDeclaration(_) => MemberKind::Constructor,
                TSSignature::TSPropertySignature(signature) => {
                    if signature.readonly {
                        MemberKind::ReadonlyField
                    } else {
                        MemberKind::Field
                    }
                }
                TSSignature::TSIndexSignature(signature) => {
                    if signature.readonly {
                        MemberKind::ReadonlySignature
                    } else {
                        MemberKind::Signature
                    }
                }
            },
        }
    }

    fn name(self, ctx: &LintContext<'a>) -> String {
        match self {
            Self::Class(member) => match member {
                ClassElement::MethodDefinition(method) => {
                    if method.kind == MethodDefinitionKind::Constructor {
                        "constructor".to_string()
                    } else {
                        member_key_name(&method.key, ctx)
                    }
                }
                ClassElement::PropertyDefinition(property) => member_key_name(&property.key, ctx),
                ClassElement::AccessorProperty(accessor) => member_key_name(&accessor.key, ctx),
                ClassElement::TSIndexSignature(signature) => index_signature_name(signature),
                ClassElement::StaticBlock(_) => "static block".to_string(),
            },
            Self::Signature(member) => match member {
                TSSignature::TSMethodSignature(signature) => member_key_name(&signature.key, ctx),
                TSSignature::TSPropertySignature(signature) => member_key_name(&signature.key, ctx),
                TSSignature::TSConstructSignatureDeclaration(_) => "new".to_string(),
                TSSignature::TSCallSignatureDeclaration(_) => "call".to_string(),
                TSSignature::TSIndexSignature(signature) => index_signature_name(signature),
            },
        }
    }

    fn accessibility(self) -> &'static str {
        match self {
            Self::Class(member) => match member {
                ClassElement::MethodDefinition(method) => {
                    if method.key.is_private_identifier() {
                        "#private"
                    } else {
                        method.accessibility.map_or("public", TSAccessibility::as_str)
                    }
                }
                ClassElement::PropertyDefinition(property) => {
                    if property.key.is_private_identifier() {
                        "#private"
                    } else {
                        property.accessibility.map_or("public", TSAccessibility::as_str)
                    }
                }
                ClassElement::AccessorProperty(accessor) => {
                    if accessor.key.is_private_identifier() {
                        "#private"
                    } else {
                        accessor.accessibility.map_or("public", TSAccessibility::as_str)
                    }
                }
                ClassElement::TSIndexSignature(_) | ClassElement::StaticBlock(_) => "public",
            },
            Self::Signature(_) => "public",
        }
    }

    fn is_optional(self) -> bool {
        match self {
            Self::Class(member) => match member {
                ClassElement::MethodDefinition(method) => method.optional,
                ClassElement::PropertyDefinition(property) => property.optional,
                ClassElement::AccessorProperty(_)
                | ClassElement::TSIndexSignature(_)
                | ClassElement::StaticBlock(_) => false,
            },
            Self::Signature(member) => match member {
                TSSignature::TSMethodSignature(signature) => signature.optional,
                TSSignature::TSPropertySignature(signature) => signature.optional,
                TSSignature::TSIndexSignature(_)
                | TSSignature::TSCallSignatureDeclaration(_)
                | TSSignature::TSConstructSignatureDeclaration(_) => false,
            },
        }
    }

    fn is_static(self) -> bool {
        match self {
            Self::Class(member) => match member {
                ClassElement::MethodDefinition(method) => method.r#static,
                ClassElement::PropertyDefinition(property) => property.r#static,
                ClassElement::AccessorProperty(accessor) => accessor.r#static,
                ClassElement::TSIndexSignature(_) | ClassElement::StaticBlock(_) => false,
            },
            Self::Signature(_) => false,
        }
    }

    fn is_abstract(self) -> bool {
        match self {
            Self::Class(member) => match member {
                ClassElement::MethodDefinition(method) => {
                    method.r#type == MethodDefinitionType::TSAbstractMethodDefinition
                }
                ClassElement::PropertyDefinition(property) => property.r#type.is_abstract(),
                ClassElement::AccessorProperty(accessor) => accessor.r#type.is_abstract(),
                ClassElement::TSIndexSignature(_) | ClassElement::StaticBlock(_) => false,
            },
            Self::Signature(_) => false,
        }
    }

    fn has_decorators(self) -> bool {
        match self {
            Self::Class(member) => match member {
                ClassElement::MethodDefinition(method) => !method.decorators.is_empty(),
                ClassElement::PropertyDefinition(property) => !property.decorators.is_empty(),
                ClassElement::AccessorProperty(accessor) => !accessor.decorators.is_empty(),
                ClassElement::TSIndexSignature(_) | ClassElement::StaticBlock(_) => false,
            },
            Self::Signature(_) => false,
        }
    }

    fn is_non_abstract_empty_method(self) -> bool {
        matches!(
            self,
            Self::Class(ClassElement::MethodDefinition(method))
                if method.r#type == MethodDefinitionType::MethodDefinition && method.value.body.is_none()
        )
    }
}

fn member_key_name(key: &PropertyKey<'_>, ctx: &LintContext<'_>) -> String {
    if let Some(name) = key.private_name() {
        return name.to_string();
    }
    if let Some(name) = key.static_name() {
        return name.into_owned();
    }
    key.span().source_text(ctx.source_text()).to_string()
}

fn index_signature_name(signature: &TSIndexSignature<'_>) -> String {
    signature
        .parameters
        .first()
        .map_or_else(|| "(index signature)".to_string(), |parameter| parameter.name.to_string())
}

fn get_rank_order(member_groups: &[String], order_config: &[MemberType]) -> i32 {
    let mut rank = -1;
    let mut stack = member_groups.to_vec();

    while !stack.is_empty() && rank == -1 {
        let member_group = stack.remove(0);
        if let Some(index) =
            order_config.iter().position(|member_type| member_type.includes(member_group.as_str()))
        {
            rank = i32::try_from(index).unwrap_or(i32::MAX);
        }
    }

    rank
}

fn get_rank(member: MemberRef<'_>, order_config: &[MemberType], supports_modifiers: bool) -> i32 {
    if member.is_non_abstract_empty_method() {
        return -1;
    }

    let member_type = member.node_type();

    let mut member_groups: Vec<String> = vec![];

    if supports_modifiers {
        let scope = if member.is_static() {
            "static"
        } else if member.is_abstract() {
            "abstract"
        } else {
            "instance"
        };
        let accessibility = member.accessibility();

        if member.has_decorators() && member_type.supports_decorators() {
            member_groups.push(format!("{accessibility}-decorated-{}", member_type.as_str()));
            member_groups.push(format!("decorated-{}", member_type.as_str()));

            if member_type == MemberKind::ReadonlyField {
                member_groups.push(format!("{accessibility}-decorated-field"));
                member_groups.push("decorated-field".to_string());
            }
        }

        if !matches!(
            member_type,
            MemberKind::ReadonlySignature
                | MemberKind::Signature
                | MemberKind::StaticInitialization
        ) {
            if member_type != MemberKind::Constructor {
                member_groups.push(format!("{accessibility}-{scope}-{}", member_type.as_str()));
                member_groups.push(format!("{scope}-{}", member_type.as_str()));

                if member_type == MemberKind::ReadonlyField {
                    member_groups.push(format!("{accessibility}-{scope}-field"));
                    member_groups.push(format!("{scope}-field"));
                }
            }

            member_groups.push(format!("{accessibility}-{}", member_type.as_str()));
            if member_type == MemberKind::ReadonlyField {
                member_groups.push(format!("{accessibility}-field"));
            }
        }
    }

    member_groups.push(member_type.as_str().to_string());
    if member_type == MemberKind::ReadonlySignature {
        member_groups.push("signature".to_string());
    } else if member_type == MemberKind::ReadonlyField {
        member_groups.push("field".to_string());
    }

    get_rank_order(&member_groups, order_config)
}

fn group_members_by_type<'a>(
    members: &[MemberRef<'a>],
    member_types: &[MemberType],
    supports_modifiers: bool,
) -> Vec<Vec<MemberRef<'a>>> {
    let mut grouped_members: Vec<Vec<MemberRef<'a>>> = vec![];
    let member_ranks = members
        .iter()
        .map(|member| get_rank(*member, member_types, supports_modifiers))
        .collect::<Vec<_>>();
    let mut previous_rank: Option<i32> = None;

    for (index, member) in members.iter().copied().enumerate() {
        if index == members.len().saturating_sub(1) {
            continue;
        }

        let rank_of_current = member_ranks[index];
        let rank_of_next = member_ranks[index + 1];

        if previous_rank == Some(rank_of_current) {
            if let Some(last_group) = grouped_members.last_mut() {
                last_group.push(member);
            }
        } else if rank_of_current == rank_of_next {
            grouped_members.push(vec![member]);
            previous_rank = Some(rank_of_current);
        }
    }

    grouped_members
}

fn get_lowest_rank(ranks: &[i32], target: i32, order: &[MemberType]) -> Option<String> {
    let mut lowest = *ranks.last()?;

    for rank in ranks.iter().copied() {
        if rank > target {
            lowest = lowest.min(rank);
        }
    }

    let lowest = usize::try_from(lowest).ok()?;
    order.get(lowest).map(MemberType::display)
}

fn check_group_sort<'a>(
    members: &[MemberRef<'a>],
    group_order: &[MemberType],
    supports_modifiers: bool,
    ctx: &LintContext<'a>,
) -> Option<Vec<Vec<MemberRef<'a>>>> {
    let mut previous_ranks: Vec<i32> = vec![];
    let mut member_groups: Vec<Vec<MemberRef<'a>>> = vec![];
    let mut is_correctly_sorted = true;

    for member in members.iter().copied() {
        let rank = get_rank(member, group_order, supports_modifiers);
        if rank == -1 {
            continue;
        }

        if let Some(&rank_last_member) = previous_ranks.last() {
            match rank.cmp(&rank_last_member) {
                Ordering::Less => {
                    let name = member.name(ctx);
                    let lowest_rank = get_lowest_rank(&previous_ranks, rank, group_order)
                        .unwrap_or_else(|| "unknown".to_string());
                    ctx.diagnostic(incorrect_group_order_diagnostic(
                        name.as_str(),
                        lowest_rank.as_str(),
                        member.key_span().unwrap_or_else(|| member.span()),
                    ));
                    is_correctly_sorted = false;
                }
                Ordering::Equal => {
                    if let Some(group) = member_groups.last_mut() {
                        group.push(member);
                    }
                }
                Ordering::Greater => {
                    previous_ranks.push(rank);
                    member_groups.push(vec![member]);
                }
            }
        } else {
            previous_ranks.push(rank);
            member_groups.push(vec![member]);
        }
    }

    if is_correctly_sorted { Some(member_groups) } else { None }
}

fn natural_out_of_order(name: &str, previous_name: &str, order: AlphabeticalOrder) -> bool {
    if name == previous_name {
        return false;
    }

    match order {
        AlphabeticalOrder::Alphabetically => name < previous_name,
        AlphabeticalOrder::AlphabeticallyCaseInsensitive => {
            name.cow_to_lowercase() < previous_name.cow_to_lowercase()
        }
        AlphabeticalOrder::Natural => compare(name, previous_name) != Ordering::Greater,
        AlphabeticalOrder::NaturalCaseInsensitive => {
            compare(name.cow_to_lowercase().as_ref(), previous_name.cow_to_lowercase().as_ref())
                != Ordering::Greater
        }
    }
}

fn check_alpha_sort<'a>(
    members: &[MemberRef<'a>],
    order: AlphabeticalOrder,
    ctx: &LintContext<'a>,
) -> bool {
    let mut previous_name = String::new();
    let mut is_correctly_sorted = true;

    for member in members.iter().copied() {
        let name = member.name(ctx);

        if natural_out_of_order(name.as_str(), previous_name.as_str(), order) {
            ctx.diagnostic(incorrect_order_diagnostic(
                name.as_str(),
                previous_name.as_str(),
                member.key_span().unwrap_or_else(|| member.span()),
            ));
            is_correctly_sorted = false;
        }

        previous_name = name;
    }

    is_correctly_sorted
}

fn check_required_order<'a>(
    members: &[MemberRef<'a>],
    optionality_order: OptionalityOrder,
    ctx: &LintContext<'a>,
) -> bool {
    if members.is_empty() {
        return true;
    }

    let switch_index = members
        .iter()
        .enumerate()
        .find_map(|(index, member)| {
            if index == 0 || member.is_optional() == members[index - 1].is_optional() {
                None
            } else {
                Some(index)
            }
        })
        .unwrap_or(0);

    let expected_optional = optionality_order == OptionalityOrder::OptionalFirst;
    if members[0].is_optional() != expected_optional {
        let member_name = members[0].name(ctx);
        let optional_or_required = if optionality_order == OptionalityOrder::RequiredFirst {
            "required"
        } else {
            "optional"
        };
        ctx.diagnostic(incorrect_required_members_order_diagnostic(
            member_name.as_str(),
            optional_or_required,
            members[0].key_span().unwrap_or_else(|| members[0].span()),
        ));
        return false;
    }

    for index in (switch_index + 1)..members.len() {
        if members[index].is_optional() != members[switch_index].is_optional() {
            let member_name = members[switch_index].name(ctx);
            let optional_or_required = if optionality_order == OptionalityOrder::RequiredFirst {
                "required"
            } else {
                "optional"
            };
            ctx.diagnostic(incorrect_required_members_order_diagnostic(
                member_name.as_str(),
                optional_or_required,
                members[switch_index].key_span().unwrap_or_else(|| members[switch_index].span()),
            ));
            return false;
        }
    }

    true
}

fn validate_members_order<'a>(
    members: &[MemberRef<'a>],
    order_config: &OrderConfig,
    supports_modifiers: bool,
    ctx: &LintContext<'a>,
) {
    if matches!(order_config, OrderConfig::Never(NeverConfig::Never)) {
        return;
    }

    let mut order: Option<Order> = None;
    let mut member_types: Option<MemberTypesConfig> = None;
    let mut optionality_order: Option<OptionalityOrder> = None;

    match order_config {
        OrderConfig::Types(types) => {
            member_types = Some(MemberTypesConfig::Types(types.clone()));
        }
        OrderConfig::Sorted(config) => {
            order = config.order;
            member_types.clone_from(&config.member_types);
            optionality_order = config.optionality_order;
        }
        OrderConfig::Never(NeverConfig::Never) => {}
    }

    let check_alpha_sort_for_all_members =
        |member_set: &[MemberRef<'a>],
         member_types: &Option<MemberTypesConfig>,
         order: Option<Order>,
         ctx: &LintContext<'a>| {
            let has_alpha_sort = order.and_then(Order::as_alphabetical);
            if let (Some(alpha_order), Some(MemberTypesConfig::Types(configured_types))) =
                (has_alpha_sort, member_types)
            {
                for grouped_members in
                    group_members_by_type(member_set, configured_types, supports_modifiers)
                {
                    let _ = check_alpha_sort(&grouped_members, alpha_order, ctx);
                }
            }
        };

    let check_order = |member_set: &[MemberRef<'a>],
                       member_types: &Option<MemberTypesConfig>,
                       order: Option<Order>,
                       ctx: &LintContext<'a>| {
        let has_alpha_sort = order.and_then(Order::as_alphabetical);

        if let Some(MemberTypesConfig::Types(configured_types)) = member_types {
            let grouped = check_group_sort(member_set, configured_types, supports_modifiers, ctx);
            if grouped.is_none() {
                check_alpha_sort_for_all_members(member_set, member_types, order, ctx);
                return false;
            }

            if let (Some(alpha_order), Some(grouped_members)) = (has_alpha_sort, grouped) {
                for members in grouped_members {
                    let _ = check_alpha_sort(&members, alpha_order, ctx);
                }
            }
        } else if let Some(alpha_order) = has_alpha_sort {
            return check_alpha_sort(member_set, alpha_order, ctx);
        }

        false
    };

    if optionality_order.is_none() {
        let _ = check_order(members, &member_types, order, ctx);
        return;
    }

    let switch_index = members.iter().enumerate().find_map(|(index, member)| {
        if index == 0 || member.is_optional() == members[index - 1].is_optional() {
            None
        } else {
            Some(index)
        }
    });

    if let Some(switch_index) = switch_index {
        if !check_required_order(
            members,
            optionality_order.unwrap_or(OptionalityOrder::RequiredFirst),
            ctx,
        ) {
            return;
        }

        let _ = check_order(&members[..switch_index], &member_types, order, ctx);
        let _ = check_order(&members[switch_index..], &member_types, order, ctx);
    } else {
        let _ = check_order(members, &member_types, order, ctx);
    }
}

fn default_member_types() -> Vec<MemberType> {
    [
        "signature",
        "call-signature",
        "public-static-field",
        "protected-static-field",
        "private-static-field",
        "#private-static-field",
        "public-decorated-field",
        "protected-decorated-field",
        "private-decorated-field",
        "public-instance-field",
        "protected-instance-field",
        "private-instance-field",
        "#private-instance-field",
        "public-abstract-field",
        "protected-abstract-field",
        "public-field",
        "protected-field",
        "private-field",
        "#private-field",
        "static-field",
        "instance-field",
        "abstract-field",
        "decorated-field",
        "field",
        "static-initialization",
        "public-constructor",
        "protected-constructor",
        "private-constructor",
        "constructor",
        "public-static-accessor",
        "protected-static-accessor",
        "private-static-accessor",
        "#private-static-accessor",
        "public-decorated-accessor",
        "protected-decorated-accessor",
        "private-decorated-accessor",
        "public-instance-accessor",
        "protected-instance-accessor",
        "private-instance-accessor",
        "#private-instance-accessor",
        "public-abstract-accessor",
        "protected-abstract-accessor",
        "public-accessor",
        "protected-accessor",
        "private-accessor",
        "#private-accessor",
        "static-accessor",
        "instance-accessor",
        "abstract-accessor",
        "decorated-accessor",
        "accessor",
        "public-static-get",
        "protected-static-get",
        "private-static-get",
        "#private-static-get",
        "public-decorated-get",
        "protected-decorated-get",
        "private-decorated-get",
        "public-instance-get",
        "protected-instance-get",
        "private-instance-get",
        "#private-instance-get",
        "public-abstract-get",
        "protected-abstract-get",
        "public-get",
        "protected-get",
        "private-get",
        "#private-get",
        "static-get",
        "instance-get",
        "abstract-get",
        "decorated-get",
        "get",
        "public-static-set",
        "protected-static-set",
        "private-static-set",
        "#private-static-set",
        "public-decorated-set",
        "protected-decorated-set",
        "private-decorated-set",
        "public-instance-set",
        "protected-instance-set",
        "private-instance-set",
        "#private-instance-set",
        "public-abstract-set",
        "protected-abstract-set",
        "public-set",
        "protected-set",
        "private-set",
        "#private-set",
        "static-set",
        "instance-set",
        "abstract-set",
        "decorated-set",
        "set",
        "public-static-method",
        "protected-static-method",
        "private-static-method",
        "#private-static-method",
        "public-decorated-method",
        "protected-decorated-method",
        "private-decorated-method",
        "public-instance-method",
        "protected-instance-method",
        "private-instance-method",
        "#private-instance-method",
        "public-abstract-method",
        "protected-abstract-method",
        "public-method",
        "protected-method",
        "private-method",
        "#private-method",
        "static-method",
        "instance-method",
        "abstract-method",
        "decorated-method",
        "method",
    ]
    .into_iter()
    .map(|member_type| MemberType::Single(member_type.to_string()))
    .collect()
}

#[cfg(test)]
#[derive(Deserialize)]
#[serde(untagged)]
enum UpstreamValidCase {
    Code(String),
    Config { code: String, options: Option<serde_json::Value> },
}

#[cfg(test)]
#[derive(Deserialize)]
struct UpstreamInvalidCase {
    code: String,
    options: Option<serde_json::Value>,
}

#[cfg(test)]
#[derive(Deserialize)]
struct UpstreamCases {
    valid: Vec<UpstreamValidCase>,
    invalid: Vec<UpstreamInvalidCase>,
}

#[cfg(test)]
fn normalize_setters_for_oxc_parser(code: &str) -> String {
    code.cow_replace("set B() {}", "set B(value) {}")
        .cow_replace("set C() {}", "set C(value) {}")
        .cow_replace("set D() {}", "set D(value) {}")
        .cow_replace("set F() {}", "set F(value) {}")
        .into_owned()
}

#[test]
fn test() {
    use crate::tester::{TestCase, Tester};

    struct JsonCase(String, Option<serde_json::Value>);

    impl From<JsonCase> for TestCase {
        fn from(value: JsonCase) -> Self {
            TestCase::from((value.0.as_str(), value.1))
        }
    }

    let cases = serde_json::from_str::<UpstreamCases>(include_str!("member_ordering.tests.json"))
        .expect("member_ordering.tests.json should be valid");

    let pass = cases
        .valid
        .into_iter()
        .map(|case| match case {
            UpstreamValidCase::Code(code) => {
                JsonCase(normalize_setters_for_oxc_parser(&code), None)
            }
            UpstreamValidCase::Config { code, options } => {
                JsonCase(normalize_setters_for_oxc_parser(&code), options)
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(pass.len(), 103, "upstream valid test count changed");

    let fail = cases
        .invalid
        .into_iter()
        .map(|case| JsonCase(normalize_setters_for_oxc_parser(&case.code), case.options))
        .collect::<Vec<_>>();
    assert_eq!(fail.len(), 73, "upstream invalid test count changed");

    Tester::new(MemberOrdering::NAME, MemberOrdering::PLUGIN, pass, fail).test_and_snapshot();
}
