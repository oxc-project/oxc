use std::collections::VecDeque;

use oxc_allocator::{Allocator, Vec};
use rustc_hash::FxHashMap;

use crate::{
    GroupId, PrettierOptions,
    ir::{Doc, Fill, Group, IfBreak, IndentIfBreak, Line},
    print::command::{Command, Indent, Mode},
};

pub struct Printer<'a> {
    // This is needed to create temporary `Doc<'a>` in `handle_fill()`
    allocator: &'a Allocator,
    options: PrettierOptions,
    /// The final output string in bytes
    out: std::vec::Vec<u8>,
    // States for `print_doc_to_string()`
    pos: usize,
    cmds: std::vec::Vec<Command<'a>>,
    line_suffix: std::vec::Vec<Command<'a>>,
    group_mode_map: FxHashMap<GroupId, Mode>,
}

impl<'a> Printer<'a> {
    pub fn new(
        allocator: &'a Allocator,
        doc: Doc<'a>,
        options: PrettierOptions,
        out_size_hint: usize,
    ) -> Self {
        Self {
            allocator,
            options,
            // Preallocate for performance
            // the output will very likely be the same size as the original text
            out: std::vec::Vec::with_capacity(out_size_hint),
            pos: 0,
            cmds: vec![Command::new(Indent::root(), Mode::Break, doc)],
            line_suffix: vec![],
            group_mode_map: FxHashMap::default(),
        }
    }

    pub fn build(mut self) -> String {
        self.print_doc_to_string();
        // SAFETY: We should have constructed valid UTF8 strings
        unsafe { String::from_utf8_unchecked(self.out) }
    }

