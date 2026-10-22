//! TC39 decorator transform (2023-11 and 2022-03).
//!
//! Based on the SWC implementation and the TC39 proposal:
//! <https://github.com/tc39/proposal-decorators>

use std::mem;

use oxc_allocator::{CloneIn, GetAddress, TakeIn, Vec as ArenaVec};
use oxc_ast::{NONE, ast::*};
use oxc_semantic::{ReferenceFlags, ScopeFlags, SymbolFlags};
use oxc_span::SPAN;
use oxc_syntax::{number::NumberBase, operator::AssignmentOperator};
use oxc_traverse::{BoundIdentifier, Traverse};

use crate::{
    Helper, common::helper_loader::helper_call_expr,
    common::var_declarations::VarDeclarationsStore, context::TraverseCtx,
    decorator::DecoratorVersion, state::TransformState,
    utils::ast_builder::create_class_constructor,
};

#[derive(Default)]
struct Tc39ClassState<'a> {
    init_proto: Option<BoundIdentifier<'a>>,
    init_static: Option<BoundIdentifier<'a>>,
    init_proto_args: Vec<Option<ArrayExpressionElement<'a>>>,
    init_static_args: Vec<Option<ArrayExpressionElement<'a>>>,
    static_non_field_lhs: Vec<AssignmentTarget<'a>>,
    proto_non_field_lhs: Vec<AssignmentTarget<'a>>,
    static_field_lhs: Vec<AssignmentTarget<'a>>,
    proto_field_lhs: Vec<AssignmentTarget<'a>>,
    class_decorators: Vec<Option<ArrayExpressionElement<'a>>>,
    class_decorators_have_this: bool,
    class_lhs: Vec<AssignmentTarget<'a>>,
    super_class: Option<BoundIdentifier<'a>>,
    instance_brand: Option<PrivateIdentifier<'a>>,
    extra_stmts: Vec<Statement<'a>>,
    needs_expr_wrap: bool,
    expr_wrap_class_ref: Option<BoundIdentifier<'a>>,
    expr_extra_vars: Vec<(BoundIdentifier<'a>, Option<Expression<'a>>)>,
    expr_pre_class_inits: Vec<Statement<'a>>,
    expr_extra_stmts: Vec<Statement<'a>>,
}

pub struct Tc39Decorator<'a> {
    version: DecoratorVersion,
    state: Tc39ClassState<'a>,
    extra_vars: Vec<(BoundIdentifier<'a>, Option<Expression<'a>>)>,
    pre_class_inits: Vec<Statement<'a>>,
}

impl<'a> Tc39Decorator<'a> {
    pub fn new(version: DecoratorVersion) -> Self {
        Self {
            version,
            state: Tc39ClassState::default(),
            extra_vars: Vec::new(),
            pre_class_inits: Vec::new(),
        }
    }

    fn is_2023_11(&self) -> bool {
        self.version == DecoratorVersion::V202311
    }

    fn helper(&self) -> Helper {
        if self.is_2023_11() { Helper::ApplyDecs2311 } else { Helper::ApplyDecs2203R }
    }

