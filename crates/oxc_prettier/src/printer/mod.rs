//! [Doc] Printer
//!
//! References:
//! * <https://github.com/prettier/prettier/blob/main/src/document/printer.js>

mod command;

use oxc_allocator::Allocator;
use std::collections::{HashMap, VecDeque};

use crate::{
    doc::{Doc, DocBuilder, Fill, IfBreak, IndentIfBreak, Line},
    GroupId, PrettierOptions,
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

    line_suffix: Vec<Command<'a>>,
    group_mode_map: HashMap<GroupId, Mode>,

    // states
    new_line: &'static str,

    allocator: &'a Allocator,
}

impl<'a> DocBuilder<'a> for Printer<'a> {
    #[inline]
    fn allocator(&self) -> &'a Allocator {
        self.allocator
    }
}

impl<'a> Printer<'a> {
    pub fn new(
        doc: Doc<'a>,
        source_text: &str,
        options: PrettierOptions,
        allocator: &'a Allocator,
    ) -> Self {
        // Preallocate for performance because the output will very likely
        // be the same size as the original text.
        let out = Vec::with_capacity(source_text.len());
        let cmds = vec![Command::new(Indent::root(), Mode::Break, doc)];
        Self {
            options,
            out,
            pos: 0,
            cmds,
            line_suffix: vec![],
            group_mode_map: HashMap::new(),
            new_line: options.end_of_line.as_str(),
            allocator,
        }
    }

    pub fn build(mut self) -> String {
        self.print_doc_to_string();
        // SAFETY: We should have constructed valid UTF8 strings
        #[allow(unsafe_code)]
        unsafe {
            String::from_utf8_unchecked(self.out)
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
                Doc::Group(_) => self.handle_group(indent, mode, doc),
                Doc::IndentIfBreak(docs) => self.handle_indent_if_break(indent, mode, docs),
                Doc::Line(line) => self.handle_line(line, indent, mode, doc),
                Doc::LineSuffix(docs) => self.handle_line_suffix(indent, mode, docs),
                Doc::IfBreak(if_break) => self.handle_if_break(if_break, indent, mode),
                Doc::Fill(fill) => self.handle_fill(indent, mode, fill),
                Doc::BreakParent => { /* No op */ }
            }

            if self.cmds.is_empty() && !self.line_suffix.is_empty() {
                self.cmds.extend(self.line_suffix.drain(..).rev());
            }
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    fn remaining_width(&self) -> isize {
        (self.options.print_width as isize) - (self.pos as isize)
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
                .map(|doc| Command::new(Indent::new(indent.length + 1), mode, doc)),
        );
    }

    fn handle_group(&mut self, indent: Indent, mode: Mode, doc: Doc<'a>) {
        match mode {
            Mode::Flat => {
                let Doc::Group(group) = doc else {
                    unreachable!();
                };
                self.cmds.extend(group.contents.into_iter().rev().map(|doc| {
                    Command::new(indent, if group.should_break { Mode::Break } else { mode }, doc)
                }));

                self.set_group_mode_from_last_cmd(group.id);
            }
            Mode::Break => {
                #[allow(clippy::cast_possible_wrap)]
                let remaining_width = self.remaining_width();
                let Doc::Group(group) = &doc else {
                    unreachable!();
                };
                let should_break = group.should_break;
                let group_id = group.id;
                let cmd = Command::new(indent, Mode::Flat, doc);
                if !should_break && self.fits(&cmd, remaining_width) {
                    self.cmds.push(Command::new(indent, Mode::Flat, cmd.doc));
                } else {
                    let Doc::Group(group) = cmd.doc else {
                        unreachable!();
                    };
                    if let Some(mut expanded_states) = group.expanded_states {
                        let most_expanded = expanded_states.pop().unwrap();
                        if group.should_break {
                            self.cmds.push(Command::new(indent, Mode::Break, most_expanded));
                            return;
                        }
                        for state in expanded_states {
                            let cmd = Command::new(indent, Mode::Flat, state);
                            if self.fits(&cmd, remaining_width) {
                                self.cmds.push(cmd);
                                return;
                            }
                        }
                        self.cmds.push(Command::new(indent, Mode::Break, most_expanded));
                    } else {
                        self.cmds.push(Command::new(
                            indent,
                            Mode::Break,
                            Doc::Array(group.contents),
                        ));
                    }
                }
                self.set_group_mode_from_last_cmd(group_id);
            }
        }
    }

