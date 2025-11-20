mod call_stack;
mod line_suffixes;
mod printer_options;
mod queue;
mod stack;

use std::num::NonZeroU8;

use oxc_data_structures::code_buffer::{self, CodeBuffer};
pub use printer_options::*;
use unicode_width::UnicodeWidthChar;

use super::{
    ActualStart, FormatElement, GroupId, InvalidDocumentError, PrintError, PrintResult, Printed,
    TextRange, TextSize,
    format_element::{BestFittingElement, LineMode, PrintMode, document::Document, tag::Condition},
    prelude::{
        Tag::EndFill,
        TextWidth,
        tag::{DedentMode, Tag, TagKind},
    },
    printer::{
        call_stack::{CallStack, FitsCallStack, PrintCallStack, PrintElementArgs, StackFrame},
        line_suffixes::{LineSuffixEntry, LineSuffixes},
        queue::{
            AllPredicate, FitsEndPredicate, FitsQueue, PrintQueue, Queue, SingleEntryPredicate,
        },
    },
};
use crate::options::IndentStyle;

/// Prints the format elements into a string
#[derive(Debug, Default)]
pub struct Printer<'a> {
    options: PrinterOptions,
    state: PrinterState<'a>,
}

impl<'a> Printer<'a> {
    pub fn new(options: PrinterOptions) -> Self {
        let (indent_char, indent_width) = match options.indent_style() {
            IndentStyle::Tab => (code_buffer::IndentChar::Tab, 1),
            IndentStyle::Space => {
                (code_buffer::IndentChar::Space, options.indent_width().value() as usize)
            }
        };
        let buffer = CodeBuffer::with_indent(indent_char, indent_width);
        Self { options, state: PrinterState { buffer, ..Default::default() } }
    }

    /// Prints the passed in element as well as all its content
    pub fn print(self, document: &'a Document) -> PrintResult<Printed> {
        self.print_with_indent(document, 0)
    }

    /// Prints the passed in element as well as all its content,
    /// starting at the specified indentation level
    pub fn print_with_indent(
        mut self,
        document: &'a Document<'a>,
        indent: u16,
    ) -> PrintResult<Printed> {
        let mut stack = PrintCallStack::new(PrintElementArgs::new());
        let mut queue: PrintQueue<'a> = PrintQueue::new(document.as_ref());
        let mut current_indent = Indention::Level(indent);
        let mut saved_for_dedent: Option<Indention> = None;
        let mut suffix_indents: Vec<Indention> = Vec::new();

        while let Some(element) = queue.pop() {
            self.print_element(
                &mut stack,
                &mut current_indent,
                &mut saved_for_dedent,
                &mut suffix_indents,
                &mut queue,
                element,
            )?;

            if queue.is_empty() {
                self.flush_line_suffixes(
                    &mut queue,
                    &mut stack,
                    &mut current_indent,
                    &mut saved_for_dedent,
                    &mut suffix_indents,
                    None,
                );
            }
        }

        Ok(Printed::new(self.state.buffer.into_string(), None))
    }

    /// Prints a single element and push the following elements to queue
    fn print_element(
        &mut self,
        stack: &mut PrintCallStack,
        current_indent: &mut Indention,
        saved_for_dedent: &mut Option<Indention>,
        suffix_indents: &mut Vec<Indention>,
        queue: &mut PrintQueue<'a>,
        element: &'a FormatElement,
    ) -> PrintResult<()> {
        use Tag::{
            EndAlign, EndConditionalContent, EndDedent, EndEntry, EndFill, EndGroup, EndIndent,
            EndIndentIfGroupBreaks, EndLabelled, EndLineSuffix, StartAlign,
            StartConditionalContent, StartDedent, StartEntry, StartFill, StartGroup, StartIndent,
            StartIndentIfGroupBreaks, StartLabelled, StartLineSuffix,
        };

        let args = stack.top();
        match element {
            FormatElement::Space | FormatElement::HardSpace => {
                if self.state.line_width > 0 {
                    self.state.pending_space = true;
                }
            }

            FormatElement::Token { text } => self.print_text(Text::Token(text)),
            FormatElement::Text { text, width } => {
                self.print_text(Text::Text { text, width: *width });
            }
            FormatElement::Line(line_mode) => {
                if args.mode().is_flat() {
                    match line_mode {
                        LineMode::Soft | LineMode::SoftOrSpace => {
                            if line_mode == &LineMode::SoftOrSpace && self.state.line_width > 0 {
                                self.state.pending_space = true;
                            }
                            return Ok(());
                        }
                        LineMode::Hard | LineMode::Empty => {
                            self.state.measured_group_fits = false;
                        }
                    }
                }

                if self.state.line_suffixes.has_pending() {
                    self.flush_line_suffixes(
                        queue,
                        stack,
                        current_indent,
                        saved_for_dedent,
                        suffix_indents,
                        Some(element),
                    );
                    return Ok(());
                }

                // Only print a newline if the current line isn't already empty
                if self.state.line_width > 0 {
                    self.print_char('\n');
                    self.state.has_empty_line = false;
                }

                // Print a second line break if this is an empty line
                if line_mode == &LineMode::Empty && !self.state.has_empty_line {
                    self.print_char('\n');
                    self.state.has_empty_line = true;
                }

                self.state.pending_space = false;
                self.state.pending_indent = *current_indent;
            }

            FormatElement::ExpandParent => {
                // Handled in `Document::propagate_expands()
            }

            FormatElement::LineSuffixBoundary => {
                const HARD_BREAK: &FormatElement = &FormatElement::Line(LineMode::Hard);
                self.flush_line_suffixes(
                    queue,
                    stack,
                    current_indent,
                    saved_for_dedent,
                    suffix_indents,
                    Some(HARD_BREAK),
                );
            }

            FormatElement::BestFitting(best_fitting) => {
                self.print_best_fitting(
                    best_fitting,
                    queue,
                    stack,
                    current_indent,
                    saved_for_dedent,
                    suffix_indents,
                )?;
            }

            FormatElement::Interned(content) => {
                queue.extend_back(content);
            }

            FormatElement::Tag(StartGroup(group)) => {
                let group_mode = if group.mode().is_flat() {
                    match args.mode() {
                        PrintMode::Flat if self.state.measured_group_fits => {
                            // A parent group has already verified that this group fits on a single line
                            // Thus, just continue in flat mode
                            PrintMode::Flat
                        }
                        // The printer is either in expanded mode or it's necessary to re-measure if the group fits
                        // because the printer printed a line break
                        _ => {
                            self.state.measured_group_fits = true;

                            if let Some(id) = group.id() {
                                self.state.group_modes.insert_print_mode(id, PrintMode::Flat);
                            }

                            // Measure to see if the group fits up on a single line. If that's the case,
                            // print the group in "flat" mode, otherwise continue in expanded mode
                            stack.push(
                                TagKind::Group,
                                args.with_print_mode(PrintMode::Flat),
                                *current_indent,
                            );
                            let fits = self.fits(
                                queue,
                                stack,
                                *current_indent,
                                *saved_for_dedent,
                                suffix_indents,
                            )?;
                            let (_args, restore) = stack.pop(TagKind::Group)?;
                            *current_indent = restore;

                            if fits { PrintMode::Flat } else { PrintMode::Expanded }
                        }
                    }
                } else {
                    PrintMode::Expanded
                };

                stack.push(TagKind::Group, args.with_print_mode(group_mode), *current_indent);

                if let Some(id) = group.id() {
                    self.state.group_modes.insert_print_mode(id, group_mode);
                }
            }

            FormatElement::Tag(StartFill) => {
                self.print_fill_entries(
                    queue,
                    stack,
                    current_indent,
                    saved_for_dedent,
                    suffix_indents,
                )?;
            }

            FormatElement::Tag(StartIndent) => {
                let restore = *current_indent;
                *current_indent = current_indent.increment_level(self.options.indent_style());
                stack.push(TagKind::Indent, args, restore);
            }

            FormatElement::Tag(StartDedent(mode)) => match mode {
                DedentMode::Level => {
                    *saved_for_dedent = Some(*current_indent);
                    stack.push(TagKind::Dedent, args, *current_indent);
                }
                DedentMode::Root => {
                    let restore = *current_indent;
                    *current_indent = Indention::default();
                    stack.push(TagKind::Dedent, args, restore);
                }
            },

            FormatElement::Tag(StartAlign(align)) => {
                let restore = *current_indent;
                *current_indent = current_indent.set_align(align.count());
                stack.push(TagKind::Align, args, restore);
            }

            FormatElement::Tag(StartConditionalContent(Condition { mode, group_id })) => {
                let group_mode = match group_id {
                    None => args.mode(),
                    Some(id) => self.state.group_modes.unwrap_print_mode(*id, element),
                };

                if group_mode == *mode {
                    stack.push(TagKind::ConditionalContent, args, *current_indent);
                } else {
                    queue.skip_content(TagKind::ConditionalContent);
                }
            }

            FormatElement::Tag(StartIndentIfGroupBreaks(group_id)) => {
                let group_mode = self.state.group_modes.unwrap_print_mode(*group_id, element);

                let restore = *current_indent;
                if group_mode == PrintMode::Expanded {
                    *current_indent = current_indent.increment_level(self.options.indent_style());
                }

                stack.push(TagKind::IndentIfGroupBreaks, args, restore);
            }

            FormatElement::Tag(StartLineSuffix) => {
                suffix_indents.push(*current_indent);
                self.state.line_suffixes.extend(args, queue.iter_content(TagKind::LineSuffix));
            }
            FormatElement::Tag(tag @ (StartLabelled(_) | StartEntry)) => {
                stack.push(tag.kind(), args, *current_indent);
            }
            FormatElement::Tag(
                tag @ (EndLabelled | EndEntry | EndGroup | EndConditionalContent | EndFill | EndIndent | EndAlign | EndLineSuffix),
            ) => {
                let (_args, restore) = stack.pop(tag.kind())?;
                *current_indent = restore;
            }
            FormatElement::Tag(tag @ EndIndentIfGroupBreaks(group_id)) => {
                let (_args, restore) = stack.pop(tag.kind())?;
                if self.state.group_modes.unwrap_print_mode(*group_id, element)
                    == PrintMode::Expanded
                {
                    *current_indent = restore;
                }
            }
            FormatElement::Tag(tag @ EndDedent(mode)) => {
                let (_args, restore) = stack.pop(tag.kind())?;
                match mode {
                    DedentMode::Level => {
                        *current_indent = saved_for_dedent.take().unwrap_or_default();
                    }
                    DedentMode::Root => {
                        *current_indent = restore;
                    }
                }
            }
        }

        Ok(())
    }

