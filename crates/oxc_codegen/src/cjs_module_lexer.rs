//! Workarounds for [cjs-module-lexer].
//!
//! `cjs-module-lexer` is the heuristic parser Node uses (under
//! `--experimental-require-module` and on `require()` of an ESM graph) to
//! discover the named exports of a CommonJS module without executing it. It
//! does **not** run a real JS parser — it pattern-matches a fixed set of
//! syntactic shapes that bundlers (Babel, TypeScript, Rollup) emit when
//! lowering ESM to CJS. If our minifier rewrites any of those shapes in a way
//! the heuristic doesn't recognise, Node silently drops the affected exports
//! and downstream `import { x } from "cjs-pkg"` fails at runtime.
//!
//! In practice every breakage we've hit comes from the same root cause:
//! `print_string_literal`'s `allow_backtick: true` tie-breaker in
//! `calculate_quote_maybe_backtick` prefers backtick on equal cost, turning
//! `"default"` into `` `default` ``. The lexer's pattern matcher accepts only
//! plain string literals at the positions below, so we hand-print those
//! positions with `allow_backtick: false`.
//!
//! The four positions we currently preserve, all `minify`-only:
//!
//! | Helper                            | Pattern                                                |
//! | --------------------------------- | ------------------------------------------------------ |
//! | [`try_print_require_call`]         | `require("…")`                                         |
//! | [`try_print_define_property_call`] | `Object.defineProperty(_, "name", …)` / `Reflect.…`     |
//! | [`try_print_exports_computed_target`] | `exports[STR] = …` / `module.exports[STR] = …`      |
//! | [`try_print_equality_string`]      | `key === "default"` / `key === "__esModule"` (and `==`/`!=`/`!==`) |
//!
//! Each helper returns `true` if it printed the construct, so the caller can
//! fall back to the default print path when the helper declines.
//!
//! Outside `minify`, `print_string_literal` already uses `self.quote` (never
//! backtick), so these helpers no-op — see <https://github.com/oxc-project/oxc/issues/22342>.
//!
//! [cjs-module-lexer]: https://github.com/nodejs/cjs-module-lexer

use oxc_ast::ast::{Argument, AssignmentTarget, CallExpression, Expression};
use oxc_syntax::precedence::Precedence;

use crate::{
    Codegen, Context,
    binary_expr_visitor::BinaryishOperator,
    r#gen::{Gen, GenExpr},
};

/// Print `require("...")` with a plain string literal so `cjs-module-lexer`
/// can detect re-export sources.
///
/// Skips the argument comments that `print_arguments` would preserve.
pub fn try_print_require_call(p: &mut Codegen<'_>, call: &CallExpression<'_>) -> bool {
    if !p.options.minify {
        return false;
    }
    let Some(str_lit) = call.common_js_require() else {
        return false;
    };
    p.print_ascii_byte(b'(');
    p.print_string_literal(str_lit, false);
    p.add_source_mapping_end(call.span);
    p.print_ascii_byte(b')');
    true
}

/// Print `Object.defineProperty(_, "name", ...)` / `Reflect.defineProperty(...)`
/// with the property-name argument as a plain string literal so
/// `cjs-module-lexer` can detect the export.
///
/// Skips the argument comments that `print_arguments` would preserve.
pub fn try_print_define_property_call(
    p: &mut Codegen<'_>,
    call: &CallExpression<'_>,
    ctx: Context,
) -> bool {
    if !p.options.minify {
        return false;
    }
    let Some(Argument::StringLiteral(name)) = call.arguments.get(1) else {
        return false;
    };
    if !call.callee.is_specific_member_access("Object", "defineProperty")
        && !call.callee.is_specific_member_access("Reflect", "defineProperty")
    {
        return false;
    }
    p.print_ascii_byte(b'(');
    for (i, arg) in call.arguments.iter().enumerate() {
        if i != 0 {
            p.print_comma();
            p.print_soft_space();
        }
        if i == 1 {
            p.print_string_literal(name, false);
        } else {
            arg.print(p, ctx);
        }
    }
    p.add_source_mapping_end(call.span);
    p.print_ascii_byte(b')');
    true
}

/// Print `exports[STR] = …` / `module.exports[STR] = …`'s LHS with the
/// computed key as a plain string literal so `cjs-module-lexer` can detect
/// the export.
pub fn try_print_exports_computed_target(
    p: &mut Codegen<'_>,
    target: &AssignmentTarget<'_>,
    ctx: Context,
) -> bool {
    if !p.options.minify {
        return false;
    }
    let AssignmentTarget::ComputedMemberExpression(member) = target else {
        return false;
    };
    let Expression::StringLiteral(key) = &member.expression else {
        return false;
    };
    if !member.object.is_specific_id("exports")
        && !member.object.is_specific_member_access("module", "exports")
    {
        return false;
    }
    member.object.print_expr(p, Precedence::Postfix, ctx.intersection(Context::FORBID_CALL));
    if member.optional {
        p.print_str("?.");
    }
    p.print_ascii_byte(b'[');
    p.print_string_literal(key, false);
    p.print_ascii_byte(b']');
    true
}

/// Print `"default"` / `"__esModule"` as a plain string literal when they
/// appear as an operand of an equality comparison, so `cjs-module-lexer` can
/// recognise the re-export filter emitted by Babel / TypeScript / Rollup:
///
/// ```js
/// Object.keys(mod).forEach(function (key) {
///   if (key === "default" || key === "__esModule") return;
///   ...
/// });
/// ```
#[inline]
pub fn try_print_equality_string(
    p: &mut Codegen,
    operator: BinaryishOperator,
    operand: &Expression,
) -> bool {
    if !p.options.minify {
        return false;
    }
    let BinaryishOperator::Binary(op) = operator else {
        return false;
    };
    if !op.is_equality() {
        return false;
    }
    let Expression::StringLiteral(str_lit) = operand else {
        return false;
    };
    if !matches!(str_lit.value.as_str(), "default" | "__esModule") {
        return false;
    }
    p.print_string_literal(str_lit, false);
    true
}
