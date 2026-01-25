use crate::{
    formatter::{Buffer, Format, Formatter},
    options::Semicolons,
    write,
};

pub struct OptionalSemicolon;

impl<'a> Format<'a> for OptionalSemicolon {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        match f.options().semicolons {
            Semicolons::Always => write!(f, ";"),
            Semicolons::AsNeeded => (),
        }
    }
}
