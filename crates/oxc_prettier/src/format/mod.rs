mod js;
mod jsx;
mod print;
mod typescript;

use crate::{ir::Doc, Prettier};

pub trait Format<'a> {
    #[must_use]
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a>;
}