    fn fits(
        &mut self,
        queue: &PrintQueue<'a>,
        stack: &PrintCallStack,
        current_indent: Indention,
        saved_for_dedent: Option<Indention>,
        suffix_indents: &[Indention],
    ) -> PrintResult<bool> {
        let mut measure =
            FitsMeasurer::new(queue, stack, current_indent, saved_for_dedent, suffix_indents, self);
        let result = measure.fits(&mut AllPredicate);
        measure.finish();
        result
    }

    fn flush_line_suffixes(
        &mut self,
        queue: &mut PrintQueue<'a>,
        stack: &mut PrintCallStack,
        current_indent: &mut Indention,
        _saved_for_dedent: &mut Option<Indention>,
        suffix_indents: &mut Vec<Indention>,
        line_break: Option<&'a FormatElement>,
    ) {
        let suffixes = self.state.line_suffixes.take_pending();

        if suffixes.len() > 0 {
            // Print this line break element again once all the line suffixes have been flushed
            if let Some(line_break) = line_break {
                queue.push(line_break);
            }

            // In the old implementation, suffix indents were reversed and pushed onto the stack.
            // When processing suffixes in reverse order, the first suffix processed (last pushed)
            // uses the first indent saved (which ended up at the top after reversal).
            // We achieve this by iterating suffix_indents forward while processing suffixes in reverse.
            let mut suffix_iter = suffix_indents.iter();

            // Track what indent to restore after processing all suffixes
            let mut restore_chain_start = *current_indent;

            // Push suffix content and EndLineSuffix tags
            // Process in reverse order so they're popped in correct order
            for entry in suffixes.rev() {
                match entry {
                    LineSuffixEntry::Suffix(suffix) => {
                        queue.push(suffix);
                    }
                    LineSuffixEntry::Args(args) => {
                        // Get the saved indent that should be active for this suffix
                        if let Some(&saved_indent) = suffix_iter.next() {
                            // Push EndLineSuffix with the restore value
                            // This creates a chain: ... -> saved_indent -> restore_chain_start
                            const LINE_SUFFIX_END: &FormatElement =
                                &FormatElement::Tag(Tag::EndLineSuffix);
                            stack.push(TagKind::LineSuffix, args, restore_chain_start);
                            queue.push(LINE_SUFFIX_END);
                            // Next EndLineSuffix should restore to this saved_indent
                            restore_chain_start = saved_indent;
                        }
                    }
                }
            }

            // Set current_indent to the last saved indent (first to be processed)
            *current_indent = restore_chain_start;

            // Clear suffix_indents since we've processed them all
            suffix_indents.clear();
        }
    }

    fn print_best_fitting(
        &mut self,
        best_fitting: &'a BestFittingElement,
        queue: &mut PrintQueue<'a>,
        stack: &mut PrintCallStack,
        current_indent: &mut Indention,
        saved_for_dedent: &mut Option<Indention>,
        suffix_indents: &mut Vec<Indention>,
    ) -> PrintResult<()> {
        let args = stack.top();

        if args.mode().is_flat() && self.state.measured_group_fits {
            queue.extend_back(best_fitting.most_flat());
            self.print_entry(queue, stack, current_indent, saved_for_dedent, suffix_indents, args)
        } else {
            self.state.measured_group_fits = true;

            for variant in best_fitting.variants() {
                // Test if this variant fits and if so, use it. Otherwise try the next
                // variant.

                // Try to fit only the first variant on a single line
                if !matches!(variant.first(), Some(&FormatElement::Tag(Tag::StartEntry))) {
                    return invalid_start_tag(TagKind::Entry, variant.first());
                }

                let entry_args = args.with_print_mode(PrintMode::Flat);

                // Skip the first element because we want to override the args for the entry and the
                // args must be popped from the stack as soon as it sees the matching end entry.
                let content = &variant[1..];

                queue.extend_back(content);
                stack.push(TagKind::Entry, entry_args, *current_indent);
                let variant_fits =
                    self.fits(queue, stack, *current_indent, *saved_for_dedent, suffix_indents)?;
                let (_args, restore) = stack.pop(TagKind::Entry)?;
                *current_indent = restore;

                // Remove the content slice because printing needs the variant WITH the start entry
                let popped_slice = queue.pop_slice();
                debug_assert_eq!(popped_slice, Some(content));

                if variant_fits {
                    queue.extend_back(variant);
                    return self.print_entry(
                        queue,
                        stack,
                        current_indent,
                        saved_for_dedent,
                        suffix_indents,
                        entry_args,
                    );
                }
            }

            // No variant fits, take the last (most expanded) as fallback
            let most_expanded = best_fitting.most_expanded();
            queue.extend_back(most_expanded);
            self.print_entry(
                queue,
                stack,
                current_indent,
                saved_for_dedent,
                suffix_indents,
                args.with_print_mode(PrintMode::Expanded),
            )
        }
    }

