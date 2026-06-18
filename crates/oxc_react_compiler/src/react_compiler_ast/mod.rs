pub mod common;
pub mod declarations;
pub mod expressions;
pub mod jsx;
pub mod literals;
pub mod operators;
pub mod patterns;
pub mod scope;
pub mod statements;
pub mod visitor;

use crate::react_compiler_ast::common::{BaseNode, Comment};
use crate::react_compiler_ast::expressions::Expression;
use crate::react_compiler_ast::patterns::PatternLike;
use crate::react_compiler_ast::statements::{Directive, Statement};

/// An original source AST node preserved verbatim for re-emission when the
/// compiler bails on a construct it does not model (`UnsupportedNode`).
///
/// Holding the typed node directly — rather than a `serde_json::Value` — lets
/// lowering stash it and codegen restore it without round-tripping through
/// serde, which is what kept the AST (de)serializers out of the generated
/// binary. The variant records which syntactic position the node came from, so
/// codegen can dispatch without re-parsing a `type` tag.
#[derive(Debug, Clone)]
pub enum OriginalNode {
    Expression(Box<Expression>),
    Statement(Box<Statement>),
    Pattern(Box<PatternLike>),
}

/// The root type returned by @babel/parser
#[derive(Debug, Clone)]
pub struct File {
    pub base: BaseNode,
    pub program: Program,
    pub comments: Vec<Comment>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub base: BaseNode,
    pub body: Vec<Statement>,
    pub directives: Vec<Directive>,
    pub source_type: SourceType,
    pub interpreter: Option<InterpreterDirective>,
    pub source_file: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SourceType {
    Module,
    Script,
}

#[derive(Debug, Clone)]
pub struct InterpreterDirective {
    pub base: BaseNode,
    pub value: String,
}
