use oxc_ast::ast::PrivateIdentifier;
use oxc_data_structures::stack::SparseStack;
use oxc_span::Atom;
use oxc_traverse::BoundIdentifier;

use super::{class_bindings::ClassBindings, FxIndexMap};

/// Stack of private props defined by classes.
///
/// Pushed to when entering a class (`None` if class has no private props, `Some` if it does).
/// Entries contain a mapping from private prop name to binding for temp var for that property,
/// and details of the class that defines the private prop.
///
/// This is used as lookup when transforming e.g. `this.#x`.
#[derive(Default)]
pub(super) struct PrivatePropsStack<'a> {
    stack: SparseStack<PrivateProps<'a>>,
}

impl<'a> PrivatePropsStack<'a> {
    // Forward methods to underlying `SparseStack`

    #[inline]
    pub fn push(&mut self, props: Option<PrivateProps<'a>>) {
        self.stack.push(props);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<PrivateProps<'a>> {
        self.stack.pop()
    }

    #[inline]
    pub fn last(&self) -> Option<&PrivateProps<'a>> {
        self.stack.last()
    }

    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut PrivateProps<'a>> {
        self.stack.last_mut()
    }

    /// Lookup details of private property referred to by `ident`.
    pub fn find<'b>(
        &'b mut self,
        ident: &PrivateIdentifier<'a>,
    ) -> Option<ResolvedPrivateProp<'a, 'b>> {
        // Check for binding in closest class first, then enclosing classes
        // TODO: Check there are tests for bindings in enclosing classes.
        for private_props in self.stack.as_mut_slice().iter_mut().rev() {
            if let Some(prop) = private_props.props.get(&ident.name) {
                return Some(ResolvedPrivateProp {
                    prop_binding: &prop.binding,
                    class_bindings: &mut private_props.class_bindings,
                    is_static: prop.is_static,
                    is_declaration: private_props.is_declaration,
                });
            }
        }
        // TODO: This should be unreachable. Only returning `None` because implementation is incomplete.
        None
    }
}

/// Details of private properties for a class.
pub(super) struct PrivateProps<'a> {
    /// Private properties for class. Indexed by property name.
    // TODO(improve-on-babel): Order that temp vars are created in is not important. Use `FxHashMap` instead.
    pub props: FxIndexMap<Atom<'a>, PrivateProp<'a>>,
    /// Bindings for class name and temp var for class
    pub class_bindings: ClassBindings<'a>,
    /// `true` for class declaration, `false` for class expression
    pub is_declaration: bool,
}

/// Details of a private property.
pub(super) struct PrivateProp<'a> {
    pub binding: BoundIdentifier<'a>,
    pub is_static: bool,
}

/// Details of a private property resolved for a private field.
///
/// This is the return value of [`PrivatePropsStack::find`].
pub(super) struct ResolvedPrivateProp<'a, 'b> {
    /// Binding for temp var representing the property
    pub prop_binding: &'b BoundIdentifier<'a>,
    /// Bindings for class name and temp var for class
    pub class_bindings: &'b mut ClassBindings<'a>,
    /// `true` if is a static property
    pub is_static: bool,
    /// `true` if class which defines this property is a class declaration
    pub is_declaration: bool,
}
