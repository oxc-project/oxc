pub mod common;
pub mod declarations;
pub mod expressions;
pub mod jsx;
pub mod literals;
pub mod operators;
pub mod patterns;
pub mod statements;
pub mod visitor;

use crate::react_compiler_ast::common::{BaseNode, Comment};
use crate::react_compiler_ast::statements::{Directive, Statement};

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
