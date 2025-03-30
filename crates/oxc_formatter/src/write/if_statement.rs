use oxc_ast::ast::*;

use super::FormatWrite;
use crate::{
    formatter::{FormatResult, Formatter, prelude::*},
    write,
};

impl<'a> FormatWrite<'a> for IfStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["if (", self.test, ")", self.consequent])?;
        if let Some(alternate) = &self.alternate {
            write!(f, ["else", alternate])?;
        }
        Ok(())
    }
}
