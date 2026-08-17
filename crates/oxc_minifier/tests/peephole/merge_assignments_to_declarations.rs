use oxc_span::SourceType;

use crate::{default_options, test, test_options_source_type, test_same};

#[test]
fn merge_assignments_to_declarations_var() {
    test("var a; a = 0", "var a = 0");
    test_same("var a = 0; a = 1"); // this can be improved to `var a = 1`
    test_same("var a = 0; a = b()"); // `b()` may access `a`
    test_same("var a = b(); a = c()"); // `c()` may access `a`
    test_same("var a, b = 1; a = 0"); // this can be improved to `var a = 0, b = 1`
    test_same("var a, b = c(); a = 0"); // `c()` may access `a`
    test("var a, b; a = 0", "var a = 0, b");
    test("var a, b; a = 0, b = 1", "var a = 0, b = 1");
    test("var a, b; a = 0; b = 1", "var a = 0, b = 1");
    test("var a, b; a = c()", "var a = c(), b");
    test("var a, b; a = c(), b = d()", "var a = c(), b = d()");
    test("var a, b; a = c(); b = d()", "var a = c(), b = d()");
    test("var a, b; a = b", "var a = b, b");

    test("var a, b, c; a = 0, b = 1, c = 2", "var a = 0, b = 1, c = 2");
    test("var a, b; a = 0, b = 1, foo()", "var a = 0, b = 1; foo()");
    test("var a; a = 0, foo(), bar()", "var a = 0; foo(), bar()");
    test_same("var a, b; foo(), bar()");
}

#[test]
fn merge_assignments_to_declarations_let() {
    test("let a; a = 0", "let a = 0");
    test_same("let a = 0; a = 1"); // this can be improved to `let a = 1`
    test_same("let a = 0; a = b()"); // `b()` may access `a`
    test_same("let a = b(); a = c()"); // `c()` may access `a`
    test_same("let a, b = 1; a = 0"); // this can be improved to `let a = 0, b = 1`
    test_same("let a, b = c(); a = 0"); // `c()` may access `a`
    test_same("let a, b; a = 0"); // this can be improved to `let a = 0, b`
    test("let a, b; a = 0; b = 1", "let a, b; a = 0, b = 1"); // this can be improved to `let a = 0, b = 1`
    test_same("let a, b; a = c()"); // `c()` may access `b`, `let a = c(), b` will cause TDZ error
    test("let a, b; a = c(); b = d()", "let a, b; a = c(), b = d()"); // same as above
    test_same("let a, b; a = b"); // `let a = b, b` will cause TDZ error; `b` reads as the implicit `undefined`, which is not worth inlining (rolldown#10174)
    test_same("let a; a = foo(a)"); // `let a = foo(a)` will cause TDZ error
    test("let a; a = (() => a)()", "let a; a = a;"); // `let a = (() => a)()` will cause TDZ error
    test("let a; a = () => a", "let a = () => a");

    // the initializer does not read `a`, so there is no TDZ error to introduce
    // https://github.com/oxc-project/oxc/issues/14310
    test(
        "export function f() { let a; a = c(); foo(a) }",
        "export function f() { let a = c(); foo(a) }",
    );
    test(
        "export function f() { let a; a = c(b); foo(a) }",
        "export function f() { let a = c(b); foo(a) }",
    );
    test(
        "export function f() { let a; a = b.c; foo(a) }",
        "export function f() { let a = b.c; foo(a) }",
    );
    // the initializer reads `a`, which `let a = <init>` would evaluate in its TDZ
    test(
        "export function f() { let a; a = c(a); foo(a) }",
        "export function f() { let a; a = c(a), foo(a) }",
    );
    test(
        "export function f() { let a; a = { b: a }; foo(a) }",
        "export function f() { let a; a = { b: a }, foo(a) }",
    );
    test(
        "export function f() { let a; a = (a = b()); foo(a) }",
        "export function f() { let a; a = a = b(), foo(a) }",
    );
    test(
        "export function f() { let a; a = class extends a {}; foo(a) }",
        "export function f() { let a; a = class extends a {}, foo(a) }",
    );

    // `c` closes over `a`, so calling it during `a`'s initialization would read
    // `a` in its TDZ, even though the initializer does not mention `a`.
    test(
        "export function f() { function c() { return a } let a; a = c(); foo(a) }",
        "export function f() { function c() { return a } let a; a = c(), foo(a) }",
    );
    test(
        "export function f() { let a; function c() { return a } a = c(); foo(a) }",
        "export function f() { let a; function c() { return a } a = c(), foo(a) }",
    );
    test(
        "export function f() { let a; a = (() => { const d = () => a; return d() })(); foo(a) }",
        "export function f() { let a; a = a, foo(a) }",
    );
}

