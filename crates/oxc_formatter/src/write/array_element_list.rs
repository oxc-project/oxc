use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    Expand, FormatTrailingCommas,
    formatter::{Buffer, Format, FormatResult, Formatter, GroupId},
    write,
};

pub struct ArrayElementList<'a, 'b> {
    elements: &'b Vec<'a, ArrayExpressionElement<'a>>,
    group_id: GroupId,
}

impl<'a, 'b> ArrayElementList<'a, 'b> {
    pub fn new(elements: &'b Vec<'a, ArrayExpressionElement<'a>>, group_id: GroupId) -> Self {
        Self { elements, group_id }
    }
}

impl<'a> Format<'a> for ArrayElementList<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let expand_lists = f.context().options().expand == Expand::Always;

        for (i, element) in self.elements.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, element)?;
        }
        Ok(())
    }
}
