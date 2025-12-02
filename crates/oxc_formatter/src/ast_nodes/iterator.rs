//! Iterator implementations for `Vec<T>` in AstNode.
//!
//! This module provides two macros for generating iterator implementations:
//! - `impl_ast_node_vec!` - For non-Option types (uses `.map()`)
//! - `impl_ast_node_vec_for_option!` - For Option types (uses `.and_then()`)

use std::cmp::min;

use oxc_allocator::{Allocator, Vec};
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};

use super::{AstNode, AstNodes};

pub struct AstNodeIterator<'a, T> {
    inner: std::iter::Peekable<std::slice::Iter<'a, T>>,
    parent: &'a AstNodes<'a>,
    allocator: &'a Allocator,
}

macro_rules! impl_ast_node_vec {
    ($type:ty) => {
        impl<'a> AstNode<'a, Vec<'a, $type>> {
            pub fn iter(&self) -> AstNodeIterator<'a, $type> {
                AstNodeIterator {
                    inner: self.inner.iter().peekable(),
                    parent: self.parent,
                    allocator: self.allocator,
                }
            }

            pub fn first(&self) -> Option<&'a AstNode<'a, $type>> {
                let mut inner_iter = self.inner.iter();
                self.allocator
                    .alloc(inner_iter.next().map(|inner| AstNode {
                        inner,
                        parent: self.parent,
                        allocator: self.allocator,
                        following_span: inner_iter.next().map(GetSpan::span),
                    }))
                    .as_ref()
            }

            pub fn last(&self) -> Option<&'a AstNode<'a, $type>> {
                self.allocator
                    .alloc(self.inner.last().map(|inner| AstNode {
                        inner,
                        parent: self.parent,
                        allocator: self.allocator,
                        following_span: None,
                    }))
                    .as_ref()
            }
        }

        impl<'a> Iterator for AstNodeIterator<'a, $type> {
            type Item = &'a AstNode<'a, $type>;
            fn next(&mut self) -> Option<Self::Item> {
                let allocator = self.allocator;
                allocator
                    .alloc(self.inner.next().map(|inner| AstNode {
                        parent: self.parent,
                        inner,
                        allocator,
                        following_span: self.inner.peek().copied().map(GetSpan::span),
                    }))
                    .as_ref()
            }
        }

        impl<'a> IntoIterator for &AstNode<'a, Vec<'a, $type>> {
            type Item = &'a AstNode<'a, $type>;
            type IntoIter = AstNodeIterator<'a, $type>;
            fn into_iter(self) -> Self::IntoIter {
                AstNodeIterator::<$type> {
                    inner: self.inner.iter().peekable(),
                    parent: self.parent,
                    allocator: self.allocator,
                }
            }
        }
    };
}

macro_rules! impl_ast_node_vec_for_option {
    ($type:ty) => {
        impl<'a> AstNode<'a, Vec<'a, $type>> {
            pub fn iter(&self) -> AstNodeIterator<'a, $type> {
                AstNodeIterator {
                    inner: self.inner.iter().peekable(),
                    parent: self.parent,
                    allocator: self.allocator,
                }
            }

            pub fn first(&self) -> Option<&'a AstNode<'a, $type>> {
                let mut inner_iter = self.inner.iter();
                self.allocator
                    .alloc(inner_iter.next().map(|inner| AstNode {
                        inner,
                        parent: self.parent,
                        allocator: self.allocator,
                        following_span:
                            inner_iter.next().and_then(|opt| opt.as_ref().map(GetSpan::span)),
                    }))
                    .as_ref()
            }

            pub fn last(&self) -> Option<&'a AstNode<'a, $type>> {
                self.allocator
                    .alloc(self.inner.last().map(|inner| AstNode {
                        inner,
                        parent: self.parent,
                        allocator: self.allocator,
                        following_span: None,
                    }))
                    .as_ref()
            }
        }

        impl<'a> Iterator for AstNodeIterator<'a, $type> {
            type Item = &'a AstNode<'a, $type>;
            fn next(&mut self) -> Option<Self::Item> {
                let allocator = self.allocator;
                allocator
                    .alloc(self.inner.next().map(|inner| {
                        AstNode {
                            parent: self.parent,
                            inner,
                            allocator,
                            following_span: self
                                .inner
                                .peek()
                                .copied()
                                .and_then(|opt| opt.as_ref().map(GetSpan::span)),
                        }
                    }))
                    .as_ref()
            }
        }

        impl<'a> IntoIterator for &AstNode<'a, Vec<'a, $type>> {
            type Item = &'a AstNode<'a, $type>;
            type IntoIter = AstNodeIterator<'a, $type>;
            fn into_iter(self) -> Self::IntoIter {
                AstNodeIterator::<$type> {
                    inner: self.inner.iter().peekable(),
                    parent: self.parent,
                    allocator: self.allocator,
                }
            }
        }
    };
}

