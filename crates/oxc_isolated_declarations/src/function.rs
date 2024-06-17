#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use oxc_allocator::Box;
use oxc_ast::ast::Function;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::SPAN;

use crate::IsolatedDeclarations;

impl<'a> IsolatedDeclarations<'a> {
    pub fn transform_function(&mut self, func: &Function<'a>) -> Option<Box<'a, Function<'a>>> {
        if func.modifiers.is_contains_declare() {
            None
        } else {
            let return_type = self.infer_function_return_type(func);
            let params = self.transform_formal_parameters(&func.params);
            Some(self.ctx.ast.function(
                func.r#type,
                func.span,
                self.ctx.ast.copy(&func.id),
                func.generator,
                func.r#async,
                self.ctx.ast.copy(&func.this_param),
                params,
                None,
                self.ctx.ast.copy(&func.type_parameters),
                return_type,
                self.modifiers_declare(),
            ))
        }
    }

    pub fn transform_formal_parameter(
        &self,
        param: &FormalParameter<'a>,
        next_param: Option<&FormalParameter<'a>>,
    ) -> FormalParameter<'a> {
        let is_assignment_pattern = param.pattern.kind.is_assignment_pattern();
        let mut pattern =
            if let BindingPatternKind::AssignmentPattern(pattern) = &param.pattern.kind {
                self.ctx.ast.copy(&pattern.left)
            } else {
                self.ctx.ast.copy(&param.pattern)
            };

        if is_assignment_pattern || pattern.type_annotation.is_none() {
            let is_next_param_optional =
                next_param.map_or(true, |next_param| next_param.pattern.optional);

            let type_annotation = pattern
              .type_annotation
              .as_ref()
              .map(|type_annotation| self.ctx.ast.copy(&type_annotation.type_annotation))
              .or_else(|| {
                  // report error for has no type annotation
                  let new_type = self
                      .infer_type_from_formal_parameter(param)
                      .unwrap_or_else(|| self.ctx.ast.ts_unknown_keyword(param.span));
                  Some(new_type)
              })
              .map(|ts_type| {
                  // jf next param is not optional and current param is assignment pattern
                  // we need to add undefined to it's type
                  if !is_next_param_optional {
                      if matches!(ts_type, TSType::TSTypeReference(_)) {
                          self.ctx.error(
                              OxcDiagnostic::error("Declaration emit for this parameter requires implicitly adding undefined to it's type. This is not supported with --isolatedDeclarations.")
                                  .with_label(param.span),
                          );
                      } else if !ts_type.is_maybe_undefined() {
                          // union with undefined
                          return self.ctx.ast.ts_type_annotation(SPAN,
                              self.ctx.ast.ts_union_type(SPAN, self.ctx.ast.new_vec_from_iter([ts_type, self.ctx.ast.ts_undefined_keyword(SPAN)]))
                          );
                      }
                  }

                  self.ctx.ast.ts_type_annotation(SPAN, ts_type)
              });

            pattern = self.ctx.ast.binding_pattern(
                self.ctx.ast.copy(&pattern.kind),
                type_annotation,
                // if it's assignment pattern, it's optional
                pattern.optional || (is_next_param_optional && is_assignment_pattern),
            );
        }

        self.ctx.ast.formal_parameter(
            param.span,
            pattern,
            None,
            param.readonly,
            false,
            self.ctx.ast.new_vec(),
        )
    }

    pub fn transform_formal_parameters(
        &self,
        params: &FormalParameters<'a>,
    ) -> Box<'a, FormalParameters<'a>> {
        if params.kind.is_signature() || (params.rest.is_none() && params.items.is_empty()) {
            return self.ctx.ast.alloc(self.ctx.ast.copy(params));
        }

        let items =
            self.ctx.ast.new_vec_from_iter(params.items.iter().enumerate().map(|(index, item)| {
                self.transform_formal_parameter(item, params.items.get(index + 1))
            }));

        if let Some(rest) = &params.rest {
            if rest.argument.type_annotation.is_none() {
                self.ctx.error(OxcDiagnostic::error(
                  "Parameter must have an explicit type annotation with --isolatedDeclarations.",
              ).with_label(rest.span));
            }
        }

        self.ctx.ast.formal_parameters(
            params.span,
            FormalParameterKind::Signature,
            items,
            self.ctx.ast.copy(&params.rest),
        )
    }
}