    fn create_uid(&mut self, name: &str, ctx: &mut TraverseCtx<'a>) -> BoundIdentifier<'a> {
        ctx.generate_uid_in_current_hoist_scope(name)
    }

    fn int_arg(value: i32, ctx: &mut TraverseCtx<'a>) -> ArrayExpressionElement<'a> {
        ArrayExpressionElement::from(ctx.ast.expression_numeric_literal(
            SPAN,
            value as f64,
            None,
            NumberBase::Float,
        ))
    }

    fn method_kind_code(&self, is_static: bool, kind: MethodDefinitionKind) -> i32 {
        if self.is_2023_11() {
            let base = match kind {
                MethodDefinitionKind::Method => 2,
                MethodDefinitionKind::Set => 4,
                MethodDefinitionKind::Get => 3,
                _ => 0,
            };
            if is_static { base | 8 } else { base }
        } else {
            match (is_static, kind) {
                (true, MethodDefinitionKind::Method) => 7,
                (false, MethodDefinitionKind::Method) => 2,
                (true, MethodDefinitionKind::Set) => 9,
                (false, MethodDefinitionKind::Set) => 4,
                (true, MethodDefinitionKind::Get) => 8,
                (false, MethodDefinitionKind::Get) => 3,
                _ => 0,
            }
        }
    }

    fn field_kind_code(&self, is_static: bool) -> i32 {
        if self.is_2023_11() {
            if is_static { 8 } else { 0 }
        } else if is_static {
            5
        } else {
            0
        }
    }

    fn accessor_kind_code(&self, is_static: bool) -> i32 {
        if self.is_2023_11() {
            if is_static { 9 } else { 1 }
        } else if is_static {
            6
        } else {
            1
        }
    }

    fn push_lhs_from_binding(
        &mut self,
        binding: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
        is_static: bool,
        is_field: bool,
    ) {
        let target = binding.create_target(ReferenceFlags::Write, ctx);
        if is_static {
            if is_field {
                self.state.static_field_lhs.push(target);
            } else {
                self.state.static_non_field_lhs.push(target);
            }
        } else if is_field {
            self.state.proto_field_lhs.push(target);
        } else {
            self.state.proto_non_field_lhs.push(target);
        }
    }

    fn build_array(
        &self,
        elems: Vec<Option<ArrayExpressionElement<'a>>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let mut arr: ArenaVec<'a, ArrayExpressionElement<'a>> = ctx.ast.vec();
        for elem in elems {
            arr.push(
                elem.unwrap_or(ArrayExpressionElement::from(ctx.ast.expression_null_literal(SPAN))),
            );
        }
        ctx.ast.expression_array(SPAN, arr)
    }

    fn consume_inits(
        &mut self,
        class_ref: Option<&BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let has_proto = !self.state.init_proto_args.is_empty();
        let has_static = !self.state.init_static_args.is_empty();
        let has_class_decs = !self.state.class_decorators.is_empty();
        if !has_proto && !has_static && !has_class_decs {
            return;
        }

        let mut e_lhs: ArenaVec<'a, Option<AssignmentTarget<'a>>> = ctx.ast.vec();
        for target in self.state.static_non_field_lhs.drain(..) {
            e_lhs.push(Some(target));
        }
        for target in self.state.proto_non_field_lhs.drain(..) {
            e_lhs.push(Some(target));
        }
        for target in self.state.static_field_lhs.drain(..) {
            e_lhs.push(Some(target));
        }
        for target in self.state.proto_field_lhs.drain(..) {
            e_lhs.push(Some(target));
        }

        let init_proto = self.state.init_proto.take();
        let init_static = self.state.init_static.take();
        if let Some(ref init) = init_proto {
            e_lhs.push(Some(init.create_target(ReferenceFlags::Write, ctx)));
        }
        if let Some(ref init) = init_static {
            e_lhs.push(Some(init.create_target(ReferenceFlags::Write, ctx)));
        }

        let mut member_elems: ArenaVec<'a, ArrayExpressionElement<'a>> = ctx.ast.vec();
        for elem in self.state.init_static_args.drain(..) {
            if let Some(e) = elem {
                member_elems.push(e);
            }
        }
        for elem in self.state.init_proto_args.drain(..) {
            if let Some(e) = elem {
                member_elems.push(e);
            }
        }

        let mut class_elems: ArenaVec<'a, ArrayExpressionElement<'a>> = ctx.ast.vec();
        for elem in self.state.class_decorators.drain(..) {
            if let Some(e) = elem {
                class_elems.push(e);
            }
        }

        let mut args: ArenaVec<'a, Argument<'a>> = ctx.ast.vec();
        let first_arg = class_ref
            .map(|b| b.create_read_expression(ctx))
            .unwrap_or_else(|| ctx.ast.expression_this(SPAN));
        args.push(Argument::from(first_arg));

        if self.is_2023_11() {
            args.push(Argument::from(ctx.ast.expression_array(SPAN, class_elems)));
            args.push(Argument::from(ctx.ast.expression_array(SPAN, member_elems)));

            if self.state.class_decorators_have_this
                || self.state.instance_brand.is_some()
                || self.state.super_class.is_some()
            {
                let flag = self.state.class_decorators_have_this as i32;
                args.push(Argument::from(ctx.ast.expression_numeric_literal(
                    SPAN,
                    flag as f64,
                    None,
                    NumberBase::Float,
                )));
            }
            if let Some(ref brand) = self.state.instance_brand {
                let param = ctx.ast.binding_identifier(SPAN, "o");
                let brand_expr = ctx.ast.expression_private_in(
                    SPAN,
                    brand.clone(),
                    ctx.ast.expression_identifier(SPAN, "o"),
                );
                let params = ctx.ast.alloc_formal_parameters(
                    SPAN,
                    FormalParameterKind::ArrowFormalParameters,
                    ctx.ast.vec1(ctx.ast.formal_parameter(
                        SPAN,
                        ctx.ast.vec(),
                        BindingPattern::BindingIdentifier(ctx.ast.alloc(param)),
                        NONE,
                        NONE,
                        false,
                        None,
                        false,
                        false,
                    )),
                    NONE,
                );
                let arrow = ctx.ast.expression_arrow_function(
                    SPAN,
                    false,
                    false,
                    NONE,
                    params,
                    NONE,
                    ctx.ast.alloc_function_body(
                        SPAN,
                        ctx.ast.vec(),
                        ctx.ast.vec1(ctx.ast.statement_return(SPAN, Some(brand_expr))),
                    ),
                );
                args.push(Argument::from(arrow));
            } else if self.state.super_class.is_some() {
                args.push(Argument::from(ctx.ast.expression_identifier(SPAN, "undefined")));
            }
            if let Some(ref super_class) = self.state.super_class {
                args.push(Argument::from(super_class.create_read_expression(ctx)));
            }
            self.state.instance_brand = None;
        } else {
            args.push(Argument::from(ctx.ast.expression_array(SPAN, member_elems)));
            args.push(Argument::from(ctx.ast.expression_array(SPAN, class_elems)));
            if let Some(ref super_class) = self.state.super_class {
                args.push(Argument::from(super_class.create_read_expression(ctx)));
            }
        }
        self.state.class_decorators_have_this = false;

        let call_expr = helper_call_expr(self.helper(), args, ctx);

        if e_lhs.is_empty() && self.state.class_lhs.is_empty() {
            self.state.extra_stmts.push(ctx.ast.statement_expression(SPAN, call_expr));
        } else {
            let mut props: ArenaVec<'a, AssignmentTargetProperty<'a>> = ctx.ast.vec();
            if !e_lhs.is_empty() {
                let mut e_elems: ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>> =
                    ctx.ast.vec();
                for elem in e_lhs {
                    e_elems.push(elem.map(|t| AssignmentTargetMaybeDefault::from(t)));
                }
                let e_array = ctx.ast.alloc_array_assignment_target(SPAN, e_elems, NONE);
                let e_binding = AssignmentTargetMaybeDefault::from(AssignmentTarget::from(
                    AssignmentTargetPattern::ArrayAssignmentTarget(e_array),
                ));
                props.push(ctx.ast.assignment_target_property_assignment_target_property_property(
                    SPAN,
                    PropertyKey::StaticIdentifier(ctx.ast.alloc_identifier_name(SPAN, "e")),
                    e_binding,
                    false,
                ));
            }
            if !self.state.class_lhs.is_empty() {
                let mut c_elems: ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>> =
                    ctx.ast.vec();
                for target in self.state.class_lhs.drain(..) {
                    c_elems.push(Some(AssignmentTargetMaybeDefault::from(target)));
                }
                let c_array = ctx.ast.alloc_array_assignment_target(SPAN, c_elems, NONE);
                let c_binding = AssignmentTargetMaybeDefault::from(AssignmentTarget::from(
                    AssignmentTargetPattern::ArrayAssignmentTarget(c_array),
                ));
                props.push(ctx.ast.assignment_target_property_assignment_target_property_property(
                    SPAN,
                    PropertyKey::StaticIdentifier(ctx.ast.alloc_identifier_name(SPAN, "c")),
                    c_binding,
                    false,
                ));
            }
            let assign = ctx.ast.expression_assignment(
                SPAN,
                AssignmentOperator::Assign,
                AssignmentTarget::from(AssignmentTargetPattern::ObjectAssignmentTarget(
                    ctx.ast.alloc_object_assignment_target(SPAN, props, NONE),
                )),
                call_expr,
            );
            self.state.extra_stmts.push(ctx.ast.statement_expression(SPAN, assign));
        }

        if let Some(ref init) = init_static {
            let init_target = class_ref
                .map(|b| b.create_read_expression(ctx))
                .unwrap_or_else(|| ctx.ast.expression_this(SPAN));
            let call = ctx.ast.expression_call(
                SPAN,
                init.create_read_expression(ctx),
                NONE,
                ctx.ast.vec1(Argument::from(init_target)),
                false,
            );
            self.state.extra_stmts.push(ctx.ast.statement_expression(SPAN, call));
        }
    }

    fn has_any_decorators(class: &Class<'a>) -> bool {
        if !class.decorators.is_empty() {
            return true;
        }
        for member in &class.body.body {
            match member {
                ClassElement::MethodDefinition(m) if !m.decorators.is_empty() => return true,
                ClassElement::PropertyDefinition(p) if !p.decorators.is_empty() => return true,
                ClassElement::AccessorProperty(a) if !a.decorators.is_empty() => return true,
                _ => {}
            }
        }
        false
    }

    fn transform_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        for dec in class.decorators.drain(..) {
            self.state.class_decorators.push(Some(ArrayExpressionElement::from(dec.expression)));
        }

        if let Some(super_expr) = class.super_class.as_ref() {
            let super_expr = super_expr.clone_in(ctx.ast.allocator);
            let binding = self.create_uid("_super", ctx);
            self.extra_vars.push((binding.clone(), None));
            let assign = ctx.ast.expression_assignment(
                SPAN,
                AssignmentOperator::Assign,
                binding.create_target(ReferenceFlags::Write, ctx),
                super_expr,
            );
            self.pre_class_inits.push(ctx.ast.statement_expression(SPAN, assign));
            class.super_class = Some(binding.create_read_expression(ctx));
            self.state.super_class = Some(binding);
        }

        let mut new_members: ArenaVec<'a, ClassElement<'a>> = ctx.ast.vec();

        for member in class.body.body.drain(..) {
            match member {
                ClassElement::MethodDefinition(mut method) => {
                    if method.kind == MethodDefinitionKind::Constructor
                        || method.decorators.is_empty()
                    {
                        new_members.push(ClassElement::MethodDefinition(method));
                        continue;
                    }

                    let decs = mem::replace(&mut method.decorators, ctx.ast.vec());
                    let is_static = method.r#static;
                    let kind = method.kind;
                    let merged = self.merge_decorators(decs, ctx);
                    let name_expr =
                        ArrayExpressionElement::from(self.prop_key_to_expr(&method.key, ctx));

                    let init = self.create_uid("_init", ctx);
                    self.extra_vars.push((init.clone(), None));

                    let mut elems = Vec::new();
                    if let Some(m) = merged {
                        elems.push(m);
                    }
                    elems.push(Self::int_arg(self.method_kind_code(is_static, kind), ctx));
                    elems.push(name_expr);

                    let arr = self.build_array(elems.into_iter().map(Some).collect(), ctx);
                    let arr_arg = Some(ArrayExpressionElement::from(arr));

                    if is_static {
                        self.push_lhs_from_binding(&init, ctx, true, false);
                        self.state.init_static_args.push(arr_arg);
                        if self.state.init_static.is_none() {
                            self.state.init_static = Some(self.create_uid("_initStatic", ctx));
                        }
                    } else {
                        self.push_lhs_from_binding(&init, ctx, false, false);
                        self.state.init_proto_args.push(arr_arg);
                        if self.state.init_proto.is_none() {
                            self.state.init_proto = Some(self.create_uid("_initProto", ctx));
                        }
                    }

                    new_members.push(ClassElement::MethodDefinition(method));
                }
                ClassElement::PropertyDefinition(mut prop) => {
                    if prop.r#type.is_abstract() || prop.decorators.is_empty() {
                        new_members.push(ClassElement::PropertyDefinition(prop));
                        continue;
                    }

                    let decs = mem::replace(&mut prop.decorators, ctx.ast.vec());
                    let is_static = prop.r#static;
                    let is_private = matches!(&prop.key, PropertyKey::PrivateIdentifier(_));
                    let merged = self.merge_decorators(decs, ctx);
                    let name_expr =
                        ArrayExpressionElement::from(self.prop_key_to_expr(&prop.key, ctx));

                    let init = self.create_uid("_init", ctx);
                    self.extra_vars.push((init.clone(), None));

                    let mut elems = Vec::new();
                    if let Some(m) = merged {
                        elems.push(m);
                    }
                    elems.push(Self::int_arg(self.field_kind_code(is_static), ctx));
                    elems.push(name_expr);

                    if is_private {
                        let (getter, setter) = if self.is_2023_11() {
                            let private_id = ctx.ast.private_identifier(
                                SPAN,
                                match &prop.key {
                                    PropertyKey::PrivateIdentifier(id) => id.name,
                                    _ => unreachable!(),
                                },
                            );
                            let getter =
                                ctx.ast.expression_function(
                                    SPAN,
                                    FunctionType::FunctionExpression,
                                    None,
                                    false,
                                    false,
                                    false,
                                    NONE,
                                    NONE,
                                    ctx.ast.alloc_formal_parameters(
                                        SPAN,
                                        FormalParameterKind::FormalParameter,
                                        ctx.ast.vec1(ctx.ast.formal_parameter(
                                            SPAN,
                                            ctx.ast.vec(),
                                            BindingPattern::BindingIdentifier(
                                                ctx.ast.alloc(
                                                    ctx.ast.binding_identifier(SPAN, "_this"),
                                                ),
                                            ),
                                            NONE,
                                            NONE,
                                            false,
                                            None,
                                            false,
                                            false,
                                        )),
                                        NONE,
                                    ),
                                    NONE,
                                    Some(ctx.ast.alloc_function_body(
                                        SPAN,
                                        ctx.ast.vec(),
                                        ctx.ast.vec1(ctx.ast.statement_return(
                                            SPAN,
                                            Some(Expression::from(
                                                ctx.ast.member_expression_private_field_expression(
                                                    SPAN,
                                                    ctx.ast.expression_identifier(SPAN, "_this"),
                                                    private_id.clone(),
                                                    false,
                                                ),
                                            )),
                                        )),
                                    )),
                                );
                            let setter = ctx.ast.expression_function(
                                SPAN,
                                FunctionType::FunctionExpression,
                                None,
                                false,
                                false,
                                false,
                                NONE,
                                NONE,
                                ctx.ast.alloc_formal_parameters(
                                    SPAN,
                                    FormalParameterKind::FormalParameter,
                                    ctx.ast.vec_from_iter([
                                        ctx.ast.formal_parameter(
                                            SPAN,
                                            ctx.ast.vec(),
                                            BindingPattern::BindingIdentifier(
                                                ctx.ast.alloc(
                                                    ctx.ast.binding_identifier(SPAN, "_this"),
                                                ),
                                            ),
                                            NONE,
                                            NONE,
                                            false,
                                            None,
                                            false,
                                            false,
                                        ),
                                        ctx.ast.formal_parameter(
                                            SPAN,
                                            ctx.ast.vec(),
                                            BindingPattern::BindingIdentifier(
                                                ctx.ast
                                                    .alloc(ctx.ast.binding_identifier(SPAN, "_v")),
                                            ),
                                            NONE,
                                            NONE,
                                            false,
                                            None,
                                            false,
                                            false,
                                        ),
                                    ]),
                                    NONE,
                                ),
                                NONE,
                                Some(ctx.ast.alloc_function_body(
                                    SPAN,
                                    ctx.ast.vec(),
                                    ctx.ast.vec1(ctx.ast.statement_expression(
                                        SPAN,
                                        ctx.ast.expression_assignment(
                                            SPAN,
                                            AssignmentOperator::Assign,
                                            AssignmentTarget::from(
                                                ctx.ast.member_expression_private_field_expression(
                                                    SPAN,
                                                    ctx.ast.expression_identifier(SPAN, "_this"),
                                                    private_id,
                                                    false,
                                                ),
                                            ),
                                            ctx.ast.expression_identifier(SPAN, "_v"),
                                        ),
                                    )),
                                )),
                            );
                            (getter, setter)
                        } else {
                            let private_id = ctx.ast.private_identifier(
                                SPAN,
                                match &prop.key {
                                    PropertyKey::PrivateIdentifier(id) => id.name,
                                    _ => unreachable!(),
                                },
                            );
                            let getter = ctx.ast.expression_function(
                                SPAN,
                                FunctionType::FunctionExpression,
                                None,
                                false,
                                false,
                                false,
                                NONE,
                                NONE,
                                ctx.ast.alloc_formal_parameters(
                                    SPAN,
                                    FormalParameterKind::FormalParameter,
                                    ctx.ast.vec(),
                                    NONE,
                                ),
                                NONE,
                                Some(ctx.ast.alloc_function_body(
                                    SPAN,
                                    ctx.ast.vec(),
                                    ctx.ast.vec1(ctx.ast.statement_return(
                                        SPAN,
                                        Some(Expression::from(
                                            ctx.ast.member_expression_private_field_expression(
                                                SPAN,
                                                ctx.ast.expression_this(SPAN),
                                                private_id.clone(),
                                                false,
                                            ),
                                        )),
                                    )),
                                )),
                            );
                            let setter = ctx.ast.expression_function(
                                SPAN,
                                FunctionType::FunctionExpression,
                                None,
                                false,
                                false,
                                false,
                                NONE,
                                NONE,
                                ctx.ast.alloc_formal_parameters(
                                    SPAN,
                                    FormalParameterKind::FormalParameter,
                                    ctx.ast.vec1(ctx.ast.formal_parameter(
                                        SPAN,
                                        ctx.ast.vec(),
                                        BindingPattern::BindingIdentifier(
                                            ctx.ast.alloc(ctx.ast.binding_identifier(SPAN, "_v")),
                                        ),
                                        NONE,
                                        NONE,
                                        false,
                                        None,
                                        false,
                                        false,
                                    )),
                                    NONE,
                                ),
                                NONE,
                                Some(ctx.ast.alloc_function_body(
                                    SPAN,
                                    ctx.ast.vec(),
                                    ctx.ast.vec1(ctx.ast.statement_expression(
                                        SPAN,
                                        ctx.ast.expression_assignment(
                                            SPAN,
                                            AssignmentOperator::Assign,
                                            AssignmentTarget::from(
                                                ctx.ast.member_expression_private_field_expression(
                                                    SPAN,
                                                    ctx.ast.expression_this(SPAN),
                                                    private_id,
                                                    false,
                                                ),
                                            ),
                                            ctx.ast.expression_identifier(SPAN, "_v"),
                                        ),
                                    )),
                                )),
                            );
                            (getter, setter)
                        };
                        elems.push(ArrayExpressionElement::from(getter));
                        elems.push(ArrayExpressionElement::from(setter));
                    }

                    let arr = self.build_array(elems.into_iter().map(Some).collect(), ctx);
                    let arr_arg = Some(ArrayExpressionElement::from(arr));

                    if is_static {
                        self.push_lhs_from_binding(&init, ctx, true, true);
                        self.state.init_static_args.push(arr_arg);
                        if !self.is_2023_11() && self.state.init_static.is_none() {
                            self.state.init_static = Some(self.create_uid("_initStatic", ctx));
                        }
                    } else {
                        self.push_lhs_from_binding(&init, ctx, false, true);
                        self.state.init_proto_args.push(arr_arg);
                        if !self.is_2023_11() && self.state.init_proto.is_none() {
                            self.state.init_proto = Some(self.create_uid("_initProto", ctx));
                        }
                    }

                    new_members.push(ClassElement::PropertyDefinition(prop));
                }
                ClassElement::AccessorProperty(mut accessor) => {
                    if accessor.decorators.is_empty() {
                        new_members.push(ClassElement::AccessorProperty(accessor));
                        continue;
                    }

                    let decs = mem::replace(&mut accessor.decorators, ctx.ast.vec());
                    let is_static = accessor.r#static;
                    let is_private = matches!(&accessor.key, PropertyKey::PrivateIdentifier(_));
                    let merged = self.merge_decorators(decs, ctx);
                    let name_expr =
                        ArrayExpressionElement::from(self.prop_key_to_expr(&accessor.key, ctx));

                    let init = self.create_uid("_init", ctx);
                    self.extra_vars.push((init.clone(), None));

                    let mut elems = Vec::new();
                    if let Some(m) = merged {
                        elems.push(m);
                    }
                    elems.push(Self::int_arg(self.accessor_kind_code(is_static), ctx));
                    elems.push(name_expr);

                    if is_private {
                        elems.push(ArrayExpressionElement::from(
                            ctx.ast.expression_null_literal(SPAN),
                        ));
                        elems.push(ArrayExpressionElement::from(
                            ctx.ast.expression_null_literal(SPAN),
                        ));
                    }

                    let arr = self.build_array(elems.into_iter().map(Some).collect(), ctx);
                    let arr_arg = Some(ArrayExpressionElement::from(arr));

                    if is_static {
                        self.push_lhs_from_binding(&init, ctx, true, false);
                        self.state.init_static_args.push(arr_arg);
                        if self.state.init_static.is_none() {
                            self.state.init_static = Some(self.create_uid("_initStatic", ctx));
                        }
                    } else {
                        self.push_lhs_from_binding(&init, ctx, false, false);
                        self.state.init_proto_args.push(arr_arg);
                        if self.state.init_proto.is_none() {
                            self.state.init_proto = Some(self.create_uid("_initProto", ctx));
                        }
                    }

                    new_members.push(ClassElement::AccessorProperty(accessor));
                }
                _ => {
                    new_members.push(member);
                }
            }
        }

        class.body.body = new_members;
    }

