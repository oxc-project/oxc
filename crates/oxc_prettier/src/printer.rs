//! [Doc] Printer
//!
//! References:
//! * <https://github.com/prettier/prettier/blob/main/src/document/printer.js>

#![allow(unused)]

use std::{collections::VecDeque, ops::Deref};

use crate::doc::Doc;

struct Command<'a, 'b> {
    indent: Indent,
    mode: Mode,
    doc: &'b Doc<'a>,
}

impl<'a, 'b> Command<'a, 'b> {
    fn new(indent: Indent, mode: Mode, doc: &'a Doc<'b>) -> Self {
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
    options: crate::PrettierOptions,
}

impl<'a> Printer<'a> {
    pub fn new(doc: Doc<'a>, options: crate::PrettierOptions) -> Self {
        Self { doc, options }
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
        let mut cmds: Vec<Command> = vec![Command::new(Indent::root(), Mode::Break, &self.doc)];

        let mut pos: u16 = 0;

        while let Some(Command { indent, doc, mode }) = cmds.pop() {
            match &doc {
                Doc::Str(string) => {
                    out.extend(string.as_bytes());
                    pos += string.len() as u16;
                }
                Doc::Array(docs) => {
                    cmds.extend(docs.into_iter().rev().map(|doc| Command::new(indent, mode, doc)));
                }
                Doc::Indent(docs) => {
                    cmds.extend(docs.into_iter().rev().map(|doc| {
                        Command::new(Indent { value: " ", length: indent.length + 1 }, mode, doc)
                    }));
                }
                Doc::Group(docs) => {
                    match mode {
                        Mode::Flat => {
                            // TODO: consider supporting `group mode` e.g. Break/Flat
                            cmds.extend(
                                docs.into_iter()
                                    .rev()
                                    .map(|doc| Command::new(indent, Mode::Flat, doc)),
                            );
                        }
                        Mode::Break => {
                            #[allow(clippy::cast_possible_wrap)]
                            let remaining_width = (self.options.print_width - pos) as isize;

                            if fits(docs, remaining_width) {
                                cmds.extend(
                                    docs.into_iter()
                                        .rev()
                                        .map(|doc| Command::new(indent, Mode::Flat, doc)),
                                );
                            } else {
                                cmds.extend(
                                    docs.into_iter()
                                        .rev()
                                        .map(|doc| Command::new(indent, Mode::Break, doc)),
                                );
                            }
                        }
                    }
                }
                #[allow(clippy::cast_lossless)]
                Doc::Line => {
                    if matches!(mode, Mode::Flat) {
                        out.push(b' ');
                    } else {
                        out.push(b'\n');
                        out.extend(indent.value.repeat(indent.length as usize).as_bytes());
                        pos = indent.length as u16;
                    }
                }
                #[allow(clippy::cast_lossless)]
                Doc::Softline => {
                    if !matches!(mode, Mode::Flat) {
                        out.push(b'\n');
                        out.extend(indent.value.repeat(indent.length as usize).as_bytes());
                        pos = indent.length as u16;
                    }
                }
                Doc::Hardline => {
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

#[allow(clippy::cast_possible_wrap)]
fn fits<'a, 'b>(doc: &'a oxc_allocator::Vec<'a, Doc<'b>>, remaining_width: isize) -> bool {
    let mut remaining_width = remaining_width;

    // TODO: these should be commands
    let mut queue: VecDeque<&Doc<'a>> = doc.iter().collect();

    while let Some(next) = queue.pop_front() {
        match next {
            Doc::Str(string) => {
                remaining_width -= string.len() as isize;
            }
            Doc::Array(docs) | Doc::Indent(docs) => {
                // Prepend docs to the queue
                for d in docs.iter().rev() {
                    queue.push_front(d);
                }
            }
            Doc::Group(doc) => {
                for d in doc.iter().rev() {
                    queue.push_front(d);
                }
            }
            // trying to fit on a single line, so we don't need to consider line breaks
            Doc::IfBreak(_, _) | Doc::Softline => {}
            Doc::Line => remaining_width += 1,
            Doc::Hardline => {
                return false;
            }
        }

        if remaining_width < 0 {
            return false;
        }
    }

    true
}
