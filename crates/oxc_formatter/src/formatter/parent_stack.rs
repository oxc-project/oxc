use oxc_ast::{AstKind, ast::Program};

use oxc_data_structures::stack::NonEmptyStack;

pub struct ParentStack<'ast> {
    inner: NonEmptyStack<AstKind<'ast>>,
}

impl<'ast> ParentStack<'ast> {
    pub fn new(program: &'ast Program<'ast>) -> Self {
        Self { inner: NonEmptyStack::new(AstKind::Program(program)) }
    }

    pub fn push(&mut self, kind: AstKind<'ast>) {
        self.inner.push(kind);
    }

    pub fn pop(&mut self) {
        /// SAFETY: push must be called.
        unsafe {
            self.inner.pop_unchecked();
        }
    }

    pub fn parent(&self) -> AstKind<'ast> {
        self.inner.as_slice().get(self.inner.len() - 2).copied().unwrap()
    }

    pub fn parent2(&self) -> Option<AstKind<'ast>> {
        self.inner.as_slice().get(self.inner.len() - 3).copied()
    }
}
