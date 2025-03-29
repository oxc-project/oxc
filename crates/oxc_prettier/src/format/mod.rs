mod js;
mod jsx;
mod print;
mod typescript;

use crate::{Prettier, ir::Doc};

pub trait Format<'a> {
    #[must_use]
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a>;
}
