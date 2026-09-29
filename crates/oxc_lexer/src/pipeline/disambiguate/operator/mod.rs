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
//!   directly ([Tokens::property_name]).
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

use crate::token::{OP_KIND_BASE, is_trivia_byte, matches_tk, tk};

use crate::pipeline::keywords::is_regex_keyword;

use super::{
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
            return ends_decl(tokens, walks, qi, tokens.next_start(qi + 1), p);
        }
        if matches_tk!(k, Number | TemplateNoSub | TemplateTail) {
            // A literal ends a value: division, unless TS ASI applies.
            return ts && ends_decl(tokens, walks, qi, tokens.next_start(qi + 1), p);
        }
        if matches_tk!(k, RegExp | PrivateIdent | PrivateIdentEscaped | JsxTagEnd | JsxLt) {
            return false;
        }
        if matches_tk!(k, TemplateHead | TemplateMiddle) {
            return true;
        }
        if matches_tk!(k, Ident | IdentEscaped) {
            let e = tokens.next_start(qi + 1);
            let newline = tokens.line_break_between(e, p);
            if tokens.property_name(qi) {
                return ts && newline && context::after(tokens, walks, qi) == After::EndsDecl;
            }
            let kw = tokens.word_kw(qi, e - qi);
            if is_regex_keyword(kw) {
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
                return context::after(tokens, walks, qi) != After::Value;
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
                return !tokens.numeric_dot(qi);
            }
            if ch == b'+' || ch == b'-' {
                // The tail of a ++ or --: postfix ends a value, prefix does not.
                if qi > 0 && src[qi - 1] == ch {
                    return context::after(tokens, walks, qi) != After::Value;
                }
                return true;
            }
            // `}` closed either a block (regex follows) or a value (division).
            if ch == b'}' {
                return context::after(tokens, walks, qi) != After::Value;
            }
            if ch == b')' && paren_close_is_regex(tokens, qi) {
                return true;
            }
            if ch == b')' || ch == b']' {
                return ts && ends_decl(tokens, walks, qi, qi + 1, p);
            }
            if ts && ch == b'>' && !(qi > 0 && src[qi - 1] == b'=') {
                return context::after(tokens, walks, qi) != After::Value;
            }
            return true;
        }
        return true;
    }
    true
}

/// Does a line break between e and p end the declaration the token at q completes?
fn ends_decl(tokens: &Tokens, walks: &mut Walks, q: usize, e: usize, p: usize) -> bool {
    tokens.line_break_between(e, p) && context::after(tokens, walks, q) == After::EndsDecl
}

fn paren_close_is_regex(tokens: &Tokens, qi: usize) -> bool {
    let word =
        |p: Option<usize>| p.filter(|&w| tokens.kind[w] == tk!(Ident) && !tokens.property_name(w));
    let Some(w) = tokens.match_delim_back(qi).and_then(|lp| word(tokens.prev_sig(lp))) else {
        return false;
    };
    if tokens.ident_is(w, b"await") {
        return word(tokens.prev_sig(w)).is_some_and(|f| tokens.ident_is(f, b"for"));
    }
    [b"if" as &[u8], b"while", b"for", b"with"].iter().any(|kw| tokens.ident_is(w, kw))
}
