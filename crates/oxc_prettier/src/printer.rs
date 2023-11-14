//! [Doc] Printer
//!
//! References:
//! * <https://github.com/prettier/prettier/blob/main/src/document/printer.js>

#![allow(unused)]

use crate::doc::Doc;

struct Command<'a> {
    indent: Indent,
    mode: Mode,
    doc: Doc<'a>,
}

impl<'a> Command<'a> {
    fn new(indent: Indent, mode: Mode, doc: Doc<'a>) -> Self {
        Self { indent, mode, doc }
    }
}

#[derive(Clone, Copy)]
enum Mode {
    Break,
    Flat,
}

#[derive(Clone, Copy)]
struct Indent {
    value: &'static str,
    length: u8,
}

impl Indent {
    fn root() -> Self {
        Self { value: "", length: 0 }
    }
}

pub struct Printer<'a> {
    doc: Doc<'a>,
}

impl<'a> Printer<'a> {
    pub fn new(doc: Doc<'a>) -> Self {
        Self { doc }
    }

    pub fn build(self) -> String {
        self.print_doc_to_string()
    }
}

impl<'a> Printer<'a> {
    /// Turn Doc into a string
    ///
    /// Reference:
    /// * <https://github.com/prettier/prettier/blob/0176a33db442e498fdb577784deaa77d7c9ae723/src/document/printer.js#L302>
    fn print_doc_to_string(self) -> String {
        let mut out = vec![];
        let mut cmds: Vec<Command> = vec![Command::new(Indent::root(), Mode::Break, self.doc)];

        while let Some(Command { indent, doc, mode }) = cmds.pop() {
            match doc {
                Doc::Str(string) => {
                    out.extend(string.as_bytes());
                }
                Doc::Array(docs) => {
                    cmds.extend(docs.into_iter().rev().map(|doc| Command::new(indent, mode, doc)));
                }
                Doc::Indent(docs) => {
                    cmds.extend(docs.into_iter().rev().map(|doc| Command::new(indent, mode, doc)));
                }
                Doc::Group(docs) => {
                    cmds.extend(docs.into_iter().rev().map(|doc| Command::new(indent, mode, doc)));
                }
                Doc::Line | Doc::Softline | Doc::Hardline => {
                    out.push(b'\n');
                }
                Doc::IfBreak(docs, _) => {
                    cmds.extend(docs.into_iter().rev().map(|doc| Command::new(indent, mode, doc)));
                }
            }
        }

        // SAFETY: We should have constructed valid UTF8 strings
        unsafe { String::from_utf8_unchecked(out) }
    }
}
