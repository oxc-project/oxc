mod command;

use std::collections::VecDeque;

use oxc_allocator::{Allocator, Vec};
use rustc_hash::FxHashMap;

use crate::{
    GroupId, PrettierOptions,
    ir::{Doc, Fill, Group, IfBreak, IndentIfBreak, Line},
    printer::command::{Command, Indent, Mode},
};

pub struct Printer<'a> {
    options: PrettierOptions,

    /// The final output string in bytes
    out: std::vec::Vec<u8>,
    pos: usize,
    cmds: std::vec::Vec<Command<'a>>,
    should_remeasure: bool,

    line_suffix: std::vec::Vec<Command<'a>>,
    group_mode_map: FxHashMap<GroupId, Mode>,

    new_line: &'static str,

    allocator: &'a Allocator,
}

impl<'a> Printer<'a> {
    pub fn new(
        doc: Doc<'a>,
        source_text: &str,
        options: PrettierOptions,
        allocator: &'a Allocator,
    ) -> Self {
        let cmds = vec![Command::new(Indent::root(), Mode::Break, doc)];

        Self {
            options,
            // Preallocate for performance
            // the output will very likely be the same size as the original text
            out: std::vec::Vec::with_capacity(source_text.len()),
            pos: 0,
            cmds,
            should_remeasure: false,
            line_suffix: vec![],
            group_mode_map: FxHashMap::default(),
            new_line: options.end_of_line.as_str(),
            allocator,
        }
    }

    pub fn build(mut self) -> String {
        self.print_doc_to_string();
        // SAFETY: We should have constructed valid UTF8 strings
        unsafe { String::from_utf8_unchecked(self.out) }
    }

    fn print_doc_to_string(&mut self) {
        while let Some(Command { indent, mut doc, mode }) = self.cmds.pop() {
            // NOTE: In Prettier, they perform this before the loop
            propagate_breaks(&mut doc);

            match doc {
                Doc::Str(s) => self.handle_str(s),
                Doc::Array(docs) => self.handle_array(indent, mode, docs),
                Doc::Indent(docs) => self.handle_indent(indent, mode, docs),
                // TODO: Doc::Align
                Doc::Group(group) => self.handle_group(indent, mode, group),
                Doc::Fill(fill) => self.handle_fill(indent, mode, fill),
                Doc::IfBreak(if_break) => self.handle_if_break(indent, mode, if_break),
                Doc::IndentIfBreak(indent_if_break) => {
                    self.handle_indent_if_break(indent, mode, indent_if_break);
                }
                Doc::LineSuffix(docs) => self.handle_line_suffix(indent, mode, docs),
                Doc::LineSuffixBoundary => self.handle_line_suffix_boundary(indent, mode),
                Doc::Line(line) => self.handle_line(indent, mode, line),
                // TODO: Doc::Label
                Doc::BreakParent => { /* No op */ }
            }

            if self.cmds.is_empty() && !self.line_suffix.is_empty() {
                self.cmds.extend(self.line_suffix.drain(..).rev());
            }
        }
    }

    fn handle_str(&mut self, s: &str) {
        // TODO: In Prettier, they replace `\r\n` and `\r` with `\n` before formatting
        // Then, they replace `\n` with `self.new_line(= options.endOfLine)` here if needed
        // And `tests/format/**` is not aware of this!
        self.out.extend(s.as_bytes());
        self.pos += s.len();
    }

    fn handle_array(&mut self, indent: Indent, mode: Mode, docs: Vec<'a, Doc<'a>>) {
        self.cmds.extend(docs.into_iter().rev().map(|doc| Command::new(indent, mode, doc)));
    }

    fn handle_indent(&mut self, indent: Indent, mode: Mode, docs: Vec<'a, Doc<'a>>) {
        self.cmds.extend(
            docs.into_iter()
                .rev()
                .map(|doc| Command::new(Indent::new(indent.length + 1), mode, doc)),
        );
    }

    // TODO: fn handle_align
    // cmds.push({ ind: makeAlign(ind, doc.n, options), mode, doc: doc.contents });

    fn handle_group(&mut self, indent: Indent, mode: Mode, group: Group<'a>) {
        let mut set_group_mode = |this: &mut Self, id| {
            let Some(id) = id else {
                return;
            };
            let Some(mode) = this.cmds.last().map(|cmd| cmd.mode) else {
                return;
            };
            this.group_mode_map.insert(id, mode);
        };

        match mode {
            Mode::Flat => {
                self.cmds.extend(group.contents.into_iter().rev().map(|doc| {
                    Command::new(indent, if group.should_break { Mode::Break } else { mode }, doc)
                }));

                set_group_mode(self, group.group_id);
            }
            Mode::Break => {
                let remaining_width = self.remaining_width();
                let should_break = group.should_break;
                let group_id = group.group_id;
                let cmd = Command::new(indent, Mode::Flat, Doc::Group(group));
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

                set_group_mode(self, group_id);
            }
        }
    }

