//! Bound repeated parsing of nested speculative type arguments.
//!
//! A comparison list such as `[a < b, a < b, ...]` can look like nested generic types.
//! Cache delimiter proofs so rewinding does not repeatedly parse the same unclosed suffix.
//! Regexes, templates and JSX can make lookahead inconclusive; cache actual parse failures
//! with their grammar context to avoid retrying those paths as well. Both caches are lazy,
//! keeping ordinary shallow type arguments on the existing parser path.
//!
//! The relevant TypeScript list has the shape `< Type (',' Type)* ','? >`. A type can
//! itself contain type arguments, generic function parameters, tuples, object members,
//! conditional types, and expressions in computed property names. Commas, whitespace,
//! and line breaks within the list therefore cannot prove that speculation must fail.
//! The grammar parser also handles diagnostics such as an empty `<>`; this scan does not.
//!
//! This is a necessary-delimiter check: `false` proves that an opening `<` cannot close
//! before its containing construct ends; `true` only means to try the grammar parser.
//! In particular, counting angles cannot distinguish nested generics from the relational
//! operators in `a < b > c`. Expression type arguments additionally need a suitable
//! following token, and can form a standalone instantiation expression such as `f<T>`.
//! Requiring a call's `(` would reject that syntax.
//!
//! References: [ECMAScript relational operators], TypeScript's [type-argument parsing]
//! and [instantiation expressions]. Delimiter rejection below is an optimization inferred
//! from those rules; lexical or grammatical uncertainty must remain inconclusive.
//!
//! [ECMAScript relational operators]: https://tc39.es/ecma262/#sec-relational-operators
//! [type-argument parsing]: https://github.com/microsoft/TypeScript/blob/v5.9.3/src/compiler/parser.ts#L6459-L6503
//! [instantiation expressions]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-7.html#instantiation-expressions

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{Context, ParserConfig as Config, ParserImpl, lexer::Kind};

/// Let ordinary generics parse without an extra scan, but bound the recursive prefix
/// before checking whether deeper speculation can actually close.
/// This is a performance threshold, not a grammar limit on valid nesting.
pub(super) const TYPE_ARGUMENT_LOOKAHEAD_DEPTH: u32 = 128;

/// Failed nested speculation can repeat even without deep native-stack recursion.
const FAILURE_CACHE_DEPTH: u32 = 4;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TypeArgumentFailure {
    /// The list itself failed under the recorded context, regardless of its use site.
    Type,
    /// Expression speculation failed, possibly only because of the token after `>`.
    Expression,
}

#[derive(Default)]
pub struct TypeArgumentLookahead {
    /// Do not repeat lookahead before this frontier. Absence of a delimiter rejection
    /// is inconclusive; the grammar still decides whether a candidate parses.
    /// Store the last token's start, not its end: a token that stopped the scan may
    /// need a different lexical goal, so its contents have not been examined safely.
    scanned_until: u32,
    /// Delimiter proofs are independent of grammar context.
    unclosed: FxHashSet<u32>,
    /// A failed type list also rules out expression type arguments, but an expression's
    /// follow-token check does not rule out a list in a type. Source type and options are
    /// fixed for this parser; grammar flags can change when the same position is revisited.
    /// For example, `DisallowConditionalTypes` changes whether `extends` starts a
    /// conditional type; expression-bearing types can also depend on `In`/`Yield`/`Await`.
    failed: FxHashMap<(u32, Context), TypeArgumentFailure>,
}

impl TypeArgumentLookahead {
    pub(crate) fn cannot_close(&self, start: u32) -> bool {
        self.unclosed.contains(&start)
    }

    pub(super) fn has_failed(
        &self,
        start: u32,
        context: Context,
        rule: TypeArgumentFailure,
    ) -> bool {
        self.failed
            .get(&(start, context))
            .is_some_and(|&failure| failure == TypeArgumentFailure::Type || failure == rule)
    }

    pub(super) fn record_failure(
        &mut self,
        start: u32,
        context: Context,
        rule: TypeArgumentFailure,
    ) {
        if rule == TypeArgumentFailure::Type {
            // A list failure is stronger than rejection by the expression follow-token rule.
            self.failed.insert((start, context), rule);
        } else {
            self.failed.entry((start, context)).or_insert(rule);
        }
    }
}

