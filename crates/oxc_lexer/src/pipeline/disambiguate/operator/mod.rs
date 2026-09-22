use crate::{
    pipeline::bytes::is_digit,
    token::{OP_KIND_BASE, tk},
};

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
pub(crate) fn not_operator_position(tokens: &Tokens, walks: &mut Walks, p: usize) -> bool {
    let ts = tokens.ts;
    let module = tokens.module;
    let src = tokens.src;
    let mut q = bits::prev1(tokens.st, p);
    while let Some(qi) = q {
        let k = tokens.kind[qi];
        if k == tk!(Whitespace)
            || k == tk!(LineComment)
            || k == tk!(BlockComment)
            || k == tk!(Hashbang)
        {
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
        if k == tk!(TemplateNoSub) || k == tk!(TemplateTail) {
            if !(ts && tokens.line_break_between(tokens.next_start(qi + 1), p)) {
                return false;
            }
            let from = if k == tk!(TemplateTail) { template_head(tokens, qi) } else { qi };
            return context::after_from(tokens, walks, qi, from) == After::EndsDecl;
        }
        if k == tk!(RegExp)
            || k == tk!(PrivateIdent)
            || k == tk!(PrivateIdentEscaped)
            || k == tk!(JsxTagEnd)
            || k == tk!(JsxLt)
        {
            return false;
        }
        if k == tk!(TemplateHead) || k == tk!(TemplateMiddle) {
            return true;
        }
        if k == tk!(Number) {
            // A word run starting with a digit is a number: division, unless TS ASI applies.
            let we = bits::next0(tokens.word, qi, tokens.n);
            return ts
                && tokens.line_break_between(we, p)
                && context::after(tokens, walks, qi) == After::EndsDecl;
        }
        if k == tk!(Ident) || k == tk!(IdentEscaped) {
            let e = tokens.next_start(qi + 1);
            let newline = tokens.line_break_between(e, p);
            if prop_name(tokens, qi) {
                return ts && newline && context::after(tokens, walks, qi) == After::EndsDecl;
            }
            if k == tk!(Ident) && tokens.tables.keywords.is_regex_keyword_at(src, qi, e - qi) {
                if ts && e - qi == 4 && tokens.ident_is(qi, b"void") {
                    // x as void / 2 is division; void /re/ is not.
                    return context::after(tokens, walks, qi) != After::Value;
                }
                if !module
                    && e - qi == 5
                    && (tokens.ident_is(qi, b"yield") || tokens.ident_is(qi, b"await"))
                {
                    return context::after_scoped(tokens, walks, qi) != After::Value;
                }
                return true;
            }
            if k == tk!(Ident) && e - qi == 2 && src[qi] == b'o' && src[qi + 1] == b'f' {
                return context::after(tokens, walks, qi) == After::Operand;
            }
            if newline {
                return context::after(tokens, walks, qi) == After::EndsDecl;
            }
            return false;
        }
        if k >= OP_KIND_BASE {
            let ch = src[qi];
            // TS postfix non-null `!`: `x! / 2` is division, not a regex -
            // look through the `!`, unless a newline sits before it (ASI
            // makes it a prefix `!/re/`).
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
                // Only a trailing-dot number (1./2) makes . end a value.
                return !bits::prev1(tokens.st, qi).is_some_and(|q2| {
                    tokens.kind[q2] == tk!(Number) && tokens.next_start(q2 + 1) == qi
                });
            }
            if ch == b'+' || ch == b'-' {
                // Tail of ++ / --: postfix ends a value, prefix does not.
                if qi > 0 && src[qi - 1] == ch {
                    return !incdec_is_postfix(tokens, qi - 1);
                }
                return true;
            }
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

/// Does a lone . (member access or ?., not ...) precede the word, trivia aside?
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
        // A ?. that coalesce has already fused starts at the ?.
        return src[w + 1] == b'.' && !is_digit(src[w + 2]);
    }
    // A lone .: not part of a ..., fused or not.
    c == b'.'
        && !(src[w + 1] == b'.' && src[w + 2] == b'.')
        && !(w >= 2 && src[w - 1] == b'.' && src[w - 2] == b'.')
}

fn template_head(tokens: &Tokens, tail: usize) -> usize {
    let mut depth = 0u32;
    let mut q = tokens.prev_sig(tail);
    while let Some(w) = q {
        match tokens.kind[w] {
            tk!(TemplateTail) => depth += 1,
            tk!(TemplateHead) => {
                if depth == 0 {
                    return w;
                }
                depth -= 1;
            }
            _ => {}
        }
        q = tokens.prev_sig(w);
    }
    tail
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
            || !tokens.tables.keywords.is_regex_keyword_at(src, sp, e - sp);
    }
    matches!(
        sk,
        tk!(Number)
            | tk!(BigInt)
            | tk!(String)
            | tk!(TemplateNoSub)
            | tk!(TemplateTail)
            | tk!(RegExp)
            | tk!(PrivateIdent)
    )
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
