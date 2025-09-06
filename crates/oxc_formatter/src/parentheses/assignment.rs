use oxc_ast::{AstKind, ast::*};
use oxc_span::GetSpan;
use oxc_syntax::{
    operator,
    precedence::{GetPrecedence, Precedence},
};

use crate::{
    formatter::Formatter,
    generated::ast_nodes::{AstNode, AstNodes},
};
