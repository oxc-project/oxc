use crate::{
    formatter::{Buffer, Format, FormatResult, Formatter},
    options::Semicolons,
    write,
};

pub(super) struct OptionalSemicolon;

impl<'a> Format<'a> for OptionalSemicolon {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match f.options().semicolons {
            Semicolons::Always => write!(f, ";"),
            Semicolons::AsNeeded => Ok(()),
        }
    }
}
