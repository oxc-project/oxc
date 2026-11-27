//! Parsing top-level statements again once an unambiguous source's goal is known.
//!
//! In unambiguous mode, `await` is an identifier until module syntax shows that the source is a
//! module. After that, `await /x/u;` is not 2 divisions, but an `await` of a regular expression.
//!
//! From the first top-level statement with `await` as an identifier on, the first parse saves a
//! checkpoint before each statement (see `parse_directives_and_statements`). It stops after the
//! statement with module syntax, if it finds any. `finish_top_level_statements` then takes one of
//! 3 paths.
//!
//! ## Module syntax found
//!
//! As in TypeScript's `reparseTopLevelAwait`, the statements before it in which `await` was an
//! identifier are parsed again, as module code. Each reparse continues until it ends where a
//! statement of the first parse started, with the same trivia before it, and that statement did
//! not have `await` as an identifier. The first parse's statements from there on are kept,
//! together with their tokens, trivia, errors and module records. Then parsing continues as a
//! module.
//!
//! ## No module syntax, but errors
//!
//! The first parse can also fail without finding module syntax, as in `await /x/; export {}`,
//! where `await / x /` is missing an operand. If it has errors from the first checkpoint on, the
//! statements from there on are parsed again as a module. Like Babel, the source is a module if
//! that finds module syntax or has no errors, counting errors reported later, such as for
//! HTML-like comments anywhere in the source. Otherwise, the first parse is put back as it was.
//!
//! ## No module syntax or errors
//!
//! The source is a script, and the first parse is kept.
//!
//! ## What this relies on
//!
//! * Every top-level statement from the first checkpoint on has a checkpoint, including one with a
//!   fatal error. Directives cannot follow the first one. So `checkpoints[i]` is before
//!   `statements[first_statement + i]`.
//! * Output is appended in parse order, so the output from a checkpoint on is what follows the
//!   lengths saved in it. The exception is the lexer error which `set_unexpected` can take as the
//!   fatal error. See `take_first_parse`.
//! * Without module syntax, the first parse stops only at the end of the source: at `Eof`, at a
//!   fatal error, which moves the lexer to the end, or at the `Undetermined` token of a lexer
//!   error, which ends the statements like `Eof`, with no fatal error. So once the source is a
//!   script, nothing is left to parse.
//! * `ParserCheckpoint` does not cover deferred errors, module records, or the parser state keyed
//!   by source position. They are handled here, as they are where `arrow.rs` parses the arguments
//!   of `async(…)` again.
//! * Some errors are reported only at the end of parsing: the deferred errors of whichever of a
//!   module or a script the source turns out to be, and the remaining `cover_initialized_name`s.

use std::{mem, ops::Range, vec};

use oxc_allocator::{ArenaVec, GetAllocator};
use oxc_ast::ast::{Comment, Statement};
use oxc_span::{GetSpan, Span};
use oxc_syntax::module_record::DynamicImport;

use crate::{
    ParserConfig as Config, ParserImpl, StatementContext, config::LexerConfig,
    cursor::ParserCheckpoint, diagnostics, diagnostics::ParserDiagnostic,
    error_handler::FatalError, lexer::Token, module_record::ModuleRecordBuilder,
    state::CoverInitializedName,
};

/// State before a top-level statement of the first parse.
///
/// The lengths outside `parser` are of output which `ParserImpl::rewind` does not truncate.
pub struct StatementCheckpoint<'a> {
    parser: ParserCheckpoint<'a>,
    deferred_module_errors_len: usize,
    deferred_script_errors_len: usize,
    dynamic_imports_len: usize,
    import_metas_len: usize,
    /// Whether the statement parsed `await` as an identifier.
    pub has_await_identifier: bool,
}

impl StatementCheckpoint<'_> {
    fn start(&self) -> u32 {
        self.parser.token().start()
    }
}

/// Output of the first parse from the first checkpoint on, moved out while statements are
/// parsed again.
struct Original<T> {
    items: vec::IntoIter<T>,
    /// Index of the first of `items` in the output of the first parse.
    index: usize,
}

