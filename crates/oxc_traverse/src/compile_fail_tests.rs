#![cfg(doctest)]

//! Tests to ensure lifetimes prevent references to escape visitor functions.
//! If they could, it'd allow aliasing, which would be undefined behavior.
//!
//! These tests were originally implemented using `trybuild` crate,
//! but it disproportionately hurt time taken to run tests.
//! So using `compile_fail` doc tests instead.
//! <https://github.com/oxc-project/oxc/issues/4537>

/**
```compile_fail
use oxc_ast::ast::IdentifierReference;
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

struct Trans<'a, 'b> {
    ancestor: Option<Ancestor<'a, 'b>>,
}

impl<'a, 'b> Traverse<'a> for Trans<'a, 'b> {
    fn enter_identifier_reference(
        &mut self,
        _node: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.ancestor = Some(ctx.parent());
    }
}
```
*/
const CANNOT_HOLD_ONTO_ANCESTOR: () = ();

/**
```compile_fail
use oxc_ast::ast::IdentifierReference;
use oxc_traverse::{ancestor::ProgramWithoutDirectives, Ancestor, Traverse, TraverseCtx};

struct Trans<'a, 'b> {
    program: Option<ProgramWithoutDirectives<'a, 'b>>,
}

impl<'a, 'b> Traverse<'a> for Trans<'a, 'b> {
    fn enter_identifier_reference(
        &mut self,
        _node: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Ancestor::ProgramDirectives(program) = ctx.parent() {
            self.program = Some(program);
        }
    }
}
```
*/
const CANNOT_HOLD_ONTO_ANCESTOR_NODE: () = ();

/**
```compile_fail
use oxc_ast::ast::{IdentifierReference, Statement};
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

struct Trans<'a, 'b> {
    stmt: Option<&'b Statement<'a>>,
}

impl<'a, 'b> Traverse<'a> for Trans<'a, 'b> {
    fn enter_identifier_reference(
        &mut self,
        _node: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Ancestor::ProgramDirectives(program) = ctx.parent() {
            let body = program.body();
            let stmt = &body[0];
            self.stmt = Some(stmt);
        }
    }
}
```
*/
const CANNOT_HOLD_ONTO_AST_NODE: () = ();

/**
```compile_fail
use oxc_ast::ast::IdentifierReference;
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

struct Trans<'a, 'b> {
    ancestor: Option<Ancestor<'a, 'b>>,
}

impl<'a, 'b> Traverse<'a> for Trans<'a, 'b> {
    fn enter_identifier_reference(
        &mut self,
        _node: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.ancestor = ctx.ancestors().next();
    }
}
```
*/
const CANNOT_HOLD_ONTO_ANCESTOR_FROM_ANCESTORS_ITERATOR: () = ();

/**
```compile_fail
use oxc_ast::ast::IdentifierReference;
use oxc_traverse::{ancestor::ProgramWithoutDirectives, Ancestor, Traverse, TraverseCtx};

struct Trans<'a, 'b> {
    program: Option<ProgramWithoutDirectives<'a, 'b>>,
}

impl<'a, 'b> Traverse<'a> for Trans<'a, 'b> {
    fn enter_identifier_reference(
        &mut self,
        _node: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let parent = ctx.ancestors().next().unwrap();
        if let Ancestor::ProgramDirectives(program) = parent {
            self.program = Some(program);
        }
    }
}
```
*/
const CANNOT_HOLD_ONTO_ANCESTOR_NODE_FROM_ANCESTORS_ITERATOR: () = ();

/**
```compile_fail
use oxc_ast::ast::{IdentifierReference, Statement};
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

struct Trans<'a, 'b> {
    stmt: Option<&'b Statement<'a>>,
}

impl<'a, 'b> Traverse<'a> for Trans<'a, 'b> {
    fn enter_identifier_reference(
        &mut self,
        _node: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let parent = ctx.ancestors().next().unwrap();
        if let Ancestor::ProgramDirectives(program) = parent {
            let body = program.body();
            let stmt = &body[0];
            self.stmt = Some(stmt);
        }
    }
}
```
*/
const CANNOT_HOLD_ONTO_AST_NODE_FROM_ANCESTORS_ITERATOR: () = ();
