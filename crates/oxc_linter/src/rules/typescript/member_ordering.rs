use std::borrow::Cow;

use cow_utils::CowUtils;

use oxc_ast::{
    AstKind,
    ast::{
        AccessorPropertyType, ClassElement, ClassType, Expression, FunctionType,
        MethodDefinitionKind, MethodDefinitionType, PropertyDefinition, PropertyDefinitionType,
        PropertyKey, TSAccessibility, TSIndexSignature, TSMethodSignatureKind, TSSignature,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use serde_json::Value;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn incorrect_group_order_diagnostic(name: &str, rank: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Member {name} should be declared before all {rank} definitions."))
        .with_label(span)
}

fn incorrect_order_diagnostic(member: &str, before_member: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Member {member} should be declared before member {before_member}."
    ))
    .with_label(span)
}

fn incorrect_required_members_order_diagnostic(
    member: &str,
    optional_or_required: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Member {member} should be declared after all {optional_or_required} members."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct MemberOrdering(Box<MemberOrderingConfig>);

impl std::ops::Deref for MemberOrdering {
    type Target = MemberOrderingConfig;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct MemberOrderingConfig {
    classes: Option<OrderConfig>,
    class_expressions: Option<OrderConfig>,
    default: Option<OrderConfig>,
    interfaces: Option<OrderConfig>,
    type_literals: Option<OrderConfig>,
}

impl Default for MemberOrderingConfig {
    fn default() -> Self {
        Self {
            classes: None,
            class_expressions: None,
            default: Some(OrderConfig::Sorted {
                member_types: SortedMemberTypes::Array(default_order()),
                order: None,
                optionality_order: None,
            }),
            interfaces: None,
            type_literals: None,
        }
    }
}

#[derive(Debug, Clone)]
enum OrderConfig {
    Never,
    Array(Vec<MemberTypeEntry>),
    Sorted {
        member_types: SortedMemberTypes,
        order: Option<Order>,
        optionality_order: Option<OptionalityOrder>,
    },
}

#[derive(Debug, Clone)]
enum SortedMemberTypes {
    Unset,
    Never,
    Array(Vec<MemberTypeEntry>),
}

#[derive(Debug, Clone)]
enum MemberTypeEntry {
    Single(String),
    Group(Vec<String>),
}

impl MemberTypeEntry {
    fn matches(&self, name: &str) -> bool {
        match self {
            Self::Single(s) => s == name,
            Self::Group(items) => items.iter().any(|s| s == name),
        }
    }

    fn as_display(&self) -> String {
        match self {
            Self::Single(s) => s.cow_replace('-', " ").into_owned(),
            Self::Group(items) => items
                .iter()
                .map(|s| s.cow_replace('-', " ").into_owned())
                .collect::<Vec<_>>()
                .join(", "),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Order {
    AsWritten,
    Alphabetically,
    AlphabeticallyCaseInsensitive,
    Natural,
    NaturalCaseInsensitive,
}

impl Order {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "as-written" => Some(Self::AsWritten),
            "alphabetically" => Some(Self::Alphabetically),
            "alphabetically-case-insensitive" => Some(Self::AlphabeticallyCaseInsensitive),
            "natural" => Some(Self::Natural),
            "natural-case-insensitive" => Some(Self::NaturalCaseInsensitive),
            _ => None,
        }
    }

    fn is_alpha(self) -> bool {
        !matches!(self, Self::AsWritten)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OptionalityOrder {
    OptionalFirst,
    RequiredFirst,
}

impl OptionalityOrder {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "optional-first" => Some(Self::OptionalFirst),
            "required-first" => Some(Self::RequiredFirst),
            _ => None,
        }
    }
}

fn parse_order_array(value: &serde_json::Value) -> Vec<MemberTypeEntry> {
    let Some(arr) = value.as_array() else { return Vec::new() };
    arr.iter()
        .filter_map(|item| match item {
            serde_json::Value::String(s) => Some(MemberTypeEntry::Single(s.clone())),
            serde_json::Value::Array(group) => {
                let items: Vec<String> =
                    group.iter().filter_map(|v| v.as_str().map(String::from)).collect();
                if items.is_empty() { None } else { Some(MemberTypeEntry::Group(items)) }
            }
            _ => None,
        })
        .collect()
}

fn parse_order_config(value: &serde_json::Value) -> Option<OrderConfig> {
    match value {
        serde_json::Value::String(s) if s == "never" => Some(OrderConfig::Never),
        serde_json::Value::Array(_) => Some(OrderConfig::Array(parse_order_array(value))),
        serde_json::Value::Object(obj) => {
            let member_types = match obj.get("memberTypes") {
                Some(serde_json::Value::String(s)) if s == "never" => SortedMemberTypes::Never,
                Some(v @ serde_json::Value::Array(_)) => {
                    SortedMemberTypes::Array(parse_order_array(v))
                }
                None => SortedMemberTypes::Array(default_order()),
                _ => SortedMemberTypes::Unset,
            };
            let order = obj.get("order").and_then(|v| v.as_str()).and_then(Order::from_str);
            let optionality_order = obj
                .get("optionalityOrder")
                .and_then(|v| v.as_str())
                .and_then(OptionalityOrder::from_str);
            Some(OrderConfig::Sorted { member_types, order, optionality_order })
        }
        _ => None,
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require a consistent member declaration order.
    ///
    /// ### Why is this bad?
    ///
    /// A consistent ordering of fields, methods and constructors can make
    /// classes, interfaces, and type literals easier to read, navigate, and edit.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// interface Foo {
    ///   B(): void;
    ///   A: string;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// interface Foo {
    ///   A: string;
    ///   B(): void;
    /// }
    /// ```
    MemberOrdering,
    typescript,
    style,
    config = Value,
    version = "next",
);

impl Rule for MemberOrdering {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let mut cfg = MemberOrderingConfig::default();
        if let Some(obj) = value.get(0).and_then(|v| v.as_object()) {
            if let Some(v) = obj.get("default") {
                cfg.default = parse_order_config(v);
            }
            if let Some(v) = obj.get("classes") {
                cfg.classes = parse_order_config(v);
            }
            if let Some(v) = obj.get("classExpressions") {
                cfg.class_expressions = parse_order_config(v);
            }
            if let Some(v) = obj.get("interfaces") {
                cfg.interfaces = parse_order_config(v);
            }
            if let Some(v) = obj.get("typeLiterals") {
                cfg.type_literals = parse_order_config(v);
            }
        }
        Ok(Self(Box::new(cfg)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Class(class) => {
                let order = if class.r#type == ClassType::ClassExpression {
                    self.class_expressions.as_ref().or(self.default.as_ref())
                } else {
                    self.classes.as_ref().or(self.default.as_ref())
                };
                if let Some(order) = order {
                    let members: Vec<MemberRef<'a, '_>> =
                        class.body.body.iter().map(MemberRef::from_class_element).collect();
                    validate_members_order(&members, order, true, ctx);
                }
            }
            AstKind::TSInterfaceDeclaration(decl) => {
                if let Some(order) = self.interfaces.as_ref().or(self.default.as_ref()) {
                    let members: Vec<MemberRef<'a, '_>> =
                        decl.body.body.iter().map(MemberRef::from_signature).collect();
                    validate_members_order(&members, order, false, ctx);
                }
            }
            AstKind::TSTypeLiteral(literal) => {
                if let Some(order) = self.type_literals.as_ref().or(self.default.as_ref()) {
                    let members: Vec<MemberRef<'a, '_>> =
                        literal.members.iter().map(MemberRef::from_signature).collect();
                    validate_members_order(&members, order, false, ctx);
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        // Class members work in JS too, but the rule's main value is in TS.
        // Still allow it to run on any source type since classes exist in JS.
        let _ = ctx;
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    Static,
    Instance,
    Abstract,
}

impl Scope {
    fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Instance => "instance",
            Self::Abstract => "abstract",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Accessibility {
    Public,
    Protected,
    Private,
    HashPrivate,
}

impl Accessibility {
    fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Protected => "protected",
            Self::Private => "private",
            Self::HashPrivate => "#private",
        }
    }
}

enum MemberRef<'a, 'b> {
    Class(&'b ClassElement<'a>),
    Signature(&'b TSSignature<'a>),
}

impl<'a, 'b> MemberRef<'a, 'b> {
    fn from_class_element(el: &'b ClassElement<'a>) -> Self {
        Self::Class(el)
    }
    fn from_signature(sig: &'b TSSignature<'a>) -> Self {
        Self::Signature(sig)
    }

    fn span(&self) -> Span {
        match self {
            Self::Class(el) => el.span(),
            Self::Signature(s) => s.span(),
        }
    }

    fn kind(&self) -> MemberKind {
        match self {
            Self::Class(el) => match el {
                ClassElement::MethodDefinition(m) => method_kind_to_member_kind(m.kind),
                ClassElement::PropertyDefinition(p) => property_def_kind(p),
                ClassElement::AccessorProperty(_) => MemberKind::Accessor,
                ClassElement::TSIndexSignature(_) => MemberKind::Signature,
                ClassElement::StaticBlock(_) => MemberKind::StaticInitialization,
            },
            Self::Signature(sig) => match sig {
                TSSignature::TSPropertySignature(_) => MemberKind::Field,
                TSSignature::TSMethodSignature(m) => match m.kind {
                    TSMethodSignatureKind::Method => MemberKind::Method,
                    TSMethodSignatureKind::Get => MemberKind::Get,
                    TSMethodSignatureKind::Set => MemberKind::Set,
                },
                TSSignature::TSCallSignatureDeclaration(_) => MemberKind::CallSignature,
                TSSignature::TSConstructSignatureDeclaration(_) => MemberKind::Constructor,
                TSSignature::TSIndexSignature(_) => MemberKind::Signature,
            },
        }
    }

    fn is_abstract(&self) -> bool {
        match self {
            Self::Class(el) => match el {
                ClassElement::MethodDefinition(m) => {
                    m.r#type == MethodDefinitionType::TSAbstractMethodDefinition
                }
                ClassElement::PropertyDefinition(p) => {
                    p.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition
                }
                ClassElement::AccessorProperty(p) => {
                    p.r#type == AccessorPropertyType::TSAbstractAccessorProperty
                }
                _ => false,
            },
            Self::Signature(_) => false,
        }
    }

    fn is_static(&self) -> bool {
        match self {
            Self::Class(el) => match el {
                ClassElement::MethodDefinition(m) => m.r#static,
                ClassElement::PropertyDefinition(p) => p.r#static,
                ClassElement::AccessorProperty(p) => p.r#static,
                ClassElement::TSIndexSignature(s) => s.r#static,
                ClassElement::StaticBlock(_) => true,
            },
            Self::Signature(_) => false,
        }
    }

    fn decorators_count(&self) -> usize {
        match self {
            Self::Class(el) => match el {
                ClassElement::MethodDefinition(m) => m.decorators.len(),
                ClassElement::PropertyDefinition(p) => p.decorators.len(),
                ClassElement::AccessorProperty(p) => p.decorators.len(),
                _ => 0,
            },
            Self::Signature(_) => 0,
        }
    }

    fn accessibility(&self) -> Accessibility {
        match self {
            Self::Class(el) => match el {
                ClassElement::MethodDefinition(m) => {
                    accessibility_from_class(m.accessibility, &m.key)
                }
                ClassElement::PropertyDefinition(p) => {
                    accessibility_from_class(p.accessibility, &p.key)
                }
                ClassElement::AccessorProperty(p) => {
                    accessibility_from_class(p.accessibility, &p.key)
                }
                _ => Accessibility::Public,
            },
            Self::Signature(_) => Accessibility::Public,
        }
    }

    fn is_optional(&self) -> bool {
        match self {
            Self::Class(el) => match el {
                ClassElement::MethodDefinition(m) => m.optional,
                ClassElement::PropertyDefinition(p) => p.optional,
                _ => false,
            },
            Self::Signature(sig) => match sig {
                TSSignature::TSPropertySignature(p) => p.optional,
                TSSignature::TSMethodSignature(m) => m.optional,
                _ => false,
            },
        }
    }

    fn is_readonly(&self) -> bool {
        match self {
            Self::Class(el) => match el {
                ClassElement::PropertyDefinition(p) => {
                    p.readonly && property_def_kind(p) == MemberKind::Field
                }
                ClassElement::TSIndexSignature(s) => s.readonly,
                _ => false,
            },
            Self::Signature(sig) => match sig {
                TSSignature::TSPropertySignature(p) => p.readonly,
                TSSignature::TSIndexSignature(s) => s.readonly,
                _ => false,
            },
        }
    }

    fn is_overload_signature(&self) -> bool {
        match self {
            Self::Class(ClassElement::MethodDefinition(m)) => {
                m.r#type != MethodDefinitionType::TSAbstractMethodDefinition
                    && m.value.r#type == FunctionType::TSEmptyBodyFunctionExpression
            }
            _ => false,
        }
    }

    fn name(&self, ctx: &LintContext<'_>) -> String {
        match self {
            Self::Class(el) => match el {
                ClassElement::MethodDefinition(m) => {
                    if m.kind == MethodDefinitionKind::Constructor {
                        "constructor".to_string()
                    } else {
                        property_key_name(&m.key, ctx)
                    }
                }
                ClassElement::PropertyDefinition(p) => property_key_name(&p.key, ctx),
                ClassElement::AccessorProperty(p) => property_key_name(&p.key, ctx),
                ClassElement::TSIndexSignature(s) => index_signature_name(s),
                ClassElement::StaticBlock(_) => "static block".to_string(),
            },
            Self::Signature(sig) => match sig {
                TSSignature::TSPropertySignature(p) => property_key_name(&p.key, ctx),
                TSSignature::TSMethodSignature(m) => property_key_name(&m.key, ctx),
                TSSignature::TSCallSignatureDeclaration(_) => "call".to_string(),
                TSSignature::TSConstructSignatureDeclaration(_) => "new".to_string(),
                TSSignature::TSIndexSignature(s) => index_signature_name(s),
            },
        }
    }
}

fn method_kind_to_member_kind(kind: MethodDefinitionKind) -> MemberKind {
    match kind {
        MethodDefinitionKind::Constructor => MemberKind::Constructor,
        MethodDefinitionKind::Method => MemberKind::Method,
        MethodDefinitionKind::Get => MemberKind::Get,
        MethodDefinitionKind::Set => MemberKind::Set,
    }
}

fn property_def_kind(p: &PropertyDefinition<'_>) -> MemberKind {
    match &p.value {
        Some(Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)) => {
            MemberKind::Method
        }
        _ => MemberKind::Field,
    }
}

fn accessibility_from_class(
    accessibility: Option<TSAccessibility>,
    key: &PropertyKey<'_>,
) -> Accessibility {
    if let Some(a) = accessibility {
        return match a {
            TSAccessibility::Public => Accessibility::Public,
            TSAccessibility::Protected => Accessibility::Protected,
            TSAccessibility::Private => Accessibility::Private,
        };
    }
    if key.is_private_identifier() {
        return Accessibility::HashPrivate;
    }
    Accessibility::Public
}

fn property_key_name(key: &PropertyKey<'_>, ctx: &LintContext<'_>) -> String {
    if let Some(name) = key.name() {
        match name {
            Cow::Borrowed(s) => s.to_string(),
            Cow::Owned(s) => s,
        }
    } else {
        ctx.source_range(key.span()).to_string()
    }
}

fn index_signature_name(sig: &TSIndexSignature<'_>) -> String {
    sig.parameters.first().map_or_else(|| "index".to_string(), |p| p.name.as_str().to_string())
}

fn member_kind_name(kind: MemberKind, readonly: bool) -> &'static str {
    match (kind, readonly) {
        (MemberKind::Field, true) => "readonly-field",
        (MemberKind::Signature, true) => "readonly-signature",
        _ => kind.as_str(),
    }
}

fn collect_member_groups(member: &MemberRef<'_, '_>, supports_modifiers: bool) -> Vec<String> {
    let mut groups: Vec<String> = Vec::new();
    let kind = member.kind();
    let readonly = member.is_readonly();

    let abstract_ = member.is_abstract();
    let scope = if member.is_static() {
        Scope::Static
    } else if abstract_ {
        Scope::Abstract
    } else {
        Scope::Instance
    };
    let access = member.accessibility();
    let specific_kind_str = member_kind_name(kind, readonly);

    if supports_modifiers {
        let decorated = member.decorators_count() > 0;
        if decorated
            && matches!(
                kind,
                MemberKind::Field
                    | MemberKind::Method
                    | MemberKind::Accessor
                    | MemberKind::Get
                    | MemberKind::Set
            )
        {
            groups.push(format!("{}-decorated-{}", access.as_str(), specific_kind_str));
            groups.push(format!("decorated-{specific_kind_str}"));
            if readonly {
                groups.push(format!("{}-decorated-field", access.as_str()));
                groups.push("decorated-field".to_string());
            }
        }

        if !matches!(kind, MemberKind::Signature | MemberKind::StaticInitialization) {
            if kind != MemberKind::Constructor {
                groups.push(format!(
                    "{}-{}-{}",
                    access.as_str(),
                    scope.as_str(),
                    specific_kind_str
                ));
                groups.push(format!("{}-{specific_kind_str}", scope.as_str()));

                if readonly {
                    groups.push(format!("{}-{}-field", access.as_str(), scope.as_str()));
                    groups.push(format!("{}-field", scope.as_str()));
                }
            }
            groups.push(format!("{}-{specific_kind_str}", access.as_str()));
            if readonly {
                groups.push(format!("{}-field", access.as_str()));
            }
        }
    }

    groups.push(specific_kind_str.to_string());
    if readonly && kind == MemberKind::Signature {
        groups.push("signature".to_string());
    } else if readonly && kind == MemberKind::Field {
        groups.push("field".to_string());
    }

    groups
}

/// Returns `None` if the member should be skipped because it is either an
/// overload signature or not part of this order's configured member types.
fn get_rank(
    member: &MemberRef<'_, '_>,
    order: &[MemberTypeEntry],
    supports_modifiers: bool,
) -> Option<usize> {
    if member.is_overload_signature() {
        return None;
    }
    let groups = collect_member_groups(member, supports_modifiers);
    for group in &groups {
        if let Some(idx) = order.iter().position(|entry| entry.matches(group)) {
            return Some(idx);
        }
    }
    None
}

fn get_lowest_rank(ranks: &[usize], target: usize, order: &[MemberTypeEntry]) -> String {
    ranks
        .iter()
        .copied()
        .filter(|rank| *rank > target)
        .min()
        .and_then(|rank| order.get(rank))
        .map_or_else(String::new, MemberTypeEntry::as_display)
}

fn validate_members_order<'a>(
    members: &[MemberRef<'a, '_>],
    order_config: &OrderConfig,
    supports_modifiers: bool,
    ctx: &LintContext<'a>,
) {
    let (member_types, order, optionality_order) = match order_config {
        OrderConfig::Never => return,
        OrderConfig::Array(arr) => (Some(arr.as_slice()), None, None),
        OrderConfig::Sorted { member_types, order, optionality_order } => match member_types {
            SortedMemberTypes::Unset | SortedMemberTypes::Never => {
                (None, *order, *optionality_order)
            }
            SortedMemberTypes::Array(arr) => (Some(arr.as_slice()), *order, *optionality_order),
        },
    };

    if members.is_empty() {
        return;
    }

    check_order(members, member_types, order, optionality_order, supports_modifiers, ctx);
}

fn check_required_order<'a>(
    members: &[&MemberRef<'a, '_>],
    optionality_order: OptionalityOrder,
    ctx: &LintContext<'a>,
) -> bool {
    let optional_first = optionality_order == OptionalityOrder::OptionalFirst;
    let label = if optional_first { "optional" } else { "required" };

    let Some(switch_idx) = optionality_switch_index(members) else { return true };

    if members[0].is_optional() != optional_first {
        let name = members[0].name(ctx);
        ctx.diagnostic(incorrect_required_members_order_diagnostic(
            &name,
            label,
            members[0].span(),
        ));
        return false;
    }

    for i in (switch_idx + 1)..members.len() {
        if members[i].is_optional() != members[switch_idx].is_optional() {
            let name = members[switch_idx].name(ctx);
            ctx.diagnostic(incorrect_required_members_order_diagnostic(
                &name,
                label,
                members[switch_idx].span(),
            ));
            return false;
        }
    }

    true
}

fn optionality_switch_index(members: &[&MemberRef<'_, '_>]) -> Option<usize> {
    members
        .iter()
        .enumerate()
        .skip(1)
        .find(|(i, member)| member.is_optional() != members[i - 1].is_optional())
        .map(|(i, _)| i)
}

fn check_order<'a>(
    members: &[MemberRef<'a, '_>],
    member_types: Option<&[MemberTypeEntry]>,
    order: Option<Order>,
    optionality_order: Option<OptionalityOrder>,
    supports_modifiers: bool,
    ctx: &LintContext<'a>,
) -> bool {
    let has_alpha_sort = order.is_some_and(Order::is_alpha);

    let (groups, group_order_valid) = if let Some(member_types) = member_types {
        collect_ranked_groups(members, member_types, supports_modifiers, ctx)
    } else {
        (vec![members.iter().collect()], true)
    };

    let mut valid = group_order_valid;
    for group in &groups {
        valid &= check_group_optionality_and_sort(
            group,
            optionality_order,
            if has_alpha_sort { order } else { None },
            ctx,
        );
    }

    valid
}

fn collect_ranked_groups<'a, 'b>(
    members: &'b [MemberRef<'a, '_>],
    member_types: &[MemberTypeEntry],
    supports_modifiers: bool,
    ctx: &LintContext<'a>,
) -> (Vec<Vec<&'b MemberRef<'a, 'b>>>, bool) {
    let mut previous_ranks: Vec<usize> = Vec::new();
    let mut groups: Vec<Vec<&MemberRef<'_, '_>>> = Vec::new();
    let mut current_group_rank: Option<usize> = None;
    let mut correctly_sorted = true;

    for member in members {
        let Some(rank) = get_rank(member, member_types, supports_modifiers) else {
            continue;
        };
        let last_rank = previous_ranks.last().copied();

        if last_rank.is_some_and(|last| rank < last) {
            let name = member.name(ctx);
            let rank_label = get_lowest_rank(&previous_ranks, rank, member_types);
            ctx.diagnostic(incorrect_group_order_diagnostic(&name, &rank_label, member.span()));
            correctly_sorted = false;
        } else if last_rank != Some(rank) {
            previous_ranks.push(rank);
        }

        if current_group_rank == Some(rank) {
            if let Some(group) = groups.last_mut() {
                group.push(member);
            }
        } else {
            groups.push(vec![member]);
            current_group_rank = Some(rank);
        }
    }

    (groups, correctly_sorted)
}

fn check_group_optionality_and_sort<'a>(
    members: &[&MemberRef<'a, '_>],
    optionality_order: Option<OptionalityOrder>,
    order: Option<Order>,
    ctx: &LintContext<'a>,
) -> bool {
    if members.is_empty() {
        return true;
    }

    let Some(order) = order else {
        if let Some(optionality_order) = optionality_order {
            return check_required_order(members, optionality_order, ctx);
        }
        return true;
    };

    let Some(optionality_order) = optionality_order else {
        return check_alpha_sort(members, order, ctx);
    };

    if !check_required_order(members, optionality_order, ctx) {
        return false;
    }

    if let Some(switch_idx) = optionality_switch_index(members) {
        check_alpha_sort(&members[..switch_idx], order, ctx)
            & check_alpha_sort(&members[switch_idx..], order, ctx)
    } else {
        check_alpha_sort(members, order, ctx)
    }
}

