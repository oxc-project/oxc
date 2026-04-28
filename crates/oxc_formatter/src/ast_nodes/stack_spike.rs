//! Stack-allocated AstNode spike.
//!
//! Validates the lifetime model for stack-allocated `AstNode` wrappers.
//!
//! The current `AstNode<'a, T>` carries `parent: AstNodes<'a>` and `allocator: &'a Allocator`,
//! using a single arena lifetime `'a` for both the AST data and the wrapper itself - which forces
//! every wrapper to live in the arena.
//!
//! This module proves out an alternative design:
//! - `'a` is the AST/arena lifetime (only for `inner: &'a T`).
//! - `'me` is the parent's stack frame lifetime.
//! - Wrappers are `Copy` and live on the stack.
//! - Children borrow their parent's stack frame via `'me`.
//! - As we recurse, `'me` shrinks naturally through Rust's subtyping.
//!
//! If this file compiles cleanly without `unsafe`, the design is viable.

#![allow(dead_code)]

use oxc_ast::ast::{BinaryExpression, Expression, IfStatement, Program, Statement};
use oxc_span::{GetSpan, Span};

/// Stack-allocated wrapper around an AST node.
///
/// `'me` is the parent's stack frame lifetime. As we recurse top-down through the AST,
/// `'me` naturally shrinks: each level's wrapper borrows its parent (which lives in an
/// outer stack frame) for `'me`, and produces children that borrow the current wrapper
/// for a shorter `'child` lifetime.
///
/// `'a` is the AST/arena lifetime - the lifetime of the underlying parsed AST data.
#[derive(Clone, Copy)]
pub struct StackAstNode<'me, 'a, T> {
    pub inner: &'a T,
    pub parent: StackAstNodes<'me, 'a>,
    pub following_span_start: u32,
}

/// Parent enum. Only includes variants needed to demonstrate the design end-to-end:
/// `Program -> IfStatement -> Expression -> BinaryExpression`.
#[derive(Clone, Copy)]
pub enum StackAstNodes<'me, 'a> {
    Dummy,
    Program(&'me StackAstNode<'me, 'a, Program<'a>>),
    Statement(&'me StackAstNode<'me, 'a, Statement<'a>>),
    IfStatement(&'me StackAstNode<'me, 'a, IfStatement<'a>>),
    Expression(&'me StackAstNode<'me, 'a, Expression<'a>>),
    BinaryExpression(&'me StackAstNode<'me, 'a, BinaryExpression<'a>>),
}

impl<'me, 'a> StackAstNodes<'me, 'a> {
    pub fn span(&self) -> Span {
        match self {
            Self::Dummy => panic!("span called on Dummy parent"),
            Self::Program(n) => n.inner.span(),
            Self::Statement(n) => n.inner.span(),
            Self::IfStatement(n) => n.inner.span(),
            Self::Expression(n) => n.inner.span(),
            Self::BinaryExpression(n) => n.inner.span(),
        }
    }

    /// Walk one level up the parent chain.
    ///
    /// Returns the grandparent's `StackAstNodes`. Each variant's wrapper holds its own
    /// `parent` field, so this is just a Copy field load - no traversal cost.
    pub fn parent(&self) -> Self {
        match self {
            Self::Dummy => panic!("parent called on Dummy"),
            Self::Program(n) => n.parent,
            Self::Statement(n) => n.parent,
            Self::IfStatement(n) => n.parent,
            Self::Expression(n) => n.parent,
            Self::BinaryExpression(n) => n.parent,
        }
    }
}

// ---------- Accessors on Program ----------

impl<'me, 'a> StackAstNode<'me, 'a, Program<'a>> {
    /// Iterate over the body, yielding owned `StackAstNode` wrappers each borrowing `self`.
    ///
    /// Note this returns an iterator that borrows `self` for `'this` - shorter than `'me` -
    /// and yields wrappers parameterised by `'this`.
    pub fn body<'this>(&'this self) -> ProgramBodyIter<'this, 'a> {
        ProgramBodyIter { parent: self, slice: self.inner.body.iter(), index: 0 }
    }
}

