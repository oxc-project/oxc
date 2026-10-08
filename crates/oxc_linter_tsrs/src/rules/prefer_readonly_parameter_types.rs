// Port of internal/rules/prefer_readonly_parameter_types/prefer_readonly_parameter_types.go.

use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, CheckFlags, Kind, ModifierFlags, Node, Symbol, SymbolFlags};
use tsrs_checker::{Checker, Type, TypeFlags};
use tsrs_compiler::Program;
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor, opt_bool, options_object};
use crate::utils::{self, TypeOrValueSpecifier};

fn build_should_be_readonly_message() -> RuleMessage {
    RuleMessage::new("shouldBeReadonly", "Parameter should be a readonly type.")
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Readonlyness {
    Unknown,
    Mutable,
    Readonly,
}

struct ReadonlynessOptions<'a> {
    allow: &'a [TypeOrValueSpecifier],
    treat_methods_as_readonly: bool,
}

type SeenTypes = FxHashSet<P<Type>>;

fn is_type_readonly_array_or_tuple(
    program: &Program,
    c: &mut Checker,
    t: P<Type>,
    opts: &ReadonlynessOptions,
    seen_types: &mut SeenTypes,
) -> Readonlyness {
    let check_type_arguments =
        |c: &mut Checker, seen_types: &mut SeenTypes, array_type: P<Type>| {
            let type_arguments = c.get_type_arguments(array_type);
            if type_arguments.is_empty() {
                return Readonlyness::Readonly;
            }
            for &type_arg in type_arguments {
                if is_type_readonly_recurser(program, c, type_arg, opts, seen_types)
                    == Readonlyness::Mutable
                {
                    return Readonlyness::Mutable;
                }
            }
            Readonlyness::Readonly
        };
    if c.is_array_type(t) {
        if t.symbol().is_some_and(|s| s.name() == "Array") {
            return Readonlyness::Mutable;
        }
        return check_type_arguments(c, seen_types, t);
    }
    if tsrs_checker::is_tuple_type_exported(t) {
        let tuple_target = t.target().unwrap_or(t);
        if !tsrs_checker::is_tuple_type_exported(tuple_target) {
            return Readonlyness::Unknown;
        }
        if !tuple_target.as_tuple_type().is_readonly() {
            return Readonlyness::Mutable;
        }
        return check_type_arguments(c, seen_types, t);
    }
    Readonlyness::Unknown
}

fn property_has_private_identifier_name(property: P<Symbol>) -> bool {
    if let Some(value_declaration) = property.value_declaration() {
        return value_declaration.name().is_some_and(|name| name.kind() == Kind::PrivateIdentifier);
    }
    property.declarations().iter().any(|declaration| {
        declaration.name().is_some_and(|name| name.kind() == Kind::PrivateIdentifier)
    })
}

fn property_is_private_mapped_property(property: P<Symbol>) -> bool {
    property.check_flags().intersects(CheckFlags::Mapped)
        && tsrs_checker::get_declaration_modifier_flags_from_symbol_exported(property)
            .intersects(ModifierFlags::Private)
}

fn property_is_readonly(c: &mut Checker, property: P<Symbol>) -> bool {
    if c.is_readonly_symbol(property) {
        return true;
    }
    let check_flags = property.check_flags();
    if check_flags.intersects(CheckFlags::Readonly) {
        return true;
    }
    if check_flags.intersects(CheckFlags::SyntheticMethod) {
        return true;
    }
    if property.flags.get().intersects(SymbolFlags::Method)
        && check_flags.intersects(CheckFlags::Mapped)
    {
        return true;
    }
    if property.value_declaration().is_none() && check_flags.intersects(CheckFlags::Mapped) {
        return true;
    }
    tsrs_checker::get_declaration_modifier_flags_from_symbol_exported(property)
        .intersects(ModifierFlags::Readonly)
}

