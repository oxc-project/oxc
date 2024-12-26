use oxc_ast::ast::*;
use oxc_data_structures::stack::NonEmptyStack;
use oxc_span::Atom;
use oxc_traverse::BoundIdentifier;

use super::{ClassBindings, ClassProperties, FxIndexMap};

/// Details of a class.
///
/// These are stored in `ClassesStack`.
pub(super) struct ClassDetails<'a> {
    /// `true` for class declaration, `false` for class expression
    pub is_declaration: bool,
    /// `true` if class requires no transformation
    pub is_transform_required: bool,
    /// Private properties.
    /// Mapping private prop name to binding for temp var.
    /// This is then used as lookup when transforming e.g. `this.#x`.
    /// `None` if class has no private properties.
    pub private_props: Option<FxIndexMap<Atom<'a>, PrivateProp<'a>>>,
    /// Bindings for class name and temp var for class
    pub bindings: ClassBindings<'a>,
}

impl<'a> ClassDetails<'a> {
    /// Create dummy `ClassDetails`.
    ///
    /// Used for dummy entry at top of `ClassesStack`.
    pub fn dummy(is_declaration: bool) -> Self {
        Self {
            is_declaration,
            is_transform_required: false,
            private_props: None,
            bindings: ClassBindings::dummy(),
        }
    }
}

/// Details of a private property.
pub(super) struct PrivateProp<'a> {
    pub binding: BoundIdentifier<'a>,
    pub is_static: bool,
    pub is_method: bool,
    pub is_accessor: bool,
}

impl<'a> PrivateProp<'a> {
    pub fn new(
        binding: BoundIdentifier<'a>,
        is_static: bool,
        is_method: bool,
        is_accessor: bool,
    ) -> Self {
        Self { binding, is_static, is_method, is_accessor }
    }
}

/// Stack of `ClassDetails`.
///
/// Pushed to when entering a class, popped when exiting.
///
/// We use a `NonEmptyStack` to make `last` and `last_mut` cheap (these are used a lot).
/// The first entry is a dummy.
///
/// This is a separate structure, rather than just storing stack as a property of `ClassProperties`
/// to work around borrow-checker. You can call `find_private_prop` and retain the return value
/// without holding a mut borrow of the whole of `&mut ClassProperties`. This allows accessing other
/// properties of `ClassProperties` while that borrow is held.
pub(super) struct ClassesStack<'a> {
    stack: NonEmptyStack<ClassDetails<'a>>,
}

impl<'a> ClassesStack<'a> {
    /// Create new `ClassesStack`.
    pub fn new() -> Self {
        // Default stack capacity is 4. That's is probably good. More than 4 nested classes is rare.
        Self { stack: NonEmptyStack::new(ClassDetails::dummy(false)) }
    }

    /// Push an entry to stack.
    #[inline]
    pub fn push(&mut self, class: ClassDetails<'a>) {
        self.stack.push(class);
    }

    /// Push last entry from stack.
    #[inline]
    pub fn pop(&mut self) -> ClassDetails<'a> {
        self.stack.pop()
    }

    /// Get details of current class.
    #[inline]
    pub fn last(&self) -> &ClassDetails<'a> {
        self.stack.last()
    }

    /// Get details of current class as `&mut` reference.
    #[inline]
    pub fn last_mut(&mut self) -> &mut ClassDetails<'a> {
        self.stack.last_mut()
    }

    /// Lookup details of private property referred to by `ident`.
    pub fn find_private_prop<'b>(
        &'b mut self,
        ident: &PrivateIdentifier<'a>,
    ) -> ResolvedPrivateProp<'a, 'b> {
        // Check for binding in closest class first, then enclosing classes.
        // We skip the first, because this is a `NonEmptyStack` with dummy first entry.
        // TODO: Check there are tests for bindings in enclosing classes.
        for class in self.stack[1..].iter_mut().rev() {
            if let Some(private_props) = &mut class.private_props {
                if let Some(prop) = private_props.get(&ident.name) {
                    return ResolvedPrivateProp {
                        prop_binding: &prop.binding,
                        class_bindings: &mut class.bindings,
                        is_static: prop.is_static,
                        is_method: prop.is_method,
                        is_accessor: prop.is_accessor,
                        is_declaration: class.is_declaration,
                    };
                }
            }
        }

        unreachable!();
    }
}

/// Details of a private property resolved for a private field.
///
/// This is the return value of [`ClassesStack::find_private_prop`].
pub(super) struct ResolvedPrivateProp<'a, 'b> {
    /// Binding for temp var representing the property
    pub prop_binding: &'b BoundIdentifier<'a>,
    /// Bindings for class name and temp var for class
    pub class_bindings: &'b mut ClassBindings<'a>,
    /// `true` if is a static property
    pub is_static: bool,
    /// `true` if is a private method
    pub is_method: bool,
    /// `true` if is a private accessor property
    pub is_accessor: bool,
    /// `true` if class which defines this property is a class declaration
    pub is_declaration: bool,
}

// Shortcut methods to get current class
impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// Get details of current class.
    pub(super) fn current_class(&self) -> &ClassDetails<'a> {
        self.classes_stack.last()
    }

    /// Get details of current class as `&mut` reference.
    pub(super) fn current_class_mut(&mut self) -> &mut ClassDetails<'a> {
        self.classes_stack.last_mut()
    }
}
