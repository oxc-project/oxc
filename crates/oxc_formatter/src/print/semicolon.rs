use crate::{
    formatter::{Buffer, Format, JsFormatContext, JsFormatter, JsFormatterExt as _},
    options::Semicolons,
    write,
};

pub struct OptionalSemicolon;

impl<'a> Format<'a, JsFormatContext<'a>> for OptionalSemicolon {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match f.options().semicolons {
            Semicolons::Always => write!(f, ";"),
            Semicolons::AsNeeded => (),
        }
    }
}
