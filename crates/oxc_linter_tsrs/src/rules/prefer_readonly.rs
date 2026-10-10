// Port of internal/rules/prefer_readonly/prefer_readonly.go.

use rustc_hash::{FxHashMap, FxHashSet};
use tsrs_ast::{self as ast, Kind, ModifierFlags, Node, SymbolFlags};
use tsrs_checker::{Checker, ObjectFlags, Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{
    Ctx, Listener, Rule, RuleDiagnostic, RuleFix, RuleMessage, RuleVisitor, opt_bool,
    options_object,
};
use crate::utils;

fn build_prefer_readonly_message(name: &str) -> RuleMessage {
    RuleMessage::with_help(
        "preferReadonly",
        format!("Member '{name}' is never reassigned."),
        "Mark it as `readonly`.",
    )
}

const OUTSIDE_CONSTRUCTOR: i32 = -1;
const DIRECTLY_INSIDE_CONSTRUCTOR: i32 = 0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum TypeToClassRelation {
    ClassAndInstance,
    Class,
    Instance,
    None,
}

struct PreparedReport {
    pos: i32,
    end: i32,
    message: RuleMessage,
    fixes: Vec<RuleFix>,
}

#[derive(Default)]
struct OrderedDeclarationMap {
    order: Vec<String>,
    nodes: FxHashMap<String, P<Node>>,
}

impl OrderedDeclarationMap {
    fn set(&mut self, name: String, node: P<Node>) {
        if !self.nodes.contains_key(&name) {
            self.order.push(name.clone());
        }
        self.nodes.insert(name, node);
    }

    fn values_without(&self, skipped: &FxHashSet<String>) -> Vec<P<Node>> {
        let mut values = Vec::with_capacity(self.order.len());
        for name in &self.order {
            if skipped.contains(name) {
                continue;
            }
            if let Some(&node) = self.nodes.get(name) {
                values.push(node);
            }
        }
        values
    }
}

struct ClassScope {
    class_type: P<Type>,
    only_inline_lambdas: bool,
    constructor_scope_depth: i32,
    member_variable_modifications: FxHashSet<String>,
    member_variable_with_constructor_modifications: FxHashSet<String>,
    private_modifiable_members: OrderedDeclarationMap,
    private_modifiable_statics: OrderedDeclarationMap,
    static_variable_modifications: FxHashSet<String>,
    deferred_reports: Vec<PreparedReport>,
}

impl ClassScope {
    fn new(c: &mut Checker, class_node: P<Node>, only_inline_lambdas: bool) -> ClassScope {
        let mut class_type = c.get_type_at_location(class_node);
        if utils::is_intersection_type(class_type) {
            if let Some(&first) = class_type.types().first() {
                class_type = first;
            }
        }
        let mut scope = ClassScope {
            class_type,
            only_inline_lambdas,
            constructor_scope_depth: OUTSIDE_CONSTRUCTOR,
            member_variable_modifications: FxHashSet::default(),
            member_variable_with_constructor_modifications: FxHashSet::default(),
            private_modifiable_members: OrderedDeclarationMap::default(),
            private_modifiable_statics: OrderedDeclarationMap::default(),
            static_variable_modifications: FxHashSet::default(),
            deferred_reports: Vec::new(),
        };
        if ast::is_class_like(class_node) {
            for &member in class_node.members() {
                if ast::is_property_declaration(member) {
                    scope.add_declared_variable(member);
                }
            }
        }
        scope
    }

    fn add_declared_variable(&mut self, node: P<Node>) {
        let Some(name_node) = node.name() else {
            return;
        };
        let flags = node.modifier_flags();
        let is_private =
            flags.intersects(ModifierFlags::Private) || name_node.kind() == Kind::PrivateIdentifier;
        if !is_private {
            return;
        }
        if flags.intersects(ModifierFlags::Accessor | ModifierFlags::Readonly) {
            return;
        }
        if ast::is_computed_property_name(name_node) {
            return;
        }
        let initializer = node.initializer();
        if self.only_inline_lambdas && !initializer.is_some_and(ast::is_arrow_function) {
            return;
        }
        let name = name_node.text();
        if name.is_empty() {
            return;
        }
        if flags.intersects(ModifierFlags::Static) {
            self.private_modifiable_statics.set(name.to_string(), node);
            return;
        }
        self.private_modifiable_members.set(name.to_string(), node);
    }

    fn add_variable_modification(&mut self, c: &mut Checker, node: P<Node>) {
        if !ast::is_property_access_expression(node) {
            return;
        }
        let modifier_type = c.get_type_at_location(node.expression().unwrap());
        let relation = self.get_type_to_class_relation(c, Some(modifier_type));
        let name = node.name().unwrap().text().to_string();
        if relation == TypeToClassRelation::Instance
            && self.constructor_scope_depth == DIRECTLY_INSIDE_CONSTRUCTOR
        {
            self.member_variable_with_constructor_modifications.insert(name);
            return;
        }
        if relation == TypeToClassRelation::Instance
            || relation == TypeToClassRelation::ClassAndInstance
        {
            self.member_variable_modifications.insert(name.clone());
        }
        if relation == TypeToClassRelation::Class
            || relation == TypeToClassRelation::ClassAndInstance
        {
            self.static_variable_modifications.insert(name);
        }
    }

    fn enter_constructor(&mut self, node: P<Node>) {
        self.constructor_scope_depth = DIRECTLY_INSIDE_CONSTRUCTOR;
        for &parameter in node.parameters() {
            if parameter.modifier_flags().intersects(ModifierFlags::Private) {
                self.add_declared_variable(parameter);
            }
        }
    }

    fn enter_non_constructor(&mut self) {
        if self.constructor_scope_depth != OUTSIDE_CONSTRUCTOR {
            self.constructor_scope_depth += 1;
        }
    }

    fn exit_constructor(&mut self) {
        self.constructor_scope_depth = OUTSIDE_CONSTRUCTOR;
    }

    fn exit_non_constructor(&mut self) {
        if self.constructor_scope_depth != OUTSIDE_CONSTRUCTOR {
            self.constructor_scope_depth -= 1;
        }
    }

    fn finalize_unmodified_private_non_readonlys(&self) -> Vec<P<Node>> {
        let mut result =
            self.private_modifiable_members.values_without(&self.member_variable_modifications);
        result.extend(
            self.private_modifiable_statics.values_without(&self.static_variable_modifications),
        );
        result
    }

    fn member_has_constructor_modifications(&self, name: &str) -> bool {
        self.member_variable_with_constructor_modifications.contains(name)
    }

    fn get_type_to_class_relation(
        &self,
        c: &mut Checker,
        t: Option<P<Type>>,
    ) -> TypeToClassRelation {
        let Some(t) = t else {
            return TypeToClassRelation::None;
        };
        if utils::is_intersection_type(t) {
            let mut result = TypeToClassRelation::None;
            for &part in t.types() {
                match self.get_type_to_class_relation(c, Some(part)) {
                    TypeToClassRelation::Class => {
                        if result == TypeToClassRelation::Instance {
                            return TypeToClassRelation::ClassAndInstance;
                        }
                        result = TypeToClassRelation::Class;
                    }
                    TypeToClassRelation::Instance => {
                        if result == TypeToClassRelation::Class {
                            return TypeToClassRelation::ClassAndInstance;
                        }
                        result = TypeToClassRelation::Instance;
                    }
                    _ => {}
                }
            }
            return result;
        }
        if utils::is_union_type(t) {
            let types = t.types();
            if types.is_empty() {
                return TypeToClassRelation::None;
            }
            // Any union of class/instance and something else cannot access private members, so we
            // assume this union contains only classes or class instances because otherwise TypeScript
            // would report an error at the access site.
            return self.get_type_to_class_relation(c, Some(types[0]));
        }
        if t.symbol().is_none() || !type_is_or_has_base_type(c, Some(t), Some(self.class_type)) {
            return TypeToClassRelation::None;
        }
        let type_is_class =
            utils::is_object_type(t) && t.object_flags().intersects(ObjectFlags::Anonymous);
        if type_is_class {
            return TypeToClassRelation::Class;
        }
        TypeToClassRelation::Instance
    }
}

fn type_is_or_has_base_type(
    c: &mut Checker,
    t: Option<P<Type>>,
    base_type: Option<P<Type>>,
) -> bool {
    let (Some(t), Some(base_type)) = (t, base_type) else {
        return false;
    };
    let Some(base_symbol) = base_type.symbol() else {
        return false;
    };
    let mut queue = vec![t];
    let mut seen: FxHashSet<P<Type>> = FxHashSet::default();
    while let Some(current) = queue.pop() {
        if !seen.insert(current) {
            continue;
        }
        if current.symbol() == Some(base_symbol) {
            return true;
        }
        if !utils::is_object_type(current)
            || !current.object_flags().intersects(ObjectFlags::ClassOrInterface)
        {
            continue;
        }
        queue.extend_from_slice(c.get_base_types(current));
    }
    false
}

fn is_destructuring_assignment(node: P<Node>) -> bool {
    let mut current = node.parent();
    while let Some(cur) = current {
        let Some(parent) = cur.parent() else {
            break;
        };
        if ast::is_object_literal_expression(parent)
            || ast::is_array_literal_expression(parent)
            || ast::is_spread_assignment(parent)
            || (ast::is_spread_element(parent)
                && parent.parent().is_some_and(ast::is_array_literal_expression))
        {
            current = Some(parent);
            continue;
        }
        if ast::is_binary_expression(parent) && !ast::is_property_access_expression(cur) {
            let bin_expr = parent.as_binary_expression();
            return bin_expr.left == cur && bin_expr.operator_token.kind() == Kind::EqualsToken;
        }
        break;
    }
    false
}

fn get_type_annotation_for_violating_node(
    ctx: &mut Ctx,
    node: P<Node>,
    violating_type: P<Type>,
    initializer_type: P<Type>,
) -> String {
    let annotation = utils::type_to_string(ctx.checker, violating_type);
    if annotation.is_empty() {
        return String::new();
    }
    if !initializer_type.flags().intersects(TypeFlags::EnumLiteral) {
        return annotation;
    }
    let Some(symbol) =
        ctx.checker.resolve_name_exported(&annotation, node, SymbolFlags::Type, false)
    else {
        return String::new();
    };
    let mut value_symbol =
        ctx.checker.resolve_name_exported(&annotation, node, SymbolFlags::Value, false);
    if let Some(v) = value_symbol {
        value_symbol = Some(ctx.checker.skip_alias(v));
    }
    let symbol = ctx.checker.skip_alias(symbol);
    if value_symbol.is_some_and(|v| v != symbol) {
        return String::new();
    }
    let definition_type = ctx.checker.get_declared_type_of_symbol(symbol);
    if definition_type != violating_type {
        return String::new();
    }
    annotation
}

pub struct PreferReadonly {
    only_inline_lambdas: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(PreferReadonly { only_inline_lambdas: opt_bool(&m, "onlyInlineLambdas", false) }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::ClassDeclaration),
    Listener::Enter(Kind::ClassExpression),
    Listener::Exit(Kind::ClassDeclaration),
    Listener::Exit(Kind::ClassExpression),
    Listener::Enter(Kind::Constructor),
    Listener::Exit(Kind::Constructor),
    Listener::Enter(Kind::ArrowFunction),
    Listener::Exit(Kind::ArrowFunction),
    Listener::Enter(Kind::FunctionDeclaration),
    Listener::Exit(Kind::FunctionDeclaration),
    Listener::Enter(Kind::FunctionExpression),
    Listener::Exit(Kind::FunctionExpression),
    Listener::Enter(Kind::MethodDeclaration),
    Listener::Exit(Kind::MethodDeclaration),
    Listener::Enter(Kind::GetAccessor),
    Listener::Exit(Kind::GetAccessor),
    Listener::Enter(Kind::SetAccessor),
    Listener::Exit(Kind::SetAccessor),
    Listener::Enter(Kind::PropertyAccessExpression),
];

impl Rule for PreferReadonly {
    fn name(&self) -> &'static str {
        "prefer-readonly"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self, class_scope_stack: Vec::new() })
    }
}