fn check_alpha_sort<'a>(
    members: &[&MemberRef<'a, '_>],
    order: Order,
    ctx: &LintContext<'a>,
) -> bool {
    let mut previous: Option<String> = None;
    let mut sorted = true;
    for m in members {
        let name = m.name(ctx);
        if let Some(prev) = previous.as_deref()
            && natural_out_of_order(&name, prev, order)
        {
            ctx.diagnostic(incorrect_order_diagnostic(&name, prev, m.span()));
            sorted = false;
        }
        previous = Some(name);
    }
    sorted
}

fn natural_out_of_order(name: &str, previous: &str, order: Order) -> bool {
    if name == previous {
        return false;
    }
    match order {
        Order::AsWritten => false,
        Order::Alphabetically => name < previous,
        Order::AlphabeticallyCaseInsensitive => {
            name.cow_to_ascii_lowercase() < previous.cow_to_ascii_lowercase()
        }
        Order::Natural => natural_compare(name, previous) != std::cmp::Ordering::Greater,
        Order::NaturalCaseInsensitive => {
            natural_compare(&name.cow_to_ascii_lowercase(), &previous.cow_to_ascii_lowercase())
                != std::cmp::Ordering::Greater
        }
    }
}

/// A simple natural compare that handles digit runs as numbers.
fn natural_compare(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let mut ai = a.chars().peekable();
    let mut bi = b.chars().peekable();
    loop {
        match (ai.peek(), bi.peek()) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(ac), Some(bc)) => {
                if ac.is_ascii_digit() && bc.is_ascii_digit() {
                    let mut a_num = String::new();
                    while let Some(&c) = ai.peek() {
                        if !c.is_ascii_digit() {
                            break;
                        }
                        a_num.push(c);
                        ai.next();
                    }
                    let mut b_num = String::new();
                    while let Some(&c) = bi.peek() {
                        if !c.is_ascii_digit() {
                            break;
                        }
                        b_num.push(c);
                        bi.next();
                    }
                    let cmp = compare_digit_runs(&a_num, &b_num);
                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                } else {
                    let ord = ac.cmp(bc);
                    if ord != Ordering::Equal {
                        return ord;
                    }
                    ai.next();
                    bi.next();
                }
            }
        }
    }
}