impl<T> Original<T> {
    fn new(items: Vec<T>, index: usize) -> Self {
        Self { items: items.into_iter(), index }
    }

    /// The items at `range` in the output of the first parse.
    /// Items between the previous range and `range` are dropped.
    fn take(&mut self, range: Range<usize>) -> impl Iterator<Item = T> {
        debug_assert!(self.index <= range.start);
        debug_assert!(range.start <= range.end);
        if range.start > self.index {
            self.items.nth(range.start - self.index - 1);
        }
        self.index = range.end;
        self.items.by_ref().take(range.len())
    }
}

struct OriginalOutput<'a> {
    statements: Original<Statement<'a>>,
    tokens: Original<Token>,
    comments: Original<Comment>,
    irregular_whitespaces: Original<Span>,
    errors: Original<ParserDiagnostic<'a>>,
    lexer_errors: Original<ParserDiagnostic<'a>>,
    deferred_module_errors: Original<ParserDiagnostic<'a>>,
    dynamic_imports: Original<DynamicImport>,
    import_metas: Original<Span>,
}

/// The first parse from the first checkpoint on, moved out of the parser while
/// `retry_as_module` parses the same statements as a module.
///
/// With the checkpoints, it holds what that parse discards or overwrites, so the first parse can
/// be put back as it was.
struct FirstParse<'a> {
    /// The checkpoints of the statements, then of where the first parse stopped.
    checkpoints: Vec<StatementCheckpoint<'a>>,
    /// Index of the first checkpoint's statement.
    first_statement: usize,
    output: OriginalOutput<'a>,
    /// Module records other than dynamic imports and `import.meta`s.
    declarations: ModuleRecordBuilder<'a>,
    fatal_error: Option<FatalError<'a>>,
    /// Whether a copy of `fatal_error`'s lexer error is in `lexer.errors` while `fatal_error` is
    /// out of the parser.
    lexer_error_copied: bool,
    reported_ambient_statement: bool,
    deferred_script_errors: Vec<ParserDiagnostic<'a>>,
    cover_initialized_names: Vec<(u32, CoverInitializedName<'a>)>,
    trailing_commas: Vec<(u32, Span)>,
}

/// Index of the current token of `checkpoint` in the tokens, if tokens are enabled.
/// Lexing it pushed it, so it is the last token before the checkpoint.
fn token_index(checkpoint: &StatementCheckpoint<'_>) -> usize {
    checkpoint.parser.lexer().tokens_len().saturating_sub(1)
}