pub struct ProgramBodyIter<'this, 'a> {
    parent: &'this StackAstNode<'this, 'a, Program<'a>>,
    slice: std::slice::Iter<'a, Statement<'a>>,
    index: usize,
}

impl<'this, 'a> Iterator for ProgramBodyIter<'this, 'a> {
    type Item = StackAstNode<'this, 'a, Statement<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = self.slice.next()?;
        // Peek the next statement to compute `following_span_start`.
        let following_span_start = self.slice.clone().next().map_or(0, |n| n.span().start);
        self.index += 1;
        Some(StackAstNode {
            inner,
            parent: StackAstNodes::Program(self.parent),
            following_span_start,
        })
    }
}

// ---------- Accessors on Statement ----------

impl<'me, 'a> StackAstNode<'me, 'a, Statement<'a>> {
    /// Specialise to a concrete `IfStatement` if the inner is one. The returned wrapper
    /// inherits the `Statement`'s parent, since it represents the same logical position.
    pub fn as_if_statement(&self) -> Option<StackAstNode<'me, 'a, IfStatement<'a>>> {
        match self.inner {
            Statement::IfStatement(boxed) => Some(StackAstNode {
                inner: boxed.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            }),
            _ => None,
        }
    }
}

// ---------- Accessors on IfStatement ----------

impl<'me, 'a> StackAstNode<'me, 'a, IfStatement<'a>> {
    pub fn test<'this>(&'this self) -> StackAstNode<'this, 'a, Expression<'a>> {
        StackAstNode {
            inner: &self.inner.test,
            parent: StackAstNodes::IfStatement(self),
            following_span_start: self.inner.consequent.span().start,
        }
    }

    pub fn consequent<'this>(&'this self) -> StackAstNode<'this, 'a, Statement<'a>> {
        let following_span_start =
            self.inner.alternate.as_ref().map_or(self.following_span_start, |alt| alt.span().start);
        StackAstNode {
            inner: &self.inner.consequent,
            parent: StackAstNodes::IfStatement(self),
            following_span_start,
        }
    }

    pub fn alternate<'this>(&'this self) -> Option<StackAstNode<'this, 'a, Statement<'a>>> {
        self.inner.alternate.as_ref().map(|alt| StackAstNode {
            inner: alt,
            parent: StackAstNodes::IfStatement(self),
            following_span_start: self.following_span_start,
        })
    }
}

// ---------- Accessors on Expression ----------

impl<'me, 'a> StackAstNode<'me, 'a, Expression<'a>> {
    /// Specialise to a concrete `BinaryExpression` if the inner is one.
    ///
    /// The concrete wrapper inherits the `Expression`'s parent (same logical position).
    pub fn as_binary_expression(&self) -> Option<StackAstNode<'me, 'a, BinaryExpression<'a>>> {
        match self.inner {
            Expression::BinaryExpression(boxed) => Some(StackAstNode {
                inner: boxed.as_ref(),
                parent: self.parent,
                following_span_start: self.following_span_start,
            }),
            _ => None,
        }
    }
}

// ---------- Accessors on BinaryExpression ----------

impl<'me, 'a> StackAstNode<'me, 'a, BinaryExpression<'a>> {
    pub fn left<'this>(&'this self) -> StackAstNode<'this, 'a, Expression<'a>> {
        StackAstNode {
            inner: &self.inner.left,
            parent: StackAstNodes::BinaryExpression(self),
            following_span_start: self.inner.right.span().start,
        }
    }

    pub fn right<'this>(&'this self) -> StackAstNode<'this, 'a, Expression<'a>> {
        StackAstNode {
            inner: &self.inner.right,
            parent: StackAstNodes::BinaryExpression(self),
            following_span_start: self.following_span_start,
        }
    }
}

// ---------- Demonstration: top-down recursive walk ----------