struct Visitor {
    rule: &'static PreferReadonly,
    class_scope_stack: Vec<ClassScope>,
}

fn build_reports(ctx: &mut Ctx, finalized_scope: &ClassScope) -> Vec<PreparedReport> {
    let mut reports = Vec::new();
    for violating_node in finalized_scope.finalize_unmodified_private_non_readonlys() {
        let Some(name_node) = violating_node.name() else {
            continue;
        };
        let (name_pos, name_end) = ctx.trim(name_node);
        let name_text = &ctx.text()[name_pos as usize..name_end as usize];
        // getCustomReportNode: a synthetic node spanning the declaration start to the name end.
        let (decl_pos, _) = ctx.trim(violating_node);
        let mut fixes = vec![ctx.fix_insert_before(name_node, "readonly ")];
        if ast::is_property_declaration(violating_node) {
            let property = violating_node.as_property_declaration();
            if property.type_.get().is_none()
                && ast::is_identifier(name_node)
                && finalized_scope.member_has_constructor_modifications(name_node.text())
            {
                if let Some(initializer) = property.initializer() {
                    let violating_type = ctx.checker.get_type_at_location(violating_node);
                    let initializer_type = ctx.checker.get_type_at_location(initializer);
                    if violating_type != initializer_type
                        && initializer_type.flags().intersects(TypeFlags::Literal)
                    {
                        let type_annotation = get_type_annotation_for_violating_node(
                            ctx,
                            violating_node,
                            violating_type,
                            initializer_type,
                        );
                        if !type_annotation.is_empty() {
                            fixes.push(
                                ctx.fix_insert_after(name_node, format!(": {type_annotation}")),
                            );
                        }
                    }
                }
            }
        }
        reports.push(PreparedReport {
            pos: decl_pos,
            end: name_end,
            message: build_prefer_readonly_message(name_text),
            fixes,
        });
    }
    reports
}

