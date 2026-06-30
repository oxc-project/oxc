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

use serde::Serialize;

use crate::common::{BaseNode, Comment};
use crate::expressions::Expression;
use crate::patterns::PatternLike;
use crate::statements::{Directive, Statement};

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
#[derive(Debug, Clone, Serialize)]
pub struct File {
    #[serde(flatten)]
    pub base: BaseNode,
    pub program: Program,
    #[serde(default)]
    pub comments: Vec<Comment>,
    #[serde(default)]
    pub errors: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Program {
    #[serde(flatten)]
    pub base: BaseNode,
    pub body: Vec<Statement>,
    #[serde(default)]
    pub directives: Vec<Directive>,
    #[serde(rename = "sourceType")]
    pub source_type: SourceType,
    #[serde(default)]
    pub interpreter: Option<InterpreterDirective>,
    #[serde(
        rename = "sourceFile",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub source_file: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Module,
    Script,
}

#[derive(Debug, Clone, Serialize)]
pub struct InterpreterDirective {
    #[serde(flatten)]
    pub base: BaseNode,
    pub value: String,
}
