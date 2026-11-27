//! Reparsing top-level statements which parsed `await` as an identifier.
//!
//! In unambiguous mode, `await` is an identifier until module syntax shows that the source is a
//! module. After that, `await /x/u;` is not 2 divisions, but an `await` of a regular expression.
//!
//! The first parse stops after the statement with the module syntax. As in TypeScript's
//! `reparseTopLevelAwait`, the statements before it in which `await` was an identifier are parsed
//! again, as module code. Each reparse continues until it ends where a statement of the first
//! parse started, with the same trivia before it, and that statement did not have `await` as an
//! identifier. The first parse's statements from there on are kept, together with their tokens,
//! trivia, errors and module records. Then parsing continues as a module.

use std::{mem, ops::Range, vec};

use oxc_allocator::{ArenaVec, GetAllocator};
use oxc_ast::ast::{Comment, Statement};
use oxc_span::{GetSpan, Span};
use oxc_syntax::module_record::DynamicImport;

use crate::{
    ParserConfig as Config, ParserImpl, StatementContext, config::LexerConfig,
    cursor::ParserCheckpoint, diagnostics, diagnostics::ParserDiagnostic, lexer::Token,
    module_record::ModuleRecordBuilder,
};

/// State before a top-level statement of the first parse.
pub struct StatementCheckpoint<'a> {
    parser: ParserCheckpoint<'a>,
    deferred_module_errors_len: usize,
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
            dynamic_imports_len: self.module_record_builder.dynamic_imports_len(),
            import_metas_len: self.module_record_builder.import_metas_len(),
            has_await_identifier: false,
        }
    }

    /// Parse a top-level statement of a module, after the first parse.
    fn parse_module_statement(
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
            self.ctx = self.ctx.and_await(true);
            // Discard what lookahead found while `await` was an identifier
            self.state.not_parenthesized_arrow.clear();
            self.state.type_argument_lookahead = None;
            if self.fatal_error.is_some() {
                self.reparse_after_fatal_error(statements);
            } else {
                self.reparse_top_level_awaits(statements, reported_ambient_statement);
            }
            while let Some(statement) = self.parse_module_statement(reported_ambient_statement) {
                statements.push(statement);
            }
        } else {
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

    /// Parse the statements from the first with an `await` identifier on again, where the first
    /// parse stopped at a fatal error in the statement with the module syntax.
    fn reparse_after_fatal_error(&mut self, statements: &mut ArenaVec<'a, Statement<'a>>) {
        let checkpoints = mem::take(&mut self.state.statement_checkpoints);
        let first = &checkpoints[0];
        let first_statement = statements.len() - checkpoints.len();
        statements.truncate(first_statement);
        self.rewind(first.parser);
        self.lexer.deferred_module_errors.truncate(first.deferred_module_errors_len);
        let start = first.start();
        self.state.cover_initialized_name.retain(|&position, _| position < start);
        self.state.trailing_commas.retain(|&position, _| position < start);
        self.take_module_record(first);
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
                let Some(statement) = self.parse_module_statement(reported_ambient_statement)
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
        // Parsed again, the module syntax may not be, as in `await /x; export {}; y/u`.
        // Like Babel, the source is still a module.
        self.module_record_builder.set_module_syntax();
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
