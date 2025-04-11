use oxc_ast::ast::*;

use super::FormatWrite;
use crate::{
    format_args,
    formatter::{Buffer, FormatResult, Formatter, prelude::*},
    write,
};

impl<'a> FormatWrite<'a> for Function<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.r#async {
            write!(f, "async");
        }
        write!(f, "function");
        if self.generator {
            write!(f, "*");
        }
        write!(f, [space(), self.id, self.params, space(), self.body]);
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for FunctionBody<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["{", block_indent(&format_args!(self.directives, self.statements)), "}"])
    }
}
