use oxc_ecmascript::side_effects::PropertyReadSideEffects;
use oxc_span::SourceType;

use crate::{
    CompressOptions, TreeShakeOptions, default_options, test, test_options_source_type,
    test_options_with_iterations, test_same, test_same_options, test_same_options_source_type,
};

#[test]
fn conflate_identifier_assignments() {
    test(
        "export let x, y; export function setVars(value) { x = value; y = value; }",
        "export let x, y; export function setVars(value) { y = x = value; }",
    );
    test(
        "export let x, y, z; export function setVars(value) { x = value, y = value, z = value; }",
        "export let x, y, z; export function setVars(value) { z = y = x = value; }",
    );
    test(
        "export let x, y, z; export function setVars(value) { y = x = value, z = value; }",
        "export let x, y, z; export function setVars(value) { z = y = x = value; }",
    );
    test(
        "export let x, y, z, w; export function setVars(value) { y = x = value, w = z = value; }",
        "export let x, y, z, w; export function setVars(value) { w = z = y = x = value; }",
    );
    test(
        "export let x, y; export function setVars(value) { return (x = value, y = value); }",
        "export let x, y; export function setVars(value) { return y = x = value; }",
    );
    test(
        "export let x, y; export function setVars() { x = 1 + 2, y = 1 + 2; }",
        "export let x, y; export function setVars() { y = x = 3; }",
    );
    test(
        "export let x, y, z, w; export function f(value) { x = value, y = value, sideEffect(), z = value, w = value; }",
        "export let x, y, z, w; export function f(value) { y = x = value, sideEffect(), w = z = value; }",
    );
    test(
        "export let x, y; export function f(value) { x = typeof value, y = typeof value; }",
        "export let x, y; export function f(value) { y = x = typeof value; }",
    );
    test(
        "export let x, y; export function f(value) { x = typeof value == 'string', y = typeof value == 'string'; }",
        "export let x, y; export function f(value) { y = x = typeof value == 'string'; }",
    );
    test(
        "export let x, y; export const value = {}; export function f() { x = value === null, y = value === null; }",
        "export let x, y; export const value = {}; export function f() { y = x = value === null; }",
    );
}

#[test]
fn statement_fusion_does_not_add_an_iteration() {
    test_options_with_iterations(
        "export let x, y; export function setVars(value) { x = value; y = value; }",
        "export let x, y; export function setVars(value) { y = x = value; }",
        1,
        &CompressOptions::smallest(),
    );
}

#[test]
fn late_statement_fusion_is_idempotent() {
    test(
        "export let a, b, c, d; export function f() { a = 0; b = 0; for (c = 0, d = 1; c < d; c++); }",
        "export let a, b, c, d; export function f() { for (c = b = a = 0, d = 1; c < d; c++); }",
    );
}

#[test]
fn conflate_static_member_assignments() {
    test(
        "export const obj = {}; export function setProps(value) { obj.x = value; obj.y = value; }",
        "export const obj = {}; export function setProps(value) { obj.y = obj.x = value; }",
    );
    test(
        "export const obj = {}; export function setProps(value) { obj.x = value, obj.y = value, obj.z = value; }",
        "export const obj = {}; export function setProps(value) { obj.z = obj.y = obj.x = value; }",
    );
    test(
        "export function setProps(value) { this.x = value, this.y = value; }",
        "export function setProps(value) { this.y = this.x = value; }",
    );
}

#[test]
fn do_not_conflate_unsafe_assignments() {
    // The assignment targets must be resolved bindings.
    test_same("function setVars(value) { x = value, y = value; }");

    // Only plain assignments with structurally equal right-hand sides qualify.
    test_same("export let x, y; export function f(value) { x += value, y = value; }");
    test_same("export let x, y; export function f(a, b) { x = a, y = b; }");

    // Re-evaluation must not allocate a distinct value or run user code.
    test_same("export let x, y; export function f() { x = {}, y = {}; }");
    test_same("export let x, y; export function f() { x = value(), y = value(); }");
    test_same(
        "export let x, y; export const value = { n: 0, valueOf() { return ++this.n; } }; export function f() { x = value == 1, y = value == 1; }",
    );

    // A mutable RHS binding could change between the original evaluations.
    test_same(
        "export let x, y, value; export function setValue(v) { value = v; } export function f() { x = value, y = value; }",
    );

    // Static member targets must use the same stable object binding.
    test_same(
        "export let obj = {}, other = {}; export function f(value) { obj.x = value, other.y = value; }",
    );
    test_same(
        "export let obj = {}; export function replace(value) { obj = value; } export function f(value) { obj.x = value, obj.y = value; }",
    );
    test_same(
        "export const obj = {}; export let x, y; export function f(value) { x = obj.a = value, y = value; }",
    );
}

#[test]
fn setter_cannot_change_repeated_rhs() {
    test_same(
        "export const obj = { set x(_) { value = 2; } }; export let value = 1; export function f() { obj.x = value, obj.y = value; }",
    );
    test_same(
        "export const obj = { set x(_) { eval('value = 2'); } }; export let value = 1; export function f() { obj.x = value, obj.y = value; }",
    );

    // Sloppy-mode parameters can be rebound through the mapped `arguments` object.
    let options = default_options();
    test_options_source_type(
        "function f(value) { const args = arguments, obj = { set x(_) { args[0] = 2; } }; obj.x = value, obj.y = value; }",
        "function f(value) { let args = arguments, obj = { set x(_) { args[0] = 2; } }; obj.x = value, obj.y = value; }",
        SourceType::script(),
        &options,
    );
    test_same_options_source_type(
        "function f(obj) { obj.x = 1, obj.y = 1; }",
        SourceType::script(),
        &options,
    );
}

#[test]
fn property_reads_are_not_repeatable() {
    let options = CompressOptions {
        treeshake: TreeShakeOptions {
            property_read_side_effects: PropertyReadSideEffects::None,
            ..TreeShakeOptions::default()
        },
        ..default_options()
    };
    test_same_options(
        "export const box = {}, obj = {}; export function f() { obj.x = box.value === 1, obj.y = box.value === 1; }",
        &options,
    );
}
