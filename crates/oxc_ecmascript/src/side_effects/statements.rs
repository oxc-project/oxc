use oxc_ast::ast::*;

use crate::constant_evaluation::{DetermineValueType, ValueType};

use super::{MayHaveSideEffects, PropertyReadSideEffects, context::MayHaveSideEffectsContext};

impl<'a> MayHaveSideEffects<'a> for Statement<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            Statement::BlockStatement(block) => block.may_have_side_effects(ctx),
            Statement::DoWhileStatement(do_while) => do_while.may_have_side_effects(ctx),
            Statement::ExpressionStatement(expr) => expr.expression.may_have_side_effects(ctx),
            Statement::IfStatement(if_stmt) => if_stmt.may_have_side_effects(ctx),
            Statement::LabeledStatement(labeled) => labeled.body.may_have_side_effects(ctx),
            Statement::ReturnStatement(return_stmt) => {
                return_stmt.argument.may_have_side_effects(ctx)
            }
            Statement::SwitchStatement(switch) => switch.may_have_side_effects(ctx),
            Statement::TryStatement(try_stmt) => try_stmt.may_have_side_effects(ctx),
            Statement::WhileStatement(while_stmt) => while_stmt.may_have_side_effects(ctx),
            Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::EmptyStatement(_) => false,
            match_declaration!(Statement) => self.to_declaration().may_have_side_effects(ctx),
            Statement::ForInStatement(_)
            | Statement::ForOfStatement(_)
            | Statement::ForStatement(_)
            | Statement::ThrowStatement(_)
            | Statement::WithStatement(_)
            | Statement::DebuggerStatement(_) => true,
            #[expect(clippy::match_same_arms)]
            match_module_declaration!(Statement) => true,
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for BlockStatement<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        self.body.iter().any(|stmt| stmt.may_have_side_effects(ctx))
    }
}

impl<'a> MayHaveSideEffects<'a> for DoWhileStatement<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        self.test.may_have_side_effects(ctx) || self.body.may_have_side_effects(ctx)
    }
}

impl<'a> MayHaveSideEffects<'a> for IfStatement<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        self.test.may_have_side_effects(ctx)
            || self.consequent.may_have_side_effects(ctx)
            || self.alternate.may_have_side_effects(ctx)
    }
}

impl<'a> MayHaveSideEffects<'a> for SwitchStatement<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        self.discriminant.may_have_side_effects(ctx)
            || self.cases.iter().any(|case| {
                case.test.may_have_side_effects(ctx)
                    || case.consequent.iter().any(|stmt| stmt.may_have_side_effects(ctx))
            })
    }
}

impl<'a> MayHaveSideEffects<'a> for TryStatement<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        self.block.may_have_side_effects(ctx)
            || self.handler.as_ref().is_some_and(|catch_clause| {
                catch_clause
                    .param
                    .as_ref()
                    .is_some_and(|param| param.pattern.may_have_side_effects(ctx))
                    || catch_clause.body.may_have_side_effects(ctx)
            })
            || self.finalizer.as_ref().is_some_and(|finalizer| finalizer.may_have_side_effects(ctx))
    }
}

impl<'a> MayHaveSideEffects<'a> for WhileStatement<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        self.test.may_have_side_effects(ctx) || self.body.may_have_side_effects(ctx)
    }
}

impl<'a> MayHaveSideEffects<'a> for Declaration<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            Declaration::VariableDeclaration(var_decl) => var_decl.may_have_side_effects(ctx),
            Declaration::FunctionDeclaration(_) => false,
            Declaration::ClassDeclaration(class_decl) => class_decl.may_have_side_effects(ctx),
            Declaration::TSEnumDeclaration(_)
            | Declaration::TSImportEqualsDeclaration(_)
            | Declaration::TSModuleDeclaration(_)
            | Declaration::TSInterfaceDeclaration(_)
            | Declaration::TSTypeAliasDeclaration(_) => unreachable!(),
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for VariableDeclaration<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        if self.kind == VariableDeclarationKind::AwaitUsing {
            return true;
        }
        if self.kind == VariableDeclarationKind::Using {
            return self.declarations.iter().any(|decl| {
                decl.init.as_ref().is_none_or(|init| {
                    !matches!(init.value_type(ctx), ValueType::Undefined | ValueType::Null)
                        || init.may_have_side_effects(ctx)
                })
            });
        }
        self.declarations
            .iter()
            .any(|decl| decl.id.may_have_side_effects(ctx) || decl.init.may_have_side_effects(ctx))
    }
}

impl<'a> MayHaveSideEffects<'a> for BindingPattern<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match &self.kind {
            BindingPatternKind::ArrayPattern(array_pattern) => {
                ctx.property_read_side_effects() != PropertyReadSideEffects::None
                    || array_pattern.elements.iter().any(|el| el.may_have_side_effects(ctx))
            }
            BindingPatternKind::ObjectPattern(object_pattern) => {
                ctx.property_read_side_effects() != PropertyReadSideEffects::None
                    || object_pattern.properties.iter().any(|prop| {
                        prop.key.may_have_side_effects(ctx) || prop.value.may_have_side_effects(ctx)
                    })
            }
            BindingPatternKind::AssignmentPattern(assignment_pattern) => {
                assignment_pattern.left.may_have_side_effects(ctx)
                    || assignment_pattern.right.may_have_side_effects(ctx)
            }
            BindingPatternKind::BindingIdentifier(_) => false,
        }
    }
}
