use oxc_ast::ast::Statement;

use crate::{
    format_args,
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        prelude::{indent, soft_line_break_or_space, space},
    },
    write,
};

pub struct FormatStatementBody<'a, 'b> {
    body: &'b Statement<'a>,
    force_space: bool,
}

impl<'a, 'b> FormatStatementBody<'a, 'b> {
    pub fn new(body: &'b Statement<'a>) -> Self {
        Self { body, force_space: false }
    }

    /// Prevents that the consequent is formatted on its own line and indented by one level and
    /// instead gets separated by a space.
    pub fn with_forced_space(mut self, forced: bool) -> Self {
        self.force_space = forced;
        self
    }
}

impl<'a> Format<'a> for FormatStatementBody<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if let Statement::EmptyStatement(empty) = &self.body {
            write!(f, empty)
        } else if matches!(&self.body, Statement::BlockStatement(_)) || self.force_space {
            write!(f, [space(), self.body])
        } else {
            write!(f, [indent(&format_args!(soft_line_break_or_space(), &self.body))])
        }
    }
}
