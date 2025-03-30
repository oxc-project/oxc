use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    formatter::{Buffer, FormatResult, Formatter, prelude::*},
    write,
};

use super::{FormatWrite, OptionalSemicolon};

impl<'a> FormatWrite<'a> for VariableDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.kind.as_str(), hard_space(), self.declarations])
    }
}

impl<'a> Format<'a> for Vec<'a, VariableDeclarator<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        for d in self {
            write!(f, d)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for VariableDeclarator<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.id)?;
        if let Some(init) = &self.init {
            write!(f, [" = ", init])?;
        }
        Ok(())
    }
}
