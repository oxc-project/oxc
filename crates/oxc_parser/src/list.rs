use oxc_diagnostics::Result;

use crate::{lexer::Kind, ParserImpl};

pub trait NormalList<'a> {
    /// Open element, e.g.. `{` `[` `(`
    fn open(&self) -> Kind;

    /// Close element, e.g.. `}` `]` `)`
    fn close(&self) -> Kind;

    fn parse_element(&mut self, p: &mut ParserImpl<'a>) -> Result<()>;

    /// Main entry point, parse the list
    fn parse(&mut self, p: &mut ParserImpl<'a>) -> Result<()> {
        p.expect(self.open())?;
        while !p.at(self.close()) && !p.at(Kind::Eof) {
            self.parse_element(p)?;
        }
        p.expect(self.close())?;
        Ok(())
    }
}

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

    /// When [`Some`], allows the parser to continue parsing when
    /// [`Self::separator`] is not found. Illegal separators will be reported as
    /// errors, but the parser will attempt to parse the rest of the list.
    fn illegal_separator(&self) -> Option<Kind> {
        Some(Kind::Semicolon)
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
                if let Err(e) = p.expect(self.separator()) {
                    let Some(illegal_sep) = self.illegal_separator() else { return Err(e) };
                    match p.expect(illegal_sep) {
                        Err(_) => return Err(e),
                        Ok(()) => {
                            // report illegal separator, but continue parsing
                            p.error(e);
                        }
                    }
                }
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