    /// Tries to fit as much content as possible on a single line.
    ///
    /// `Fill` is a sequence of *item*, *separator*, *item*, *separator*, *item*, ... entries.
    /// The goal is to fit as many items (with their separators) on a single line as possible and
    /// first expand the *separator* if the content exceeds the print width and only fallback to expanding
    /// the *item*s if the *item* or the *item* and the expanded *separator* don't fit on the line.
    ///
    /// The implementation handles the following 5 cases:
    ///
    /// * The *item*, *separator*, and the *next item* fit on the same line.
    ///   Print the *item* and *separator* in flat mode.
    /// * The *item* and *separator* fit on the line but there's not enough space for the *next item*.
    ///   Print the *item* in flat mode and the *separator* in expanded mode.
    /// * The *item* fits on the line but the *separator* does not in flat mode.
    ///   Print the *item* in flat mode and the *separator* in expanded mode.
    /// * The *item* fits on the line but the *separator* does not in flat **NOR** expanded mode.
    ///   Print the *item* and *separator* in expanded mode.
    /// * The *item* does not fit on the line.
    ///   Print the *item* and *separator* in expanded mode.
    fn print_fill_entries(
        &mut self,
        queue: &mut PrintQueue<'a>,
        stack: &mut PrintCallStack,
        current_indent: &mut Indention,
        saved_for_dedent: &mut Option<Indention>,
        suffix_indents: &mut Vec<Indention>,
    ) -> PrintResult<()> {
        let args = stack.top();

        // It's already known that the content fit, print all items in flat mode.
        if self.state.measured_group_fits && args.mode().is_flat() {
            stack.push(TagKind::Fill, args.with_print_mode(PrintMode::Flat), *current_indent);
            return Ok(());
        }

        stack.push(TagKind::Fill, args, *current_indent);

        while matches!(queue.top(), Some(FormatElement::Tag(Tag::StartEntry))) {
            let mut measurer = FitsMeasurer::new_flat(
                queue,
                stack,
                *current_indent,
                *saved_for_dedent,
                suffix_indents,
                self,
            );

            // The number of item/separator pairs that fit on the same line.
            let mut flat_pairs = 0usize;
            let mut item_fits = measurer.fill_item_fits()?;

            let last_pair_layout = if item_fits {
                // Measure the remaining pairs until the first item or separator that does not fit (or the end of the fill element).
                // Optimisation to avoid re-measuring the next-item twice:
                // * Once when measuring if the *item*, *separator*, *next-item* fit
                // * A second time when measuring if *next-item*, *separator*, *next-next-item* fit.
                loop {
                    // Item that fits without a following separator.
                    if !matches!(measurer.queue.top(), Some(FormatElement::Tag(Tag::StartEntry))) {
                        break FillPairLayout::Flat;
                    }

                    let separator_fits = measurer.fill_separator_fits(PrintMode::Flat)?;

                    // Item fits but the flat separator does not.
                    if !separator_fits {
                        break FillPairLayout::ItemMaybeFlat;
                    }

                    // Last item/separator pair that both fit
                    if !matches!(measurer.queue.top(), Some(FormatElement::Tag(Tag::StartEntry))) {
                        break FillPairLayout::Flat;
                    }

                    item_fits = measurer.fill_item_fits()?;

                    if item_fits {
                        flat_pairs += 1;
                    } else {
                        // Item and separator both fit, but the next element doesn't.
                        // Print the separator in expanded mode and then re-measure if the item now
                        // fits in the next iteration of the outer loop.
                        break FillPairLayout::ItemFlatSeparatorExpanded;
                    }
                }
            } else {
                // Neither item nor separator fit, print both in expanded mode.
                FillPairLayout::Expanded
            };

            measurer.finish();

            self.state.measured_group_fits = true;

            // Print all pairs that fit in flat mode.
            for _ in 0..flat_pairs {
                self.print_fill_item(
                    queue,
                    stack,
                    current_indent,
                    saved_for_dedent,
                    suffix_indents,
                    args.with_print_mode(PrintMode::Flat),
                )?;
                self.print_fill_separator(
                    queue,
                    stack,
                    current_indent,
                    saved_for_dedent,
                    suffix_indents,
                    args.with_print_mode(PrintMode::Flat),
                )?;
            }

            let item_mode = match last_pair_layout {
                FillPairLayout::Flat | FillPairLayout::ItemFlatSeparatorExpanded => PrintMode::Flat,
                FillPairLayout::Expanded => PrintMode::Expanded,
                FillPairLayout::ItemMaybeFlat => {
                    let mut measurer = FitsMeasurer::new_flat(
                        queue,
                        stack,
                        *current_indent,
                        *saved_for_dedent,
                        suffix_indents,
                        self,
                    );
                    // SAFETY: That the item fits is guaranteed by `ItemMaybeFlat`.
                    // Re-measuring is required to get the measurer in the correct state for measuring the separator.
                    assert!(measurer.fill_item_fits()?);
                    let separator_fits = measurer.fill_separator_fits(PrintMode::Expanded)?;
                    measurer.finish();

                    if separator_fits { PrintMode::Flat } else { PrintMode::Expanded }
                }
            };

            self.print_fill_item(
                queue,
                stack,
                current_indent,
                saved_for_dedent,
                suffix_indents,
                args.with_print_mode(item_mode),
            )?;

            if matches!(queue.top(), Some(FormatElement::Tag(Tag::StartEntry))) {
                let separator_mode = match last_pair_layout {
                    FillPairLayout::Flat => PrintMode::Flat,
                    FillPairLayout::ItemFlatSeparatorExpanded
                    | FillPairLayout::Expanded
                    | FillPairLayout::ItemMaybeFlat => PrintMode::Expanded,
                };

                // Push a new stack frame with print mode `Flat` for the case where the separator gets printed in expanded mode
                // but does contain a group to ensure that the group will measure "fits" with the "flat" versions of the next item/separator.
                stack.push(TagKind::Fill, args.with_print_mode(PrintMode::Flat), *current_indent);
                self.print_fill_separator(
                    queue,
                    stack,
                    current_indent,
                    saved_for_dedent,
                    suffix_indents,
                    args.with_print_mode(separator_mode),
                )?;
                let (_args, restore) = stack.pop(TagKind::Fill)?;
                *current_indent = restore;
            }
        }

        if queue.top() == Some(&FormatElement::Tag(EndFill)) {
            Ok(())
        } else {
            invalid_end_tag(TagKind::Fill, stack.top_kind())
        }
    }

    /// Semantic alias for [Self::print_entry] for fill items.
    fn print_fill_item(
        &mut self,
        queue: &mut PrintQueue<'a>,
        stack: &mut PrintCallStack,
        current_indent: &mut Indention,
        saved_for_dedent: &mut Option<Indention>,
        suffix_indents: &mut Vec<Indention>,
        args: PrintElementArgs,
    ) -> PrintResult<()> {
        self.print_entry(queue, stack, current_indent, saved_for_dedent, suffix_indents, args)
    }

