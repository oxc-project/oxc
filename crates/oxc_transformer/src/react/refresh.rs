use std::borrow::Cow;

use oxc_ast::{
    ast::{
        ExportDefaultDeclaration, ExportDefaultDeclarationKind, Expression, IdentifierReference,
        Program, Statement, VariableDeclaration,
    },
    match_expression,
};
use oxc_semantic::{ReferenceFlag, SymbolFlags, SymbolId};
use oxc_span::{Atom, CompactStr, GetSpan, SPAN};
use oxc_syntax::operator::AssignmentOperator;
use oxc_traverse::TraverseCtx;

use super::options::ReactRefreshOptions;

use crate::context::Ctx;

/// React Fast Refresh
///
/// Transform React components to integrate Fast Refresh.
///
/// References:
///
/// * <https://github.com/facebook/react/issues/16604#issuecomment-528663101>
/// * <https://github.com/facebook/react/blob/main/packages/react-refresh/src/ReactFreshBabelPlugin.js>
pub struct ReactRefresh<'a> {
    options: ReactRefreshOptions,
    ctx: Ctx<'a>,
    registrations: std::vec::Vec<(SymbolId, CompactStr)>,
}

impl<'a> ReactRefresh<'a> {
    pub fn new(options: ReactRefreshOptions, ctx: Ctx<'a>) -> Self {
        Self { options, ctx, registrations: std::vec::Vec::default() }
    }

    fn create_registration(
        &mut self,
        persistent_id: CompactStr,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        let symbol_id = ctx.generate_uid_in_root_scope("c", SymbolFlags::FunctionScopedVariable);
        self.registrations.push((symbol_id, persistent_id));
        let name = ctx.ast.atom(ctx.symbols().get_name(symbol_id));
        ctx.create_reference_id(SPAN, name, Some(symbol_id), ReferenceFlag::Write)
    }

    /// Similar to the `findInnerComponents` function in `react-refresh/babel`.
    fn replace_inner_components(
        &mut self,
        inferred_name: Cow<CompactStr>,
        expr: &mut Expression<'a>,
        is_variable_declarator: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        match expr {
            Expression::Identifier(ref ident) => {
                if !is_componentish_name(&ident.name) {
                    return false;
                }
                // For case like:
                // export const Something = hoc(Foo)
                // we don't want to wrap Foo inside the call.
                // Instead we assume it's registered at definition.
                return true;
            }
            Expression::FunctionExpression(_) => {}
            Expression::ArrowFunctionExpression(arrow_expr) => {
                // () => () => {}
                let is_arrow_function = arrow_expr.expression
                    && {
                        arrow_expr.body.statements.first()
                .is_some_and(|stmt| {
                    matches!(stmt, Statement::ExpressionStatement(expr) if matches!(expr.expression, Expression::ArrowFunctionExpression(_)))
                })
                    };

                if is_arrow_function {
                    return false;
                }
            }
            Expression::CallExpression(ref mut call_expr) => {
                if call_expr.arguments.len() == 0 {
                    return false;
                }
                let allowed_callee = matches!(
                    call_expr.callee,
                    Expression::Identifier(_)
                        | Expression::ComputedMemberExpression(_)
                        | Expression::StaticMemberExpression(_)
                );

                if allowed_callee {
                    let callee_span = call_expr.callee.span();
                    let Some(argument_expr) =
                        call_expr.arguments.first_mut().and_then(|e| e.as_expression_mut())
                    else {
                        return false;
                    };

                    let found_inside = self.replace_inner_components(
                        Cow::Owned(CompactStr::from(format!(
                            "{}${}",
                            inferred_name.clone(),
                            callee_span.source_text(self.ctx.source_text)
                        ))),
                        argument_expr,
                        /* is_variable_declarator */ false,
                        ctx,
                    );

                    if !found_inside {
                        return false;
                    }

                    // const Foo = hoc1(hoc2(() => {}))
                    // export default memo(React.forwardRef(function() {}))
                    if is_variable_declarator {
                        return true;
                    }
                } else {
                    return false;
                }
            }
            _ => {
                return false;
            }
        }

        let ident = self.create_registration(inferred_name.into_owned(), ctx);
        *expr = ctx.ast.expression_assignment(
            SPAN,
            AssignmentOperator::Assign,
            ctx.ast.assignment_target_simple(
                ctx.ast.simple_assignment_target_from_identifier_reference(ident),
            ),
            ctx.ast.move_expression(expr),
        );

        true
    }
}