fn compare_digit_runs(a: &str, b: &str) -> std::cmp::Ordering {
    let a_significant = a.trim_start_matches('0');
    let b_significant = b.trim_start_matches('0');
    let a_significant = if a_significant.is_empty() { "0" } else { a_significant };
    let b_significant = if b_significant.is_empty() { "0" } else { b_significant };

    a_significant
        .len()
        .cmp(&b_significant.len())
        .then_with(|| a_significant.cmp(b_significant))
        .then_with(|| a.cmp(b))
}

fn default_order() -> Vec<MemberTypeEntry> {
    [
        // Index signature
        "signature",
        "call-signature",
        // Fields
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
        // Static initialization
        "static-initialization",
        // Constructors
        "public-constructor",
        "protected-constructor",
        "private-constructor",
        "constructor",
        // Accessors
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
        // Getters
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
        // Setters
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
        // Methods
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
    .iter()
    .map(|s| MemberTypeEntry::Single((*s).to_string()))
    .collect()
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass: Vec<(&str, Option<serde_json::Value>)> = vec![
        // Default config: passes for ordered interface
        ("interface Foo { [Z: string]: any; A: string; B(): void; }", None),
        // Class default order
        (
            "class Foo {
                public static a: string;
                protected static b: string;
                private static c: string;
                public d: string;
                constructor() {}
                public e(): void {}
            }",
            None,
        ),
        // never disables ordering
        ("interface Foo { B(): void; A: string; }", Some(json!([{ "default": "never" }]))),
        // Custom interface order
        (
            "interface Foo { method(): void; new(): void; field: number; [k: string]: any; }",
            Some(json!([{ "interfaces": ["method", "constructor", "field", "signature"] }])),
        ),
        // Decorated field comes before regular field
        (
            "class Foo { @Dec a: number; b: number; }",
            Some(json!([{ "default": ["decorated-field", "field"] }])),
        ),
        // Group: get and set adjacent allowed
        (
            "class Foo {
                a: number;
                constructor() {}
                get b() { return 1; }
                set b(v: number) {}
                m() {}
            }",
            Some(json!([{
                "default": ["field", "constructor", ["get", "set"], "method"]
            }])),
        ),
        // Static init block ordering
        (
            "class Foo {
                static {}
                m(): void {}
                f = 1;
            }",
            Some(json!([{ "default": ["static-initialization", "method", "field"] }])),
        ),
        // Alphabetical - memberTypes never
        (
            "interface Foo { a: number; b: string; c: boolean; }",
            Some(json!([{
                "default": { "memberTypes": "never", "order": "alphabetically" }
            }])),
        ),
        // Natural order keeps leading-zero numeric runs distinct.
        (
            "interface Foo { a02: string; a2: string; a10: string; }",
            Some(json!([{
                "default": { "memberTypes": "never", "order": "natural" }
            }])),
        ),
        // Type literal default
        ("type Foo = { a: string; b(): void; };", None),
        // Abstract members ordering
        (
            "abstract class Foo {
                public abstract a: number;
                protected abstract b: number;
                public c(): void {}
                protected d(): void {}
            }",
            None,
        ),
        // Overload signatures should not affect ordering (skip-rank)
        (
            "class Foo {
                a: string;
                m(x: number): void;
                m(x: string): void;
                m(x: number | string): void {}
            }",
            None,
        ),
        // Abstract methods after fields are ordered.
        (
            "abstract class Foo {
                a: string;
                abstract m(x: number): void;
                abstract m(x: string): void;
            }",
            None,
        ),
        // Optionality order: required-first
        (
            "interface Foo { a: string; b: number; c?: boolean; d?: string; }",
            Some(json!([{
                "default": { "memberTypes": "never", "optionalityOrder": "required-first" }
            }])),
        ),
        // Optionality order: optional-first
        (
            "interface Foo { a?: string; b?: number; c: boolean; d: string; }",
            Some(json!([{
                "default": { "memberTypes": "never", "optionalityOrder": "optional-first" }
            }])),
        ),
        // Optionality order allows groups with no optionality transition.
        (
            "interface Foo { a?: string; b?: number; }",
            Some(json!([{
                "default": { "memberTypes": "never", "optionalityOrder": "required-first" }
            }])),
        ),
        (
            "interface Foo { a: string; b: number; }",
            Some(json!([{
                "default": { "memberTypes": "never", "optionalityOrder": "optional-first" }
            }])),
        ),
        // typeLiterals option overrides default
        ("type Foo = { B(): void; A: string; };", Some(json!([{ "typeLiterals": "never" }]))),
        // classExpressions config separate from classes
        (
            "const Foo = class { B(): void {} A = 1; };",
            Some(json!([{ "classExpressions": "never" }])),
        ),
        // Unmatched members are irrelevant to custom ordering.
        (
            "class Foo {
                private c: string;
                constructor() {}
                public b(): void {}
                public static a: string;
            }",
            Some(json!([{ "default": ["public-instance-method", "public-static-field"] }])),
        ),
        // Sorted configs use the default member groups when memberTypes is omitted.
        (
            "interface Foo { B: string; a: number; c: boolean; B(): void; a(): void; c(): void; }",
            Some(json!([{ "default": { "order": "alphabetically" } }])),
        ),
        // Optionality is enforced within each member group.
        (
            "interface Foo { a(): void; b?(): void; c: string; d?: string; }",
            Some(json!([{
                "default": { "memberTypes": ["method", "field"], "optionalityOrder": "required-first" }
            }])),
        ),
        // Private identifier (#private) field ordering
        (
            "class Foo {
                public a: number;
                #b: number;
            }",
            None,
        ),
        // Default config does not distinguish readonly members from their base groups.
        ("interface Foo { readonly b: string; a: string; }", None),
        ("interface Foo { readonly [k: string]: string; a: string; }", None),
        ("class Foo { public static readonly b: string; public static a: string; }", None),
    ];

    let fail: Vec<(&str, Option<serde_json::Value>)> = vec![
        // Method declared before field (default class order: fields before methods)
        (
            "class Foo {
                m(): void {}
                a: string;
            }",
            None,
        ),
        // Interface: method declared before field
        ("interface Foo { B(): void; A: string; }", None),
        // Custom order violation
        (
            "interface Foo { field: number; method(): void; }",
            Some(json!([{ "interfaces": ["method", "field"] }])),
        ),
        // Alphabetical violation
        (
            "interface Foo { b: number; a: string; }",
            Some(json!([{
                "default": { "memberTypes": "never", "order": "alphabetically" }
            }])),
        ),
        // Natural order violation with leading-zero numeric runs.
        (
            "interface Foo { a2: string; a02: string; }",
            Some(json!([{
                "default": { "memberTypes": "never", "order": "natural" }
            }])),
        ),
        // optionality-order: required-first violation
        (
            "interface Foo { a?: string; b: number; }",
            Some(json!([{
                "default": { "optionalityOrder": "required-first", "memberTypes": "never" }
            }])),
        ),
        // Decorated field should come before non-decorated
        (
            "class Foo { a: number; @Dec b: number; }",
            Some(json!([{ "default": ["decorated-field", "field"] }])),
        ),
        // Static method before instance method (default order)
        (
            "class Foo {
                instanceMethod() {}
                static staticMethod() {}
            }",
            None,
        ),
        // Interface optionality-order: optional-first violation
        (
            "interface Foo { a: string; b?: number; }",
            Some(json!([{
                "default": { "optionalityOrder": "optional-first", "memberTypes": "never" }
            }])),
        ),
        // Type literal violation when typeLiterals is configured
        (
            "type Foo = { method(): void; field: number; };",
            Some(json!([{ "typeLiterals": ["field", "method"] }])),
        ),
        // Group config violated: alphabetical sort within group
        (
            "interface Foo { b: string; a: string; d(): void; c(): void; }",
            Some(json!([{
                "default": { "memberTypes": ["field", "method"], "order": "alphabetically" }
            }])),
        ),
        // Readonly-specific groups are still supported in custom orders.
        (
            "class Foo { public static a: string; public static readonly b: string; }",
            Some(json!([{ "default": ["public-static-readonly-field", "public-static-field"] }])),
        ),
        // Abstract methods still participate in member ordering.
        (
            "abstract class Foo {
                abstract m(): void;
                a: string;
            }",
            None,
        ),
    ];

    Tester::new(MemberOrdering::NAME, MemberOrdering::PLUGIN, pass, fail).test_and_snapshot();
}
