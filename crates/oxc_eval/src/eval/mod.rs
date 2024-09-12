//! Constant-time evaluation through the [`Eval`] trait.

mod expr;
mod stmt;

use oxc_ast::{
    ast::{Expression, Program, Statement},
    match_declaration, match_module_declaration,
};
use oxc_diagnostics::OxcDiagnostic;

use crate::{EvalContext, Value};

/// [`None`] for unimplemented logic or logic that cannot be evaluated without non-constant
/// techniques. [`Some`] for an error that occurred during evaluation.
///
/// The [`Some`] variant is equivalent to a throw completion. This and [`EvalResult`] will both be
/// replaced in the future with [CompletionRecord](`crate::completion::CompletionRecord`) when the
/// `try_trait_v2` feature stabilizes.
pub type EvalError = Option<OxcDiagnostic>;

/// A [`Result`] type that is a simplified version of a completion record.
/// 1. [`Ok`] for a normal completion.
/// 2. [`Err`] with `None` for an unimplemented logic or logic that cannot be evaluated without
///    non-constant techniques.
/// 3. [`Err`] with [`Some`] for an error that occurred during evaluation, i.e. a throw completion.
///
/// This will be replaced in the future with
/// [CompletionRecord](`crate::completion::CompletionRecord`) when the `try_trait_v2` feature
/// stabilizes.
pub type EvalResult<'a> = Result<Value<'a>, EvalError>;
pub const TODO: EvalResult<'static> = Err(None);
pub const VOID: EvalResult<'static> = Ok(Value::Undefined);

pub trait Eval<'a> {
    fn eval(&self, ctx: &mut EvalContext<'a>) -> EvalResult<'a>;
}

impl<'a> Eval<'a> for Expression<'a> {
    fn eval(&self, ctx: &mut EvalContext<'a>) -> EvalResult<'a> {
        match self {
            Self::BooleanLiteral(lit) => lit.eval(ctx),
            Self::NullLiteral(lit) => lit.eval(ctx),
            Self::NumericLiteral(lit) => lit.eval(ctx),
            Self::BigIntLiteral(lit) => lit.eval(ctx),
            Self::RegExpLiteral(_) => TODO,
            Self::StringLiteral(lit) => lit.eval(ctx),
            Self::TemplateLiteral(_) => TODO,
            Self::Identifier(_) => TODO,
            Self::MetaProperty(_) => TODO,
            Self::Super(_) => TODO,
            Self::ArrayExpression(_) => TODO,
            Self::ArrowFunctionExpression(_) => TODO,
            Self::AssignmentExpression(_) => TODO,
            Self::AwaitExpression(_) => TODO,
            // Self::BinaryExpression(bin) => bin.eval(ctx),
            Self::BinaryExpression(_) => TODO,
            Self::CallExpression(_) => TODO,
            Self::ChainExpression(_) => TODO,
            Self::ClassExpression(_) => TODO,
            Self::ConditionalExpression(expr) => expr.eval(ctx),
            Self::FunctionExpression(_) => TODO,
            Self::ImportExpression(_) => TODO,
            Self::LogicalExpression(expr) => expr.eval(ctx),
            Self::NewExpression(_) => TODO,
            Self::ObjectExpression(_) => TODO,
            Self::ParenthesizedExpression(expr) => expr.expression.eval(ctx),
            Self::SequenceExpression(expr) => expr.eval(ctx),
            Self::TaggedTemplateExpression(_) => TODO,
            Self::ThisExpression(_) => TODO,
            Self::UnaryExpression(_) => TODO,
            Self::UpdateExpression(_) => TODO,
            Self::YieldExpression(_) => TODO,
            Self::PrivateInExpression(_) => TODO,
            // jsx
            Self::JSXElement(_) => TODO,
            Self::JSXFragment(_) => TODO,
            // ts
            Self::TSAsExpression(expr) => expr.expression.eval(ctx),
            Self::TSSatisfiesExpression(expr) => expr.expression.eval(ctx),
            Self::TSTypeAssertion(expr) => expr.expression.eval(ctx),
            Self::TSNonNullExpression(expr) => expr.expression.eval(ctx),
            Self::TSInstantiationExpression(expr) => expr.expression.eval(ctx),
            Self::ComputedMemberExpression(_) => TODO,
            Self::StaticMemberExpression(_) => TODO,
            Self::PrivateFieldExpression(_) => TODO,
        }
    }
}

impl<'a> Eval<'a> for Statement<'a> {
    fn eval(&self, ctx: &mut EvalContext<'a>) -> EvalResult<'a> {
        match self {
            Self::BlockStatement(stmt) => stmt.eval(ctx),
            Self::BreakStatement(_) => TODO,
            Self::ContinueStatement(_) => TODO,
            Self::DebuggerStatement(_) => TODO,
            Self::DoWhileStatement(_) => TODO,
            Self::EmptyStatement(_) => VOID,
            Self::ExpressionStatement(expr) => expr.expression.eval(ctx),
            Self::ForInStatement(_) => TODO,
            Self::ForOfStatement(_) => TODO,
            Self::ForStatement(_) => TODO,
            Self::IfStatement(_) => TODO,
            Self::LabeledStatement(_) => TODO,
            Self::ReturnStatement(_) => TODO,
            Self::SwitchStatement(_) => TODO,
            Self::ThrowStatement(_) => TODO,
            Self::TryStatement(_) => TODO,
            Self::WhileStatement(_) => TODO,
            Self::WithStatement(_) => TODO,
            _stmt @ match_declaration!(Self) => TODO,
            _stmt @ match_module_declaration!(Self) => TODO,
        }
    }
}

impl<'a> Eval<'a> for Program<'a> {
    fn eval(&self, ctx: &mut EvalContext<'a>) -> EvalResult<'a> {
        if self.is_strict() {
            ctx.enter_strict();
        }
        for stmt in &self.body {
            let _ = stmt.eval(ctx)?;
        }
        VOID
    }
}