impl<'a, C: Config> ParserImpl<'a, C> {
    #[inline]
    pub(crate) fn statement_checkpoint(&self) -> StatementCheckpoint<'a> {
        StatementCheckpoint {
            parser: self.checkpoint(),
            deferred_module_errors_len: self.lexer.deferred_module_errors.len(),
            deferred_script_errors_len: self.deferred_script_errors.len(),
            dynamic_imports_len: self.module_record_builder.dynamic_imports_len(),
            import_metas_len: self.module_record_builder.import_metas_len(),
            has_await_identifier: false,
        }
    }

    /// Parse a top-level statement, after the first parse.
    fn parse_top_level_statement(
        &mut self,
        reported_ambient_statement: &mut bool,
    ) -> Option<Statement<'a>> {
        if self.has_fatal_error() {
            return None;
        }
        let statement = self.parse_statement_list_item(StatementContext::Program);
        if self.ctx.has_ambient() {
            self.check_ambient_statement(&statement, reported_ambient_statement);
        }
        Some(statement)
    }

    /// Parse the remaining top-level statements, after the first parse.
    fn parse_top_level_statements(
        &mut self,
        statements: &mut ArenaVec<'a, Statement<'a>>,
        reported_ambient_statement: &mut bool,
    ) {
        while let Some(statement) = self.parse_top_level_statement(reported_ambient_statement) {
            statements.push(statement);
        }
    }

    /// Finish the top-level statements, after the first parse stopped with
    /// `self.state.statement_checkpoints` the checkpoints of its last statements.
    ///
    /// See the module docs.
    #[cold]
    #[inline(never)]
    pub(crate) fn finish_top_level_statements(
        &mut self,
        statements: &mut ArenaVec<'a, Statement<'a>>,
        reported_ambient_statement: &mut bool,
    ) {
        if self.module_record_builder.has_module_syntax() {
            self.set_await_context(true);
            if self.fatal_error.is_some() {
                // The fatal error is in the statement with the module syntax, so the first parse
                // has no statement to resume at. Parse everything from the first checkpoint on
                // again.
                let checkpoints = mem::take(&mut self.state.statement_checkpoints);
                let first_statement = statements.len() - checkpoints.len();
                self.rewind_to_statement(&checkpoints[0], statements, first_statement);
            } else {
                self.reparse_top_level_awaits(statements, reported_ambient_statement);
            }
            // Parsed again, the module syntax may not be, as in `await /x; export {}; y/u`.
            // Like Babel, the source is still a module.
            self.module_record_builder.set_module_syntax();
            self.parse_top_level_statements(statements, reported_ambient_statement);
        } else if self.has_errors_since(&self.state.statement_checkpoints[0]) {
            self.retry_as_module(statements, reported_ambient_statement);
        } else {
            // A script, so the first parse is kept. Statements with checkpoints skipped the
            // ambient context check when parsed, because they might have been parsed again.
            let mut checkpoints = mem::take(&mut self.state.statement_checkpoints);
            if self.ctx.has_ambient() {
                checkpoints.push(self.statement_checkpoint());
                let first_statement = statements.len() + 1 - checkpoints.len();
                self.check_original_ambient_statements(
                    &statements[first_statement..],
                    &checkpoints,
                    checkpoints[0].parser.errors_len(),
                    reported_ambient_statement,
                );
            }
        }
    }

    /// Parse the statements from the first checkpoint on again as a module, after the first parse
    /// had errors in them and found no module syntax. Put the first parse back if that fails.
    fn retry_as_module(
        &mut self,
        statements: &mut ArenaVec<'a, Statement<'a>>,
        reported_ambient_statement: &mut bool,
    ) {
        let first_parse = self.take_first_parse(statements, *reported_ambient_statement);
        let first = &first_parse.checkpoints[0];
        self.rewind(first.parser);
        // `take_first_parse` moved out the first checkpoint's token with the tokens after it
        if self.lexer.config.tokens() {
            self.lexer.tokens_mut().push(first.parser.token());
        }
        self.set_await_context(true);
        self.parse_top_level_statements(statements, reported_ambient_statement);

        // HTML-like comments are errors in a module wherever they are, even before `first`
        if self.module_record_builder.has_module_syntax()
            || (!self.has_errors_since(first) && self.lexer.deferred_module_errors.is_empty())
        {
            self.module_record_builder.set_module_syntax();
            return;
        }
        self.put_back_first_parse(first_parse, statements, reported_ambient_statement);
    }

    /// Move the first parse from the first checkpoint on out of the parser.
    ///
    /// Leaves the parser where the first parse stopped, without its fatal error.
    fn take_first_parse(
        &mut self,
        statements: &mut ArenaVec<'a, Statement<'a>>,
        reported_ambient_statement: bool,
    ) -> FirstParse<'a> {
        let mut checkpoints = mem::take(&mut self.state.statement_checkpoints);
        let first_statement = statements.len() - checkpoints.len();

        // At `Eof`, `set_unexpected` takes the last lexer error as the fatal error, even one from
        // before the first checkpoint, as in `'\u{zz}'; await(1); (`. Then there are fewer lexer
        // errors than the checkpoint counts. `rewind` gives such an error back when it discards
        // the fatal error, but this only moves the fatal error out, so it pushes a copy instead.
        let fatal_error = self.fatal_error.take();
        let lexer_error_copied = if let Some(fatal_error) = &fatal_error
            && self.lexer.errors.len() < checkpoints[0].parser.lexer().errors_len()
        {
            self.lexer.errors.push(fatal_error.error.clone());
            true
        } else {
            false
        };

        // Where the first parse stopped. Taken without the fatal error, as `resume` requires, and
        // with the copy, so its lexer errors start with the first checkpoint's.
        checkpoints.push(self.statement_checkpoint());

        let first = &checkpoints[0];
        let start = first.start();
        let (output, declarations) = self.take_original_output(statements, first_statement, first);
        let deferred_script_errors =
            self.deferred_script_errors.split_off(first.deferred_script_errors_len);
        // Keyed by source position. Left in, the module parse could find them at the same
        // positions.
        let cover_initialized_names = self
            .state
            .cover_initialized_name
            .extract_if(|&position, _| position >= start)
            .collect();
        let trailing_commas =
            self.state.trailing_commas.extract_if(|&position, _| position >= start).collect();

        FirstParse {
            checkpoints,
            first_statement,
            output,
            declarations,
            fatal_error,
            lexer_error_copied,
            reported_ambient_statement,
            deferred_script_errors,
            cover_initialized_names,
            trailing_commas,
        }
    }

    /// Discard the module parse, and put back the first parse.
    ///
    /// Leaves the parser where the first parse stopped, as if it had not been taken.
    fn put_back_first_parse(
        &mut self,
        first_parse: FirstParse<'a>,
        statements: &mut ArenaVec<'a, Statement<'a>>,
        reported_ambient_statement: &mut bool,
    ) {
        let FirstParse {
            checkpoints,
            first_statement,
            mut output,
            declarations,
            fatal_error,
            lexer_error_copied,
            reported_ambient_statement: reported,
            deferred_script_errors,
            cover_initialized_names,
            trailing_commas,
        } = first_parse;
        let (first, end) = (&checkpoints[0], &checkpoints[checkpoints.len() - 1]);

        self.rewind_to_statement(first, statements, first_statement);
        self.set_await_context(false);
        if lexer_error_copied {
            // `rewind` kept it, giving it back if the module parse's fatal error took it
            debug_assert_eq!(self.lexer.errors.len(), first.parser.lexer().errors_len());
            self.lexer.errors.pop();
        }
        // The first parse's errors are put back at the same indexes, so `errors_len` of its fatal
        // error stays right. It is set before `keep_original`, whose ambient statement check skips
        // the statement with the fatal error and keeps `errors_len` in step.
        self.fatal_error = fatal_error;
        *reported_ambient_statement = reported;
        // `keep_original` puts back the first parse's tokens, from the first checkpoint's on
        if self.lexer.config.tokens() {
            self.lexer.tokens_mut().pop();
        }
        self.keep_original(
            &mut output,
            statements,
            &checkpoints,
            first_statement,
            reported_ambient_statement,
        );
        self.deferred_script_errors.extend(deferred_script_errors);
        self.state.cover_initialized_name.extend(cover_initialized_names);
        self.state.trailing_commas.extend(trailing_commas);
        self.module_record_builder.append(declarations);
        // Its token is `Eof` or `Undetermined`, which `finalize_tokens` expects to be the last
        // token, unless there is a fatal error, which discards the tokens.
        self.resume_at(end);
    }

    /// Whether parsing since `checkpoint` had errors other than deferred errors, which depend on
    /// whether the source is a module.
    ///
    /// That includes a fatal error, which may have taken a lexer error from before `checkpoint`
    /// (see `take_first_parse`), and the `cover_initialized_name`s which
    /// `check_unfinished_errors` reports at the end of parsing.
    fn has_errors_since(&self, checkpoint: &StatementCheckpoint<'a>) -> bool {
        let start = checkpoint.start();
        self.fatal_error.is_some()
            || self.errors.len() > checkpoint.parser.errors_len()
            || self.lexer.errors.len() > checkpoint.parser.lexer().errors_len()
            || self.state.cover_initialized_name.keys().any(|&position| position >= start)
    }

    /// Parse with `await` as a keyword or not from now on.
    fn set_await_context(&mut self, has_await: bool) {
        self.ctx = self.ctx.and_await(has_await);
        // Discard what lookahead found in the other context
        self.state.not_parenthesized_arrow.clear();
        self.state.type_argument_lookahead = None;
    }

    /// Rewind to the statement of `checkpoint`, which is `statements[index]`, discarding all
    /// output from it on, including the output which `rewind` does not truncate.
    fn rewind_to_statement(
        &mut self,
        checkpoint: &StatementCheckpoint<'a>,
        statements: &mut ArenaVec<'a, Statement<'a>>,
        index: usize,
    ) {
        statements.truncate(index);
        self.rewind(checkpoint.parser);
        self.lexer.deferred_module_errors.truncate(checkpoint.deferred_module_errors_len);
        self.deferred_script_errors.truncate(checkpoint.deferred_script_errors_len);
        let start = checkpoint.start();
        self.state.cover_initialized_name.retain(|&position, _| position < start);
        self.state.trailing_commas.retain(|&position, _| position < start);
        self.take_module_record(checkpoint);
    }

    /// Reparse the statements of the first parse which parsed `await` as an identifier.
    fn reparse_top_level_awaits(
        &mut self,
        statements: &mut ArenaVec<'a, Statement<'a>>,
        reported_ambient_statement: &mut bool,
    ) {
        let mut checkpoints = mem::take(&mut self.state.statement_checkpoints);
        debug_assert!(checkpoints[0].has_await_identifier);
        // The first parse stopped at the start of the statement after the last checkpoint
        checkpoints.push(self.statement_checkpoint());
        let end_index = checkpoints.len() - 1;
        let checkpoint = |index: usize| &checkpoints[index];
        let first_statement = statements.len() - end_index;

        let (mut original, declarations) =
            self.take_original_output(statements, first_statement, &checkpoints[0]);
        let original_cover_initialized_names = mem::take(&mut self.state.cover_initialized_name);
        self.state.trailing_commas.retain(|&position, _| position < checkpoints[0].start());

        // Source which was parsed again
        let mut reparsed = vec![];
        // Index of the checkpoint from which the first parse is kept
        let mut kept = 0;
        while let Some(start) =
            (kept..end_index).find(|&index| checkpoint(index).has_await_identifier)
        {
            self.keep_original(
                &mut original,
                statements,
                &checkpoints[kept..=start],
                first_statement + kept,
                reported_ambient_statement,
            );
            self.resume_at(checkpoint(start));

            // Parse statements until the first parse is reached again
            let mut next = start + 1;
            let overran = loop {
                let Some(statement) = self.parse_top_level_statement(reported_ambient_statement)
                else {
                    break true;
                };
                statements.push(statement);
                if self.has_fatal_error() {
                    break true;
                }
                let position = self.cur_start();
                while next <= end_index && checkpoint(next).start() < position {
                    next += 1;
                }
                if next > end_index {
                    break true;
                }
                if checkpoint(next).start() == position
                    && (next == end_index
                        || (!checkpoint(next).has_await_identifier
                            && checkpoint(next).parser.prev_token_end() == self.prev_token_end))
                {
                    break false;
                }
            };
            if overran {
                reparsed.push(Span::new(checkpoint(start).start(), u32::MAX));
                kept = end_index;
                break;
            }
            reparsed.push(Span::new(checkpoint(start).start(), checkpoint(next).start()));
            kept = next;
            if kept == end_index {
                break;
            }
            // Continue with the first parse's token. Its parse may have lexed it again.
            if self.lexer.config.tokens() {
                self.lexer.tokens_mut().pop();
            }
        }

        if kept < end_index {
            self.keep_original(
                &mut original,
                statements,
                &checkpoints[kept..],
                first_statement + kept,
                reported_ambient_statement,
            );
            self.resume_at(checkpoint(end_index));
            // The statement with the module syntax is kept, which the other records are of
            self.module_record_builder.append(declarations);
        }

        for (start, cover) in original_cover_initialized_names {
            if !reparsed.iter().any(|span: &Span| span.start <= start && start < span.end) {
                self.state.cover_initialized_name.entry(start).or_insert(cover);
            }
        }
    }

    /// Move the output of the first parse after `first` out of the parser.
    ///
    /// Also returns the module records other than dynamic imports and `import.meta`s,
    /// which are all of the statement with the module syntax.
    fn take_original_output(
        &mut self,
        statements: &mut ArenaVec<'a, Statement<'a>>,
        first_statement: usize,
        first: &StatementCheckpoint<'a>,
    ) -> (OriginalOutput<'a>, ModuleRecordBuilder<'a>) {
        let lexer = first.parser.lexer();
        let token_index = token_index(first);
        let tokens = self.lexer.tokens_mut().drain(token_index..).collect();
        let trivia = &mut self.lexer.trivia_builder;
        // The comments before a statement's token are output with it: whether they apply depends on
        // its parse. They stay before the current token too.
        let comments_start = trivia
            .comments
            .partition_point(|comment| comment.span.start < first.parser.prev_token_end());
        let comments = trivia.comments.split_off(comments_start);
        for comment in &comments[..lexer.comments_len() - comments_start] {
            trivia.comments.push(comment.clone());
        }
        let irregular_whitespaces =
            trivia.irregular_whitespaces.split_off(lexer.irregular_whitespaces_len());

        let (declarations, dynamic_imports, import_metas) = self.take_module_record(first);

        let original = OriginalOutput {
            statements: Original::new(
                statements.drain(first_statement..).collect(),
                first_statement,
            ),
            tokens: Original::new(tokens, token_index),
            comments: Original::new(comments, comments_start),
            irregular_whitespaces: Original::new(
                irregular_whitespaces,
                lexer.irregular_whitespaces_len(),
            ),
            errors: Original::new(
                self.errors.split_off(first.parser.errors_len()),
                first.parser.errors_len(),
            ),
            lexer_errors: Original::new(
                self.lexer.errors.split_off(lexer.errors_len()),
                lexer.errors_len(),
            ),
            deferred_module_errors: Original::new(
                self.lexer.deferred_module_errors.split_off(first.deferred_module_errors_len),
                first.deferred_module_errors_len,
            ),
            dynamic_imports: Original::new(dynamic_imports, first.dynamic_imports_len),
            import_metas: Original::new(import_metas, first.import_metas_len),
        };
        (original, declarations)
    }

    /// Start a module record with the dynamic imports and `import.meta`s before `first`.
    ///
    /// Returns the records which were not kept: the declarations, which are all of the statement
    /// with the module syntax, and the dynamic imports and `import.meta`s from `first` on.
    fn take_module_record(
        &mut self,
        first: &StatementCheckpoint<'a>,
    ) -> (ModuleRecordBuilder<'a>, Vec<DynamicImport>, Vec<Span>) {
        let module_record_builder = ModuleRecordBuilder::new(self.allocator(), self.source_type);
        let mut declarations = mem::replace(&mut self.module_record_builder, module_record_builder);
        let (mut dynamic_imports, mut import_metas) = declarations.take_expression_records();
        let later_dynamic_imports = dynamic_imports.drain(first.dynamic_imports_len..).collect();
        let later_import_metas = import_metas.drain(first.import_metas_len..).collect();
        for dynamic_import in dynamic_imports {
            self.module_record_builder.add_dynamic_import(dynamic_import);
        }
        for import_meta in import_metas {
            self.module_record_builder.add_import_meta(import_meta);
        }
        (declarations, later_dynamic_imports, later_import_metas)
    }

    /// Put back the output of the first parse from the first to the last of `checkpoints`,
    /// which is of its statements from `first_statement` on.
    fn keep_original(
        &mut self,
        original: &mut OriginalOutput<'a>,
        statements: &mut ArenaVec<'a, Statement<'a>>,
        checkpoints: &[StatementCheckpoint<'a>],
        first_statement: usize,
        reported_ambient_statement: &mut bool,
    ) {
        let (from, to) = (&checkpoints[0], &checkpoints[checkpoints.len() - 1]);
        let (from_lexer, to_lexer) = (from.parser.lexer(), to.parser.lexer());
        let statements_len = statements.len();
        statements.extend(
            original.statements.take(first_statement..first_statement + checkpoints.len() - 1),
        );
        if self.lexer.config.tokens() {
            let tokens = original.tokens.take(token_index(from)..token_index(to));
            self.lexer.tokens_mut().extend(tokens);
        }
        let trivia = &mut self.lexer.trivia_builder;
        // Replace the comments before the first token with the first parse's, which it applied
        let comments_start = trivia
            .comments
            .partition_point(|comment| comment.span.start < from.parser.prev_token_end());
        let leading_comments = trivia.comments.len() - comments_start;
        let mut comments = original
            .comments
            .take(from_lexer.comments_len() - leading_comments..to_lexer.comments_len());
        for (comment, original) in trivia.comments[comments_start..].iter_mut().zip(&mut comments) {
            debug_assert_eq!(comment.span, original.span);
            *comment = original;
        }
        for comment in comments {
            trivia.comments.push(comment);
        }
        trivia.irregular_whitespaces.extend(
            original
                .irregular_whitespaces
                .take(from_lexer.irregular_whitespaces_len()..to_lexer.irregular_whitespaces_len()),
        );
        let errors_len = self.errors.len();
        self.errors.extend(original.errors.take(from.parser.errors_len()..to.parser.errors_len()));
        self.lexer
            .errors
            .extend(original.lexer_errors.take(from_lexer.errors_len()..to_lexer.errors_len()));
        self.lexer.deferred_module_errors.extend(
            original
                .deferred_module_errors
                .take(from.deferred_module_errors_len..to.deferred_module_errors_len),
        );
        for dynamic_import in
            original.dynamic_imports.take(from.dynamic_imports_len..to.dynamic_imports_len)
        {
            self.module_record_builder.add_dynamic_import(dynamic_import);
        }
        for import_meta in original.import_metas.take(from.import_metas_len..to.import_metas_len) {
            self.module_record_builder.add_import_meta(import_meta);
        }
        if self.ctx.has_ambient() {
            self.check_original_ambient_statements(
                &statements[statements_len..],
                checkpoints,
                errors_len,
                reported_ambient_statement,
            );
        }
    }

    /// Continue parsing at the statement of `checkpoint`, after the output before it.
    ///
    /// The statement's token was moved out with the tokens from it on. It is pushed back as the
    /// last token, as the lexer pushed it when it lexed it.
    fn resume_at(&mut self, checkpoint: &StatementCheckpoint<'a>) {
        if self.lexer.config.tokens() {
            self.lexer.tokens_mut().push(checkpoint.parser.token());
        }
        self.resume(&checkpoint.parser);
    }

    /// Report the first statement of an ambient block which is not a declaration.
    pub(crate) fn check_ambient_statement(
        &mut self,
        statement: &Statement<'a>,
        reported_ambient_statement: &mut bool,
    ) {
        if !*reported_ambient_statement && !is_ambient_statement(statement) {
            self.error(diagnostics::statement_in_ambient_context(statement.span()));
            *reported_ambient_statement = true;
        }
    }

    /// Check the first parse's `statements`, of `checkpoints` but the last, in an ambient
    /// context, where its errors from the first checkpoint on are now at `errors_len`.
    ///
    /// The error is inserted where it would be if the statements had been checked when parsed.
    fn check_original_ambient_statements(
        &mut self,
        statements: &[Statement<'a>],
        checkpoints: &[StatementCheckpoint<'a>],
        errors_len: usize,
        reported_ambient_statement: &mut bool,
    ) {
        if *reported_ambient_statement {
            return;
        }
        // The last statement has the fatal error. Its error is discarded with the errors after it.
        let statements = &statements[..statements.len() - usize::from(self.fatal_error.is_some())];
        let Some(index) = statements.iter().position(|statement| !is_ambient_statement(statement))
        else {
            return;
        };
        let errors_len = errors_len
            + (checkpoints[index + 1].parser.errors_len() - checkpoints[0].parser.errors_len());
        let error = diagnostics::statement_in_ambient_context(statements[index].span());
        self.errors.insert(errors_len, error);
        if let Some(fatal_error) = &mut self.fatal_error {
            fatal_error.errors_len += 1;
        }
        *reported_ambient_statement = true;
    }
}

/// Whether `statement` is allowed in an ambient context.
fn is_ambient_statement(statement: &Statement<'_>) -> bool {
    statement.is_declaration() || statement.is_module_declaration()
}
