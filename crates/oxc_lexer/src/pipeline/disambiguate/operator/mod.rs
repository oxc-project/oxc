//! Is a position directly after a complete value?
//!
//! [`not_operator_position`] answers this for `carve`, to tell a regex from division,
//! and a JSX element from less-than. Its doc comment explains the question in detail.
//!
//! The token before the position settles nearly every case. Reading it takes most of this file,
//! because the token stream `carve` has built so far is not final:
//!
//! - Operators and numbers such as `x++`, `a?.b` and `1.5e+3` aren't formed into their final
//!   tokens until `coalesce`, which runs later, so the bytes around a punctuator are read
//!   directly ([`incdec_is_postfix`], [`prop_name`]).
//! - After `)`, what the parens closed decides it: `if (c) /re/` has a regex, but `f(c) / 2` has
//!   a division ([`paren_close_is_regex`]).
//!
//! The cases left over are questions about the parser's state, which [`context`] answers by
//! walking forward to the position:
//!
//! - After `}`, whether the braces closed a block or a value.
//! - Outside modules, whether `yield` and `await` are keywords, which depends on the enclosing
//!   functions, and whether `of` is the keyword of a `for ... of` head.
//! - In TypeScript, a `>` which may close type arguments, a `void` which is a type rather than
//!   the operator, and the line breaks after which a declaration ends without a semicolon.

use crate::{
    pipeline::bytes::is_digit,
    token::{OP_KIND_BASE, is_trivia_byte, matches_tk, tk},
};

use super::{
    WALK_SCAN_CAP,
    common::{Tokens, bits},
    context::{self, After, Walks},
};

#[cfg(test)]
mod tests;

/// Check if `p` is a position where an operator cannot go.
///
/// Operator position is directly after a complete value, e.g. after `a`, `f(x)`, or `a[0]`.
/// Anywhere else, an operator can't go, and something must start instead:
///
/// - An expression e.g. `x = /re/`, `x = <Foo />`.
/// - A statement e.g. `if (c) /re/.test(s)`, `if (c) <Foo />`.
/// - In TypeScript, a type e.g. `let f: <T>(x: T) => T`.
///
/// Returns `true` if `p` is not in operator position, `false` if it is.
///
/// Callers use this to decide:
///
/// - `/` starts a regex if `true`, or is a division operator (`/` or `/=`) if `false`.
/// - `<` may start a JSX element if `true`, or is a less-than operator if `false`.
///   In TS, a `<` in operator position can also open type arguments e.g. `f<T>()`.
///
/// The previous significant token decides most cases; `}`, `yield` / `await`, `of`, a TS `>` and
/// the TS line-break cases depend on unbounded left context and ask [`context`].
pub(crate) fn not_operator_position(tokens: &Tokens, walks: &mut Walks, p: usize) -> bool {
    let ts = tokens.ts;
    let module = tokens.module;
    let src = tokens.src;
    let mut q = bits::prev1(tokens.st, p);
    while let Some(qi) = q {
        let k = tokens.kind[qi];
        if is_trivia_byte(k) {
            q = bits::prev1(tokens.st, qi);
            continue;
        }
        if k == tk!(String) {
            let e = tokens.next_start(qi + 1);
            if !tokens.line_break_between(e, p) {
                return false;
            }
            return module_specifier_asi(tokens, qi)
                || (ts && context::after(tokens, walks, qi) == After::EndsDecl);
        }
        if matches_tk!(k, TemplateNoSub | TemplateTail) {
            if !(ts && tokens.line_break_between(tokens.next_start(qi + 1), p)) {
                return false;
            }
            let from = if k == tk!(TemplateTail) {
                tokens.template_head(qi, &mut 0, WALK_SCAN_CAP).unwrap_or(qi)
            } else {
                qi
            };
            return context::after_from(tokens, walks, qi, from) == After::EndsDecl;
        }
        if matches_tk!(k, RegExp | PrivateIdent | PrivateIdentEscaped | JsxTagEnd | JsxLt) {
            return false;
        }
        if matches_tk!(k, TemplateHead | TemplateMiddle) {
            return true;
        }
        if k == tk!(Number) {
            // A word run that starts with a digit is a numeric literal, and a numeric literal ends
            // a value: division, unless TS ASI applies.
            let we = bits::next0(tokens.word, qi, tokens.n);
            return ts
                && tokens.line_break_between(we, p)
                && context::after(tokens, walks, qi) == After::EndsDecl;
        }
        if matches_tk!(k, Ident | IdentEscaped) {
            let e = tokens.next_start(qi + 1);
            let newline = tokens.line_break_between(e, p);
            if prop_name(tokens, qi) {
                return ts && newline && context::after(tokens, walks, qi) == After::EndsDecl;
            }
            let kw = tokens.word_kw(qi, e - qi);
            if tokens.tables.keywords.is_regex_keyword(kw) {
                if ts && kw == tk!(KwVoid) {
                    // `x as void / 2` is division; `void /re/` is not.
                    return context::after(tokens, walks, qi) != After::Value;
                }
                if !module && matches_tk!(kw, KwYield | KwAwait) {
                    return context::after_scoped(tokens, walks, qi) != After::Value;
                }
                return true;
            }
            if kw == tk!(KwOf) {
                return context::after(tokens, walks, qi) == After::Operand;
            }
            if newline {
                return context::after(tokens, walks, qi) == After::EndsDecl;
            }
            return false;
        }
        if k >= OP_KIND_BASE {
            let ch = src[qi];
            // TS postfix non-null `!`: `x! / 2` is division, so look through the `!` unless a line
            // terminator precedes it (ASI makes it a prefix `!/re/`).
            if ts && ch == b'!' {
                let Some(q2) = tokens.prev_sig(qi) else {
                    return true;
                };
                if tokens.line_break_between(tokens.next_start(q2 + 1), qi) {
                    return true;
                }
                q = Some(q2);
                continue;
            }
            if ch == b'.' {
                // Only a trailing-dot numeric literal (`1./2`) makes `.` end a value; member
                // access, `...` and `?.` all precede an operand.
                return !bits::prev1(tokens.st, qi).is_some_and(|q2| {
                    tokens.kind[q2] == tk!(Number) && tokens.next_start(q2 + 1) == qi
                });
            }
            if ch == b'+' || ch == b'-' {
                // Tail of `++`/`--`: postfix ends a value, prefix does not; a run of three or more
                // always ends in a prefix pair or a lone sign.
                if qi > 0 && src[qi - 1] == ch {
                    return !incdec_is_postfix(tokens, qi - 1);
                }
                return true;
            }
            // `}` closed either a block (regex follows) or a value (division).
            if ch == b'}' {
                let from = tokens.match_delim_back(qi, b'{', b'}').unwrap_or(qi);
                return context::after_from(tokens, walks, qi, from) != After::Value;
            }
            if ch == b')' {
                if paren_close_is_regex(tokens, qi) {
                    return true;
                }
                if !(ts && tokens.line_break_between(qi + 1, p)) {
                    return false;
                }
                let from = tokens.match_delim_back(qi, b'(', b')').unwrap_or(qi);
                return context::after_from(tokens, walks, qi, from) == After::EndsDecl;
            }
            if ts && ch == b'>' && !(qi > 0 && src[qi - 1] == b'=') {
                return context::after(tokens, walks, qi) != After::Value;
            }
            if ch == b']' {
                if !(ts && tokens.line_break_between(qi + 1, p)) {
                    return false;
                }
                let from = tokens.match_delim_back(qi, b'[', b']').unwrap_or(qi);
                return context::after_from(tokens, walks, qi, from) == After::EndsDecl;
            }
            return true;
        }
        return true;
    }
    true
}

