// Ports of the internal/utils helpers the implemented rules use (ts_api_utils.go, ts_eslint.go,
// builtin_symbol_likes.go, utils.go).

// New helpers go into the submodule named after the Go file they port (internal/utils/<file>.go;
// go_utils.rs is utils.go). Everything is re-exported, so callers use crate::utils::name. The helpers
// above the submodules predate the split; check them before porting a helper again.
mod base_type_utils;
pub use base_type_utils::*;
mod bfs;
#[expect(unused_imports, reason = "bfs.rs has no helpers yet")]
pub use bfs::*;
mod builtin_symbol_likes;
pub use builtin_symbol_likes::*;
mod diagnostic_helpers;
#[expect(unused_imports, reason = "diagnostic_helpers.rs has no helpers yet")]
pub use diagnostic_helpers::*;
mod map;
#[expect(unused_imports, reason = "map.rs has no helpers yet")]
pub use map::*;
mod set;
#[expect(unused_imports, reason = "set.rs has no helpers yet")]
pub use set::*;
mod string_literal;
pub use string_literal::*;
mod ts_api_utils;
pub use ts_api_utils::*;
mod ts_eslint;
pub use ts_eslint::*;
mod type_matches_specifier;
pub use type_matches_specifier::*;
mod go_utils;
pub use go_utils::*;

use tsrs_ast::{self as ast, Kind, Node, SourceFile, Symbol, SymbolFlags};
use tsrs_checker::{Checker, Signature, SignatureKind, Type, TypeFlags};
use tsrs_compiler::Program;
use tsrs_core::CompilerOptions;
use tsrs_core::{P, Tristate, tspath};

/// utils.TrimNodeTextRange: the node's range without leading trivia.
pub fn trim_node_text_range(file: P<SourceFile>, node: P<Node>) -> (i32, i32) {
    let r = tsrs_scanner::get_range_of_token_at_position(file, node.pos());
    (r.pos(), node.end())
}

pub fn is_type_flag_set(t: P<Type>, flags: TypeFlags) -> bool {
    t.flags().intersects(flags)
}
pub fn is_union_type(t: P<Type>) -> bool {
    is_type_flag_set(t, TypeFlags::Union)
}
pub fn is_intersection_type(t: P<Type>) -> bool {
    is_type_flag_set(t, TypeFlags::Intersection)
}
pub fn is_type_parameter(t: P<Type>) -> bool {
    is_type_flag_set(t, TypeFlags::TypeParameter)
}
pub fn is_type_any_type(t: P<Type>) -> bool {
    is_type_flag_set(t, TypeFlags::Any)
}
pub fn is_type_unknown_type(t: P<Type>) -> bool {
    is_type_flag_set(t, TypeFlags::Unknown)
}

pub fn union_type_parts(t: P<Type>) -> Vec<P<Type>> {
    if is_union_type(t) { t.types().to_vec() } else { vec![t] }
}
pub fn intersection_type_parts(t: P<Type>) -> Vec<P<Type>> {
    if is_intersection_type(t) { t.types().to_vec() } else { vec![t] }
}

pub fn get_call_signatures(c: &mut Checker, t: P<Type>) -> &'static [P<Signature>] {
    c.get_signatures_of_type(t, SignatureKind::Call)
}

pub fn is_symbol_flag_set(symbol: Option<P<Symbol>>, flag: SymbolFlags) -> bool {
    symbol.is_some_and(|s| s.flags.get().intersects(flag))
}

pub fn is_strict_compiler_option_enabled(options: &CompilerOptions, option: Tristate) -> bool {
    options.get_strict_option_value(option)
}

/// ts_api_utils.go IsCallback.
pub fn is_callback(c: &mut Checker, param: P<Symbol>, node: P<Node>) -> bool {
    let at = c.get_type_of_symbol_at_location(param, Some(node)).unwrap();
    let mut t = Some(c.get_apparent_type(at));
    if let Some(decl) = param.value_declaration() {
        if ast::is_parameter_declaration(decl)
            && decl.as_parameter_declaration().dot_dot_dot_token().is_some()
        {
            let number_type = c.number_type;
            t = c.get_index_type_of_type(t.unwrap(), number_type);
        }
    }
    let Some(t) = t else { return false };
    for sub in union_type_parts(t) {
        if !get_call_signatures(c, sub).is_empty() {
            return true;
        }
    }
    false
}

