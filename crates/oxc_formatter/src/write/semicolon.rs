use oxc_ast::ast::*;

use crate::{
    formatter::{Buffer, Format, FormatResult, Formatter},
    generated::ast_nodes::AstNode,
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
