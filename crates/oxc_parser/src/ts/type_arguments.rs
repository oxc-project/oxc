//! Bound repeated parsing of nested speculative type arguments.
//!
//! A comparison list such as `[a < b, a < b, ...]` can look like nested generic types.
//! Cache delimiter proofs so rewinding does not repeatedly parse the same unclosed suffix.
//! Regexes, templates and JSX can make lookahead inconclusive; cache actual parse failures
//! with their grammar context to avoid retrying those paths as well. Both caches are lazy,
//! keeping ordinary shallow type arguments on the existing parser path.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{Context, ParserConfig as Config, ParserImpl, lexer::Kind};

/// Let ordinary generics parse without an extra scan, but bound the recursive prefix
/// before checking whether deeper speculation can actually close.
pub(super) const TYPE_ARGUMENT_LOOKAHEAD_DEPTH: u32 = 128;

/// Failed nested speculation can repeat even without deep native-stack recursion.
const FAILURE_CACHE_DEPTH: u32 = 4;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TypeArgumentFailure {
    Type,
    Expression,
}

#[derive(Default)]
pub struct TypeArgumentLookahead {
    /// Do not repeat lookahead before this frontier. Absence of a delimiter rejection
    /// is inconclusive; the grammar still decides whether a candidate parses.
    scanned_until: u32,
    /// Delimiter proofs are independent of grammar context.
    unclosed: FxHashSet<u32>,
    /// A failed type list also rules out expression type arguments, but an expression's
    /// follow-token check does not rule out a list in a type. Source type and options are
    /// fixed for this parser; grammar flags can change when the same position is revisited.
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

        let checkpoint = self.lexer.checkpoint();
        let errors_len = self.lexer.errors.len();
        let mut angles = vec![start];
        if matches!(self.cur_kind(), Kind::ShiftLeft | Kind::ShiftLeftEq) {
            angles.push(start + 1);
        }
        // Delimiters inside a group cannot close angles outside that group.
        let mut groups = Vec::new();
        let jsx = self.source_type.is_jsx();
        let cache = self.state.type_argument_lookahead.get_or_insert_with(Box::default);
        let mut scanned_until;
        let can_close = loop {
            let token = self.lexer.next_token();
            scanned_until = token.start();
            if self.lexer.errors.len() != errors_len {
                break true;
            }
            let kind = token.kind();
            match kind {
                // These need the parser to select a lexical goal. Do not draw conclusions
                // from tokenizing a regex, template substitution, or invalid token as types.
                Kind::Slash | Kind::SlashEq | Kind::TemplateHead | Kind::Undetermined => {
                    break true;
                }
                Kind::LAngle | Kind::ShiftLeft | Kind::LtEq | Kind::ShiftLeftEq => {
                    if jsx && !groups.is_empty() {
                        break true;
                    }
                    angles.push(token.start());
                    if matches!(kind, Kind::ShiftLeft | Kind::ShiftLeftEq) {
                        angles.push(token.start() + 1);
                    }
                }
                Kind::LParen | Kind::LBrack | Kind::LCurly => {
                    groups.push((kind, angles.len()));
                }
                Kind::RParen | Kind::RBrack | Kind::RCurly => {
                    let Some((opening, first_angle)) = groups.pop() else { break false };
                    if !matches!(
                        (opening, kind),
                        (Kind::LParen, Kind::RParen)
                            | (Kind::LBrack, Kind::RBrack)
                            | (Kind::LCurly, Kind::RCurly)
                    ) {
                        break true;
                    }
                    cache.unclosed.extend(angles.drain(first_angle..));
                }
                Kind::Semicolon => {
                    let first_angle = groups.last().map_or(0, |&(_, first_angle)| first_angle);
                    cache.unclosed.extend(angles.drain(first_angle..));
                    if groups.is_empty() {
                        break false;
                    }
                }
                Kind::RAngle => {
                    let first_angle = groups.last().map_or(0, |&(_, first_angle)| first_angle);
                    if angles.len() > first_angle {
                        angles.pop();
                    }
                    if angles.is_empty() {
                        break true;
                    }
                }
                Kind::Eof => break false,
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