fn is_type_readonly_object(
    program: &Program,
    c: &mut Checker,
    t: P<Type>,
    opts: &ReadonlynessOptions,
    seen_types: &mut SeenTypes,
) -> Readonlyness {
    let properties = c.get_properties_of_type(t);
    for &property in properties {
        if opts.treat_methods_as_readonly && property.flags.get().intersects(SymbolFlags::Method) {
            continue;
        }
        if property_is_readonly(c, property) {
            continue;
        }
        if property_has_private_identifier_name(property) {
            continue;
        }
        // `private` class properties are not exposed by a mapped type such as `Readonly<T>`, so
        // their original declaration's mutability is irrelevant.
        if property_is_private_mapped_property(property) {
            continue;
        }
        return Readonlyness::Mutable;
    }
    for &property in properties {
        if property.flags.get().intersects(SymbolFlags::Method) {
            continue;
        }
        if property_has_private_identifier_name(property) {
            continue;
        }
        if property_is_private_mapped_property(property) {
            continue;
        }
        let property_type = match c.get_type_of_property_of_type(t, property.name()) {
            Some(pt) => pt,
            None => c.get_type_of_symbol(property),
        };
        if seen_types.contains(&property_type) {
            continue;
        }
        if is_type_readonly_recurser(program, c, property_type, opts, seen_types)
            == Readonlyness::Mutable
        {
            return Readonlyness::Mutable;
        }
    }
    for &info in c.get_index_infos_of_type(t) {
        if !info.is_readonly() {
            return Readonlyness::Mutable;
        }
        let value_type = info.value_type();
        if value_type == t {
            continue;
        }
        if seen_types.contains(&value_type) {
            continue;
        }
        if is_type_readonly_recurser(program, c, value_type, opts, seen_types)
            == Readonlyness::Mutable
        {
            return Readonlyness::Mutable;
        }
    }
    Readonlyness::Readonly
}

fn is_type_readonly_recurser(
    program: &Program,
    c: &mut Checker,
    t: P<Type>,
    opts: &ReadonlynessOptions,
    seen_types: &mut SeenTypes,
) -> Readonlyness {
    seen_types.insert(t);
    if utils::type_matches_some_specifier(t, opts.allow, program) {
        return Readonlyness::Readonly;
    }
    if utils::is_union_type(t) {
        for &sub_type in t.types() {
            if seen_types.contains(&sub_type) {
                continue;
            }
            if is_type_readonly_recurser(program, c, sub_type, opts, seen_types)
                != Readonlyness::Readonly
            {
                return Readonlyness::Mutable;
            }
        }
        return Readonlyness::Readonly;
    }
    if utils::is_intersection_type(t) {
        let mut has_array_or_tuple = false;
        for &sub_type in t.types() {
            if c.is_array_type(sub_type) || tsrs_checker::is_tuple_type_exported(sub_type) {
                has_array_or_tuple = true;
                break;
            }
        }
        if has_array_or_tuple {
            for &sub_type in t.types() {
                if seen_types.contains(&sub_type) {
                    continue;
                }
                if is_type_readonly_recurser(program, c, sub_type, opts, seen_types)
                    != Readonlyness::Readonly
                {
                    return Readonlyness::Mutable;
                }
            }
            return Readonlyness::Readonly;
        }
        let readonly_object = is_type_readonly_object(program, c, t, opts, seen_types);
        if readonly_object != Readonlyness::Unknown {
            return readonly_object;
        }
    }
    if !utils::is_object_type(t) {
        return Readonlyness::Readonly;
    }
    if !utils::get_call_signatures(c, t).is_empty() && c.get_properties_of_type(t).is_empty() {
        return Readonlyness::Readonly;
    }
    let readonly_array = is_type_readonly_array_or_tuple(program, c, t, opts, seen_types);
    if readonly_array != Readonlyness::Unknown {
        return readonly_array;
    }
    let readonly_object = is_type_readonly_object(program, c, t, opts, seen_types);
    if readonly_object != Readonlyness::Unknown {
        return readonly_object;
    }
    Readonlyness::Readonly
}

fn is_type_readonly(
    program: &Program,
    c: &mut Checker,
    t: P<Type>,
    opts: &ReadonlynessOptions,
) -> bool {
    is_type_readonly_recurser(program, c, t, opts, &mut FxHashSet::default())
        == Readonlyness::Readonly
}

fn is_literal_or_taggable_primitive_like(t: P<Type>) -> bool {
    let flags = t.flags();
    if flags.intersects(TypeFlags::Literal) {
        return true;
    }
    flags.intersects(
        TypeFlags::BigInt | TypeFlags::Number | TypeFlags::String | TypeFlags::TemplateLiteral,
    )
}