impl<C: Config> ParserImpl<'_, C> {
    pub(super) fn record_type_argument_failure(
        &mut self,
        start: u32,
        context: Context,
        rule: TypeArgumentFailure,
        depth: u32,
    ) {
        let cache = if depth >= FAILURE_CACHE_DEPTH {
            self.state.type_argument_lookahead.get_or_insert_with(Box::default)
        } else if let Some(cache) = &mut self.state.type_argument_lookahead {
            cache
        } else {
            return;
        };
        cache.record_failure(start, context, rule);
    }

    pub(super) fn speculative_type_arguments_can_close(&mut self) -> bool {
        let start = self.cur_start();
        if let Some(cache) = &self.state.type_argument_lookahead {
            if cache.cannot_close(start) {
                return false;
            }
            if start < cache.scanned_until {
                return true;
            }
        }

        // The parser's current token already consumed the opening angle from the lexer.
        // Seed it explicitly, then inspect subsequent tokens without committing lexer state.
        let checkpoint = self.lexer.checkpoint();
        let errors_len = self.lexer.errors.len();
        let mut angles = vec![start];
        if matches!(self.cur_kind(), Kind::ShiftLeft | Kind::ShiftLeftEq) {
            // `f<<T>() => T>()` starts with two openings in one lexer token.
            angles.push(start + 1);
        }
        // Types contain (), [] and {} (function/parenthesized types, tuples/indexed
        // accesses, and object/mapped types). Computed keys can contain expressions too.
        // Each group saves the first angle opened inside it: `>` in `{ [x > y]: T }`
        // cannot close an outer type argument, and `<` in `[x < y]` cannot escape `]`.
        let mut groups = Vec::new();
        let jsx = self.source_type.is_jsx();
        let cache = self.state.type_argument_lookahead.get_or_insert_with(Box::default);
        let mut scanned_until;
        let can_close = loop {
            let token = self.lexer.next_token();
            scanned_until = token.start();
            if self.lexer.errors.len() != errors_len {
                // Recovery tokenization is not evidence about the intended delimiters.
                break true;
            }
            let kind = token.kind();
            match kind {
                // A computed key can contain `/[<>]/` (or a regex starting `/=`), whose
                // punctuation is not type syntax. `${...}` in template types/expressions
                // requires the parser to re-lex `}` as a template continuation. Stop before
                // either lexical goal can make subsequent ordinary tokens misleading.
                // An undetermined token likewise cannot support a delimiter proof.
                Kind::Slash | Kind::SlashEq | Kind::TemplateHead | Kind::Undetermined => {
                    break true;
                }
                Kind::LAngle | Kind::ShiftLeft | Kind::LtEq | Kind::ShiftLeftEq => {
                    if jsx && !groups.is_empty() {
                        // A computed-key expression may start JSX: its tag/text scanner
                        // gives `<` and `>` different roles. Without parsing the group we
                        // cannot distinguish this from nested generics or a comparison.
                        break true;
                    }
                    // Count every leading `<`, including generic function type parameters.
                    // Type-context re-lexing can split all four token kinds. `=` makes some
                    // candidates invalid, but validating their contents belongs to the parser.
                    angles.push(token.start());
                    if matches!(kind, Kind::ShiftLeft | Kind::ShiftLeftEq) {
                        angles.push(token.start() + 1);
                    }
                }
                Kind::LParen | Kind::LBrack | Kind::LCurly => {
                    groups.push((kind, angles.len()));
                }
                Kind::RParen | Kind::RBrack | Kind::RCurly => {
                    // With no group opened by this scan, the closer ends the construct
                    // containing the candidate. Its required `>` has not appeared.
                    let Some((opening, first_angle)) = groups.pop() else { break false };
                    if !matches!(
                        (opening, kind),
                        (Kind::LParen, Kind::RParen)
                            | (Kind::LBrack, Kind::RBrack)
                            | (Kind::LCurly, Kind::RCurly)
                    ) {
                        // Mismatched groups need the parser's own diagnostic/recovery path.
                        break true;
                    }
                    // Only openings inside this group are ruled out. Outer angles may
                    // still close later, as in `f<{ [x < y]: T }>()`.
                    cache.unclosed.extend(angles.drain(first_angle..));
                }
                Kind::Semicolon => {
                    // A semicolon cannot occur directly in a type-argument list. Within
                    // a group it can separate object-type members or statements in a
                    // computed-key function body, so retain openings outside that group:
                    // `f<{ a: T; b: U }>()` still has a possible closing angle.
                    let first_angle = groups.last().map_or(0, |&(_, first_angle)| first_angle);
                    cache.unclosed.extend(angles.drain(first_angle..));
                    if groups.is_empty() {
                        break false;
                    }
                }
                Kind::RAngle => {
                    // Ordinary lexing yields individual `>` tokens even in `>>`, `>>>`
                    // and `>=`; expression parsing re-lexes the operators later. This
                    // deliberately leaves their disambiguation to the grammar parser.
                    // The `>` in a function type's `=>` is part of an Arrow token instead.
                    let first_angle = groups.last().map_or(0, |&(_, first_angle)| first_angle);
                    if angles.len() > first_angle {
                        angles.pop();
                    }
                    if angles.is_empty() {
                        // Balanced delimiters are necessary, but contents and following
                        // tokens may still make this a comparison rather than type arguments.
                        break true;
                    }
                }
                // No later token can provide the missing closing angle.
                Kind::Eof => break false,
                // The lexer skips comments/trivia and consumes strings and templates
                // without substitutions as whole tokens, hiding their punctuation. Other
                // tokens (commas, unions, intersections, `extends`, `?`, `=>`, etc.) can be
                // part of a type or an embedded expression; leave them to the real parser.
                _ => {}
            }
        };
        cache.scanned_until = cache.scanned_until.max(scanned_until);
        if !can_close {
            cache.unclosed.extend(angles);
        }
        self.lexer.rewind(checkpoint);
        can_close
    }
}
