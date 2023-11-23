//! [Doc] Printer
//!
//! References:
//! * <https://github.com/prettier/prettier/blob/main/src/document/printer.js>

mod command;

use std::collections::VecDeque;

use crate::{
    doc::{Doc, Group},
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

    // states
    new_line: &'static str,
}

impl<'a> Printer<'a> {
    pub fn new(doc: Doc<'a>, source_text: &str, options: PrettierOptions) -> Self {
        // Preallocate for performance because the output will very likely
        // be the same size as the original text.
        let out = Vec::with_capacity(source_text.len());
        let cmds = vec![Command::new(Indent::root(), Mode::Break, doc)];
        Self { options, out, pos: 0, cmds, new_line: options.end_of_line.as_str() }
    }

    pub fn build(mut self) -> String {
        self.print_doc_to_string();
        // SAFETY: We should have constructed valid UTF8 strings
        unsafe { String::from_utf8_unchecked(self.out) }
    }

    /// Reference:
    /// * https://github.com/prettier/prettier/blob/main/src/document/utils.js#L156-L185
    pub fn propagate_breaks(doc: &mut Doc<'_>) -> bool {
        match doc {
            Doc::Hardline => true,
            Doc::Group(group) => {
                let should_break =
                    group.contents.iter_mut().rev().any(|doc| Self::propagate_breaks(doc));
                if should_break {
                    group.should_break = should_break;
                }
                group.should_break
            }
            Doc::IfBreak(d) => Self::propagate_breaks(d),
            Doc::Array(arr) | Doc::Indent(arr) | Doc::IndentIfBreak(arr) => {
                arr.iter_mut().any(|doc| Self::propagate_breaks(doc))
            }
            _ => false,
        }
    }

    /// Turn Doc into a string
    ///
    /// Reference:
    /// * <https://github.com/prettier/prettier/blob/0176a33db442e498fdb577784deaa77d7c9ae723/src/document/printer.js#L302>
    pub fn print_doc_to_string(&mut self) {
        while let Some(Command { indent, mut doc, mode }) = self.cmds.pop() {
            Self::propagate_breaks(&mut doc);

            match doc {
                Doc::Str(s) => self.handle_str(s),
                Doc::Array(docs) => self.handle_array(indent, mode, docs),
                Doc::Indent(docs) => self.handle_indent(indent, mode, docs),
                Doc::Group(group) => self.handle_group(indent, mode, group),
                Doc::IndentIfBreak(docs) => self.handle_indent_if_break(indent, mode, docs),
                Doc::Line => self.handle_line(indent, mode),
                Doc::Softline => self.handle_softline(indent, mode),
                Doc::Hardline => self.handle_hardline(indent),
                Doc::IfBreak(doc) => self.handle_if_break(doc.unbox(), indent, mode),
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

    fn handle_group(&mut self, indent: Indent, mode: Mode, group: Group<'a>) {
        match mode {
            Mode::Flat => {
                // TODO: consider supporting `group mode` e.g. Break/Flat
                self.cmds.extend(group.contents.into_iter().rev().map(|doc| {
                    Command::new(
                        indent,
                        if group.should_break { Mode::Break } else { Mode::Flat },
                        doc,
                    )
                }));
            }
            Mode::Break => {
                #[allow(clippy::cast_possible_wrap)]
                let remaining_width = (self.options.print_width as isize) - (self.pos as isize);

                if !group.should_break && self.fits(&group.contents, remaining_width) {
                    self.cmds.push(Command::new(indent, Mode::Flat, Doc::Group(group)));
                } else {
                    self.cmds.extend(
                        group
                            .contents
                            .into_iter()
                            .rev()
                            .map(|doc| Command::new(indent, Mode::Break, doc)),
                    );
                }
            }
        }
    }

    fn handle_indent_if_break(
        &mut self,
        indent: Indent,
        mode: Mode,
        docs: oxc_allocator::Vec<'a, Doc<'a>>,
    ) {
        match mode {
            Mode::Flat => {
                self.cmds.extend(
                    docs.into_iter().rev().map(|doc| Command::new(indent, Mode::Flat, doc)),
                );
            }
            Mode::Break => {
                self.cmds.extend(docs.into_iter().rev().map(|doc| {
                    Command::new(Indent { length: indent.length + 1 }, Mode::Break, doc)
                }));
            }
        }
    }

    fn handle_line(&mut self, indent: Indent, mode: Mode) {
        if matches!(mode, Mode::Flat) {
            self.out.push(b' ');
        } else {
            self.out.extend(self.new_line.as_bytes());
            self.pos = self.indent(indent.length);
        }
    }

    fn handle_softline(&mut self, indent: Indent, mode: Mode) {
        if !matches!(mode, Mode::Flat) {
            self.out.extend(self.new_line.as_bytes());
            self.pos = self.indent(indent.length);
        }
    }

    fn handle_hardline(&mut self, indent: Indent) {
        self.out.extend(self.new_line.as_bytes());
        self.pos = self.indent(indent.length);
    }

    fn handle_if_break(&mut self, doc: Doc<'a>, indent: Indent, mode: Mode) {
        if mode == Mode::Break {
            self.cmds.push(Command::new(indent, Mode::Break, doc));
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    fn fits(&self, docs: &oxc_allocator::Vec<'a, Doc<'a>>, remaining_width: isize) -> bool {
        let mut remaining_width = remaining_width;

        // TODO: these should be commands
        let mut queue: VecDeque<(Mode, &Doc)> = docs.iter().map(|doc| (Mode::Flat, doc)).collect();
        let mut cmds = self.cmds.iter().rev();

        while let Some((mode, doc)) = queue.pop_front() {
            let is_break = matches!(mode, Mode::Break);
            match doc {
                Doc::Str(string) => {
                    remaining_width -= string.len() as isize;
                }
                Doc::IndentIfBreak(docs) | Doc::Indent(docs) | Doc::Array(docs) => {
                    // Prepend docs to the queue
                    for d in docs.iter().rev() {
                        queue.push_front((mode, d));
                    }
                }
                Doc::Group(group) => {
                    let mode = if group.should_break { Mode::Break } else { mode };
                    for d in group.contents.iter().rev() {
                        queue.push_front((mode, d));
                    }
                }
                Doc::IfBreak(doc) => {
                    if is_break {
                        queue.push_front((mode, doc));
                    }
                }
                Doc::Line => {
                    if is_break {
                        return true;
                    }
                    remaining_width -= 1_isize;
                }
                Doc::Softline => {
                    if is_break {
                        return true;
                    }
                }
                Doc::Hardline => {
                    return true;
                }
            }

            if remaining_width < 0 {
                return false;
            }

            if queue.is_empty() {
                if let Some(cmd) = cmds.next() {
                    queue.push_back((cmd.mode, &cmd.doc));
                }
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
