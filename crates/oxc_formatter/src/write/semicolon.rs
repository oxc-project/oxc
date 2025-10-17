use oxc_ast::ast::*;

use crate::{
    ast_nodes::AstNode,
    formatter::{Buffer, Format, FormatResult, Formatter},
    options::Semicolons,
    write,
};

pub struct OptionalSemicolon;

impl<'a> Format<'a> for OptionalSemicolon {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match f.options().semicolons {
            Semicolons::Always => write!(f, ";"),
            Semicolons::AsNeeded => Ok(()),
        }
    }
}

pub struct MaybeOptionalSemicolon(pub bool);

impl<'a> Format<'a> for MaybeOptionalSemicolon {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.0 { OptionalSemicolon.fmt(f) } else { Ok(()) }
    }
}