    fn handle_indent_if_break(&mut self, indent: Indent, mode: Mode, doc: IndentIfBreak<'a>) {
        let IndentIfBreak { contents, group_id } = doc;
        let group_mode = group_id.map_or(Some(mode), |id| self.group_mode_map.get(&id).copied());

        match group_mode {
            Some(Mode::Flat) => {
                self.cmds
                    .extend(contents.into_iter().rev().map(|doc| Command::new(indent, mode, doc)));
            }
            Some(Mode::Break) => {
                self.cmds.extend(
                    contents
                        .into_iter()
                        .rev()
                        .map(|doc| Command::new(Indent::new(indent.length + 1), mode, doc)),
                );
            }
            None => {}
        }
    }

    fn handle_line(&mut self, line: Line, indent: Indent, mode: Mode, doc: Doc<'a>) {
        if mode.is_flat() {
            if line.hard {
                // shouldRemeasure = true;
            } else {
                if !line.soft {
                    self.out.push(b' ');
                    self.pos += 1;
                }
                return;
            }
        }

        if !self.line_suffix.is_empty() {
            self.cmds.push(Command::new(indent, mode, doc));
            self.cmds.extend(self.line_suffix.drain(..).rev());
            return;
        }

        if line.literal {
            self.out.extend(self.new_line.as_bytes());
            if !indent.root {
                self.pos = 0;
            }
        } else {
            self.trim();
            self.out.extend(self.new_line.as_bytes());
            self.pos = self.indent(indent.length);
        }
    }

    fn handle_line_suffix(
        &mut self,
        indent: Indent,
        mode: Mode,
        docs: oxc_allocator::Vec<'a, Doc<'a>>,
    ) {
        self.line_suffix.push(Command { indent, mode, doc: Doc::Array(docs) });
    }

    fn handle_if_break(&mut self, if_break: IfBreak<'a>, indent: Indent, mode: Mode) {
        let IfBreak { break_contents, flat_content, group_id } = if_break;
        let group_mode = group_id.map_or(Some(mode), |id| self.group_mode_map.get(&id).copied());

        match group_mode {
            Some(Mode::Flat) => {
                self.cmds.push(Command::new(indent, Mode::Flat, flat_content.unbox()));
            }
            Some(Mode::Break) => {
                self.cmds.push(Command::new(indent, Mode::Break, break_contents.unbox()));
            }
            None => {}
        }
    }