    // Fills each line with as much code as possible before moving to a new line with the same indentation.
    //
    // Expects doc.parts to be an array of alternating content and whitespace.
    // The whitespace contains the linebreaks.
    //
    // For example:
    //   ["I", line, "love", line, "monkeys"]
    // or
    //   [{ type: group, ... }, softline, { type: group, ... }]
    //
    // It uses this parts structure to handle three main layout cases:
    // - The first two content items fit on the same line without breaking
    //   -> output the first content item and the whitespace "flat".
    // - Only the first content item fits on the line without breaking
    //   -> output the first content item "flat" and the whitespace with
    //   "break".
    // - Neither content item fits on the line without breaking
    //   -> output the first content item and the whitespace with "break".
    fn handle_fill(&mut self, indent: Indent, mode: Mode, fill: Fill<'a>) {
        let mut fill = fill;

        let remaining_width = self.remaining_width();
        let original_parts_len = fill.len();
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

        let mut docs = Vec::new_in(self.allocator);
        docs.push(content_flat_cmd.doc);
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

    fn handle_if_break(&mut self, indent: Indent, mode: Mode, if_break: IfBreak<'a>) {
        let IfBreak { break_contents, flat_contents, group_id } = if_break;
        let group_mode = group_id
            .map_or(Some(mode), |id| self.group_mode_map.get(&id).copied())
            .expect("`group_mode` should be exists");

        match group_mode {
            Mode::Break => {
                self.cmds.push(Command::new(indent, mode, break_contents.unbox()));
            }
            Mode::Flat => {
                self.cmds.push(Command::new(indent, mode, flat_contents.unbox()));
            }
        }
    }

    fn handle_indent_if_break(
        &mut self,
        indent: Indent,
        mode: Mode,
        indent_if_break: IndentIfBreak<'a>,
    ) {
        let IndentIfBreak { contents, group_id } = indent_if_break;
        let group_mode =
            self.group_mode_map.get(&group_id).copied().expect("`group_mode` should be exists");

        match group_mode {
            Mode::Break => {
                // Same effect as doing:
                // `self.cmds.push(Command::new(indent, mode, crates::indent!(self, [contents.unbox()])))`
                self.cmds.push(Command::new(
                    Indent::new(indent.length + 1),
                    mode,
                    contents.unbox(),
                ));
            }
            Mode::Flat => {
                self.cmds.push(Command::new(indent, mode, contents.unbox()));
            }
        }
    }

    fn handle_line_suffix(&mut self, indent: Indent, mode: Mode, docs: Vec<'a, Doc<'a>>) {
        self.line_suffix.push(Command { indent, mode, doc: Doc::Array(docs) });
    }

    fn handle_line_suffix_boundary(&mut self, indent: Indent, mode: Mode) {
        if !self.line_suffix.is_empty() {
            let hardline_without_break_parent = Doc::Line(Line { hard: true, ..Line::default() });
            self.cmds.push(Command { indent, mode, doc: hardline_without_break_parent });
        }
    }

    fn handle_line(&mut self, indent: Indent, mode: Mode, line: Line) {
        let Line { hard, soft, literal } = line;

        if matches!(mode, Mode::Flat) {
            if hard {
                // This line was forced into the output even if we were in flattened mode,
                // so we need to tell the next group that no matter what, it needs to remeasure
                // because the previous measurement didn't accurately capture the entire expression.
                // (this is necessary for nested groups)
                self.should_remeasure = true;
            } else {
                if !soft {
                    self.out.push(b' ');
                    self.pos += 1;
                }
                return;
            }
        }

        // `Mode::Break` or `Mode:Flat` w/ `should_remeasure`

        if !self.line_suffix.is_empty() {
            self.cmds.push(Command::new(indent, mode, Doc::Line(line)));
            self.cmds.extend(self.line_suffix.drain(..).rev());
            return;
        }

        if literal {
            self.out.extend(self.new_line.as_bytes());
            if !indent.root {
                self.pos = 0;
            }
        } else {
            // Trim `Tab(U+0009)` and `Space(U+0020)` at the end of line
            while let Some(&last) = self.out.last() {
                if last == b' ' || last == b'\t' {
                    self.out.pop();
                } else {
                    break;
                }
            }

            self.out.extend(self.new_line.as_bytes());
            // Resolve indent type and value
            let size = indent.length;
            if self.options.use_tabs {
                self.out.extend("\t".repeat(size).as_bytes());
                self.pos = size;
            } else {
                let count = self.options.tab_width * size;
                self.out.extend(" ".repeat(count).as_bytes());
                self.pos = count;
            }
        }
    }

    // TODO: fn handle_label
    // cmds.push({ ind, mode, doc: doc.contents });

    // ---

    #[expect(clippy::cast_possible_wrap)]
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
                Doc::IndentIfBreak(IndentIfBreak { contents, .. }) => {
                    queue.push_front((mode, contents));
                }
                Doc::Indent(docs) | Doc::Array(docs) => {
                    // Prepend docs to the queue
                    for d in docs.iter().rev() {
                        queue.push_front((mode, d));
                    }
                }
                Doc::Group(group) => {
                    let mode = if group.should_break { Mode::Break } else { mode };
                    if group.expanded_states.is_some() && matches!(mode, Mode::Break) {
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

                    let contents = if matches!(group_mode, Mode::Break) {
                        &if_break_doc.break_contents
                    } else {
                        &if_break_doc.flat_contents
                    };

                    queue.push_front((mode, contents));
                }
                Doc::Line(line) => {
                    if matches!(mode, Mode::Break) || line.hard {
                        return true;
                    }
                    if !line.soft {
                        remaining_width -= 1_isize;
                    }
                }
                Doc::Fill(fill) => {
                    for part in fill.parts.iter().rev() {
                        queue.push_front((mode, part));
                    }
                }
                Doc::LineSuffix(_) => {
                    break;
                }
                Doc::LineSuffixBoundary => {
                    if !self.line_suffix.is_empty() {
                        return false;
                    }
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

    #[expect(clippy::cast_possible_wrap)]
    fn remaining_width(&self) -> isize {
        (self.options.print_width as isize) - (self.pos as isize)
    }
}

// TODO: I tried to write a similar code in Prettier, there was a test that failed
// - Almost all tests pass, but for some reason, only a few cases in the `jsx/text-wrap` fails
// - I'm not sure whether this logic has a problem or the `Doc` printing logic has a problem
// PERF: When taking a `Doc` other than `Group` as a target, unnecessary traversal occurs
// - In this implementation, `should_break` is updated only when the argument is `Group`
// - This can be resolved by separating the recursive part from the entry, but it can be done later
// PERF: When `Group` is nested, intermediate results should be reused
// - This occurs when the structure is like `Group > Group > BreakParent`
// - When processing the 1st `Group`, it should be known that the 2nd `Group` also breaks
fn propagate_breaks(doc: &mut Doc<'_>) -> bool {
    let apply_vec = |arr: &mut Vec<'_, Doc<'_>>| arr.iter_mut().any(propagate_breaks);

    match doc {
        Doc::BreakParent => true,
        Doc::Group(group) => {
            // NOTE: This is important, propagating breaks
            if group.expanded_states.is_none() && apply_vec(&mut group.contents) {
                // In Prettier, they seem to use a string `"propagated"`(as truthy value)
                // to distinguish from original `shouldBreak: true` for `printDocToDebug()`
                group.should_break = true;
            }
            group.should_break
        }
        // TODO: | Doc::Align(arr)
        Doc::Array(arr)
        | Doc::Fill(Fill { parts: arr })
        | Doc::Indent(arr)
        | Doc::LineSuffix(arr) => apply_vec(arr),
        Doc::IndentIfBreak(IndentIfBreak { contents, .. }) => propagate_breaks(contents),
        Doc::IfBreak(IfBreak { break_contents, flat_contents, .. }) => {
            propagate_breaks(flat_contents) || propagate_breaks(break_contents)
        }
        _ => false,
    }
}

impl<'a> Fill<'a> {
    pub fn drain_out_pair(&mut self) -> (Option<Doc<'a>>, Option<Doc<'a>>) {
        let content = if self.parts.len() > 0 { Some(self.parts.remove(0)) } else { None };
        let whitespace = if self.parts.len() > 0 { Some(self.parts.remove(0)) } else { None };
        (content, whitespace)
    }

    pub fn dequeue(&mut self) -> Option<Doc<'a>> {
        if self.parts.len() > 0 { Some(self.parts.remove(0)) } else { None }
    }

    pub fn enqueue(&mut self, doc: Doc<'a>) {
        self.parts.insert(0, doc);
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }
}