impl Visitor {
    fn flush_finalized_scope(&mut self, ctx: &mut Ctx, finalized_scope: ClassScope) {
        let mut reports = build_reports(ctx, &finalized_scope);
        reports.extend(finalized_scope.deferred_reports);
        if let Some(parent) = self.class_scope_stack.last_mut() {
            parent.deferred_reports.extend(reports);
            return;
        }
        for report in reports {
            let fixes = report.fixes;
            ctx.report_diagnostic_with_fixes(
                RuleDiagnostic {
                    pos: report.pos,
                    end: report.end,
                    message: report.message,
                    labeled_ranges: Vec::new(),
                },
                |_| fixes,
            );
        }
    }

    fn handle_property_access_expression(&mut self, ctx: &mut Ctx, node: P<Node>) {
        let scope = self.class_scope_stack.last_mut().unwrap();
        let parent = node.parent().unwrap();
        if ast::is_binary_expression(parent) {
            if parent.as_binary_expression().left == node
                && ast::is_assignment_expression(parent, false)
            {
                scope.add_variable_modification(ctx.checker, node);
            }
            return;
        }
        if parent.kind() == Kind::DeleteExpression || is_destructuring_assignment(node) {
            scope.add_variable_modification(ctx.checker, node);
            return;
        }
        if parent.kind() == Kind::PostfixUnaryExpression {
            let p = parent.as_postfix_unary_expression();
            if p.operator == Kind::PlusPlusToken || p.operator == Kind::MinusMinusToken {
                let operand = p.operand;
                if ast::is_property_access_expression(operand) {
                    scope.add_variable_modification(ctx.checker, operand);
                }
            }
            return;
        }
        if ast::is_prefix_unary_expression(parent) {
            let p = parent.as_prefix_unary_expression();
            if p.operator == Kind::PlusPlusToken || p.operator == Kind::MinusMinusToken {
                let operand = p.operand;
                if ast::is_property_access_expression(operand) {
                    scope.add_variable_modification(ctx.checker, operand);
                }
            }
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        match listener {
            Listener::Enter(Kind::ClassDeclaration | Kind::ClassExpression) => {
                let scope = ClassScope::new(ctx.checker, node, self.rule.only_inline_lambdas);
                self.class_scope_stack.push(scope);
            }
            Listener::Exit(Kind::ClassDeclaration | Kind::ClassExpression) => {
                if let Some(scope) = self.class_scope_stack.pop() {
                    self.flush_finalized_scope(ctx, scope);
                }
            }
            Listener::Enter(Kind::Constructor) => {
                if let Some(scope) = self.class_scope_stack.last_mut() {
                    scope.enter_constructor(node);
                }
            }
            Listener::Exit(Kind::Constructor) => {
                if let Some(scope) = self.class_scope_stack.last_mut() {
                    scope.exit_constructor();
                }
            }
            Listener::Enter(Kind::PropertyAccessExpression)
                if !self.class_scope_stack.is_empty() =>
            {
                self.handle_property_access_expression(ctx, node);
            }
            Listener::Enter(Kind::PropertyAccessExpression) => {}
            Listener::Enter(_) => {
                if let Some(scope) = self.class_scope_stack.last_mut() {
                    scope.enter_non_constructor();
                }
            }
            Listener::Exit(_) => {
                if let Some(scope) = self.class_scope_stack.last_mut() {
                    scope.exit_non_constructor();
                }
            }
            _ => {}
        }
    }
}