/// ts_api_utils.go IsThenableType.
pub fn is_thenable_type(c: &mut Checker, node: P<Node>, t: Option<P<Type>>) -> bool {
    let t = t.unwrap_or_else(|| c.get_type_at_location(node));
    let apparent = c.get_apparent_type(t);
    for part in union_type_parts(apparent) {
        let Some(then) = c.get_property_of_type(part, "then") else {
            continue;
        };
        let then_type = c.get_type_of_symbol_at_location(then, Some(node)).unwrap();
        for sub in union_type_parts(then_type) {
            for &sig in c.get_signatures_of_type(sub, SignatureKind::Call) {
                let params = sig.parameters.get();
                if !params.is_empty() && is_callback(c, params[0], node) {
                    return true;
                }
            }
        }
    }
    false
}

/// ts_eslint.go GetConstraintInfo: (constraint, is_type_parameter).
pub fn get_constraint_info(c: &mut Checker, t: P<Type>) -> (Option<P<Type>>, bool) {
    if is_type_parameter(t) {
        return (c.get_base_constraint_of_type(t), true);
    }
    (Some(t), false)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TypeAwaitable {
    Always,
    Never,
    May,
}

/// ts_eslint.go NeedsToBeAwaited.
pub fn needs_to_be_awaited(c: &mut Checker, node: P<Node>, t: P<Type>) -> TypeAwaitable {
    let (constraint, is_type_parameter) = get_constraint_info(c, t);
    let Some(constraint) = constraint else {
        return if is_type_parameter { TypeAwaitable::May } else { TypeAwaitable::Never };
    };
    if is_type_any_type(constraint) || is_type_unknown_type(constraint) {
        return TypeAwaitable::May;
    }
    if is_thenable_type(c, node, Some(constraint)) {
        return TypeAwaitable::Always;
    }
    TypeAwaitable::Never
}

/// ts_eslint.go GetConstrainedTypeAtLocation.
pub fn get_constrained_type_at_location(c: &mut Checker, node: P<Node>) -> P<Type> {
    let t = c.get_type_at_location(node);
    c.get_base_constraint_of_type(t).unwrap_or(t)
}

pub fn is_higher_precedence_than_await(node: P<Node>) -> bool {
    let node_precedence = ast::get_expression_precedence(node);
    let await_precedence = ast::get_operator_precedence(
        Kind::AwaitExpression,
        Kind::Unknown,
        ast::OperatorPrecedenceFlags::None,
    );
    (node_precedence as i32) > (await_precedence as i32)
}

pub fn is_strong_precedence_node(inner: P<Node>) -> bool {
    ast::is_literal_kind(inner.kind())
        || ast::is_boolean_literal(inner)
        || ast::is_parenthesized_expression(inner)
        || matches!(
            inner.kind(),
            Kind::Identifier
                | Kind::TypeReference
                | Kind::TypeOperator
                | Kind::ArrayLiteralExpression
                | Kind::ObjectLiteralExpression
                | Kind::PropertyAccessExpression
                | Kind::ElementAccessExpression
                | Kind::CallExpression
                | Kind::NewExpression
                | Kind::TaggedTemplateExpression
                | Kind::ExpressionWithTypeArguments
        )
}

/// utils.go FindModifier.
pub fn find_modifier(node: P<Node>, modifier: Kind) -> Option<P<Node>> {
    node.modifier_nodes().iter().copied().find(|m| m.kind() == modifier)
}
pub fn includes_modifier(node: P<Node>, modifier: Kind) -> bool {
    find_modifier(node, modifier).is_some()
}

// ---- builtin_symbol_likes.go ----

pub fn is_source_file_default_library(program: &Program, file: P<SourceFile>) -> bool {
    if !file.is_declaration_file() {
        return false;
    }
    if program.is_source_file_default_library(file.path()) {
        return true;
    }
    let options = program.options();
    if options.no_lib.is_true() {
        return false;
    }
    let lib_dir = program.host().default_library_path().to_string();
    let libs: Vec<String> = match &options.lib {
        None => vec![tspath::combine_paths(
            &lib_dir,
            &[tsrs_tsoptions::get_default_lib_file_name(&options)],
        )],
        Some(lib) => lib
            .iter()
            .filter_map(|l| tsrs_tsoptions::get_lib_file_name(l))
            .map(|name| tspath::combine_paths(&lib_dir, &[&name]))
            .collect(),
    };
    let opts = tspath::ComparePathsOptions {
        current_directory: program.host().get_current_directory().to_string(),
        use_case_sensitive_file_names: program.host().fs().use_case_sensitive_file_names(),
    };
    libs.iter().any(|lib| tspath::compare_paths(file.file_name(), lib, &opts) == 0)
}

pub fn is_symbol_from_default_library(program: &Program, symbol: Option<P<Symbol>>) -> bool {
    let Some(symbol) = symbol else { return false };
    symbol.declarations().iter().any(|&d| {
        ast::get_source_file_of_node(d)
            .is_some_and(|sf| is_source_file_default_library(program, sf))
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BuiltinMatch {
    Unknown,
    False,
    True,
}

#[expect(
    clippy::only_used_in_recursion,
    reason = "Go signature: IsBuiltinSymbolLikeRecurser passes program through the recursion"
)]
pub fn is_builtin_symbol_like_recurser(
    program: &Program,
    c: &mut Checker,
    t: P<Type>,
    predicate: &mut dyn FnMut(&mut Checker, P<Type>) -> BuiltinMatch,
) -> bool {
    if is_intersection_type(t) {
        return intersection_type_parts(t)
            .into_iter()
            .any(|t| is_builtin_symbol_like_recurser(program, c, t, predicate));
    }
    if is_union_type(t) {
        return union_type_parts(t)
            .into_iter()
            .all(|t| is_builtin_symbol_like_recurser(program, c, t, predicate));
    }
    if is_type_parameter(t) {
        return match c.get_base_constraint_of_type(t) {
            Some(constraint) => is_builtin_symbol_like_recurser(program, c, constraint, predicate),
            None => false,
        };
    }
    match predicate(c, t) {
        BuiltinMatch::True => return true,
        BuiltinMatch::False => return false,
        BuiltinMatch::Unknown => {}
    }
    if let Some(symbol) = t.symbol() {
        if symbol.flags.get().intersects(SymbolFlags::Class | SymbolFlags::Interface) {
            let declared = c.get_declared_type_of_symbol(symbol);
            for &base in c.get_base_types(declared) {
                if is_builtin_symbol_like_recurser(program, c, base, predicate) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn is_builtin_symbol_like(
    program: &Program,
    c: &mut Checker,
    t: P<Type>,
    names: &[&str],
) -> bool {
    is_builtin_symbol_like_recurser(program, c, t, &mut |_c, sub| {
        let Some(symbol) = sub.symbol() else {
            return BuiltinMatch::False;
        };
        if names.contains(&symbol.name()) && is_symbol_from_default_library(program, Some(symbol)) {
            BuiltinMatch::True
        } else {
            BuiltinMatch::Unknown
        }
    })
}

pub fn is_promise_like(program: &Program, c: &mut Checker, t: P<Type>) -> bool {
    is_builtin_symbol_like(program, c, t, &["Promise"])
}

thread_local! {
    /// Type strings printed for the file being linted: several rules print the same types (labels and messages of
    /// no-unsafe-*), and printing is one of the costlier checker calls. Cleared per file by the linter.
    static TYPE_STRINGS: std::cell::RefCell<rustc_hash::FxHashMap<P<Type>, String>> = Default::default();
}

pub fn clear_type_strings() {
    TYPE_STRINGS.with(|m| m.borrow_mut().clear());
}

/// `checker.TypeToString(t)`, memoized per file.
pub fn type_to_string(c: &mut Checker, t: P<Type>) -> String {
    if let Some(s) = TYPE_STRINGS.with(|m| m.borrow().get(&t).cloned()) {
        return s;
    }
    let s = c.type_to_string_exported(t);
    TYPE_STRINGS.with(|m| m.borrow_mut().insert(t, s.clone()));
    s
}