// Transform
impl<'a> ReactRefresh<'a> {
    pub fn transform_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut new_statements = ctx.ast.vec_with_capacity(program.body.len());
        for mut statement in program.body.drain(..) {
            let next_statement = self.transform_statement(&mut statement, ctx);
            new_statements.push(statement);
            if let Some(assignment_expression) = next_statement {
                new_statements.push(assignment_expression);
            }
        }
        // TODO *=
        program.body.extend(new_statements);
    }
    fn transform_statement(
        &mut self,
        statement: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        match statement {
            Statement::VariableDeclaration(stmt_decl) => {
                self.transform_variable_declaration(stmt_decl, ctx)
            }
            Statement::ExportDefaultDeclaration(ref mut stmt_decl) => {
                match &mut stmt_decl.declaration {
                    declaration @ match_expression!(ExportDefaultDeclarationKind) => {
                        let expression = declaration.to_expression_mut();
                        if !matches!(expression, Expression::CallExpression(_)) {
                            // For now, we only support possible HOC calls here.
                            // Named function declarations are handled in FunctionDeclaration.
                            // Anonymous direct exports like export default function() {}
                            // are currently ignored.
                            return None;
                        }

                        // This code path handles nested cases like:
                        // export default memo(() => {})
                        // In those cases it is more plausible people will omit names
                        // so they're worth handling despite possible false positives.
                        // More importantly, it handles the named case:
                        // export default memo(function Named() {})
                        self.replace_inner_components(
                            Cow::Owned(CompactStr::from("%default%")),
                            expression,
                            false,
                            ctx,
                        );

                        None
                    }
                    ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                        if let Some(id) = &func.id {
                            let reference = self.create_registration(id.name.to_compact_str(), ctx);
                            let expr = ctx.ast.expression_assignment(
                                SPAN,
                                AssignmentOperator::Assign,
                                ctx.ast.assignment_target_simple(
                                    ctx.ast.simple_assignment_target_from_identifier_reference(
                                        reference,
                                    ),
                                ),
                                ctx.ast.expression_identifier_reference(SPAN, id.name.clone()),
                            );
                            return Some(ctx.ast.statement_expression(SPAN, expr));
                        }
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
    pub fn transform_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        if decl.declarations.len() != 1 {
            return None;
        }

        let declarator = decl.declarations.first_mut().unwrap_or_else(|| unreachable!());
        let init = declarator.init.as_mut()?;
        let inferred_name = declarator.id.get_identifier()?;

        if !is_componentish_name(&inferred_name) {
            return None;
        }

        let compact_inferred_name = inferred_name.to_compact_str();

        match init {
            // Likely component definitions.
            Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_)
            // Maybe something like styled.div`...`
            | Expression::TaggedTemplateExpression(_) => {
                // Special case when a variable would get an inferred name:
                // let Foo = () => {}
                // let Foo = function() {}
                // let Foo = styled.div``;
                // We'll register it on next line so that
                // we don't mess up the inferred 'Foo' function name.
                // (eg: with @babel/plugin-transform-react-display-name or
                // babel-plugin-styled-components)
            }
            Expression::CallExpression(call_expr) => {
                if matches!(call_expr.callee, Expression::ImportExpression(_))
                    || call_expr.is_symbol_or_symbol_for_call()
                {
                    return None;
                }

                // Maybe a HOC.
                // Try to determine if this is some form of import.
                let found_inside = self.replace_inner_components(Cow::Borrowed(&compact_inferred_name), init, true, ctx);
                if !found_inside {
                    return None;
                }

                // See if this identifier is used in JSX. Then it's a component.
            }
            _ => {
                return None;
            }
        }

        let reference = self.create_registration(compact_inferred_name, ctx);
        let expr = ctx.ast.expression_assignment(
            SPAN,
            AssignmentOperator::Assign,
            ctx.ast.assignment_target_simple(
                ctx.ast.simple_assignment_target_from_identifier_reference(reference),
            ),
            ctx.ast.expression_identifier_reference(SPAN, inferred_name),
        );
        Some(ctx.ast.statement_expression(SPAN, expr))
    }
}

fn is_componentish_name(name: &Atom) -> bool {
    name.chars().next().unwrap().is_ascii_uppercase()
}
