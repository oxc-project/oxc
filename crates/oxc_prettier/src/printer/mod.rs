//! [Doc] Printer
//!
//! References:
//! * <https://github.com/prettier/prettier/blob/main/src/document/printer.js>

mod command;

use std::collections::VecDeque;

use crate::{
    doc::{Doc, IfBreak},
    PrettierOptions,
};

use self::command::{Command, Indent, Mode};

pub struct Printer<'a> {
    options: PrettierOptions,
    /// The final output string in bytes
    out: Vec<u8>,
    /// The current position in the output
    pos: usize,
    /// cmds is basically a stack. We've turned a recursive call into a
    /// while loop which is much faster. The while loop below adds new
    /// cmds to the array instead of recursively calling `print`.
    cmds: Vec<Command<'a>>,
}

impl<'a> Printer<'a> {
    pub fn new(doc: Doc<'a>, options: PrettierOptions) -> Self {
        // TODO(perf): `with_capacity(source_text.len())`
        Self {
            options,
            out: vec![],
            pos: 0,
            cmds: vec![Command::new(Indent::root(), Mode::Break, doc)],
        }
    }

    pub fn build(mut self) -> String {
        self.print_doc_to_string();
        // SAFETY: We should have constructed valid UTF8 strings
        unsafe { String::from_utf8_unchecked(self.out) }
    }

    /// Turn Doc into a string
    ///
    /// Reference:
    /// * <https://github.com/prettier/prettier/blob/0176a33db442e498fdb577784deaa77d7c9ae723/src/document/printer.js#L302>
    pub fn print_doc_to_string(&mut self) {
        while let Some(Command { indent, doc, mode }) = self.cmds.pop() {
            match doc {
                Doc::Str(s) => self.handle_str(s),
                Doc::Array(docs) => self.handle_array(indent, mode, docs),
                Doc::Indent(docs) => self.handle_indent(indent, mode, docs),
                Doc::Group(docs) => self.handle_group(indent, mode, docs),
                Doc::Line => self.handle_line(indent, mode),
                Doc::Softline => self.handle_softline(indent, mode),
                Doc::Hardline => self.handle_hardline(indent),
                Doc::IfBreak(if_break) => self.handle_if_break(if_break, indent, mode),
            }
        }
    }

    fn handle_str(&mut self, s: &str) {
        self.out.extend(s.as_bytes());
        self.pos += s.len();
    }

    fn handle_array(&mut self, indent: Indent, mode: Mode, docs: oxc_allocator::Vec<'a, Doc<'a>>) {
        self.cmds.extend(docs.into_iter().rev().map(|doc| Command::new(indent, mode, doc)));
    }

    fn handle_indent(&mut self, indent: Indent, mode: Mode, docs: oxc_allocator::Vec<'a, Doc<'a>>) {
        self.cmds.extend(
            docs.into_iter()
                .rev()
                .map(|doc| Command::new(Indent { length: indent.length + 1 }, mode, doc)),
        );
    }

    fn handle_group(&mut self, indent: Indent, mode: Mode, docs: oxc_allocator::Vec<'a, Doc<'a>>) {
        match mode {
            Mode::Flat => {
                // TODO: consider supporting `group mode` e.g. Break/Flat
                self.cmds.extend(
                    docs.into_iter().rev().map(|doc| Command::new(indent, Mode::Flat, doc)),
                );
            }
            Mode::Break => {
                #[allow(clippy::cast_possible_wrap)]
                let remaining_width = (self.options.print_width as isize) - (self.pos as isize);

                if Self::fits(&docs, remaining_width) {
                    self.cmds.extend(
                        docs.into_iter().rev().map(|doc| Command::new(indent, Mode::Flat, doc)),
                    );
                } else {
                    self.cmds.extend(
                        docs.into_iter().rev().map(|doc| Command::new(indent, Mode::Break, doc)),
                    );
                }
            }
        }
    }

    fn handle_line(&mut self, indent: Indent, mode: Mode) {
        if matches!(mode, Mode::Flat) {
            self.out.push(b' ');
        } else {
            self.out.push(b'\n');
            self.pos = self.indent(indent.length);
        }
    }

    fn handle_softline(&mut self, indent: Indent, mode: Mode) {
        if !matches!(mode, Mode::Flat) {
            self.out.push(b'\n');
            self.pos = self.indent(indent.length);
        }
    }

    fn handle_hardline(&mut self, indent: Indent) {
        self.out.push(b'\n');
        self.pos = self.indent(indent.length);
    }

    fn handle_if_break(&mut self, if_break: IfBreak<'a>, indent: Indent, mode: Mode) {
        let IfBreak { break_contents, .. } = if_break;
        self.cmds
            .extend(break_contents.into_iter().rev().map(|doc| Command::new(indent, mode, doc)));
    }

    #[allow(clippy::cast_possible_wrap)]
    fn fits(doc: &oxc_allocator::Vec<'a, Doc<'a>>, remaining_width: isize) -> bool {
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
                Doc::IfBreak { .. } | Doc::Softline => {}
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

    fn indent(&mut self, size: usize) -> usize {
        if self.options.use_tabs {
            self.out.extend("\t".repeat(size).as_bytes());
            size
        } else {
            let count = self.options.tab_width * size;
            self.out.extend(" ".repeat(count).as_bytes());
            count
        }
    }
}