    fn handle_fill(&mut self, indent: Indent, mode: Mode, fill: Fill<'a>) {
        let mut fill = fill;
        let remaining_width = self.remaining_width();
        let original_parts_len = fill.parts().len();
        let (content, whitespace) = fill.drain_out_pair();

        let Some(content) = content else {
            return;
        };
        let content_flat_cmd = Command::new(indent, Mode::Flat, content);
        let content_fits = self.fits(&content_flat_cmd, remaining_width);

        if original_parts_len == 1 {
            if content_fits {
                self.cmds.push(content_flat_cmd);
            } else {
                let content_break_cmd = content_flat_cmd.with_mode(Mode::Break);
                self.cmds.push(content_break_cmd);
            }
            return;
        }

        let Some(whitespace) = whitespace else {
            return;
        };
        let whitespace_flat_cmd = Command::new(indent, Mode::Flat, whitespace);

        if original_parts_len == 2 {
            if content_fits {
                self.cmds.push(whitespace_flat_cmd);
                self.cmds.push(content_flat_cmd);
            } else {
                let content_break_cmd = content_flat_cmd.with_mode(Mode::Break);
                let whitespace_break_cmd = whitespace_flat_cmd.with_mode(Mode::Break);
                self.cmds.push(whitespace_break_cmd);
                self.cmds.push(content_break_cmd);
            }
            return;
        }

        let Some(second_content) = fill.dequeue() else {
            return;
        };
        let mut docs = self.vec();
        let content = content_flat_cmd.doc;
        docs.push(content);
        docs.push(whitespace_flat_cmd.doc);
        docs.push(second_content);

        let first_and_second_content_fit_cmd = Command::new(indent, Mode::Flat, Doc::Array(docs));
        let first_and_second_content_fits =
            self.fits(&first_and_second_content_fit_cmd, remaining_width);
        let Doc::Array(mut doc) = first_and_second_content_fit_cmd.doc else {
            return;
        };
        if let Some(second_content) = doc.pop() {
            fill.enqueue(second_content);
        }

        let Some(whitespace) = doc.pop() else {
            return;
        };
        let Some(content) = doc.pop() else {
            return;
        };

        let remaining_cmd = Command::new(indent, mode, Doc::Fill(fill));
        let whitespace_flat_cmd = Command::new(indent, Mode::Flat, whitespace);
        let content_flat_cmd = Command::new(indent, Mode::Flat, content);

        if first_and_second_content_fits {
            self.cmds.extend(vec![remaining_cmd, whitespace_flat_cmd, content_flat_cmd]);
        } else if content_fits {
            let whitespace_break_cmd = whitespace_flat_cmd.with_mode(Mode::Break);
            self.cmds.extend(vec![remaining_cmd, whitespace_break_cmd, content_flat_cmd]);
        } else {
            let content_break_cmd = content_flat_cmd.with_mode(Mode::Break);
            let whitespace_break_cmd = whitespace_flat_cmd.with_mode(Mode::Break);
            self.cmds.extend(vec![remaining_cmd, whitespace_break_cmd, content_break_cmd]);
        };
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

    fn trim(&mut self) {
        while let Some(&last) = self.out.last() {
            if last == b' ' || last == b'\t' {
                self.out.pop();
            } else {
                break;
            }
        }
    }

    fn set_group_mode_from_last_cmd(&mut self, id: Option<GroupId>) {
        let Some(id) = id else { return };
        let Some(mode) = self.cmds.last().map(|cmd| cmd.mode) else { return };
        self.group_mode_map.insert(id, mode);
    }

    #[allow(clippy::cast_possible_wrap)]
    fn fits(&self, next: &Command<'a>, width: isize) -> bool {
        let mut remaining_width = width;
        let mut queue: VecDeque<(Mode, &Doc)> = VecDeque::new();
        queue.push_front((next.mode, &next.doc));
        let mut cmds = self.cmds.iter().rev();

        while let Some((mode, doc)) = queue.pop_front() {
            match doc {
                Doc::Str(string) => {
                    remaining_width -= string.len() as isize;
                }
                Doc::IndentIfBreak(IndentIfBreak { contents: docs, .. })
                | Doc::Indent(docs)
                | Doc::Array(docs) => {
                    // Prepend docs to the queue
                    for d in docs.iter().rev() {
                        queue.push_front((mode, d));
                    }
                }
                Doc::Group(group) => {
                    let mode = if group.should_break { Mode::Break } else { mode };
                    if group.expanded_states.is_some() && mode.is_break() {
                        queue.push_front((
                            mode,
                            group.expanded_states.as_ref().unwrap().last().unwrap(),
                        ));
                    } else {
                        for d in group.contents.iter().rev() {
                            queue.push_front((mode, d));
                        }
                    };
                }
                Doc::IfBreak(if_break_doc) => {
                    let group_mode = if_break_doc
                        .group_id
                        .map_or(mode, |id| *self.group_mode_map.get(&id).unwrap_or(&Mode::Flat));

                    let contents = if group_mode.is_break() {
                        &if_break_doc.break_contents
                    } else {
                        &if_break_doc.flat_content
                    };

                    queue.push_front((mode, contents));
                }
                Doc::Line(line) => {
                    if mode.is_break() || line.hard {
                        return true;
                    }
                    if !line.soft {
                        remaining_width -= 1_isize;
                    }
                }
                Doc::Fill(fill) => {
                    for part in fill.parts().iter().rev() {
                        queue.push_front((mode, part));
                    }
                }
                Doc::LineSuffix(_) => {
                    break;
                }
                Doc::BreakParent => {}
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

    /// Reference:
    /// * https://github.com/prettier/prettier/blob/main/src/document/utils.js#L156-L185
    pub fn propagate_breaks(doc: &mut Doc<'_>) -> bool {
        let check_array = |arr: &mut oxc_allocator::Vec<'_, Doc<'_>>| {
            arr.iter_mut().rev().any(|doc| Self::propagate_breaks(doc))
        };

        match doc {
            Doc::BreakParent => true,
            Doc::Group(group) => {
                let mut should_break = false;
                if let Some(expanded_states) = &mut group.expanded_states {
                    should_break = expanded_states.iter_mut().rev().any(Self::propagate_breaks);
                }
                if !should_break {
                    should_break = check_array(&mut group.contents);
                }
                if group.expanded_states.is_none() && should_break {
                    group.should_break = should_break;
                }
                group.should_break
            }
            Doc::IfBreak(d) => Self::propagate_breaks(&mut d.break_contents),
            Doc::Array(arr)
            | Doc::Indent(arr)
            | Doc::IndentIfBreak(IndentIfBreak { contents: arr, .. }) => check_array(arr),
            _ => false,
        }
    }
}