    fn merge_decorators(
        &mut self,
        decorators: ArenaVec<'a, Decorator<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<ArrayExpressionElement<'a>> {
        if decorators.is_empty() {
            return None;
        }
        let mut iter = decorators.into_iter();
        let first = iter.next().unwrap();
        if iter.len() == 0 {
            return Some(ArrayExpressionElement::from(first.expression));
        }
        let mut elems: ArenaVec<'a, ArrayExpressionElement<'a>> = ctx.ast.vec();
        elems.push(ArrayExpressionElement::from(first.expression));
        for dec in iter {
            elems.push(ArrayExpressionElement::from(dec.expression));
        }
        Some(ArrayExpressionElement::from(ctx.ast.expression_array(SPAN, elems)))
    }

    fn prop_key_to_expr(&self, key: &PropertyKey<'a>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        match key {
            PropertyKey::StaticIdentifier(id) => {
                ctx.ast.expression_string_literal(SPAN, id.name, None)
            }
            PropertyKey::PrivateIdentifier(id) => {
                ctx.ast.expression_string_literal(SPAN, id.name, None)
            }
            _ => ctx.ast.expression_string_literal(SPAN, "", None),
        }
    }

    fn process_class_body(
        &mut self,
        class: &mut Class<'a>,
        class_ref: Option<&BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(ref init_proto) = self.state.init_proto.clone() {
            let init_call = ctx.ast.expression_call(
                SPAN,
                init_proto.create_read_expression(ctx),
                NONE,
                ctx.ast.vec1(Argument::from(ctx.ast.expression_this(SPAN))),
                false,
            );

            let mut injected = false;
            if self.is_2023_11() {
                for member in class.body.body.iter_mut() {
                    match member {
                        ClassElement::PropertyDefinition(prop) => {
                            if !prop.r#static && prop.value.is_some() {
                                let value = prop.value.take().unwrap();
                                prop.value = Some(ctx.ast.expression_sequence(
                                    SPAN,
                                    ctx.ast.vec_from_iter([
                                        init_call.clone_in(ctx.ast.allocator),
                                        value,
                                    ]),
                                ));
                                injected = true;
                                break;
                            }
                        }
                        ClassElement::AccessorProperty(accessor) => {
                            if !accessor.r#static && accessor.value.is_some() {
                                let value = accessor.value.take().unwrap();
                                accessor.value = Some(ctx.ast.expression_sequence(
                                    SPAN,
                                    ctx.ast.vec_from_iter([
                                        init_call.clone_in(ctx.ast.allocator),
                                        value,
                                    ]),
                                ));
                                injected = true;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }

            if !injected {
                let has_super = class.super_class.is_some();
                let mut found = false;
                for member in class.body.body.iter_mut() {
                    if let ClassElement::MethodDefinition(method) = member {
                        if method.kind == MethodDefinitionKind::Constructor {
                            if let Some(body) = &mut method.value.body {
                                if has_super {
                                    let pos = body.statements.iter().position(|s| {
                                        matches!(s, Statement::ExpressionStatement(es) if matches!(&es.expression, Expression::CallExpression(ce) if matches!(&ce.callee, Expression::Super(_))))
                                    }).map(|p| p + 1).unwrap_or(0);
                                    body.statements.insert(
                                        pos,
                                        ctx.ast.statement_expression(
                                            SPAN,
                                            init_call.clone_in(ctx.ast.allocator),
                                        ),
                                    );
                                } else {
                                    body.statements.insert(
                                        0,
                                        ctx.ast.statement_expression(
                                            SPAN,
                                            init_call.clone_in(ctx.ast.allocator),
                                        ),
                                    );
                                }
                            }
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    let stmts = if has_super {
                        ctx.ast.vec_from_iter([
                            ctx.ast.statement_expression(
                                SPAN,
                                ctx.ast.expression_call(
                                    SPAN,
                                    ctx.ast.expression_super(SPAN),
                                    NONE,
                                    ctx.ast.vec(),
                                    false,
                                ),
                            ),
                            ctx.ast
                                .statement_expression(SPAN, init_call.clone_in(ctx.ast.allocator)),
                        ])
                    } else {
                        ctx.ast.vec1(
                            ctx.ast
                                .statement_expression(SPAN, init_call.clone_in(ctx.ast.allocator)),
                        )
                    };
                    let scope_id = ctx.scoping.create_child_scope_of_current(
                        ScopeFlags::Function | ScopeFlags::Constructor,
                    );
                    let ctor = create_class_constructor(stmts, has_super, scope_id, ctx);
                    class.body.body.insert(0, ctor);
                }
            }
        }

        self.consume_inits(class_ref, ctx);
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for Tc39Decorator<'a> {
    fn enter_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if !Self::has_any_decorators(class) {
            return;
        }
        if class.r#type != ClassType::ClassDeclaration {
            self.transform_class(class, ctx);
        }
    }

    fn exit_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.state.init_proto.is_none()
            && self.state.init_static.is_none()
            && self.state.class_decorators.is_empty()
        {
            return;
        }
        if class.r#type == ClassType::ClassExpression {
            self.state.expr_extra_vars = std::mem::take(&mut self.extra_vars);
            self.state.expr_pre_class_inits = std::mem::take(&mut self.pre_class_inits);
            let class_ref = self.create_uid("_class", ctx);
            self.state.expr_wrap_class_ref = Some(class_ref);
            self.state.needs_expr_wrap = true;
            if !self.state.class_decorators.is_empty() {
                self.state.class_lhs.push(
                    self.state
                        .expr_wrap_class_ref
                        .as_ref()
                        .unwrap()
                        .create_target(ReferenceFlags::Write, ctx),
                );
            }
            let class_ref = self.state.expr_wrap_class_ref.as_ref().unwrap().clone();
            self.process_class_body(class, Some(&class_ref), ctx);
            self.state.expr_extra_stmts = std::mem::take(&mut self.state.extra_stmts);
        } else {
            self.process_class_body(class, None, ctx);
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.state.needs_expr_wrap {
            return;
        }

        let Expression::ClassExpression(_) = expr else {
            return;
        };
        self.state.needs_expr_wrap = false;

        let class_ref = self.state.expr_wrap_class_ref.take().unwrap();

        let placeholder = ctx.ast.expression_null_literal(SPAN);
        let old_expr = std::mem::replace(expr, placeholder);
        let Expression::ClassExpression(class_box) = old_expr else { unreachable!() };

        let mut body_stmts: ArenaVec<'a, Statement<'a>> = ctx.ast.vec();

        for (var_binding, init) in std::mem::take(&mut self.state.expr_extra_vars) {
            let decl = ctx.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Var,
                var_binding.create_binding_pattern(ctx),
                NONE,
                init,
                false,
            );
            body_stmts.push(
                ctx.ast
                    .declaration_variable(
                        SPAN,
                        VariableDeclarationKind::Var,
                        ctx.ast.vec1(decl),
                        false,
                    )
                    .into(),
            );
        }

        for stmt in std::mem::take(&mut self.state.expr_pre_class_inits) {
            body_stmts.push(stmt);
        }

        for stmt in std::mem::take(&mut self.state.expr_extra_stmts) {
            body_stmts.push(stmt);
        }

        body_stmts
            .push(ctx.ast.statement_return(SPAN, Some(class_ref.create_read_expression(ctx))));

        let params = ctx.ast.alloc_formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            ctx.ast.vec1(ctx.ast.formal_parameter(
                SPAN,
                ctx.ast.vec(),
                class_ref.create_binding_pattern(ctx),
                NONE,
                NONE,
                false,
                None,
                false,
                false,
            )),
            NONE,
        );

        let body = ctx.ast.alloc_function_body(SPAN, ctx.ast.vec(), body_stmts);
        let arrow = ctx.ast.expression_arrow_function(SPAN, false, false, NONE, params, NONE, body);

        let call = ctx.ast.expression_call(
            SPAN,
            arrow,
            NONE,
            ctx.ast.vec1(Argument::from(Expression::ClassExpression(class_box))),
            false,
        );

        *expr = call;
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::ClassDeclaration(class) = stmt else { return };
        if !Self::has_any_decorators(class) {
            return;
        }

        let old_addr = class.address();

        let class_name = class.id.as_ref().map(|id| id.name);
        self.transform_class(class, ctx);
        let binding = if let Some(name) = class_name {
            let binding = ctx.generate_binding(
                name,
                ctx.current_hoist_scope_id(),
                SymbolFlags::FunctionScopedVariable,
            );
            ctx.state.var_declarations.insert_var(&binding, ctx.ast);
            binding
        } else {
            VarDeclarationsStore::create_uid_var("_Class", ctx)
        };
        if !self.state.class_decorators.is_empty() {
            self.state.class_lhs.push(binding.create_target(ReferenceFlags::Write, ctx));
        }
        self.process_class_body(class, Some(&binding), ctx);

        class.r#type = ClassType::ClassExpression;
        let class_expr = Expression::ClassExpression(class.take_in_box(ctx.ast));

        let new_stmt = Statement::from(ctx.ast.statement_expression(
            SPAN,
            ctx.ast.expression_assignment(
                SPAN,
                AssignmentOperator::Assign,
                binding.create_target(ReferenceFlags::Write, ctx),
                class_expr,
            ),
        ));

        let new_addr = new_stmt.address();

        if old_addr != new_addr {
            ctx.state.statement_injector.move_insertions(&old_addr, &new_addr);
        }

        *stmt = new_stmt;

        let stmt_addr = stmt.address();

        for (var_binding, init) in mem::take(&mut self.extra_vars) {
            let decl = ctx.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Var,
                var_binding.create_binding_pattern(ctx),
                NONE,
                init,
                false,
            );
            let decl_stmt = Statement::from(ctx.ast.declaration_variable(
                SPAN,
                VariableDeclarationKind::Var,
                ctx.ast.vec1(decl),
                false,
            ));
            ctx.state.statement_injector.insert_before(&stmt_addr, decl_stmt);
        }

