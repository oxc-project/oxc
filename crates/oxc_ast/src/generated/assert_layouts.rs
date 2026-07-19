// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`.

// PROTOTYPE: The original layout assertions have been removed because converting `Expression`
// and `MemberExpression` to 8-byte tagged-pointer structs changes the size and field offsets of
// every type containing them. Only the assertions for the new tagged types are kept.
// Do NOT re-run `just ast` - it would overwrite this file.

#![allow(unused_imports)]

use std::mem::{align_of, offset_of, size_of};

use crate::ast::*;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(size_of::<Expression>() == 8);
    assert!(align_of::<Expression>() == 8);
    assert!(size_of::<Option<Expression>>() == 8);

    assert!(size_of::<MemberExpression>() == 8);
    assert!(align_of::<MemberExpression>() == 8);
    assert!(size_of::<Option<MemberExpression>>() == 8);
};

#[cfg(not(target_pointer_width = "64"))]
const _: () = panic!("The tagged-pointer `Expression` prototype only supports 64-bit platforms");