    /// Semantic alias for [Self::print_entry] for fill separators.
    fn print_fill_separator(
        &mut self,
        queue: &mut PrintQueue<'a>,
        stack: &mut PrintCallStack,
        current_indent: &mut Indention,
        saved_for_dedent: &mut Option<Indention>,
        suffix_indents: &mut Vec<Indention>,
        args: PrintElementArgs,
    ) -> PrintResult<()> {
        self.print_entry(queue, stack, current_indent, saved_for_dedent, suffix_indents, args)
    }

    /// Fully print an element (print the element itself and all its descendants)
    ///
    /// Unlike [print_element], this function ensures the entire element has
    /// been printed when it returns and the queue is back to its original state
    fn print_entry(
        &mut self,
        queue: &mut PrintQueue<'a>,
        stack: &mut PrintCallStack,
        current_indent: &mut Indention,
        saved_for_dedent: &mut Option<Indention>,
        suffix_indents: &mut Vec<Indention>,
        args: PrintElementArgs,
    ) -> PrintResult<()> {
        let start_entry = queue.top();

        if !matches!(start_entry, Some(&FormatElement::Tag(Tag::StartEntry))) {
            return invalid_start_tag(TagKind::Entry, start_entry);
        }

        let mut depth = 0;

        while let Some(element) = queue.pop() {
            match element {
                FormatElement::Tag(Tag::StartEntry) => {
                    // Handle the start of the first element by pushing the args on the stack.
                    if depth == 0 {
                        depth = 1;
                        stack.push(TagKind::Entry, args, *current_indent);
                        continue;
                    }

                    depth += 1;
                }
                FormatElement::Tag(Tag::EndEntry) => {
                    depth -= 1;
                    // Reached the end entry, pop the entry from the stack and return.
                    if depth == 0 {
                        let (_args, restore) = stack.pop(TagKind::Entry)?;
                        *current_indent = restore;
                        return Ok(());
                    }
                }
                _ => {
                    // Fall through
                }
            }

            self.print_element(
                stack,
                current_indent,
                saved_for_dedent,
                suffix_indents,
                queue,
                element,
            )?;
        }

        invalid_end_tag(TagKind::Entry, stack.top_kind())
    }

    fn print_text(&mut self, text: Text) {
        if !self.state.pending_indent.is_empty() {
            let indent = std::mem::take(&mut self.state.pending_indent);

            let level = indent.level() as usize;
            self.state.buffer.print_indent(level);
            self.state.line_width += level * self.options.indent_width().value() as usize;

            let align_count = indent.align() as usize;
            for _ in 0..align_count {
                // SAFETY: `' '` is an valid ASCII character
                unsafe {
                    self.state.buffer.print_byte_unchecked(b' ');
                }
            }
            self.state.line_width += align_count;
        }

        // Print pending spaces
        if self.state.pending_space {
            // SAFETY: `' '` is an valid ASCII character
            unsafe {
                self.state.buffer.print_byte_unchecked(b' ');
            }
            self.state.pending_space = false;
            self.state.line_width += 1;
        }

        match text {
            Text::Token(text) => {
                // SAFETY: `text` is a ASCII-only string
                unsafe {
                    self.state.buffer.print_bytes_unchecked(text.as_bytes());
                }
                self.state.line_width += text.len();
            }
            Text::Text { text, width } => {
                if width.is_multiline() {
                    let line_break_position = text.find('\n').unwrap_or(text.len());
                    let (first_line, remaining) = text.split_at(line_break_position);
                    self.state.buffer.print_str(first_line);
                    self.state.line_width += width.value() as usize;
                    // Print the remaining lines
                    for char in remaining.chars() {
                        self.print_char(char);
                    }
                } else {
                    self.state.buffer.print_str(text);
                    self.state.line_width += width.value() as usize;
                }
            }
        }

        self.state.has_empty_line = false;
    }

    fn print_char(&mut self, char: char) {
        if char == '\n' {
            // SAFETY: `line_ending` is one of `\n`, `\r\n` or `\r`, all valid ASCII sequences
            unsafe {
                self.state.buffer.print_bytes_unchecked(self.options.line_ending.as_bytes());
            }

            self.state.line_width = 0;

            // Fit's only tests if groups up to the first line break fit.
            // The next group must re-measure if it still fits.
            self.state.measured_group_fits = false;
        } else {
            let char_width = if char == '\t' {
                // SAFETY: `'\t'` is an valid ASCII character
                unsafe {
                    self.state.buffer.print_byte_unchecked(b'\t');
                }
                self.options.indent_width().value() as usize
            } else {
                self.state.buffer.print_char(char);
                char.width().unwrap_or(0)
            };

            self.state.line_width += char_width;
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum FillPairLayout {
    /// The item, separator, and next item fit. Print the first item and the separator in flat mode.
    Flat,

    /// The item and separator fit but the next element does not. Print the item in flat mode and
    /// the separator in expanded mode.
    ItemFlatSeparatorExpanded,

    /// The item does not fit. Print the item and any potential separator in expanded mode.
    Expanded,

    /// The item fits but the separator does not in flat mode. If the separator fits in expanded mode then
    /// print the item in flat and the separator in expanded mode, otherwise print both in expanded mode.
    ItemMaybeFlat,
}

/// Printer state that is global to all elements.
/// Stores the result of the print operation (buffer and mappings) and at what
/// position the printer currently is.
#[derive(Default, Debug)]
struct PrinterState<'a> {
    buffer: CodeBuffer,
    pending_indent: Indention,
    pending_space: bool,
    measured_group_fits: bool,
    line_width: usize,
    has_empty_line: bool,
    line_suffixes: LineSuffixes<'a>,
    group_modes: GroupModes,
    // Re-used queue to measure if a group fits. Optimisation to avoid re-allocating a new
    // vec everytime a group gets measured
    fits_stack: Vec<StackFrame>,
    fits_queue: Vec<&'a [FormatElement<'a>]>,
}

/// Tracks the mode in which groups with ids are printed. Stores the groups at `group.id()` index.
/// This is based on the assumption that the group ids for a single document are dense.
#[derive(Debug, Default)]
struct GroupModes(Vec<Option<PrintMode>>);

impl GroupModes {
    fn insert_print_mode(&mut self, group_id: GroupId, mode: PrintMode) {
        let index = u32::from(group_id) as usize;

        if self.0.len() <= index {
            self.0.resize(index + 1, None);
        }

        self.0[index] = Some(mode);
    }

    fn get_print_mode(&self, group_id: GroupId) -> Option<PrintMode> {
        let index = u32::from(group_id) as usize;
        self.0.get(index).and_then(|option| option.as_ref().copied())
    }