// Concrete implementations for all Vec types used in the AST
impl_ast_node_vec!(Expression<'a>);
impl_ast_node_vec!(ArrayExpressionElement<'a>);
impl_ast_node_vec!(ObjectPropertyKind<'a>);
impl_ast_node_vec!(TemplateElement<'a>);
impl_ast_node_vec!(Argument<'a>);
impl_ast_node_vec!(AssignmentTargetProperty<'a>);
impl_ast_node_vec!(VariableDeclarator<'a>);
impl_ast_node_vec!(SwitchCase<'a>);
impl_ast_node_vec!(BindingProperty<'a>);
impl_ast_node_vec!(FormalParameter<'a>);
impl_ast_node_vec!(ClassElement<'a>);
impl_ast_node_vec!(ImportDeclarationSpecifier<'a>);
impl_ast_node_vec!(ImportAttribute<'a>);
impl_ast_node_vec!(ExportSpecifier<'a>);
impl_ast_node_vec!(JSXAttributeItem<'a>);
impl_ast_node_vec!(JSXChild<'a>);
impl_ast_node_vec!(TSEnumMember<'a>);
impl_ast_node_vec!(TSType<'a>);
impl_ast_node_vec!(TSTupleElement<'a>);
impl_ast_node_vec!(TSTypeParameter<'a>);
impl_ast_node_vec!(TSClassImplements<'a>);
impl_ast_node_vec!(TSSignature<'a>);
impl_ast_node_vec!(TSIndexSignatureName<'a>);
impl_ast_node_vec!(TSInterfaceHeritage<'a>);
impl_ast_node_vec!(Decorator<'a>);
impl_ast_node_vec_for_option!(Option<AssignmentTargetMaybeDefault<'a>>);
impl_ast_node_vec_for_option!(Option<BindingPattern<'a>>);

// Manual implementation for Vec<Statement> because we have to handle span
// for ExportDefaultDeclaration and ExportNamedDeclaration that may include
// decorators.
// <https://github.com/oxc-project/oxc/issues/10409>
impl<'a> AstNode<'a, Vec<'a, Statement<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Statement<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, Statement<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_span: inner_iter.next().map(GetSpan::span),
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, Statement<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_span: None,
            }))
            .as_ref()
    }
}
impl<'a> Iterator for AstNodeIterator<'a, Statement<'a>> {
    type Item = &'a AstNode<'a, Statement<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| AstNode {
                parent: self.parent,
                inner,
                allocator,
                following_span: {
                    match self.inner.peek() {
                        // `@decorator export default class A {}`
                        // Get the span of the decorator.
                        Some(Statement::ExportDefaultDeclaration(export)) => {
                            if let ExportDefaultDeclarationKind::ClassDeclaration(class) =
                                &export.declaration
                                && let Some(decorator) = class.decorators.first()
                            {
                                Some(Span::new(
                                    min(decorator.span.start, export.span.start),
                                    export.span.end,
                                ))
                            } else {
                                Some(export.span)
                            }
                        }
                        // `@decorator export class A {}`
                        // Get the span of the decorator.
                        Some(Statement::ExportNamedDeclaration(export)) => {
                            if let Some(Declaration::ClassDeclaration(class)) = &export.declaration
                                && let Some(decorator) = class.decorators.first()
                            {
                                Some(Span::new(
                                    min(decorator.span.start, export.span.start),
                                    export.span.end,
                                ))
                            } else {
                                Some(export.span)
                            }
                        }
                        Some(next) => Some(next.span()),
                        None => None,
                    }
                },
            }))
            .as_ref()
    }
}
impl<'a> IntoIterator for &AstNode<'a, Vec<'a, Statement<'a>>> {
    type Item = &'a AstNode<'a, Statement<'a>>;
    type IntoIter = AstNodeIterator<'a, Statement<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<Statement<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}

fn get_following_span_for_directive_parent(parent: &AstNodes<'_>) -> Option<Span> {
    match parent {
        AstNodes::Program(program) => program.body().first().map(GetSpan::span),
        AstNodes::FunctionBody(function_body) => {
            function_body.statements().first().map(GetSpan::span)
        }
        AstNodes::TSModuleBlock(ts_module_block) => {
            ts_module_block.body().first().map(GetSpan::span)
        }
        _ => None,
    }
}

// Manual implementation for Vec<Directive> because we need to handle
// following_span for the last directive in Program.body.
impl<'a> AstNode<'a, Vec<'a, Directive<'a>>> {
    pub fn iter(&self) -> AstNodeIterator<'a, Directive<'a>> {
        AstNodeIterator {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
    pub fn first(&self) -> Option<&'a AstNode<'a, Directive<'a>>> {
        let mut inner_iter = self.inner.iter();
        self.allocator
            .alloc(inner_iter.next().map(|inner| {
                AstNode {
                    inner,
                    parent: self.parent,
                    allocator: self.allocator,
                    following_span: inner_iter
                        .next()
                        .map(GetSpan::span)
                        .or_else(|| get_following_span_for_directive_parent(self.parent)),
                }
            }))
            .as_ref()
    }
    pub fn last(&self) -> Option<&'a AstNode<'a, Directive<'a>>> {
        self.allocator
            .alloc(self.inner.last().map(|inner| AstNode {
                inner,
                parent: self.parent,
                allocator: self.allocator,
                following_span: get_following_span_for_directive_parent(self.parent),
            }))
            .as_ref()
    }
}
impl<'a> Iterator for AstNodeIterator<'a, Directive<'a>> {
    type Item = &'a AstNode<'a, Directive<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let allocator = self.allocator;
        allocator
            .alloc(self.inner.next().map(|inner| {
                AstNode {
                    parent: self.parent,
                    inner,
                    allocator,
                    following_span: self
                        .inner
                        .peek()
                        .copied()
                        .map(GetSpan::span)
                        .or_else(|| get_following_span_for_directive_parent(self.parent)),
                }
            }))
            .as_ref()
    }
}
impl<'a> IntoIterator for &AstNode<'a, Vec<'a, Directive<'a>>> {
    type Item = &'a AstNode<'a, Directive<'a>>;
    type IntoIter = AstNodeIterator<'a, Directive<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        AstNodeIterator::<Directive<'a>> {
            inner: self.inner.iter().peekable(),
            parent: self.parent,
            allocator: self.allocator,
        }
    }
}
