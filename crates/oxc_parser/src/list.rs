use oxc_diagnostics::Result;

use crate::{lexer::Kind, ParserImpl};

pub trait SeparatedList<'a>: Sized {
    fn new(p: &ParserImpl<'a>) -> Self;

    fn parse(p: &mut ParserImpl<'a>) -> Result<Self> {
        let mut list = Self::new(p);
        list.parse_list(p)?;
        Ok(list)
    }

    /// Open element, e.g.. `{` `[` `(`
    fn open(&self) -> Kind;

    /// Close element, e.g.. `}` `]` `)`
    fn close(&self) -> Kind;

    /// Separator element, e.g. `,`
    fn separator(&self) -> Kind {
        Kind::Comma
    }

    fn parse_element(&mut self, p: &mut ParserImpl<'a>) -> Result<()>;

    /// Main entry point, parse the list
    fn parse_list(&mut self, p: &mut ParserImpl<'a>) -> Result<()> {
        p.expect(self.open())?;

        let mut first = true;

        while !p.at(self.close()) && !p.at(Kind::Eof) {
            if first {
                first = false;
            } else {
                p.expect(self.separator())?;
                if p.at(self.close()) {
                    break;
                }
            }

            self.parse_element(p)?;
        }

        p.expect(self.close())?;
        Ok(())
    }
}
