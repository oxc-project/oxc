use oxc_diagnostics::Result;

use crate::{lexer::Kind, Parser};

pub trait NormalList<'a> {
    /// Open element, e.g.. `{` `[` `(`
    fn open(&self) -> Kind;

    /// Close element, e.g.. `}` `]` `)`
    fn close(&self) -> Kind;

    fn parse_element(&mut self, p: &mut Parser<'a>) -> Result<()>;

    /// Main entry point, parse the list
    fn parse(&mut self, p: &mut Parser<'a>) -> Result<()> {
        p.expect(self.open())?;
        while !p.at(self.close()) && !p.at(Kind::Eof) {
            self.parse_element(p)?;
        }
        p.expect(self.close())?;
        Ok(())
    }
}

pub trait SeparatedList<'a>: Sized {
    fn new(p: &Parser<'a>) -> Self;

    fn parse(p: &mut Parser<'a>) -> Result<Self> {
        let mut list = Self::new(p);
        list.parse_list(p)?;
        Ok(list)
    }

    /// Open element, e.g.. `{` `[` `(`
    fn open(&self) -> Kind;

    /// Close element, e.g.. `}` `]` `)`
    fn close(&self) -> Kind;

    /// Seperator element, e.g. `,`
    fn seperator(&self) -> Kind {
        Kind::Comma
    }

    fn start_sequence(&mut self, _p: &mut Parser<'a>) {}
    fn finish_sequence(&mut self, _p: &mut Parser<'a>) {}

    fn parse_element(&mut self, p: &mut Parser<'a>) -> Result<()>;

    /// Main entry point, parse the list
    fn parse_list(&mut self, p: &mut Parser<'a>) -> Result<()> {
        p.expect(self.open())?;
        self.start_sequence(p);

        let mut first = true;

        while !p.at(self.close()) && !p.at(Kind::Eof) {
            if first {
                first = false;
            } else {
                p.expect(self.seperator())?;
                if p.at(self.close()) {
                    break;
                }
            }

            self.parse_element(p)?;
        }

        self.finish_sequence(p);
        p.expect(self.close())?;
        Ok(())
    }
}