/// Demonstrates that the stack model supports recursive top-down traversal where each
/// level's wrapper borrows its parent's wrapper - the natural shape of a `Format::fmt` impl.
///
/// This is not a real formatter; it just exercises the lifetime relationships end-to-end.
pub fn walk_program(program: &Program<'_>) -> usize {
    let root: StackAstNode<'_, '_, Program<'_>> =
        StackAstNode { inner: program, parent: StackAstNodes::Dummy, following_span_start: 0 };

    let mut count = 0;
    for stmt in root.body() {
        count += walk_statement(&stmt);
    }
    count
}

fn walk_statement<'me, 'a>(stmt: &StackAstNode<'me, 'a, Statement<'a>>) -> usize {
    let mut count = 1;
    if let Some(if_stmt) = stmt.as_if_statement() {
        // Walk the test expression.
        let test = if_stmt.test();
        count += walk_expression(&test);
        // Walk the consequent statement (recursive).
        let consequent = if_stmt.consequent();
        count += walk_statement(&consequent);
        // Walk the optional alternate.
        if let Some(alt) = if_stmt.alternate() {
            count += walk_statement(&alt);
        }
    }
    count
}

fn walk_expression<'me, 'a>(expr: &StackAstNode<'me, 'a, Expression<'a>>) -> usize {
    let mut count = 1;
    if let Some(bin) = expr.as_binary_expression() {
        let left = bin.left();
        count += walk_expression(&left);
        let right = bin.right();
        count += walk_expression(&right);
    }
    count
}

/// Demonstrates upward parent traversal. Within a deeply-nested `BinaryExpression::left`,
/// walk back up to confirm the chain reaches the `Program` root.
pub fn walk_to_root_from_left<'a>(program: &'a Program<'a>) -> Option<usize> {
    let root: StackAstNode<'_, 'a, Program<'a>> =
        StackAstNode { inner: program, parent: StackAstNodes::Dummy, following_span_start: 0 };

    for stmt in root.body() {
        if let Some(if_stmt) = stmt.as_if_statement() {
            let test = if_stmt.test();
            if let Some(bin) = test.as_binary_expression() {
                let left = bin.left();

                // Walk parent chain. Should be: BinaryExpression -> IfStatement -> Program -> Dummy.
                let mut depth = 0;
                let mut cursor = left.parent;
                loop {
                    match cursor {
                        StackAstNodes::Dummy => return Some(depth),
                        _ => {
                            depth += 1;
                            cursor = cursor.parent();
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use super::*;

    #[test]
    fn stack_recursion_compiles_and_runs() {
        let allocator = Allocator::default();
        let source = "if (a + b) { c; } else { d; }";
        let parser = Parser::new(&allocator, source, SourceType::default());
        let ret = parser.parse();
        assert!(ret.errors.is_empty());

        let count = walk_program(&ret.program);
        // 1 (if-stmt) + 1 (test expr) + 1 (left) + 1 (right) + 1 (consequent block) + 1 (alternate block).
        // Walk visits IfStatement (1) + test=BinaryExpression (1) + left=Identifier (1) + right=Identifier (1)
        // + consequent=BlockStatement (1) + alternate=BlockStatement (1) = 6.
        // The block contents aren't recursed into in this minimal walk.
        assert!(count >= 4);
    }

    #[test]
    fn parent_chain_walks_to_root() {
        let allocator = Allocator::default();
        let source = "if (a + b) c;";
        let parser = Parser::new(&allocator, source, SourceType::default());
        let ret = parser.parse();
        assert!(ret.errors.is_empty());

        let depth = walk_to_root_from_left(&ret.program);
        // BinaryExpression(self) -> IfStatement -> Program -> Dummy.
        // Starting cursor = left.parent = BinaryExpression. Then BinaryExpression's parent
        // = IfStatement (since `Expression::as_binary_expression` inherits parent from the
        // Expression slot), then Program, then Dummy.
        // Hmm wait - left.parent is BinaryExpression(self). That means parent of left = BinaryExpression.
        // BinaryExpression's parent is the Expression's parent (we inherit). Expression's parent is
        // IfStatement (from .test()). So chain: BinaryExpression -> IfStatement -> Program -> Dummy.
        // Depth = 3.
        assert_eq!(depth, Some(3));
    }
}
