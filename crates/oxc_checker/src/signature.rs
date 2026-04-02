use oxc_ast::ast::{BindingPattern, Function, Statement};
use oxc_span::CompactStr;
use oxc_types::{ParameterInfo, Signature, SignatureFlags, TypeId};
use smallvec::SmallVec;

use crate::checker::CheckMode;
use crate::Checker;

impl Checker<'_> {
    /// Build a Signature from a function's formal parameters and return type.
    ///
    /// Used for function declarations, function expressions, and arrow functions.
    /// When there is no return type annotation, infers the return type from the body.
    pub fn build_signature_from_function(&mut self, func: &Function<'_>) -> Signature {
        self.build_signature_from_function_with_context(func, None)
    }

    /// Build a Signature from formal parameters and an optional return type annotation.
    pub fn build_signature_from_params(
        &mut self,
        params: &oxc_ast::ast::FormalParameters<'_>,
        return_type_ann: Option<&oxc_ast::ast::TSTypeAnnotation<'_>>,
    ) -> Signature {
        self.build_signature_from_params_with_context(params, return_type_ann, None)
    }

    /// Build a Signature from formal parameters with contextual type information.
    ///
    /// When a contextual signature is provided (e.g., from a variable declaration
    /// annotation or a call site parameter type), parameters without type annotations
    /// use the corresponding type from the contextual signature.
    pub fn build_signature_from_params_with_context(
        &mut self,
        params: &oxc_ast::ast::FormalParameters<'_>,
        return_type_ann: Option<&oxc_ast::ast::TSTypeAnnotation<'_>>,
        contextual_sig: Option<&Signature>,
    ) -> Signature {
        let mut parameters = Vec::new();
        let mut min_argument_count: u32 = 0;
        let mut has_rest = false;

        for (i, param) in params.items.iter().enumerate() {
            let name = match &param.pattern {
                BindingPattern::BindingIdentifier(id) => CompactStr::new(id.name.as_str()),
                _ => CompactStr::new("_"),
            };
            let type_id = if let Some(ann) = &param.type_annotation {
                // Explicit annotation takes priority
                self.get_type_from_type_node(&ann.type_annotation)
            } else if let Some(ctx_sig) = contextual_sig {
                // Fall back to contextual parameter type
                ctx_sig.parameters.get(i).map(|p| p.type_id).unwrap_or(self.any_type)
            } else {
                self.any_type
            };
            let is_optional = param.optional || param.initializer.is_some();
            parameters.push(ParameterInfo { name, type_id, is_optional, is_rest: false });
            if !is_optional {
                min_argument_count += 1;
            }
        }

        // Handle rest parameter
        if let Some(rest) = &params.rest {
            let name = match &rest.rest.argument {
                BindingPattern::BindingIdentifier(id) => CompactStr::new(id.name.as_str()),
                _ => CompactStr::new("_"),
            };
            let type_id = if let Some(ann) = &rest.type_annotation {
                self.get_type_from_type_node(&ann.type_annotation)
            } else {
                self.any_type
            };
            parameters.push(ParameterInfo { name, type_id, is_optional: false, is_rest: true });
            has_rest = true;
        }

        let return_type = if let Some(rt) = return_type_ann {
            self.get_type_from_type_node(&rt.type_annotation)
        } else {
            self.any_type
        };

        let mut flags = SignatureFlags::None;
        if has_rest {
            flags |= SignatureFlags::HasRestParameter;
        }

        Signature {
            flags,
            min_argument_count,
            parameters,
            return_type,
            type_parameters: SmallVec::new(),
        }
    }

    /// Build a Signature from a function declaration/expression with contextual typing.
    ///
    /// Like `build_signature_from_function`, but passes through a contextual signature
    /// for parameter type inference.
    pub fn build_signature_from_function_with_context(
        &mut self,
        func: &Function<'_>,
        contextual_sig: Option<&Signature>,
    ) -> Signature {
        // Create type parameters FIRST so that parameter/return type annotations
        // that reference them (e.g., `T` in `function id<T>(x: T): T`) resolve
        // to the correct TypeParameter TypeIds via the declared_type_cache.
        let type_parameters =
            self.get_type_parameters_from_declaration(func.type_parameters.as_deref());
        let mut sig = self.build_signature_from_params_with_context(
            &func.params,
            func.return_type.as_deref(),
            contextual_sig,
        );
        sig.type_parameters = type_parameters;
        // Infer return type from body when there's no annotation.
        if func.return_type.is_none() {
            if let Some(body) = &func.body {
                let contextual_return_type = contextual_sig.map(|s| s.return_type);
                sig.return_type =
                    self.infer_return_type_from_body(&body.statements, contextual_return_type);
            }
        }
        sig
    }

    /// Infer the return type of a function from its body.
    ///
    /// Collects all return expression types, checks end-of-function reachability
    /// via the flow graph, and produces a union type. If the function end is
    /// reachable (implicit return), `void` is included in the union.
    pub(crate) fn infer_return_type_from_body(
        &mut self,
        stmts: &[Statement<'_>],
        contextual_return_type: Option<TypeId>,
    ) -> TypeId {
        // Collect types from all return statements (non-recursive into nested functions).
        let mut return_types = Vec::new();
        self.collect_return_types(stmts, &mut return_types);

        // Build a flow graph to check end-of-function reachability.
        let flow_graph = crate::flow_builder::FlowGraphBuilder::build(stmts, &self.semantic);
        let end_reachable = flow_graph.is_end_reachable();

        // If the function end is reachable without a return, add void.
        if end_reachable {
            return_types.push(self.void_type);
        }

        if return_types.is_empty() {
            return self.void_type;
        }

        let return_type = self.get_or_create_union_type(return_types);
        self.widen_return_type(return_type, contextual_return_type)
    }

    /// Walk statements collecting return expression types.
    /// Does NOT descend into nested function/arrow bodies.
    fn collect_return_types(&mut self, stmts: &[Statement<'_>], out: &mut Vec<TypeId>) {
        for stmt in stmts {
            self.collect_return_types_from_statement(stmt, out);
        }
    }

    fn collect_return_types_from_statement(&mut self, stmt: &Statement<'_>, out: &mut Vec<TypeId>) {
        match stmt {
            Statement::ReturnStatement(ret) => {
                let return_type = if let Some(arg) = &ret.argument {
                    self.get_type_of_expression(arg, None, CheckMode::NORMAL)
                } else {
                    self.undefined_type
                };
                if !out.contains(&return_type) {
                    out.push(return_type);
                }
            }
            // Recurse into compound statements but NOT into nested functions.
            Statement::BlockStatement(block) => {
                self.collect_return_types(&block.body, out);
            }
            Statement::IfStatement(if_stmt) => {
                self.collect_return_types_from_statement(&if_stmt.consequent, out);
                if let Some(alt) = &if_stmt.alternate {
                    self.collect_return_types_from_statement(alt, out);
                }
            }
            Statement::ForStatement(for_stmt) => {
                self.collect_return_types_from_statement(&for_stmt.body, out);
            }
            Statement::ForInStatement(for_in) => {
                self.collect_return_types_from_statement(&for_in.body, out);
            }
            Statement::ForOfStatement(for_of) => {
                self.collect_return_types_from_statement(&for_of.body, out);
            }
            Statement::WhileStatement(while_stmt) => {
                self.collect_return_types_from_statement(&while_stmt.body, out);
            }
            Statement::DoWhileStatement(do_while) => {
                self.collect_return_types_from_statement(&do_while.body, out);
            }
            Statement::SwitchStatement(switch_stmt) => {
                for case in &switch_stmt.cases {
                    self.collect_return_types(&case.consequent, out);
                }
            }
            Statement::TryStatement(try_stmt) => {
                self.collect_return_types(&try_stmt.block.body, out);
                if let Some(handler) = &try_stmt.handler {
                    self.collect_return_types(&handler.body.body, out);
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    self.collect_return_types(&finalizer.body, out);
                }
            }
            Statement::LabeledStatement(labeled) => {
                self.collect_return_types_from_statement(&labeled.body, out);
            }
            Statement::WithStatement(with_stmt) => {
                self.collect_return_types_from_statement(&with_stmt.body, out);
            }
            // Function/class declarations — do NOT descend (separate scope).
            Statement::FunctionDeclaration(_)
            | Statement::ClassDeclaration(_)
            | Statement::ExportNamedDeclaration(_)
            | Statement::ExportDefaultDeclaration(_) => {}
            // All other statements — no return statements possible.
            _ => {}
        }
    }
}
