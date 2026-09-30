use crate::config::LexerConfig as Config;

use super::{Kind, Lexer, Token};

impl<C: Config> Lexer<'_, C> {
    /// Re-tokenize `<<`, `<=`, or `<<=` to `<`.
    ///
    /// Called when the parser encounters a compound token starting with `<` (e.g. `<<`)
    /// in a position where it could be the start of type arguments (e.g. `foo<<T>()>`).
    /// The lexer eagerly produces the compound token, but the parser needs a single `<`
    /// to speculatively try parsing type arguments.
    ///
    /// Only `<<` reaches here in code which parses. `<=` and `<<=` arrive only from the type-context
    /// callers in `ts/types.rs` and always go on to fail, for the reason documented on
    /// `parse_type_arguments_in_expression`, which rejects them before reaching here.
    ///
    /// The compound token was pushed to the collected token stream *before* `try_parse`
    /// created its checkpoint (since it was the "current" token at that point), so it is
    /// the last token in the stream. Overwrite it there with the single `<`. The remaining
    /// characters after the first `<` (e.g. the second `<` in `<<`) are lexed as separate
    /// tokens on subsequent `next_token` calls, so the stream ends up with all of them.
    ///
    /// If the speculative parse fails and `try_parse` rewinds, `truncate` leaves this `<`
    /// as the last collected token, and the caller (`expression.rs`) restores the original
    /// compound token over it via `rewrite_last_collected_token`.
    pub(crate) fn re_lex_as_typescript_l_angle(&mut self, offset: u32) -> Token {
        self.token.set_start(self.offset() - offset);
        self.source.back(offset as usize - 1);
        let token = self.finish_re_lex(Kind::LAngle);
        self.rewrite_last_collected_token(token);
        token
    }

    /// Re-tokenize `>>` or `>>>` to `>`.
    ///
    /// Called during speculative type argument parsing when the parser encounters
    /// `>>` or `>>>` and needs to split off a single `>` to close the type arguments.
    ///
    /// Unlike `re_lex_as_typescript_l_angle`, this does NOT need to pop from the
    /// collected token stream. The `>` character is initially lexed as just `RAngle`
    /// (the lexer handles `>` lazily). The compound `>>` / `>>>` is only created
    /// later by `re_lex_right_angle` (Replace mode), which happens *during* the
    /// speculative parse (i.e. after `try_parse`'s checkpoint). This is because
    /// the checkpoint is created when the parser is at `<` (the opening bracket),
    /// and the `>` being replaced is the closing bracket, encountered later during
    /// the parse. Since both the original push and the Replace are at a post-checkpoint
    /// position, `truncate(checkpoint.tokens_len)` on rewind removes it automatically.
    pub(crate) fn re_lex_as_typescript_r_angle(&mut self, offset: u32) -> Token {
        self.token.set_start(self.offset() - offset);
        self.source.back(offset as usize - 1);
        self.finish_re_lex(Kind::RAngle)
    }
}
