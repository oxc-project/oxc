use super::detector_utils::{
    PrimitiveType, can_change_strict_to_loose, extract_member_expr_chain, is_primitive_literal,
    is_side_effect_free_member_expr_of_len_three, is_side_effect_free_member_expr_of_len_two,
    is_side_effect_free_unbound_identifier_ref, is_well_known_global_ident_ref,
    known_primitive_type, maybe_side_effect_free_global_constructor,
    maybe_side_effect_free_global_function_call,
};
use super::{
    MayHaveSideEffectsContext, PropertyReadSideEffects, PropertyWriteSideEffects, SideEffectDetail,
};
use bitflags::bitflags;
use oxc_ast::ast::{
    self, Argument, ArrayExpressionElement, AssignmentTarget, BindingPattern, CallExpression,
    ChainElement, Expression, IdentifierReference, PropertyKey, UnaryOperator,
    VariableDeclarationKind,
};
use oxc_ast::{match_expression, match_member_expression};
use oxc_span::Ident;

bitflags! {
  #[derive(Debug, Clone, Copy)]
  /// Property could be read and write at the same time, so we need to distinguish them. e.g.
  /// For a UpdateExpr `obj.prop++`, it is both read and write.
  pub struct PropertyAccessFlag: u8 {
    const Read = 1 << 0;
    const Write = 1 << 1;
  }
}