fn is_object_literal_like(c: &mut Checker, t: P<Type>) -> bool {
    utils::get_call_signatures(c, t).is_empty()
        && utils::get_construct_signatures(c, t).is_empty()
        && utils::is_object_type(t)
}

fn is_type_branded_literal(c: &mut Checker, t: P<Type>) -> bool {
    if !utils::is_intersection_type(t) {
        return false;
    }
    let mut had_object_like = false;
    let mut had_primitive_like = false;
    for &constituent in t.types() {
        if is_object_literal_like(c, constituent) {
            had_object_like = true;
        } else if is_literal_or_taggable_primitive_like(constituent) {
            had_primitive_like = true;
        } else {
            return false;
        }
    }
    had_object_like && had_primitive_like
}

fn is_type_branded_literal_like(c: &mut Checker, t: P<Type>) -> bool {
    if utils::is_union_type(t) {
        return t.types().iter().all(|&part| is_type_branded_literal(c, part));
    }
    is_type_branded_literal(c, t)
}

fn is_parameter_property(parameter: P<Node>) -> bool {
    if !ast::is_parameter_declaration(parameter) {
        return false;
    }
    if parameter.parent().is_none_or(|p| p.kind() != Kind::Constructor) {
        return false;
    }
    parameter.modifier_flags().intersects(ModifierFlags::ParameterPropertyModifier)
}

fn get_parameter_type(c: &mut Checker, parameter: P<Node>) -> P<Type> {
    if let Some(type_node) = parameter.type_node() {
        return c.get_type_from_type_node(type_node);
    }
    c.get_type_at_location(parameter)
}

pub struct PreferReadonlyParameterTypes {
    allow: Vec<TypeOrValueSpecifier>,
    check_parameter_properties: bool,
    ignore_inferred_types: bool,
    treat_methods_as_readonly: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(PreferReadonlyParameterTypes {
        allow: utils::unmarshal_type_or_value_specifiers(m.get("allow"))
            .map_err(|e| format!("prefer-readonly-parameter-types: {e}"))?,
        check_parameter_properties: opt_bool(&m, "checkParameterProperties", true),
        ignore_inferred_types: opt_bool(&m, "ignoreInferredTypes", false),
        treat_methods_as_readonly: opt_bool(&m, "treatMethodsAsReadonly", false),
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::ArrowFunction),
    Listener::Enter(Kind::CallSignature),
    Listener::Enter(Kind::ConstructSignature),
    Listener::Enter(Kind::Constructor),
    Listener::Enter(Kind::FunctionDeclaration),
    Listener::Enter(Kind::FunctionExpression),
    Listener::Enter(Kind::FunctionType),
    Listener::Enter(Kind::MethodDeclaration),
    Listener::Enter(Kind::MethodSignature),
];

impl Rule for PreferReadonlyParameterTypes {
    fn name(&self) -> &'static str {
        "prefer-readonly-parameter-types"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static PreferReadonlyParameterTypes,
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let opts = self.rule;
        for &parameter in node.parameters() {
            let parameter_is_property = is_parameter_property(parameter);
            if !opts.check_parameter_properties && parameter_is_property {
                continue;
            }
            let actual_parameter = parameter;
            if opts.ignore_inferred_types && actual_parameter.type_node().is_none() {
                continue;
            }
            let t = get_parameter_type(ctx.checker, actual_parameter);
            let is_read_only = is_type_readonly(
                ctx.program,
                ctx.checker,
                t,
                &ReadonlynessOptions {
                    allow: &opts.allow,
                    treat_methods_as_readonly: opts.treat_methods_as_readonly,
                },
            );
            if !is_read_only && !is_type_branded_literal_like(ctx.checker, t) {
                if parameter_is_property {
                    if let Some(name) = actual_parameter.name() {
                        let name_start = ctx.trim(name).0;
                        ctx.report_range(
                            name_start,
                            actual_parameter.end(),
                            build_should_be_readonly_message(),
                        );
                        continue;
                    }
                }
                ctx.report_node(actual_parameter, build_should_be_readonly_message());
            }
        }
    }
}