/// Reads of the binding that are deferred past the initializer, or that this
/// analysis cannot place, must block the merge.
#[test]
fn merge_assignments_to_declarations_let_deferred_reads() {
    // A class field initializer runs at construction time, so `new D()` reads `a`
    // while it is still uninitialized. The reference sits in the class scope, which
    // carries no flag distinguishing it from a plain block.
    test(
        "export function f() { class D { x = a } let a; a = new D(); console.log(a.x) }",
        "export function f() { class D { x = a } let a; a = new D(), console.log(a.x) }",
    );
    test(
        "export function f() { const D = class { x = a }; let a; a = new D(); console.log(a.x) }",
        "export function f() { let D = class { x = a }, a; a = new D(), console.log(a.x) }",
    );

    // A read in a nested block executes inline and would be safe to merge past, but
    // it is not in the binding's scope, so the conservative rule rejects it.
    //
    // NOTE: the block here holds a `let` so it survives minification. A block that
    // gets flattened (`if (x) { foo(a) }`) makes this transform iteration-dependent:
    // the flattening pass removes the block but leaves the reference's `scope_id`
    // pointing at it, so the merge is rejected on one pass and accepted on the next.
    // The staleness only ever reports *more* scopes than exist, so it errs safe.
    test(
        "export function f() { let a; a = c(); { let b = a; foo(b) } }",
        "export function f() { let a; a = c(); { let b = a; foo(b) } }",
    );

    // A direct `eval` can read the binding without producing a resolved reference.
    test(
        "export function f() { let a; a = c(); eval('foo'); return a }",
        "export function f() { let a; return a = c(), eval('foo'), a }",
    );
}

/// A program-scope lexical is reachable from code the reference scan cannot see,
/// so the initializer must not be moved into the declaration.
#[test]
fn merge_assignments_to_declarations_let_program_scope() {
    // A cyclic import can call back into this module and read the exported `a`,
    // which is initialized (`undefined`) before the assignment but in its TDZ
    // once the initializer moves into the declaration.
    test_same("import { f } from './b.mjs'; let a; a = f(); export { a }");
    test_same("import { f } from './b.mjs'; let a; a = f(); export default () => a");

    // A Script root's `let` joins the global lexical environment, so a function
    // from another script can read it while this initializer runs.
    test_options_source_type(
        "let a; a = f(); console.log(a)",
        "let a; a = f(), console.log(a)",
        SourceType::cjs(),
        &default_options(),
    );

    // the untouched `is_literal_value` branch still merges at program scope: a
    // literal initializer executes no code, so nothing can observe the binding
    test("let a; a = 0; foo(a)", "let a = 0; foo(0)");
}

#[test]
fn merge_assignments_to_declarations_other() {
    test_same("const a = 0; a = 1");
    test_same("using a = 0; a = 1");
    test_same("await using a = 0; a = 1");
}

/// A definite assignment assertion cannot coexist with an initializer (TS1263),
/// so the merge has to drop the `!` while keeping the type annotation. Covers all
/// three branches that reach the merge: `var`, a literal RHS, and the lexical
/// safety analysis.
#[test]
fn merge_assignments_to_declarations_drops_definite_assertion() {
    test_options_source_type(
        "export function f() { var a!: number; a = foo(); bar(a, a) }",
        "export function f() { var a: number = foo(); bar(a, a) }",
        SourceType::ts(),
        &default_options(),
    );
    test_options_source_type(
        "export function f() { let a!: number; a = 0; bar(a, a) }",
        "export function f() { let a: number = 0; bar(0, 0) }",
        SourceType::ts(),
        &default_options(),
    );
    test_options_source_type(
        "export function f() { let a!: number; a = foo(); bar(a, a) }",
        "export function f() { let a: number = foo(); bar(a, a) }",
        SourceType::ts(),
        &default_options(),
    );
}