/// Detect if a statement "may" have side effect.
pub struct SideEffectDetector<'ctx, 'a, Ctx: MayHaveSideEffectsContext<'a>> {
    ctx: &'ctx Ctx,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'ctx, 'a, Ctx: MayHaveSideEffectsContext<'a>> SideEffectDetector<'ctx, 'a, Ctx> {
    pub fn new(ctx: &'ctx Ctx) -> Self {
        Self { ctx, _marker: std::marker::PhantomData }
    }

    #[inline]
    fn is_unresolved_reference(&self, ident_ref: &IdentifierReference<'a>) -> bool {
        self.ctx.is_global_reference(ident_ref)
    }

    fn detect_side_effect_of_property_key(
        &self,
        key: &PropertyKey<'a>,
        is_computed: bool,
    ) -> SideEffectDetail {
        match key {
            PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_) => false.into(),
            key @ match_expression!(PropertyKey) => (is_computed && {
                let key_expr = key.to_expression();
                match key_expr {
                    match_member_expression!(Expression) => {
                        if let Some((ident_ref, chain)) =
                            extract_member_expr_chain(key_expr.to_member_expression(), 2)
                        {
                            !(chain == ["Symbol", "iterator"]
                                && self.is_unresolved_reference(ident_ref))
                        } else {
                            true
                        }
                    }
                    _ => !is_primitive_literal(self.ctx, key_expr),
                }
            })
            .into(),
        }
    }

    /// ref: https://github.com/evanw/esbuild/blob/360d47230813e67d0312ad754cad2b6ee09b151b/internal/js_ast/js_ast_helpers.go#L2298-L2393
    fn detect_side_effect_of_class(&self, cls: &ast::Class<'a>) -> SideEffectDetail {
        use oxc_ast::ast::ClassElement;
        if !cls.decorators.is_empty() {
            return true.into();
        }
        cls.body
            .body
            .iter()
            .any(|elm| match elm {
                ClassElement::StaticBlock(static_block) => static_block
                    .body
                    .iter()
                    .any(|stmt| self.detect_side_effect_of_stmt(stmt).has_side_effect()),
                ClassElement::MethodDefinition(def) => {
                    if !def.decorators.is_empty() {
                        return true;
                    }
                    if self
                        .detect_side_effect_of_property_key(&def.key, def.computed)
                        .has_side_effect()
                    {
                        return true;
                    }

                    def.value.params.items.iter().any(|item| !item.decorators.is_empty())
                }
                ClassElement::PropertyDefinition(def) => {
                    if !def.decorators.is_empty() {
                        return true;
                    }
                    if self
                        .detect_side_effect_of_property_key(&def.key, def.computed)
                        .has_side_effect()
                    {
                        return true;
                    }

                    def.r#static
                        && def.value.as_ref().is_some_and(|init| {
                            self.detect_side_effect_of_expr(init).has_side_effect()
                        })
                }
                ClassElement::AccessorProperty(def) => {
                    if !def.decorators.is_empty() {
                        return true;
                    }
                    (match &def.key {
                        PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_) => {
                            false
                        }
                        key @ match_expression!(PropertyKey) => {
                            self.detect_side_effect_of_expr(key.to_expression()).has_side_effect()
                        }
                    } || def.value.as_ref().is_some_and(|init| {
                        self.detect_side_effect_of_expr(init).has_side_effect()
                    }))
                }
                ClassElement::TSIndexSignature(_) => true,
            })
            .into()
    }

    fn detect_side_effect_of_computed_member_expr(
        &self,
        expr: &ast::ComputedMemberExpression<'a>,
        property_access_kind: PropertyAccessFlag,
    ) -> SideEffectDetail {
        let mut property_access_side_effects = false;
        if property_access_kind.contains(PropertyAccessFlag::Read) {
            property_access_side_effects |=
                self.ctx.property_read_side_effects() != PropertyReadSideEffects::None;
        }
        if property_access_kind.contains(PropertyAccessFlag::Write) {
            property_access_side_effects |=
                self.ctx.property_write_side_effects() != PropertyWriteSideEffects::None;
        }

        let mut side_effects_detail = SideEffectDetail::empty();
        let max_len = 3;
        let mut chains = vec![];
        if let ast::Expression::StringLiteral(ref str) = expr.expression {
            chains.push(str.value.into());
        } else {
            side_effects_detail |= self.detect_side_effect_of_expr(&expr.expression);
        }
        let cur = &expr.object;
        self.common_member_chain_processing(
            property_access_side_effects,
            &mut side_effects_detail,
            max_len,
            &mut chains,
            cur,
            property_access_kind,
        );
        side_effects_detail
    }

    fn detect_side_effect_of_static_member_expr(
        &self,
        expr: &ast::StaticMemberExpression<'a>,
        property_access_kind: PropertyAccessFlag,
    ) -> SideEffectDetail {
        let mut property_access_side_effects = false;
        if property_access_kind.contains(PropertyAccessFlag::Read) {
            property_access_side_effects |=
                self.ctx.property_read_side_effects() != PropertyReadSideEffects::None;
        }
        if property_access_kind.contains(PropertyAccessFlag::Write) {
            property_access_side_effects |=
                self.ctx.property_write_side_effects() != PropertyWriteSideEffects::None;
        }

        let mut side_effects_detail = SideEffectDetail::empty();
        let max_len = 3;
        let mut chains = vec![expr.property.name];
        let cur = &expr.object;
        self.common_member_chain_processing(
            property_access_side_effects,
            &mut side_effects_detail,
            max_len,
            &mut chains,
            cur,
            property_access_kind,
        );
        side_effects_detail
    }

    fn detect_side_effect_of_member_expr(
        &self,
        expr: &ast::MemberExpression<'a>,
        property_access_kind: PropertyAccessFlag,
    ) -> SideEffectDetail {
        if self.is_expr_manual_pure_functions(expr.object()) {
            return false.into();
        }

        match expr {
            ast::MemberExpression::ComputedMemberExpression(computed_expr) => {
                self.detect_side_effect_of_computed_member_expr(computed_expr, property_access_kind)
            }
            ast::MemberExpression::StaticMemberExpression(static_expr) => {
                self.detect_side_effect_of_static_member_expr(static_expr, property_access_kind)
            }
            ast::MemberExpression::PrivateFieldExpression(_) => true.into(),
        }
    }

    fn common_member_chain_processing(
        &self,
        property_access_side_effects: bool,
        side_effects_detail: &mut SideEffectDetail,
        max_len: usize,
        chains: &mut Vec<Ident<'a>>,
        mut cur: &Expression<'a>,
        property_access_flag: PropertyAccessFlag,
    ) {
        loop {
            match cur {
                ast::Expression::StaticMemberExpression(expr) => {
                    cur = &expr.object;
                    chains.push(expr.property.name);
                }
                ast::Expression::ComputedMemberExpression(computed_expr) => {
                    if let ast::Expression::StringLiteral(ref str) = computed_expr.expression {
                        chains.push(str.value.into());
                    } else {
                        *side_effects_detail |=
                            self.detect_side_effect_of_expr(&computed_expr.expression);
                    }
                    cur = &computed_expr.object;
                }
                ast::Expression::Identifier(ident_ref) => {
                    chains.push(ident_ref.name);
                    chains.reverse();
                    side_effects_detail.set(
                        SideEffectDetail::GlobalVarAccess,
                        self.is_unresolved_reference(ident_ref),
                    );
                    break;
                }
                ast::Expression::MetaProperty(_) => {
                    // Only `import.meta.url` is a spec-defined side-effect-free property read.
                    // Other accesses like `import.meta.hot.accept()` may have side effects.
                    if chains.len() == 1 && chains[0] == "url" {
                        return;
                    }
                    break;
                }
                _ => {
                    *side_effects_detail |= self.detect_side_effect_of_expr(cur);
                    break;
                }
            }
            if chains.len() >= max_len && property_access_side_effects {
                *side_effects_detail = true.into();
                return;
            }
        }

        if property_access_flag.contains(PropertyAccessFlag::Write)
            && side_effects_detail.contains(SideEffectDetail::GlobalVarAccess)
        {
            // If it is a write operation on a global variable, we consider it has side effect.
            *side_effects_detail |= true.into();
            return;
        }
        if !property_access_side_effects {
            return;
        }
        *side_effects_detail |= (match chains.len() {
            2 => !is_side_effect_free_member_expr_of_len_two(chains),
            3 => !is_side_effect_free_member_expr_of_len_three(chains),
            _ => true,
        })
        .into();
    }

    fn detect_side_effect_of_assignment_target(
        &self,
        expr: &AssignmentTarget<'a>,
    ) -> SideEffectDetail {
        match expr {
            AssignmentTarget::ComputedMemberExpression(_)
            | AssignmentTarget::StaticMemberExpression(_) => {
                let member_expr = expr.to_member_expression();
                match member_expr.object() {
                    Expression::Identifier(ident) => {
                        // - exports.a = ...;
                        // - exports['a'] = ...;
                        if self.is_unresolved_reference(ident)
                            && ident.name == "exports"
                            && member_expr.static_property_name().is_some()
                        {
                            SideEffectDetail::PureCjs
                        } else if self.ctx.property_write_side_effects()
                            != PropertyWriteSideEffects::None
                        {
                            true.into()
                        } else {
                            self.detect_side_effect_of_member_expr(
                                member_expr,
                                PropertyAccessFlag::Write,
                            )
                        }
                    }
                    _ => {
                        if self.ctx.property_write_side_effects() != PropertyWriteSideEffects::None
                        {
                            true.into()
                        } else {
                            self.detect_side_effect_of_member_expr(
                                member_expr,
                                PropertyAccessFlag::Write,
                            )
                        }
                    }
                }
            }

            AssignmentTarget::AssignmentTargetIdentifier(_)
            | AssignmentTarget::PrivateFieldExpression(_)
            | AssignmentTarget::TSAsExpression(_)
            | AssignmentTarget::TSSatisfiesExpression(_)
            | AssignmentTarget::TSNonNullExpression(_)
            | AssignmentTarget::TSTypeAssertion(_) => true.into(),

            AssignmentTarget::ArrayAssignmentTarget(array_pattern) => {
                (!array_pattern.elements.is_empty() || array_pattern.rest.is_some()).into()
            }
            AssignmentTarget::ObjectAssignmentTarget(object_pattern) => {
                (!object_pattern.properties.is_empty() || object_pattern.rest.is_some()).into()
            }
        }
    }

    fn detect_side_effect_of_call_expr(&self, expr: &CallExpression<'a>) -> SideEffectDetail {
        if self.is_expr_manual_pure_functions(&expr.callee) {
            return false.into();
        }

        // TODO: with cjs tree shaking remove this may cause some runtime behavior incorrect.
        // But marking `Object.defineProperty(exports, "__esModule", { value: true })` as has side effect may incraese bundle size a little.
        // if is_object_define_property_es_module(self.scope, expr).unwrap_or_default() {
        //   return StmtSideEffect::Unknown;
        // }

        let is_pure =
            self.ctx.annotations() && (expr.pure || self.ctx.is_pure_call_expression(expr));
        if is_pure {
            // Even it is pure, we also wants to know if the callee has access global var
            // But we need to ignore the `Unknown` flag, since it is already marked as `pure`.
            let mut detail = SideEffectDetail::PureAnnotation;
            detail |= self.detect_side_effect_of_expr(&expr.callee) - SideEffectDetail::Unknown;
            for arg in &expr.arguments {
                detail |= match arg {
                    Argument::SpreadElement(_) => true.into(),
                    _ => self.detect_side_effect_of_expr(arg.to_expression()),
                };
                if detail.has_side_effect() {
                    break;
                }
            }
            detail
        } else {
            let is_side_effect_free_global_function =
                maybe_side_effect_free_global_function_call(self.ctx, expr);
            if is_side_effect_free_global_function {
                SideEffectDetail::GlobalVarAccess
            } else {
                true.into()
            }
        }
    }

    fn is_expr_manual_pure_functions(&self, expr: &Expression<'a>) -> bool {
        self.ctx.manual_pure_functions(expr)
    }

    #[expect(clippy::too_many_lines)]
    pub fn detect_side_effect_of_expr(&self, expr: &Expression<'a>) -> SideEffectDetail {
        match expr {
      Expression::BooleanLiteral(_)
      | Expression::NullLiteral(_)
      | Expression::NumericLiteral(_)
      | Expression::BigIntLiteral(_)
      | Expression::RegExpLiteral(_)
      | Expression::FunctionExpression(_)
      | Expression::ArrowFunctionExpression(_)
      | Expression::MetaProperty(_)
      | Expression::ThisExpression(_)
      | Expression::StringLiteral(_) => false.into(),
      Expression::ObjectExpression(obj_expr) => {
        let mut detail = SideEffectDetail::empty();
        for obj_prop in &obj_expr.properties {
          detail |= match obj_prop {
            ast::ObjectPropertyKind::ObjectProperty(prop) => {
              self.detect_side_effect_of_property_key(&prop.key, prop.computed)
                | self.detect_side_effect_of_expr(&prop.value)
            }
            // refer https://github.com/rollup/rollup/blob/f7633942/src/ast/nodes/SpreadElement.ts#L32
            ast::ObjectPropertyKind::SpreadProperty(res) => {
              if self.ctx.property_read_side_effects() != PropertyReadSideEffects::None {
                return true.into();
              }
              self.detect_side_effect_of_expr(&res.argument)
            }
          };
          if detail.has_side_effect() {
            break;
          }
        }
        detail
      }
      // https://github.com/evanw/esbuild/blob/d34e79e2a998c21bb71d57b92b0017ca11756912/internal/js_ast/js_ast_helpers.go#L2533-L2539
      Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
        ast::UnaryOperator::Typeof if matches!(unary_expr.argument, Expression::Identifier(_)) => {
          false.into()
        }
        _ => self.detect_side_effect_of_expr(&unary_expr.argument),
      },
      oxc_ast::match_member_expression!(Expression) => self
        .detect_side_effect_of_member_expr(expr.to_member_expression(), PropertyAccessFlag::Read),
      Expression::ClassExpression(cls) => self.detect_side_effect_of_class(cls),
      // Accessing global variables considered as side effect.
      Expression::Identifier(ident) => self.detect_side_effect_of_identifier(ident),
      // https://github.com/evanw/esbuild/blob/360d47230813e67d0312ad754cad2b6ee09b151b/internal/js_ast/js_ast_helpers.go#L2576-L2588
      Expression::TemplateLiteral(literal) => {
        let mut detail = SideEffectDetail::empty();
        for expr in &literal.expressions {
          // Primitive type detection is more strict and faster than side_effects detection of
          // `Expr`, put it first to fail fast.
          detail |= (known_primitive_type(self.ctx, expr) == PrimitiveType::Unknown).into();
          detail |= self.detect_side_effect_of_expr(expr);
          if detail.has_side_effect() {
            break;
          }
        }
        detail
      }
      Expression::LogicalExpression(logic_expr) => match logic_expr.operator {
        ast::LogicalOperator::Or => {
          let lhs = self.detect_side_effect_of_expr(&logic_expr.left);
          let mut rhs = self.detect_side_effect_of_expr(&logic_expr.right);
          rhs.set(
            SideEffectDetail::Unknown,
            !is_side_effect_free_unbound_identifier_ref(
              self.ctx,
              &logic_expr.right,
              &logic_expr.left,
              false,
            )
            .unwrap_or_default()
              && rhs.contains(SideEffectDetail::Unknown),
          );
          lhs | rhs
        }
        ast::LogicalOperator::And => {
          let lhs = self.detect_side_effect_of_expr(&logic_expr.left);
          let mut rhs = self.detect_side_effect_of_expr(&logic_expr.right);
          rhs.set(
            SideEffectDetail::Unknown,
            !is_side_effect_free_unbound_identifier_ref(
              self.ctx,
              &logic_expr.right,
              &logic_expr.left,
              true,
            )
            .unwrap_or_default()
              && rhs.contains(SideEffectDetail::Unknown),
          );
          lhs | rhs
        }
        ast::LogicalOperator::Coalesce => {
          self.detect_side_effect_of_expr(&logic_expr.left)
            | self.detect_side_effect_of_expr(&logic_expr.right)
        }
      },
      Expression::ParenthesizedExpression(paren_expr) => {
        self.detect_side_effect_of_expr(&paren_expr.expression)
      }
      Expression::SequenceExpression(seq_expr) => {
        let mut detail = SideEffectDetail::empty();

        for expr in &seq_expr.expressions {
          detail |= self.detect_side_effect_of_expr(expr);
          if detail.has_side_effect() {
            break;
          }
        }
        detail
      }
      // https://github.com/evanw/esbuild/blob/d34e79e2a998c21bb71d57b92b0017ca11756912/internal/js_ast/js_ast_helpers.go#L2460-L2463
      Expression::ConditionalExpression(cond_expr) => {
        let detail = self.detect_side_effect_of_expr(&cond_expr.test);
        let mut consequent_detail = self.detect_side_effect_of_expr(&cond_expr.consequent);
        consequent_detail.set(
          SideEffectDetail::Unknown,
          !is_side_effect_free_unbound_identifier_ref(
            self.ctx,
            &cond_expr.consequent,
            &cond_expr.test,
            true,
          )
          .unwrap_or_default()
            && consequent_detail.contains(SideEffectDetail::Unknown),
        );
        let mut alternate_detail = self.detect_side_effect_of_expr(&cond_expr.alternate);
        alternate_detail.set(
          SideEffectDetail::Unknown,
          !is_side_effect_free_unbound_identifier_ref(
            self.ctx,
            &cond_expr.alternate,
            &cond_expr.test,
            false,
          )
          .unwrap_or_default()
            && alternate_detail.contains(SideEffectDetail::Unknown),
        );
        detail | consequent_detail | alternate_detail
      }
      // Untranspiled TS/JSX syntax should be caught during scan stage.
      // Conservatively treat as side-effectful since they should not appear here.
      Expression::TSAsExpression(_)
      | Expression::TSSatisfiesExpression(_)
      | Expression::TSTypeAssertion(_)
      | Expression::TSNonNullExpression(_)
      | Expression::TSInstantiationExpression(_)
      | Expression::JSXElement(_)
      | Expression::JSXFragment(_)
      // Inherently side-effectful expressions.
      | Expression::Super(_)
      | Expression::AwaitExpression(_)
      | Expression::ImportExpression(_)
      | Expression::YieldExpression(_)
      | Expression::V8IntrinsicExpression(_) => true.into(),
      // https://github.com/evanw/esbuild/blob/d34e79e2a998c21bb71d57b92b0017ca11756912/internal/js_ast/js_ast_helpers.go#L2541-L2574
      Expression::BinaryExpression(binary_expr) => {
        match binary_expr.operator {
          ast::BinaryOperator::StrictEquality | ast::BinaryOperator::StrictInequality => {
            self.detect_side_effect_of_expr(&binary_expr.left)
              | self.detect_side_effect_of_expr(&binary_expr.right)
          }
          // Special-case "<" and ">" with string, number, or bigint arguments
          ast::BinaryOperator::GreaterThan
          | ast::BinaryOperator::LessThan
          | ast::BinaryOperator::GreaterEqualThan
          | ast::BinaryOperator::LessEqualThan => {
            let lt = known_primitive_type(self.ctx, &binary_expr.left);
            match lt {
              PrimitiveType::Number | PrimitiveType::String | PrimitiveType::BigInt => {
                SideEffectDetail::from(known_primitive_type(self.ctx, &binary_expr.right) != lt)
                  | self.detect_side_effect_of_expr(&binary_expr.left)
                  | self.detect_side_effect_of_expr(&binary_expr.right)
              }
              _ => true.into(),
            }
          }

          // For "==" and "!=", pretend the operator was actually "===" or "!==". If
          // we know that we can convert it to "==" or "!=", then we can consider the
          // operator itself to have no side effects. This matters because our mangle
          // logic will convert "typeof x === 'object'" into "typeof x == 'object'"
          // and since "typeof x === 'object'" is considered to be side-effect free,
          // we must also consider "typeof x == 'object'" to be side-effect free.
          ast::BinaryOperator::Equality | ast::BinaryOperator::Inequality => {
            SideEffectDetail::from(!can_change_strict_to_loose(
              self.ctx,
              &binary_expr.left,
              &binary_expr.right,
            )) | self.detect_side_effect_of_expr(&binary_expr.left)
              | self.detect_side_effect_of_expr(&binary_expr.right)
          }

          _ => true.into(),
        }
      }
      Expression::PrivateInExpression(private_in_expr) => {
        self.detect_side_effect_of_expr(&private_in_expr.right)
      }
      Expression::AssignmentExpression(expr) => {
        self.detect_side_effect_of_assignment_target(&expr.left)
          | self.detect_side_effect_of_expr(&expr.right)
      }

      Expression::ChainExpression(expr) => match &expr.expression {
        ChainElement::CallExpression(call_expr) => self.detect_side_effect_of_call_expr(call_expr),
        ChainElement::TSNonNullExpression(expr) => {
          self.detect_side_effect_of_expr(&expr.expression)
        }
        match_member_expression!(ChainElement) => self.detect_side_effect_of_member_expr(
          expr.expression.to_member_expression(),
          PropertyAccessFlag::Read,
        ),
      },
      Expression::TaggedTemplateExpression(expr) => {
        (!self.is_expr_manual_pure_functions(&expr.tag)).into()
      }
      Expression::UpdateExpression(expr) => {
        // Handle update expressions like obj.prop++ or obj[prop]++
        match &expr.argument {
          ast::SimpleAssignmentTarget::StaticMemberExpression(static_member_expr) => {
            if self.ctx.property_write_side_effects() != PropertyWriteSideEffects::None {
              true.into()
            } else {
              // If property_write_side_effects is false, we consider property updates
              // as side-effect-free

              self.detect_side_effect_of_static_member_expr(
                static_member_expr,
                PropertyAccessFlag::all(),
              )
            }
          }
          ast::SimpleAssignmentTarget::ComputedMemberExpression(computed_expr) => self
            .detect_side_effect_of_computed_member_expr(computed_expr, PropertyAccessFlag::all()),
          _ => true.into(),
        }
      }
      Expression::ArrayExpression(expr) => self.detect_side_effect_of_array_expr(expr),
      Expression::NewExpression(expr) => {
        let is_side_effect_free_global_constructor =
          maybe_side_effect_free_global_constructor(self.ctx, expr);
        let is_pure = expr.pure || is_side_effect_free_global_constructor;

        let mut detail = SideEffectDetail::empty();
        detail.set(SideEffectDetail::GlobalVarAccess, is_side_effect_free_global_constructor);
        detail.set(SideEffectDetail::Unknown, !is_pure);
        detail.set(SideEffectDetail::PureAnnotation, expr.pure);

        for arg in &expr.arguments {
          detail |= match arg {
            Argument::SpreadElement(_) => true.into(),
            _ => self.detect_side_effect_of_expr(arg.to_expression()),
          };
          if detail.has_side_effect() {
            break;
          }
        }
        detail
      }
      Expression::CallExpression(expr) => self.detect_side_effect_of_call_expr(expr),
    }
    }

    fn detect_side_effect_of_array_expr(
        &self,
        expr: &ast::ArrayExpression<'a>,
    ) -> SideEffectDetail {
        let mut detail = SideEffectDetail::empty();
        for elem in &expr.elements {
            let cur = match elem {
                ArrayExpressionElement::SpreadElement(ele) => {
                    // https://github.com/evanw/esbuild/blob/d34e79e2a998c21bb71d57b92b0017ca11756912/internal/js_ast/js_ast_helpers.go#L2466-L2477
                    // Spread of an inline array such as "[...[x]]" is side-effect free
                    match &ele.argument {
                        Expression::ArrayExpression(arr) => {
                            self.detect_side_effect_of_array_expr(arr)
                        }
                        _ => return true.into(),
                    }
                }
                ArrayExpressionElement::Elision(_) => false.into(),
                match_expression!(ArrayExpressionElement) => {
                    self.detect_side_effect_of_expr(elem.to_expression())
                }
            };
            detail |= cur;
        }
        detail
    }

    fn detect_side_effect_of_var_decl(
        &self,
        var_decl: &ast::VariableDeclaration<'a>,
    ) -> SideEffectDetail {
        match var_decl.kind {
            VariableDeclarationKind::AwaitUsing => true.into(),
            VariableDeclarationKind::Using => {
                self.detect_side_effect_of_using_declarators(&var_decl.declarations)
            }
            _ => {
                let mut detail = SideEffectDetail::empty();
                for declarator in &var_decl.declarations {
                    // Whether to destructure import.meta
                    if let BindingPattern::ObjectPattern(ref obj_pat) = declarator.id {
                        if !obj_pat.properties.is_empty() {
                            if let Some(Expression::MetaProperty(_)) = declarator.init {
                                return true.into();
                            }
                        }
                    }
                    detail |= match &declarator.id {
                        // Destructuring the initializer has no side effects if the
                        // initializer is an array, since we assume the iterator is then
                        // the built-in side-effect free array iterator.
                        BindingPattern::ObjectPattern(_) => {
                            // Object destructuring only has side effects when property_read_side_effects is Always
                            if self.ctx.property_read_side_effects()
                                != PropertyReadSideEffects::None
                            {
                                true.into()
                            } else {
                                declarator
                                    .init
                                    .as_ref()
                                    .map(|init| self.detect_side_effect_of_expr(init))
                                    .unwrap_or(false.into())
                            }
                        }
                        BindingPattern::ArrayPattern(pat) => {
                            for p in &pat.elements {
                                if p.as_ref().is_some_and(|pat| {
                                    !matches!(pat, BindingPattern::BindingIdentifier(_))
                                }) {
                                    return true.into();
                                }
                            }
                            declarator
                                .init
                                .as_ref()
                                .map(|init| self.detect_side_effect_of_expr(init))
                                .unwrap_or(false.into())
                        }
                        BindingPattern::BindingIdentifier(_)
                        | BindingPattern::AssignmentPattern(_) => declarator
                            .init
                            .as_ref()
                            .map(|init| self.detect_side_effect_of_expr(init))
                            .unwrap_or(false.into()),
                    };
                }
                detail
            }
        }
    }

    fn detect_side_effect_of_decl(&self, decl: &ast::Declaration<'a>) -> SideEffectDetail {
        use oxc_ast::ast::Declaration;
        match decl {
            Declaration::VariableDeclaration(var_decl) => {
                self.detect_side_effect_of_var_decl(var_decl)
            }
            Declaration::FunctionDeclaration(_) => false.into(),
            Declaration::ClassDeclaration(cls_decl) => self.detect_side_effect_of_class(cls_decl),
            Declaration::TSTypeAliasDeclaration(_)
            | Declaration::TSInterfaceDeclaration(_)
            | Declaration::TSEnumDeclaration(_)
            | Declaration::TSModuleDeclaration(_)
            | Declaration::TSImportEqualsDeclaration(_)
            | Declaration::TSGlobalDeclaration(_) => true.into(),
        }
    }

    fn detect_side_effect_of_using_declarators(
        &self,
        declarators: &[ast::VariableDeclarator<'a>],
    ) -> SideEffectDetail {
        let mut detail = SideEffectDetail::empty();
        for decl in declarators {
            detail |= decl
                .init
                .as_ref()
                .map(|init| match init {
                    Expression::NullLiteral(_) => false.into(),
                    // Side effect detection of identifier is different with other position when as initialization of using declaration.
                    // Global variable `undefined` is considered as side effect free.
                    Expression::Identifier(id) => {
                        (!(id.name == "undefined" && self.is_unresolved_reference(id))).into()
                    }
                    Expression::UnaryExpression(expr)
                        if matches!(expr.operator, UnaryOperator::Void) =>
                    {
                        self.detect_side_effect_of_expr(&expr.argument)
                    }
                    _ => true.into(),
                })
                .unwrap_or(SideEffectDetail::empty());
            if detail.has_side_effect() {
                break;
            }
        }
        detail
    }

    fn detect_side_effect_of_identifier(
        &self,
        ident_ref: &IdentifierReference<'a>,
    ) -> SideEffectDetail {
        let mut detail = SideEffectDetail::empty();
        detail.set(SideEffectDetail::GlobalVarAccess, self.is_unresolved_reference(ident_ref));
        if detail.contains(SideEffectDetail::GlobalVarAccess) {
            detail.set(
                SideEffectDetail::Unknown,
                detail.contains(SideEffectDetail::GlobalVarAccess)
                    && self.ctx.unknown_global_side_effects()
                    && !is_well_known_global_ident_ref(ident_ref.name.as_str()),
            );
        }
        detail
    }

    pub fn detect_side_effect_of_stmt(&self, stmt: &ast::Statement<'a>) -> SideEffectDetail {
        use oxc_ast::ast::Statement;
        match stmt {
            oxc_ast::match_declaration!(Statement) => {
                self.detect_side_effect_of_decl(stmt.to_declaration())
            }
            Statement::ExpressionStatement(expr) => {
                self.detect_side_effect_of_expr(&expr.expression)
            }
            oxc_ast::match_module_declaration!(Statement) => match stmt.to_module_declaration() {
                ast::ModuleDeclaration::ExportAllDeclaration(_)
                | ast::ModuleDeclaration::ImportDeclaration(_) => {
                    // We consider `import ...` has no side effect. However, `import ...` might be rewritten to other statements by the bundler.
                    // In that case, we will mark the statement as having side effect in link stage.
                    false.into()
                }
                ast::ModuleDeclaration::ExportDefaultDeclaration(default_decl) => {
                    use oxc_ast::ast::ExportDefaultDeclarationKind;
                    match &default_decl.declaration {
                        decl @ match_expression!(ExportDefaultDeclarationKind) => {
                            self.detect_side_effect_of_expr(decl.to_expression())
                        }
                        ast::ExportDefaultDeclarationKind::FunctionDeclaration(_) => false.into(),
                        ast::ExportDefaultDeclarationKind::ClassDeclaration(decl) => {
                            self.detect_side_effect_of_class(decl)
                        }
                        ast::ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => true.into(),
                    }
                }
                ast::ModuleDeclaration::ExportNamedDeclaration(named_decl) => {
                    if named_decl.source.is_some() {
                        false.into()
                    } else {
                        named_decl
                            .declaration
                            .as_ref()
                            .map(|decl| self.detect_side_effect_of_decl(decl))
                            .unwrap_or(false.into())
                    }
                }
                ast::ModuleDeclaration::TSExportAssignment(_)
                | ast::ModuleDeclaration::TSNamespaceExportDeclaration(_) => true.into(),
            },
            Statement::BlockStatement(block) => self.detect_side_effect_of_block(block),
            Statement::DoWhileStatement(do_while) => {
                self.detect_side_effect_of_stmt(&do_while.body)
                    | self.detect_side_effect_of_expr(&do_while.test)
            }
            Statement::WhileStatement(while_stmt) => {
                self.detect_side_effect_of_expr(&while_stmt.test)
                    | self.detect_side_effect_of_stmt(&while_stmt.body)
            }
            Statement::IfStatement(if_stmt) => {
                self.detect_side_effect_of_expr(&if_stmt.test)
                    | self.detect_side_effect_of_stmt(&if_stmt.consequent)
                    | if_stmt
                        .alternate
                        .as_ref()
                        .map(|stmt| self.detect_side_effect_of_stmt(stmt))
                        .unwrap_or(false.into())
            }
            Statement::ReturnStatement(ret_stmt) => ret_stmt
                .argument
                .as_ref()
                .map(|expr| self.detect_side_effect_of_expr(expr))
                .unwrap_or(false.into()),
            Statement::LabeledStatement(labeled_stmt) => {
                self.detect_side_effect_of_stmt(&labeled_stmt.body)
            }
            Statement::TryStatement(try_stmt) => {
                let mut detail = self.detect_side_effect_of_block(&try_stmt.block);
                detail |= try_stmt
                    .handler
                    .as_ref()
                    .map(|handler| self.detect_side_effect_of_block(&handler.body))
                    .unwrap_or(SideEffectDetail::empty());
                detail |= try_stmt
                    .finalizer
                    .as_ref()
                    .map(|finalizer| self.detect_side_effect_of_block(finalizer))
                    .unwrap_or(SideEffectDetail::empty());
                detail
            }
            Statement::SwitchStatement(switch_stmt) => {
                let mut detail = self.detect_side_effect_of_expr(&switch_stmt.discriminant);
                if detail.has_side_effect() {
                    return detail;
                }
                'outer: for case in &switch_stmt.cases {
                    detail |= case
                        .test
                        .as_ref()
                        .map(|expr| self.detect_side_effect_of_expr(expr))
                        .unwrap_or(SideEffectDetail::empty());
                    for stmt in &case.consequent {
                        detail |= self.detect_side_effect_of_stmt(stmt);
                        if detail.has_side_effect() {
                            break 'outer;
                        }
                    }

                    if detail.has_side_effect() {
                        break;
                    }
                }
                detail
            }

            Statement::EmptyStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::BreakStatement(_) => false.into(),

            Statement::DebuggerStatement(_)
            | Statement::ForInStatement(_)
            | Statement::ForOfStatement(_)
            | Statement::ForStatement(_)
            | Statement::ThrowStatement(_)
            | Statement::WithStatement(_) => true.into(),
        }
    }

    fn detect_side_effect_of_block(&self, block: &ast::BlockStatement<'a>) -> SideEffectDetail {
        let mut detail = SideEffectDetail::empty();
        for stmt in &block.body {
            detail |= self.detect_side_effect_of_stmt(stmt);
            if detail.has_side_effect() {
                break;
            }
        }
        detail
    }
}
