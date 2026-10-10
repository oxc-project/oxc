//! Ports of internal/utils/builtin_symbol_likes.go helpers, added as rules need them.

use super::*;

/// builtin_symbol_likes.go IsPromiseConstructorLike.
pub fn is_promise_constructor_like(program: &Program, c: &mut Checker, t: P<Type>) -> bool {
    is_builtin_symbol_like(program, c, t, &["PromiseConstructor"])
}

/// builtin_symbol_likes.go IsErrorLike.
pub fn is_error_like(program: &Program, c: &mut Checker, t: P<Type>) -> bool {
    is_builtin_symbol_like(program, c, t, &["Error"])
}

/// builtin_symbol_likes.go IsReadonlyErrorLike.
pub fn is_readonly_error_like(program: &Program, c: &mut Checker, t: P<Type>) -> bool {
    is_readonly_type_like(program, c, t, &mut |c, subtype| {
        let type_argument = subtype.alias().unwrap().type_arguments()[0];
        is_error_like(program, c, type_argument)
            || is_readonly_error_like(program, c, type_argument)
    })
}

/// builtin_symbol_likes.go IsReadonlyTypeLike.
pub fn is_readonly_type_like(
    program: &Program,
    c: &mut Checker,
    t: P<Type>,
    predicate: &mut dyn FnMut(&mut Checker, P<Type>) -> bool,
) -> bool {
    is_builtin_type_alias_like(program, c, t, &mut |c, subtype| {
        subtype.alias().unwrap().symbol().unwrap().name() == "Readonly" && predicate(c, subtype)
    })
}

/// builtin_symbol_likes.go IsBuiltinTypeAliasLike.
pub fn is_builtin_type_alias_like(
    program: &Program,
    c: &mut Checker,
    t: P<Type>,
    predicate: &mut dyn FnMut(&mut Checker, P<Type>) -> bool,
) -> bool {
    is_builtin_symbol_like_recurser(program, c, t, &mut |c, subtype| {
        let Some(alias) = subtype.alias() else {
            return BuiltinMatch::False;
        };
        if alias.type_arguments().is_empty() {
            return BuiltinMatch::False;
        }
        if is_symbol_from_default_library(program, alias.symbol()) && predicate(c, subtype) {
            return BuiltinMatch::True;
        }
        BuiltinMatch::Unknown
    })
}

/// builtin_symbol_likes.go IsAnyBuiltinSymbolLike.
pub fn is_any_builtin_symbol_like(program: &Program, c: &mut Checker, t: P<Type>) -> bool {
    is_builtin_symbol_like_recurser(program, c, t, &mut |_c, sub| {
        let Some(symbol) = sub.symbol() else {
            return BuiltinMatch::False;
        };
        if is_symbol_from_default_library(program, Some(symbol)) {
            BuiltinMatch::True
        } else {
            BuiltinMatch::Unknown
        }
    })
}