    fn print_doc_to_string(&mut self) {
        while let Some(Command { indent, mut doc, mode }) = self.cmds.pop() {
            // TODO: In Prettier, this process is done outside the loop first. (P1)
            // When I try to rewrite the Prettier code to do this inside the loop like us(P2), some of the `jsx/text-wrap` tests fail.
            // By the way, in P1 and P2, the structure of `Doc` after propagation is the same!
            // It means that there is a timing in this printing process where
            // what should originally be `break: true` is not judged as `true`, and that leads to differences in behavior.
            // We may have no choice but to implement it like Prettier?
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
    // Current implementation resolves indent values manually instead of using `makeAlign()` in Prettier, may need to rework.
    // See also `handle_indent()` and the last part of `handle_line`
    // cmds.push({ ind: makeAlign(ind, doc.n, options), mode, doc: doc.contents });

    fn handle_group(&mut self, indent: Indent, mode: Mode, group: Group<'a>) {
        let Group { contents, should_break, expanded_states, group_id } = group;

        match mode {
            Mode::Flat => {
                self.cmds.extend(contents.into_iter().rev().map(|doc| {
                    Command::new(indent, if should_break { Mode::Break } else { Mode::Flat }, doc)
                }));
            }
            Mode::Break => {
                let cmd = Command::new(indent, Mode::Flat, Doc::Array(contents));

                if !should_break && self.fits(&cmd, false) {
                    self.cmds.push(cmd);
                } else {
                    // Expanded states are a rare case
                    // where a document can manually provide multiple representations of itself.
                    // It provides an array of documents going from
                    // the least expanded (most flattened) representation first to the most expanded.
                    // If a group has these,
                    // we need to manually go through these states and find the first one that fits.
                    if let Some(mut expanded_states) = expanded_states {
                        let most_expanded =
                            expanded_states.pop().expect("`expanded_states` should not be empty");

                        if should_break {
                            self.cmds.push(Command::new(indent, Mode::Break, most_expanded));
                            return;
                        }

                        for state in expanded_states {
                            let cmd = Command::new(indent, Mode::Flat, state);
                            if self.fits(&cmd, false) {
                                self.cmds.push(cmd);
                                return;
                            }
                        }

                        self.cmds.push(Command::new(indent, Mode::Break, most_expanded));
                    } else {
                        self.cmds.push(cmd.with_mode(Mode::Break));
                    }
                }
            }
        }

        if let (Some(id), Some(mode)) = (group_id, self.cmds.last().map(|cmd| cmd.mode)) {
            self.group_mode_map.insert(id, mode);
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

        let original_parts_len = fill.len();
        let (content, whitespace) = fill.drain_out_pair();

        let Some(content) = content else {
            return;
        };
        let content_flat_cmd = Command::new(indent, Mode::Flat, content);
        let content_fits = self.fits(&content_flat_cmd, true);

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
        let first_and_second_content_fits = self.fits(&first_and_second_content_fit_cmd, true);
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
        let new_line = self.options.end_of_line.as_str().as_bytes();

        if matches!(mode, Mode::Flat) {
            if hard {
                // This is defined in Prettier but it does not seem to take effect?
                // self.should_remeasure = true;
            } else {
                if !soft {
                    self.out.push(b' ');
                    self.pos += 1;
                }
                return;
            }
        }

        // `Mode::Break` or `Mode:Flat` w/ `hard: true`

        if !self.line_suffix.is_empty() {
            self.cmds.push(Command::new(indent, mode, Doc::Line(line)));
            self.cmds.extend(self.line_suffix.drain(..).rev());
            return;
        }

        if literal {
            self.out.extend(new_line);
            if !indent.root {
                self.pos = 0;
            }
        } else {
            // Trim `Tab(U+0009)` and `Space(U+0020)` at the end of line
            while let Some(&last) = self.out.last() {
                if last == b' ' || last == b'\t' {
                    self.out.pop();
                    self.pos -= 1;
                } else {
                    break;
                }
            }

            self.out.extend(new_line);
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

    fn fits(&self, next: &Command<'a>, in_fill: bool) -> bool {
        #[expect(clippy::cast_possible_wrap)]
        let mut remaining_width = (self.options.print_width as isize) - (self.pos as isize);
        let mut has_line_suffix = !self.line_suffix.is_empty();
        let mut cmds = self.cmds.iter().rev();

        let mut queue: VecDeque<(Mode, &Doc)> = VecDeque::new();
        queue.push_front((next.mode, &next.doc));

        while let Some((mode, doc)) = queue.pop_front() {
            match doc {
                #[expect(clippy::cast_possible_wrap)]
                Doc::Str(s) => {
                    remaining_width -= s.len() as isize;
                }
                // TODO: Doc::Align
                // TODO: Doc::Label
                Doc::Array(docs) | Doc::Fill(Fill { parts: docs }) | Doc::Indent(docs) => {
                    // In Prettier, `DOC_FILL_PRINTED_LENGTH` is used for `Fill`, but always returns `0`?
                    for doc in docs.iter().rev() {
                        queue.push_front((mode, doc));
                    }
                }
                Doc::IndentIfBreak(IndentIfBreak { contents, .. }) => {
                    queue.push_front((mode, contents));
                }
                Doc::Group(Group { contents, should_break, expanded_states, .. }) => {
                    if in_fill && *should_break {
                        return false;
                    }

                    let mode = if *should_break { Mode::Break } else { mode };

                    // The most expanded state takes up the least space on the current line.
                    if expanded_states.is_some() && matches!(mode, Mode::Break) {
                        queue.push_front((mode, expanded_states.as_ref().unwrap().last().unwrap()));
                    } else {
                        for doc in contents.iter().rev() {
                            queue.push_front((mode, doc));
                        }
                    };
                }
                Doc::IfBreak(IfBreak { break_contents, flat_contents, group_id }) => {
                    let group_mode = group_id
                        .map_or(mode, |id| *self.group_mode_map.get(&id).unwrap_or(&Mode::Flat));
                    let doc = if matches!(group_mode, Mode::Break) {
                        &break_contents
                    } else {
                        &flat_contents
                    };

                    queue.push_front((mode, doc));
                }
                Doc::Line(line) => {
                    if matches!(mode, Mode::Break) || line.hard {
                        return true;
                    }

                    if !line.soft {
                        remaining_width -= 1_isize;
                    }
                }
                Doc::LineSuffix(_) => {
                    has_line_suffix = true;
                    break;
                }
                Doc::LineSuffixBoundary => {
                    if has_line_suffix {
                        return false;
                    }
                    break;
                }
                Doc::BreakParent => {}
            }

            if remaining_width < 0 {
                return false;
            }

            if in_fill && queue.is_empty() {
                return true;
            }
            if let Some(cmd) = cmds.next() {
                queue.push_back((cmd.mode, &cmd.doc));
            }
        }

        true
    }
}

fn propagate_breaks(doc: &mut Doc<'_>) -> bool {
    let apply_vec = |arr: &mut Vec<'_, Doc<'_>>| arr.iter_mut().any(propagate_breaks);

    match doc {
        Doc::BreakParent => true,
        Doc::Group(group) => {
            // NOTE: This is important, propagating breaks
            if group.expanded_states.is_none() && apply_vec(&mut group.contents) {
                // In Prettier, they use a string `"propagated"`(as truthy value)
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

// NOTE: In Prettier, the secret property `DOC_FILL_PRINTED_LENGTH` is used for `Fill`.
// It stores the offset already printed in the former printing and used in later printing.
// However, we do not maange this and directly update `parts` itself.
// The following is a utility for this.
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