/// Is the word at `pos` a property name: does a lone `.` (member access or `?.`, not the last dot
/// of `...`) precede it as a token? Trivia in between does not matter: `x. return / 2` divides.
#[inline]
fn prop_name(tokens: &Tokens, pos: usize) -> bool {
    let Some(w) = tokens.prev_sig(pos) else {
        return false;
    };
    if tokens.kind[w] < OP_KIND_BASE {
        return false;
    }
    let src = tokens.src;
    let c = src[w];
    if c == b'?' {
        // A `?.` that `coalesce` has already fused starts at the `?`.
        return src[w + 1] == b'.' && !is_digit(src[w + 2]);
    }
    // A lone `.`: not a `...`, whether the run is still three token starts (the previous token is
    // then its last dot) or already fused (its first).
    c == b'.'
        && !(src[w + 1] == b'.' && src[w + 2] == b'.')
        && !(w >= 2 && src[w - 1] == b'.' && src[w - 2] == b'.')
}

#[inline]
fn module_specifier_asi(tokens: &Tokens, spec: usize) -> bool {
    let Some(w) = tokens.prev_sig(spec) else {
        return false;
    };
    tokens.kind[w] == tk!(Ident)
        && !prop_name(tokens, w)
        && (tokens.ident_is(w, b"from") || tokens.ident_is(w, b"import"))
}

/// Is the `++`/`--` whose first byte is at `first` postfix? It is when a value ends right before it
/// on the same line; `tail_before` is the shared definition of "ends a value".
fn incdec_is_postfix(tokens: &Tokens, first: usize) -> bool {
    tokens.prev_sig(first).is_some_and(|q| {
        tail_before(tokens, first) && !tokens.line_break_between(tokens.next_start(q + 1), first)
    })
}

fn tail_before(tokens: &Tokens, pos: usize) -> bool {
    let src = tokens.src;
    let mut s = tokens.prev_sig(pos);
    while let Some(w) = s {
        if tokens.base_kind(w) >= OP_KIND_BASE && src[w] == b'!' && src[w + 1] != b'=' {
            s = tokens.prev_sig(w);
            continue;
        }
        break;
    }
    let Some(sp) = s else {
        return false;
    };
    let sk = tokens.base_kind(sp);
    if sk >= OP_KIND_BASE {
        return matches!(src[sp], b')' | b']');
    }
    if sk == tk!(Ident) {
        let e = tokens.next_start(sp + 1);
        return prop_name(tokens, sp)
            || !tokens.tables.keywords.is_regex_keyword(tokens.word_kw(sp, e - sp));
    }
    matches_tk!(sk, Number | BigInt | String | TemplateNoSub | TemplateTail | RegExp | PrivateIdent)
}

fn paren_close_is_regex(tokens: &Tokens, qi: usize) -> bool {
    let Some(lp) = tokens.match_delim_back(qi, b'(', b')') else {
        return false;
    };
    let Some(mut w) = tokens.prev_sig(lp) else {
        return false;
    };
    if tokens.kind[w] != tk!(Ident) {
        return false;
    }
    if tokens.ident_is(w, b"await") {
        let Some(q2) = tokens.prev_sig(w) else {
            return false;
        };
        if tokens.kind[q2] != tk!(Ident) {
            return false;
        }
        w = q2;
        return !prop_name(tokens, w) && tokens.ident_is(w, b"for");
    }
    !prop_name(tokens, w)
        && (tokens.ident_is(w, b"if")
            || tokens.ident_is(w, b"while")
            || tokens.ident_is(w, b"for")
            || tokens.ident_is(w, b"with"))
}
