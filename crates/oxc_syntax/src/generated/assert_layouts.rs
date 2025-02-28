// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`

#![allow(unused_imports)]

use std::mem::{align_of, offset_of, size_of};

use nonmax::NonMaxU32;

use crate::{number::*, operator::*, reference::*, scope::*, symbol::*};

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(size_of::<NonMaxU32>() == 4);
    assert!(align_of::<NonMaxU32>() == 4);

    assert!(size_of::<NumberBase>() == 1);
    assert!(align_of::<NumberBase>() == 1);

    assert!(size_of::<BigintBase>() == 1);
    assert!(align_of::<BigintBase>() == 1);

    assert!(size_of::<AssignmentOperator>() == 1);
    assert!(align_of::<AssignmentOperator>() == 1);

    assert!(size_of::<BinaryOperator>() == 1);
    assert!(align_of::<BinaryOperator>() == 1);

    assert!(size_of::<LogicalOperator>() == 1);
    assert!(align_of::<LogicalOperator>() == 1);

    assert!(size_of::<UnaryOperator>() == 1);
    assert!(align_of::<UnaryOperator>() == 1);

    assert!(size_of::<UpdateOperator>() == 1);
    assert!(align_of::<UpdateOperator>() == 1);

    assert!(size_of::<ScopeId>() == 4);
    assert!(align_of::<ScopeId>() == 4);

    assert!(size_of::<SymbolId>() == 4);
    assert!(align_of::<SymbolId>() == 4);

    assert!(size_of::<ReferenceId>() == 4);
    assert!(align_of::<ReferenceId>() == 4);
};

#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(size_of::<NonMaxU32>() == 4);
    assert!(align_of::<NonMaxU32>() == 4);

    assert!(size_of::<NumberBase>() == 1);
    assert!(align_of::<NumberBase>() == 1);

    assert!(size_of::<BigintBase>() == 1);
    assert!(align_of::<BigintBase>() == 1);

    assert!(size_of::<AssignmentOperator>() == 1);
    assert!(align_of::<AssignmentOperator>() == 1);

    assert!(size_of::<BinaryOperator>() == 1);
    assert!(align_of::<BinaryOperator>() == 1);

    assert!(size_of::<LogicalOperator>() == 1);
    assert!(align_of::<LogicalOperator>() == 1);

    assert!(size_of::<UnaryOperator>() == 1);
    assert!(align_of::<UnaryOperator>() == 1);

    assert!(size_of::<UpdateOperator>() == 1);
    assert!(align_of::<UpdateOperator>() == 1);

    assert!(size_of::<ScopeId>() == 4);
    assert!(align_of::<ScopeId>() == 4);

    assert!(size_of::<SymbolId>() == 4);
    assert!(align_of::<SymbolId>() == 4);

    assert!(size_of::<ReferenceId>() == 4);
    assert!(align_of::<ReferenceId>() == 4);
};

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