    fn unwrap_print_mode(&self, group_id: GroupId, next_element: &FormatElement) -> PrintMode {
        self.get_print_mode(group_id).unwrap_or_else(|| {
            panic!("Expected group with id {group_id:?} to exist but it wasn't present in the document. Ensure that a group with such a document appears in the document before the element {next_element:?}.")
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Indention {
    /// Indent the content by `count` levels by using the indention sequence specified by the printer options.
    Level(u16),

    /// Indent the content by n-`level`s using the indention sequence specified by the printer options and `align` spaces.
    Align { level: u16, align: NonZeroU8, align_count: u16 },
}

impl Indention {
    const fn is_empty(self) -> bool {
        matches!(self, Indention::Level(0))
    }

    /// Creates a new indention level with a zero-indent.
    const fn new() -> Self {
        Indention::Level(0)
    }

    /// Returns the indention level
    fn level(self) -> u16 {
        match self {
            Indention::Level(count) => count,
            Indention::Align { level: indent, .. } => indent,
        }
    }

    /// Returns the number of trailing align spaces or 0 if none
    fn align(self) -> u8 {
        match self {
            Indention::Level(_) => 0,
            Indention::Align { align, .. } => align.into(),
        }
    }

    /// Increments the level by one.
    ///
    /// The behaviour depends on the [`indent_style`][IndentStyle] if this is an [Indent::Align]:
    /// * **Tabs**: `align` is converted into an indent. This results in `level` increasing by two: once for the align, once for the level increment
    /// * **Spaces**: Increments the `level` by one and keeps the `align` unchanged.
    ///
    /// Keeps any  the current value is [Indent::Align] and increments the level by one.
    fn increment_level(self, indent_style: IndentStyle) -> Self {
        match self {
            Indention::Level(count) => Indention::Level(count + 1),
            // Increase the indent AND convert the align to an indent
            Indention::Align { level, align_count, .. } if indent_style.is_tab() => {
                Indention::Level(level + align_count + 1)
            }
            Indention::Align { level: indent, align, align_count } => {
                Indention::Align { level: indent + 1, align, align_count }
            }
        }
    }

    /// Adds an `align` of `count` spaces to the current indention.
    ///
    /// It increments the `level` value if the current value is [Indent::IndentAlign].
    fn set_align(self, count: NonZeroU8) -> Self {
        match self {
            Indention::Level(indent_count) => {
                Indention::Align { level: indent_count, align: count, align_count: 1 }
            }

            // Convert the existing align to an indent
            Indention::Align { level: indent, align, align_count } => Indention::Align {
                level: indent,
                align: align.saturating_add(count.get()),
                align_count: align_count + 1,
            },
        }
    }
}

impl Default for Indention {
    fn default() -> Self {
        Indention::new()
    }
}

#[must_use = "FitsMeasurer must be finished."]
struct FitsMeasurer<'a, 'print> {
    state: FitsState,
    queue: FitsQueue<'a, 'print>,
    stack: FitsCallStack<'print>,
    current_indent: Indention,
    saved_for_dedent: Option<Indention>,
    printer: &'print mut Printer<'a>,
    must_be_flat: bool,
}

impl<'a, 'print> FitsMeasurer<'a, 'print> {
    fn new_flat(
        print_queue: &'print PrintQueue<'a>,
        print_stack: &'print PrintCallStack,
        current_indent: Indention,
        saved_for_dedent: Option<Indention>,
        suffix_indents: &[Indention],
        printer: &'print mut Printer<'a>,
    ) -> Self {
        let mut measurer = Self::new(
            print_queue,
            print_stack,
            current_indent,
            saved_for_dedent,
            suffix_indents,
            printer,
        );
        measurer.must_be_flat = true;
        measurer
    }

    fn new(
        print_queue: &'print PrintQueue<'a>,
        print_stack: &'print PrintCallStack,
        current_indent: Indention,
        saved_for_dedent: Option<Indention>,
        suffix_indents: &[Indention],
        printer: &'print mut Printer<'a>,
    ) -> Self {
        let saved_stack = std::mem::take(&mut printer.state.fits_stack);
        let saved_queue = std::mem::take(&mut printer.state.fits_queue);
        debug_assert!(saved_stack.is_empty());
        debug_assert!(saved_queue.is_empty());

        let fits_queue = FitsQueue::new(print_queue, saved_queue);
        let fits_stack = FitsCallStack::new(print_stack, saved_stack);

        let fits_state = FitsState {
            pending_indent: printer.state.pending_indent,
            pending_space: printer.state.pending_space,
            line_width: printer.state.line_width,
            has_line_suffix: printer.state.line_suffixes.has_pending(),
        };

        Self {
            state: fits_state,
            queue: fits_queue,
            stack: fits_stack,
            current_indent,
            saved_for_dedent,
            must_be_flat: false,
            printer,
        }
    }

    /// Tests if it's possible to print the content of the queue up to the first hard line break
    /// or the end of the document on a single line without exceeding the line width.
    fn fits<P>(&mut self, predicate: &mut P) -> PrintResult<bool>
    where
        P: FitsEndPredicate,
    {
        while let Some(element) = self.queue.pop() {
            match self.fits_element(element)? {
                Fits::Yes => return Ok(true),
                Fits::No => return Ok(false),
                Fits::Maybe => {
                    if predicate.is_end(element)? {
                        break;
                    }
                }
            }
        }

        Ok(true)
    }

    /// Tests if the content of a `Fill` item fits in [PrintMode::Flat].
    ///
    /// Returns `Err` if the top element of the queue is not a [Tag::StartEntry]
    /// or if the document has any mismatching start/end tags.
    fn fill_item_fits(&mut self) -> PrintResult<bool> {
        self.fill_entry_fits(PrintMode::Flat)
    }

    /// Tests if the content of a `Fill` separator fits with `mode`.
    ///
    /// Returns `Err` if the top element of the queue is not a [Tag::StartEntry]
    /// or if the document has any mismatching start/end tags.
    fn fill_separator_fits(&mut self, mode: PrintMode) -> PrintResult<bool> {
        self.fill_entry_fits(mode)
    }

    /// Tests if the elements between the [Tag::StartEntry] and [Tag::EndEntry]
    /// of a fill item or separator fits with `mode`.
    ///
    /// Returns `Err` if the queue isn't positioned at a [Tag::StartEntry] or if
    /// the matching [Tag::EndEntry] is missing.
    fn fill_entry_fits(&mut self, mode: PrintMode) -> PrintResult<bool> {
        let start_entry = self.queue.top();

        if !matches!(start_entry, Some(&FormatElement::Tag(Tag::StartEntry))) {
            return invalid_start_tag(TagKind::Entry, start_entry);
        }

        self.stack.push(TagKind::Fill, self.stack.top().with_print_mode(mode), self.current_indent);
        let mut predicate = SingleEntryPredicate::default();
        let fits = self.fits(&mut predicate)?;

        if predicate.is_done() {
            let (_args, restore) = self.stack.pop(TagKind::Fill)?;
            self.current_indent = restore;
        }

        Ok(fits)
    }

    /// Tests if the passed element fits on the current line or not.
    fn fits_element(&mut self, element: &'a FormatElement) -> PrintResult<Fits> {
        use Tag::{
            EndAlign, EndConditionalContent, EndDedent, EndEntry, EndFill, EndGroup, EndIndent,
            EndIndentIfGroupBreaks, EndLabelled, EndLineSuffix, StartAlign,
            StartConditionalContent, StartDedent, StartEntry, StartFill, StartGroup, StartIndent,
            StartIndentIfGroupBreaks, StartLabelled, StartLineSuffix,
        };

        let args = self.stack.top();

        match element {
            FormatElement::Space => {
                if self.state.line_width > 0 {
                    self.state.pending_space = true;
                }
            }
            FormatElement::HardSpace => {
                self.state.line_width += 1;
                if self.state.line_width > usize::from(self.options().print_width) {
                    return Ok(Fits::No);
                }
            }
            FormatElement::Line(line_mode) => {
                if args.mode().is_flat() {
                    match line_mode {
                        LineMode::SoftOrSpace => {
                            self.state.pending_space = true;
                        }
                        LineMode::Soft => {}
                        LineMode::Hard | LineMode::Empty => {
                            // Even in flat mode, content that _directly_ contains a hard or empty
                            // line is considered to fit when a hard break is reached, since that
                            // break is always going to exist, regardless of the print mode.
                            // This is particularly important for `Fill` entries, where _ungrouped_
                            // content that contains hard breaks shouldn't force the surrounding
                            // elements to also expand. Example:
                            //   [
                            //     -1, -2, 3
                            //     // leading comment
                            //     -4, -5, -6
                            //   ]
                            // Here, `-4` contains a hardline because of the leading comment, but that
                            // doesn't cause the element (`-4`) nor the separator (`,`) to print in
                            // expanded mode, allowing the rest of the elements to fill in. If this
                            // _did_ respect `must_be_flat` and returned `Fits::No` instead, the result
                            // would put the `-4` on its own line, which is not preferable (at least,
                            // it doesn't match Prettier):
                            //   [
                            //     -1, -2, 3
                            //     // leading comment
                            //     -4,
                            //     -5, -6
                            //   ]
                            // The perception here is that most comments inline for fills are used to
                            // separate _groups_ rather than to single out an individual element.
                            //
                            // The alternative case is when the fill entry is grouped, in which case
                            // this fit returns `Yes`, but the parent group is already known to
                            // expand _because_ of this hard line, and so the fill entry and separator
                            // are automatically printed in expanded mode anyway, and this fit check
                            // has no bearing on that (so it doesn't need to care about flat or not):
                            //   <div>
                            //     <span a b>
                            //       <Foo />
                            //     </span>{" "}
                            //     ({variable})
                            //   </div>
                            // Here the `<span>...</span>` _is_ grouped and contains a hardline, so it
                            // is known to break and _not_ fit already because the check is performed
                            // on the group. But within the group itself, the content with hardlines
                            // (the `\n<Foo />\n`) _does_ fit, for the same logic in the first case.
                            return Ok(Fits::Yes);
                        }
                    }
                } else {
                    // Reachable if the restQueue contains an element with mode expanded because Expanded
                    // is what the mode's initialized to by default
                    // This means, the printer is outside of the current element at this point and any
                    // line break should be printed as regular line break -> Fits
                    return Ok(Fits::Yes);
                }
            }

            FormatElement::Token { text } => {
                return Ok(self.fits_text(Text::Token(text)));
            }
            FormatElement::Text { text, width } => {
                return Ok(self.fits_text(Text::Text { text, width: *width }));
            }

            FormatElement::LineSuffixBoundary => {
                if self.state.has_line_suffix {
                    return Ok(Fits::No);
                }
            }

            FormatElement::ExpandParent => {
                if self.must_be_flat {
                    return Ok(Fits::No);
                }
            }

            FormatElement::BestFitting(best_fitting) => {
                let slice = match args.mode() {
                    PrintMode::Flat => best_fitting.most_flat(),
                    PrintMode::Expanded => best_fitting.most_expanded(),
                };

                if !matches!(slice.first(), Some(FormatElement::Tag(Tag::StartEntry))) {
                    return invalid_start_tag(TagKind::Entry, slice.first());
                }

                self.queue.extend_back(slice);
            }

            FormatElement::Interned(content) => self.queue.extend_back(content),

            FormatElement::Tag(StartIndent) => {
                let restore = self.current_indent;
                self.current_indent =
                    self.current_indent.increment_level(self.options().indent_style());
                self.stack.push(TagKind::Indent, args, restore);
            }

            FormatElement::Tag(StartDedent(mode)) => match mode {
                DedentMode::Level => {
                    self.saved_for_dedent = Some(self.current_indent);
                    self.stack.push(TagKind::Dedent, args, self.current_indent);
                }
                DedentMode::Root => {
                    let restore = self.current_indent;
                    self.current_indent = Indention::default();
                    self.stack.push(TagKind::Dedent, args, restore);
                }
            },

            FormatElement::Tag(StartAlign(align)) => {
                let restore = self.current_indent;
                self.current_indent = self.current_indent.set_align(align.count());
                self.stack.push(TagKind::Align, args, restore);
            }

            FormatElement::Tag(StartGroup(group)) => {
                if self.must_be_flat && !group.mode().is_flat() {
                    return Ok(Fits::No);
                }

                let group_mode =
                    if group.mode().is_flat() { args.mode() } else { PrintMode::Expanded };

                self.stack.push(
                    TagKind::Group,
                    args.with_print_mode(group_mode),
                    self.current_indent,
                );

                if let Some(id) = group.id() {
                    self.group_modes_mut().insert_print_mode(id, group_mode);
                }
            }

            FormatElement::Tag(StartConditionalContent(condition)) => {
                let group_mode = match condition.group_id {
                    None => args.mode(),
                    Some(group_id) => {
                        self.group_modes().get_print_mode(group_id).unwrap_or_else(|| args.mode())
                    }
                };

                if group_mode == condition.mode {
                    self.stack.push(TagKind::ConditionalContent, args, self.current_indent);
                } else {
                    self.queue.skip_content(TagKind::ConditionalContent);
                }
            }

            FormatElement::Tag(StartIndentIfGroupBreaks(id)) => {
                let group_mode =
                    self.group_modes().get_print_mode(*id).unwrap_or_else(|| args.mode());

                let restore = self.current_indent;
                match group_mode {
                    PrintMode::Flat => {
                        self.stack.push(TagKind::IndentIfGroupBreaks, args, restore);
                    }
                    PrintMode::Expanded => {
                        self.current_indent =
                            self.current_indent.increment_level(self.options().indent_style());
                        self.stack.push(TagKind::IndentIfGroupBreaks, args, restore);
                    }
                }
            }

            FormatElement::Tag(StartLineSuffix) => {
                self.queue.skip_content(TagKind::LineSuffix);
                self.state.has_line_suffix = true;
            }

            FormatElement::Tag(EndLineSuffix) => {
                return invalid_end_tag(TagKind::LineSuffix, self.stack.top_kind());
            }

            FormatElement::Tag(tag @ (StartFill | StartLabelled(_) | StartEntry)) => {
                self.stack.push(tag.kind(), args, self.current_indent);
            }
            FormatElement::Tag(
                tag @ (EndLabelled | EndEntry | EndGroup | EndConditionalContent | EndFill | EndIndent | EndAlign),
            ) => {
                let (_args, restore) = self.stack.pop(tag.kind())?;
                self.current_indent = restore;
            }
            FormatElement::Tag(tag @ EndIndentIfGroupBreaks(group_id)) => {
                let group_mode =
                    self.group_modes().get_print_mode(*group_id).unwrap_or_else(|| args.mode());
                let (_args, restore) = self.stack.pop(tag.kind())?;
                if group_mode == PrintMode::Expanded {
                    self.current_indent = restore;
                }
            }
            FormatElement::Tag(tag @ EndDedent(mode)) => {
                let (_args, restore) = self.stack.pop(tag.kind())?;
                match mode {
                    DedentMode::Level => {
                        self.current_indent = self.saved_for_dedent.unwrap_or_default();
                    }
                    DedentMode::Root => {
                        self.current_indent = restore;
                    }
                }
            }
        }

        Ok(Fits::Maybe)
    }

    fn fits_text(&mut self, text: Text) -> Fits {
        let indent = std::mem::take(&mut self.state.pending_indent);
        self.state.line_width += indent.level() as usize
            * self.options().indent_width().value() as usize
            + indent.align() as usize;

        if self.state.pending_space {
            self.state.line_width += 1;
        }

        match text {
            Text::Token(text) => {
                self.state.line_width += text.len();
            }
            Text::Text { text, width } => {
                if width.is_multiline() {
                    return if self.must_be_flat
                        || self.state.line_width + width.value() as usize
                            > usize::from(self.options().print_width)
                    {
                        Fits::No
                    } else {
                        Fits::Yes
                    };
                }

                self.state.line_width += width.value() as usize;
            }
        }

        if self.state.line_width > usize::from(self.options().print_width) {
            return Fits::No;
        }

        self.state.pending_space = false;

        Fits::Maybe
    }

    fn finish(self) {
        let mut queue = self.queue.finish();
        queue.clear();
        self.printer.state.fits_queue = queue;

        let mut stack = self.stack.finish();
        stack.clear();
        self.printer.state.fits_stack = stack;
    }

    fn options(&self) -> &PrinterOptions {
        &self.printer.options
    }

    fn group_modes(&self) -> &GroupModes {
        &self.printer.state.group_modes
    }

    fn group_modes_mut(&mut self) -> &mut GroupModes {
        &mut self.printer.state.group_modes
    }
}

#[cold]
fn invalid_end_tag<R>(end_tag: TagKind, start_tag: Option<TagKind>) -> PrintResult<R> {
    Err(PrintError::InvalidDocument(match start_tag {
        None => InvalidDocumentError::StartTagMissing { kind: end_tag },
        Some(kind) => {
            InvalidDocumentError::StartEndTagMismatch { start_kind: end_tag, end_kind: kind }
        }
    }))
}

#[cold]
fn invalid_start_tag<R>(expected: TagKind, actual: Option<&FormatElement>) -> PrintResult<R> {
    let start = match actual {
        None => ActualStart::EndOfDocument,
        Some(FormatElement::Tag(tag)) => {
            if tag.is_start() {
                ActualStart::Start(tag.kind())
            } else {
                ActualStart::End(tag.kind())
            }
        }
        Some(_) => ActualStart::Content,
    };

    Err(PrintError::InvalidDocument(InvalidDocumentError::ExpectedStart {
        actual: start,
        expected_start: expected,
    }))
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Fits {
    // Element fits
    Yes,
    // Element doesn't fit
    No,
    // Element may fit, depends on the elements following it
    Maybe,
}

impl From<bool> for Fits {
    fn from(value: bool) -> Self {
        if value { Fits::Yes } else { Fits::No }
    }
}

/// State used when measuring if a group fits on a single line
#[derive(Debug)]
struct FitsState {
    pending_indent: Indention,
    pending_space: bool,
    has_line_suffix: bool,
    line_width: usize,
}

#[derive(Copy, Clone, Debug)]
enum Text<'a> {
    /// ASCII only text that contains no line breaks or tab characters.
    Token(&'a str),
    /// Arbitrary text. May contain `\n` line breaks, tab characters, or unicode characters.
    Text { text: &'a str, width: TextWidth },
}

impl Text<'_> {
    fn len(&self) -> usize {
        match self {
            Text::Token(text) | Text::Text { text, .. } => text.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;

    use crate::formatter::prelude::document::Document;
    use crate::formatter::printer::{PrintWidth, Printer, PrinterOptions};
    use crate::formatter::{FormatContext, FormatState, Printed, VecBuffer};
    use crate::{IndentStyle, LineEnding};
    use crate::{format_args, formatter::prelude::*, write};

    fn format<'a>(allocator: &'a Allocator, root: &dyn Format<'a>) -> Printed {
        format_with_options(
            allocator,
            root,
            PrinterOptions {
                indent_style: IndentStyle::Space,
                indent_width: 2.try_into().unwrap(),
                line_ending: LineEnding::Lf,
                ..PrinterOptions::default()
            },
        )
    }

    fn format_with_options<'a>(
        allocator: &'a Allocator,
        root: &dyn Format<'a>,
        options: PrinterOptions,
    ) -> Printed {
        let formatted = crate::format!(FormatContext::dummy(allocator), [root]).unwrap();

        Printer::new(options).print(formatted.document()).expect("Document to be valid")
    }

    #[test]
    fn it_prints_a_group_on_a_single_line_if_it_fits() {
        let allocator = Allocator::default();
        let result = format(
            &allocator,
            &FormatArrayElements {
                items: vec![&token("\"a\""), &token("\"b\""), &token("\"c\""), &token("\"d\"")],
            },
        );

        assert_eq!(r#"["a", "b", "c", "d"]"#, result.as_code());
    }

    #[test]
    fn it_tracks_the_indent_for_each_token() {
        let allocator = Allocator::default();
        let formatted = format(
            &allocator,
            &format_args!(
                token("a"),
                soft_block_indent(&format_args!(
                    token("b"),
                    soft_block_indent(&format_args!(
                        token("c"),
                        soft_block_indent(
                            &format_args!(token("d"), soft_line_break(), token("d"),)
                        ),
                        token("c"),
                    )),
                    token("b"),
                )),
                token("a")
            ),
        );

        assert_eq!(
            r"a
  b
    c
      d
      d
    c
  b
a",
            formatted.as_code()
        );
    }

    #[test]
    fn it_converts_line_endings() {
        let allocator = Allocator::default();
        let options = PrinterOptions {
            indent_style: IndentStyle::Tab,
            line_ending: LineEnding::Crlf,
            ..PrinterOptions::default()
        };

        let result = format_with_options(
            &allocator,
            &format_args!(
                token("function main() {"),
                block_indent(&text("let x = `This is a multiline\nstring`;")),
                token("}"),
                hard_line_break()
            ),
            options,
        );

        assert_eq!(
            "function main() {\r\n\tlet x = `This is a multiline\r\nstring`;\r\n}\r\n",
            result.as_code()
        );
    }

    #[test]
    fn it_converts_line_endings_to_cr() {
        let allocator = Allocator::default();
        let options = PrinterOptions {
            indent_style: IndentStyle::Tab,
            line_ending: LineEnding::Cr,
            ..PrinterOptions::default()
        };

        let result = format_with_options(
            &allocator,
            &format_args!(
                token("function main() {"),
                block_indent(&text("let x = `This is a multiline\nstring`;")),
                token("}"),
                hard_line_break()
            ),
            options,
        );

        assert_eq!(
            "function main() {\r\tlet x = `This is a multiline\rstring`;\r}\r",
            result.as_code()
        );
    }

    #[test]
    fn it_breaks_a_group_if_a_string_contains_a_newline() {
        let allocator = Allocator::default();
        let result = format(
            &allocator,
            &FormatArrayElements {
                items: vec![&text("`This is a string spanning\ntwo lines`"), &token("\"b\"")],
            },
        );

        assert_eq!(
            r#"[
  `This is a string spanning
two lines`,
  "b",
]"#,
            result.as_code()
        );
    }
    #[test]
    fn it_breaks_a_group_if_it_contains_a_hard_line_break() {
        let allocator = Allocator::default();
        let result =
            format(&allocator, &group(&format_args!(token("a"), block_indent(&token("b")))));

        assert_eq!("a\n  b\n", result.as_code());
    }

    #[test]
    fn it_breaks_parent_groups_if_they_dont_fit_on_a_single_line() {
        let allocator = Allocator::default();
        let result = format(
            &allocator,
            &FormatArrayElements {
                items: vec![
                    &token("\"a\""),
                    &token("\"b\""),
                    &token("\"c\""),
                    &token("\"d\""),
                    &FormatArrayElements {
                        items: vec![
                            &token("\"0123456789\""),
                            &token("\"0123456789\""),
                            &token("\"0123456789\""),
                            &token("\"0123456789\""),
                            &token("\"0123456789\""),
                        ],
                    },
                ],
            },
        );

        assert_eq!(
            r#"[
  "a",
  "b",
  "c",
  "d",
  ["0123456789", "0123456789", "0123456789", "0123456789", "0123456789"],
]"#,
            result.as_code()
        );
    }

    #[test]
    fn it_use_the_indent_character_specified_in_the_options() {
        let allocator = Allocator::default();
        let options = PrinterOptions {
            indent_style: IndentStyle::Tab,
            indent_width: 2.try_into().unwrap(),
            print_width: PrintWidth::new(19),
            ..PrinterOptions::default()
        };

        let result = format_with_options(
            &allocator,
            &FormatArrayElements {
                items: vec![&token("'a'"), &token("'b'"), &token("'c'"), &token("'d'")],
            },
            options,
        );

        assert_eq!("[\n\t'a',\n\t\'b',\n\t\'c',\n\t'd',\n]", result.as_code());
    }

    #[test]
    fn it_prints_consecutive_hard_lines_as_one() {
        let allocator = Allocator::default();
        let result = format(
            &allocator,
            &format_args!(
                token("a"),
                hard_line_break(),
                hard_line_break(),
                hard_line_break(),
                token("b"),
            ),
        );

        assert_eq!("a\nb", result.as_code());
    }

    #[test]
    fn it_prints_consecutive_empty_lines_as_one() {
        let allocator = Allocator::default();
        let result = format(
            &allocator,
            &format_args!(token("a"), empty_line(), empty_line(), empty_line(), token("b"),),
        );

        assert_eq!("a\n\nb", result.as_code());
    }

    #[test]
    fn it_prints_consecutive_mixed_lines_as_one() {
        let allocator = Allocator::default();
        let result = format(
            &allocator,
            &format_args!(
                token("a"),
                empty_line(),
                hard_line_break(),
                empty_line(),
                hard_line_break(),
                token("b"),
            ),
        );

        assert_eq!("a\n\nb", result.as_code());
    }

    #[test]
    fn test_fill_breaks() {
        let allocator = Allocator::default();
        let mut state = FormatState::new(FormatContext::dummy(&allocator));
        let mut buffer = VecBuffer::new(&mut state);
        let mut formatter = Formatter::new(&mut buffer);

        formatter
            .fill()
            // These all fit on the same line together
            .entry(&soft_line_break_or_space(), &format_args!(token("1"), token(",")))
            .entry(&soft_line_break_or_space(), &format_args!(token("2"), token(",")))
            .entry(&soft_line_break_or_space(), &format_args!(token("3"), token(",")))
            // This one fits on a line by itself,
            .entry(&soft_line_break_or_space(), &format_args!(token("723493294"), token(",")))
            // fits without breaking
            .entry(
                &soft_line_break_or_space(),
                &group(&format_args!(token("["), soft_block_indent(&token("5")), token("],"))),
            )
            // this one must be printed in expanded mode to fit
            .entry(
                &soft_line_break_or_space(),
                &group(&format_args!(
                    token("["),
                    soft_block_indent(&token("123456789")),
                    token("]"),
                )),
            )
            .finish()
            .unwrap();

        let document = Document::from(buffer.into_vec());

        let printed = Printer::new(
            PrinterOptions::default()
                .with_indent_style(IndentStyle::Tab)
                .with_print_width(PrintWidth::new(10)),
        )
        .print(&document)
        .unwrap();

        assert_eq!(printed.as_code(), "1, 2, 3,\n723493294,\n[5],\n[\n\t123456789\n]");
    }

    #[test]
    fn line_suffix_printed_at_end() {
        let allocator = Allocator::default();
        let printed = format(
            &allocator,
            &format_args!(
                group(&format_args!(
                    token("["),
                    soft_block_indent(&format_with(|f| {
                        f.fill()
                            .entry(
                                &soft_line_break_or_space(),
                                &format_args!(token("1"), token(",")),
                            )
                            .entry(
                                &soft_line_break_or_space(),
                                &format_args!(token("2"), token(",")),
                            )
                            .entry(
                                &soft_line_break_or_space(),
                                &format_args!(token("3"), if_group_breaks(&token(","))),
                            )
                            .finish()
                    })),
                    token("]")
                )),
                token(";"),
                &line_suffix(&format_args!(space(), token("// trailing"), space()))
            ),
        );

        assert_eq!(printed.as_code(), "[1, 2, 3]; // trailing");
    }
    #[test]
    fn conditional_with_group_id_in_fits() {
        let allocator = Allocator::default();
        let content = format_with(|f| {
            let group_id = f.group_id("test");
            write!(
                f,
                [
                    group(&format_args!(
                        token("The referenced group breaks."),
                        hard_line_break()
                    ))
                    .with_group_id(Some(group_id)),
                    group(&format_args!(
                        token("This group breaks because:"),
                        soft_line_break_or_space(),
                        if_group_fits_on_line(&token("This content fits but should not be printed.")).with_group_id(Some(group_id)),
                        if_group_breaks(&token("It measures with the 'if_group_breaks' variant because the referenced group breaks and that's just way too much text.")).with_group_id(Some(group_id)),
                    ))
                ]
            )
        });

        let printed = format(&allocator, &content);

        assert_eq!(
            printed.as_code(),
            "The referenced group breaks.\nThis group breaks because:\nIt measures with the 'if_group_breaks' variant because the referenced group breaks and that's just way too much text."
        );
    }

    #[test]
    fn out_of_order_group_ids() {
        let allocator = Allocator::default();
        let content = format_with(|f| {
            let id_1 = f.group_id("id-1");
            let id_2 = f.group_id("id-2");

            write!(
                f,
                [group(&token("Group with id-2")).with_group_id(Some(id_2)), hard_line_break()]
            )?;

            write!(
                f,
                [
                    group(&token("Group with id-1 does not fit on the line because it exceeds the line width of 80 characters by")).with_group_id(Some(id_1)),
                    hard_line_break()
                ]
            )?;

            write!(
                f,
                [
                    if_group_fits_on_line(&token("Group 2 fits")).with_group_id(Some(id_2)),
                    hard_line_break(),
                    if_group_breaks(&token("Group 1 breaks")).with_group_id(Some(id_1))
                ]
            )
        });

        let printed = format(&allocator, &content);

        assert_eq!(
            printed.as_code(),
            r"Group with id-2
Group with id-1 does not fit on the line because it exceeds the line width of 80 characters by
Group 2 fits
Group 1 breaks"
        );
    }

    #[test]
    fn break_group_if_partial_string_exceeds_print_width() {
        let allocator = Allocator::default();
        let options =
            PrinterOptions { print_width: PrintWidth::new(10), ..PrinterOptions::default() };

        let result = format_with_options(
            &allocator,
            &format_args!(group(&format_args!(
                token("("),
                soft_line_break(),
                text("This is a string\n containing a newline"),
                soft_line_break(),
                token(")")
            ))),
            options,
        );

        assert_eq!("(\nThis is a string\n containing a newline\n)", result.as_code());
    }

    struct FormatArrayElements<'a> {
        items: Vec<&'a dyn Format<'a>>,
    }

    impl<'a> Format<'a> for FormatArrayElements<'a> {
        fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
            write!(
                f,
                [group(&format_args!(
                    token("["),
                    soft_block_indent(&format_args!(
                        format_with(|f| f
                            .join_with(format_args!(token(","), soft_line_break_or_space()))
                            .entries(&self.items)
                            .finish()),
                        if_group_breaks(&token(",")),
                    )),
                    token("]")
                ))]
            )
        }
    }
}