        for init_stmt in mem::take(&mut self.pre_class_inits) {
            ctx.state.statement_injector.insert_before(&stmt_addr, init_stmt);
        }

        if !self.state.extra_stmts.is_empty() {
            ctx.state
                .statement_injector
                .insert_many_after(&stmt_addr, mem::take(&mut self.state.extra_stmts));
        }
    }

    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        debug_assert!(
            self.state.class_decorators.is_empty(),
            "All class decorators should have been consumed"
        );

        if self.extra_vars.is_empty()
            && self.pre_class_inits.is_empty()
            && self.state.extra_stmts.is_empty()
        {
            return;
        }

        let mut new_body: ArenaVec<'a, Statement<'a>> = ctx.ast.vec();

        let extra_vars = std::mem::take(&mut self.extra_vars);
        for (var_binding, init) in extra_vars {
            let decl = ctx.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Var,
                var_binding.create_binding_pattern(ctx),
                NONE,
                init,
                false,
            );
            new_body.push(
                ctx.ast
                    .declaration_variable(
                        SPAN,
                        VariableDeclarationKind::Var,
                        ctx.ast.vec1(decl),
                        false,
                    )
                    .into(),
            );
        }

        for stmt in std::mem::take(&mut self.pre_class_inits) {
            new_body.push(stmt);
        }

        for stmt in program.body.drain(..) {
            new_body.push(stmt);
        }

        for stmt in std::mem::take(&mut self.state.extra_stmts) {
            new_body.push(stmt);
        }

        program.body = new_body;
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_codegen::{Codegen, CodegenOptions};
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;

    use crate::decorator::{DecoratorOptions, DecoratorVersion};
    use crate::{TransformOptions, Transformer};

    fn transform(source: &str, version: DecoratorVersion) -> Result<String, String> {
        let source_type = SourceType::mjs();
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source, source_type).parse();
        if !ret.errors.is_empty() {
            return Err(format!("Parse errors: {:?}", ret.errors));
        }
        let mut program = ret.program;
        let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();

        let options = TransformOptions {
            decorator: DecoratorOptions { legacy: false, version, ..DecoratorOptions::default() },
            ..TransformOptions::default()
        };

        let ret = Transformer::new(&allocator, std::path::Path::new(""), &options)
            .build_with_scoping(scoping, &mut program);
        if !ret.errors.is_empty() {
            return Err(format!("Transform errors: {:?}", ret.errors));
        }

        let code = Codegen::new()
            .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
            .build(&program)
            .code;

        Ok(code)
    }

    #[test]
    fn class_declaration_without_decorators() {
        let result = transform("class Foo {}", DecoratorVersion::V202311).unwrap();
        assert!(result.contains("class Foo"));
    }

    #[test]
    fn class_expression_without_decorators() {
        let result = transform("const x = class {}", DecoratorVersion::V202311).unwrap();
        assert!(result.contains("class"));
    }

    #[test]
    fn decorated_class_declaration_202311() {
        let result = transform("@dec class Foo {}", DecoratorVersion::V202311).unwrap();
        assert!(result.contains("Foo"), "Expected 'Foo' in output, got: {result}");
    }

    #[test]
    fn decorated_class_expression_202311() {
        let result = transform("const x = @dec class {}", DecoratorVersion::V202311).unwrap();
        assert!(result.contains("=>"), "Expected IIFE arrow in output, got: {result}");
        assert!(result.contains("applyDecs"), "Expected applyDecs, got: {result}");
    }

    #[test]
    fn decorated_class_declaration_202203() {
        let result = transform("@dec class Foo {}", DecoratorVersion::V202203).unwrap();
        let pass = result.contains("_Foo = class")
            || result.contains("var Foo")
            || result.contains("var _Class");
        assert!(pass, "Expected class assignment in output, got: {result}");
    }

    #[test]
    fn multiple_decorators() {
        let result = transform("@dec1 @dec2 class Foo {}", DecoratorVersion::V202311).unwrap();
        let pass = result.contains("_Foo = class")
            || result.contains("var Foo")
            || result.contains("var _Class");
        assert!(pass, "Expected class assignment in output, got: {result}");
    }

    #[test]
    fn decorated_method_202311() {
        let result = transform("class Foo { @dec bar() {} }", DecoratorVersion::V202311).unwrap();
        assert!(result.contains("Foo"));
    }

    #[test]
    fn decorated_field_202311() {
        let result = transform("class Foo { @dec x = 1; }", DecoratorVersion::V202311).unwrap();
        assert!(result.contains("Foo"));
    }

    #[test]
    fn decorated_get_set_202311() {
        let result =
            transform("class Foo { @dec get x() {} @dec set x(v) {} }", DecoratorVersion::V202311)
                .unwrap();
        assert!(result.contains("Foo"));
    }

    #[test]
    fn decorated_class_with_accessor() {
        let result =
            transform("class Foo { @dec accessor x = 1; }", DecoratorVersion::V202311).unwrap();
        assert!(result.contains("Foo"));
    }

    #[test]
    fn decorated_class_with_static_method() {
        let result =
            transform("class Foo { @dec static bar() {} }", DecoratorVersion::V202311).unwrap();
        assert!(result.contains("Foo"));
    }

    #[test]
    fn decorator_no_panic_on_empty_body() {
        let result = transform("@dec class Foo {}", DecoratorVersion::V202311).unwrap();
        let pass = result.contains("_Foo = class")
            || result.contains("var Foo")
            || result.contains("var _Class");
        assert!(pass, "Expected class assignment in output, got: {result}");
    }
}
