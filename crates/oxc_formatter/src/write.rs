use oxc_ast::{AstKind, ast::*};

use crate::{
    formatter::{Buffer, Format, FormatResult, Formatter, prelude::*},
    write,
};

pub trait FormatWrite<'ast> {
    fn write(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()>;
}

impl<'ast> FormatWrite<'ast> for Program<'ast> {
    fn write(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'ast> FormatWrite<'ast> for Hashbang<'ast> {
    fn write(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        write!(f, [text("#!"), dynamic_text(self.value.as_str(), self.span.start)])
    }
}
