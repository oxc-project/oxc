use oxc_ast::ast::*;
use oxc_data_structures::stack::NonEmptyStack;
use oxc_traverse::BoundIdentifier;

use crate::{
    common::class_lowering::ClassLoweringMode,
    context::TraverseCtx,
    decorator::standard::{ClassDecoratorState, StandardDecoratorState},
    utils::ast_builder::create_variable_declaration,
};

use super::{ClassBindings, ClassProperties, IdentIndexMap};

/// Details of a class.
///
/// These are stored in `ClassesStack`.
pub(super) struct ClassDetails<'a> {
    /// Which transform owns this class, recorded before the class was rewritten.
    pub mode: ClassLoweringMode,
    /// `true` for class declaration, `false` for class expression
    pub is_declaration: bool,
    /// `true` if private method helpers must be emitted in the class expression sequence.
    pub private_method_helpers_in_class_sequence: bool,
    /// `true` if class requires no transformation
    pub is_transform_required: bool,
    /// Native private names or bindings used to lower this class's private properties.
    pub private_props: PrivateProps<'a>,
    /// Bindings for class name and temp var for class
    pub bindings: ClassBindings<'a>,
    /// Standard decorators state. `Some` if class is decorated.
    pub decorators: Option<ClassDecoratorState<'a>>,
    /// Class contained auto-accessors before they were lowered.
    pub has_auto_accessor: bool,
}

impl<'a> ClassDetails<'a> {
    /// Create dummy `ClassDetails`.
    ///
    /// Used for dummy entry at top of `ClassesStack`.
    pub fn dummy(mode: ClassLoweringMode, is_declaration: bool) -> Self {
        Self {
            mode,
            is_declaration,
            private_method_helpers_in_class_sequence: false,
            is_transform_required: false,
            private_props: PrivateProps::Lowered(IdentIndexMap::default()),
            bindings: ClassBindings::dummy(false),
            decorators: None,
            has_auto_accessor: false,
        }
    }

    /// Declare a class-scoped temporary binding.
    ///
    /// A decorated class uses `let`, because a class decorator can replace the binding.
    pub fn declare_temp(&self, binding: &BoundIdentifier<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.decorators.is_some() {
            ctx.state.var_declarations.insert_let(binding, None, &ctx.ast);
        } else {
            ctx.state.var_declarations.insert_var(binding, &ctx.ast);
        }
    }

    /// Create a class-scoped temporary declaration, using `let` for a decorated class.
    pub fn create_temp_declaration(
        &self,
        binding: &BoundIdentifier<'a>,
        init: Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Statement<'a> {
        if self.decorators.is_some() {
            StandardDecoratorState::create_let(binding, init, ctx)
        } else {
            create_variable_declaration(binding, init, ctx)
        }
    }
}

/// Private names belonging to one class. Native names only participate in lexical resolution.
pub(super) enum PrivateProps<'a> {
    Native(IdentIndexMap<'a, ()>),
    Lowered(IdentIndexMap<'a, PrivateProp<'a>>),
}

impl<'a> PrivateProps<'a> {
    pub fn lowered(&self) -> Option<&IdentIndexMap<'a, PrivateProp<'a>>> {
        match self {
            Self::Native(_) => None,
            Self::Lowered(props) => Some(props),
        }
    }
}

/// A private name resolved to its owning class, including that class's effective storage mode.
pub(super) enum ResolvedPrivateName<'a, 'b> {
    Native,
    Lowered {
        prop: &'b PrivateProp<'a>,
        class_bindings: &'b mut ClassBindings<'a>,
        is_declaration: bool,
    },
}

/// Details of a private property.
pub(super) struct PrivateProp<'a> {
    pub binding: BoundIdentifier<'a>,
    pub is_static: bool,
    pub method_kind: Option<MethodDefinitionKind>,
    pub is_accessor: bool,
    /// Store a static field in a WeakMap, initialized on the final decorated constructor.
    /// Source staticness is retained in `is_static`; private uses take the instance-field path.
    pub uses_weakmap: bool,
    // For accessor methods, they have two bindings,
    // one for getter and another for setter.
    pub binding2: Option<BoundIdentifier<'a>>,
}

impl<'a> PrivateProp<'a> {
    pub fn new(
        binding: BoundIdentifier<'a>,
        is_static: bool,
        method_kind: Option<MethodDefinitionKind>,
        is_accessor: bool,
    ) -> Self {
        Self { binding, is_static, method_kind, is_accessor, uses_weakmap: false, binding2: None }
    }

    pub fn is_method(&self) -> bool {
        self.method_kind.is_some()
    }

    pub fn is_accessor(&self) -> bool {
        self.is_accessor || self.method_kind.is_some_and(MethodDefinitionKind::is_accessor)
    }

    /// Whether a private use takes the static-box / constructor-brand path.
    /// Static fields stored in WeakMaps reuse the instance-field lowering path.
    pub fn is_static_for_lowering(&self) -> bool {
        self.is_static && !self.uses_weakmap
    }

    pub fn set_binding2(&mut self, binding: BoundIdentifier<'a>) {
        self.binding2 = Some(binding);
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
    pub fn new(dummy_mode: ClassLoweringMode) -> Self {
        // Default stack capacity is 4. That's is probably good. More than 4 nested classes is rare.
        Self { stack: NonEmptyStack::new(ClassDetails::dummy(dummy_mode, false)) }
    }

    /// Push an entry to stack.
    #[inline]
    pub fn push(&mut self, class: ClassDetails<'a>) {
        debug_assert_eq!(
            matches!(class.mode, ClassLoweringMode::Native | ClassLoweringMode::Preserve),
            matches!(class.private_props, PrivateProps::Native(_)),
            "class lowering mode must agree with its private-name storage"
        );
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

    /// Resolve the nearest private name without inventing bindings for untouched classes.
    pub fn resolve_private_name<'b>(
        &'b mut self,
        ident: &PrivateIdentifier<'a>,
    ) -> ResolvedPrivateName<'a, 'b> {
        // Skip the dummy entry and search the closest class first.
        for class in self.stack[1..].iter_mut().rev() {
            match &class.private_props {
                PrivateProps::Native(names) => {
                    if names.contains_key(&ident.name) {
                        return ResolvedPrivateName::Native;
                    }
                }
                PrivateProps::Lowered(props) => {
                    if let Some(prop) = props.get(&ident.name) {
                        return ResolvedPrivateName::Lowered {
                            prop,
                            class_bindings: &mut class.bindings,
                            is_declaration: class.is_declaration,
                        };
                    }
                }
            }
        }
        // Native names need not be registered when there is no enclosing lowered name to shadow.
        ResolvedPrivateName::Native
    }

    fn lookup_private_prop<
        'b,
        Ret,
        RetFn: Fn(&'b PrivateProp<'a>, &'b mut ClassBindings<'a>, bool) -> Ret,
    >(
        &'b mut self,
        ident: &PrivateIdentifier<'a>,
        ret_fn: RetFn,
    ) -> Ret {
        let ResolvedPrivateName::Lowered { prop, class_bindings, is_declaration } =
            self.resolve_private_name(ident)
        else {
            unreachable!("native private names do not have lowered bindings");
        };
        ret_fn(prop, class_bindings, is_declaration)
    }

    /// Use the owning class's effective private-storage mode.
    pub fn private_fields_as_properties(&mut self, ident: &PrivateIdentifier<'a>) -> bool {
        self.lookup_private_prop(ident, |_, class_bindings, _| {
            class_bindings.private_fields_as_properties()
        })
    }

    /// Lookup details of private property referred to by `ident`.
    pub fn find_private_prop<'b>(
        &'b mut self,
        ident: &PrivateIdentifier<'a>,
    ) -> ResolvedPrivateProp<'a, 'b> {
        self.lookup_private_prop(ident, move |prop, class_bindings, is_declaration| {
            ResolvedPrivateProp {
                prop_binding: &prop.binding,
                class_bindings,
                is_static: prop.is_static_for_lowering(),
                is_method: prop.is_method(),
                is_accessor: prop.is_accessor(),
                is_declaration,
            }
        })
    }

    /// Lookup details of readable private property referred to by `ident`.
    pub fn find_readable_private_prop<'b>(
        &'b mut self,
        ident: &PrivateIdentifier<'a>,
    ) -> Option<ResolvedPrivateProp<'a, 'b>> {
        self.lookup_private_prop(ident, move |prop, class_bindings, is_declaration| {
            let prop_binding = if matches!(prop.method_kind, Some(MethodDefinitionKind::Set)) {
                prop.binding2.as_ref()
            } else {
                Some(&prop.binding)
            };
            prop_binding.map(|prop_binding| ResolvedPrivateProp {
                prop_binding,
                class_bindings,
                is_static: prop.is_static_for_lowering(),
                is_method: prop.is_method(),
                is_accessor: prop.is_accessor(),
                is_declaration,
            })
        })
    }

    /// Lookup details of writable private property referred to by `ident`.
    /// Returns `Some` if it refers to a private prop and setter method
    pub fn find_writable_private_prop<'b>(
        &'b mut self,
        ident: &PrivateIdentifier<'a>,
    ) -> Option<ResolvedPrivateProp<'a, 'b>> {
        self.lookup_private_prop(ident, move |prop, class_bindings, is_declaration| {
            let prop_binding = if matches!(prop.method_kind, Some(MethodDefinitionKind::Set) | None)
            {
                Some(&prop.binding)
            } else {
                prop.binding2.as_ref()
            };
            prop_binding.map(|prop_binding| ResolvedPrivateProp {
                prop_binding,
                class_bindings,
                is_static: prop.is_static_for_lowering(),
                is_method: prop.is_method(),
                is_accessor: prop.is_accessor(),
                is_declaration,
            })
        })
    }

    /// Look up details of the private property referred to by ident and it can either be read or written.
    pub fn find_get_set_private_prop<'b>(
        &'b mut self,
        ident: &PrivateIdentifier<'a>,
    ) -> ResolvedGetSetPrivateProp<'a, 'b> {
        self.lookup_private_prop(ident, move |prop, class_bindings, is_declaration| {
            let (get_binding, set_binding) = match prop.method_kind {
                Some(MethodDefinitionKind::Set) => (prop.binding2.as_ref(), Some(&prop.binding)),
                Some(_) => (Some(&prop.binding), prop.binding2.as_ref()),
                _ => (Some(&prop.binding), Some(&prop.binding)),
            };
            ResolvedGetSetPrivateProp {
                get_binding,
                set_binding,
                class_bindings,
                is_static: prop.is_static_for_lowering(),
                is_method: prop.is_method(),
                is_accessor: prop.is_accessor(),
                is_declaration,
            }
        })
    }
}

/// Details of a private property resolved for a private field.
///
/// This is the return value of [`ClassesStack::find_private_prop`],
/// [`ClassesStack::find_readable_private_prop`] and
/// [`ClassesStack::find_writable_private_prop`].
pub(super) struct ResolvedPrivateProp<'a, 'b> {
    /// Binding for temp var representing the property
    pub prop_binding: &'b BoundIdentifier<'a>,
    /// Bindings for class name and temp var for class
    pub class_bindings: &'b mut ClassBindings<'a>,
    /// `true` if the use needs static-box / constructor-brand lowering.
    /// Static fields with `uses_weakmap` take the instance-field path instead.
    pub is_static: bool,
    /// `true` if is a private method or accessor property
    pub is_method: bool,
    /// `true` if is a private accessor property or [`PrivateProp::method_kind`] is
    /// `Some(MethodDefinitionKind::Get)` or `Some(MethodDefinitionKind::Set)`
    pub is_accessor: bool,
    /// `true` if class which defines this property is a class declaration
    pub is_declaration: bool,
}

/// Details of a private property resolved for a private field.
///
/// This is the return value of [`ClassesStack::find_get_set_private_prop`].
pub(super) struct ResolvedGetSetPrivateProp<'a, 'b> {
    /// Binding for temp var representing the property or getter method
    pub get_binding: Option<&'b BoundIdentifier<'a>>,
    /// Binding for temp var representing the property or setter method
    pub set_binding: Option<&'b BoundIdentifier<'a>>,
    /// Bindings for class name and temp var for class
    pub class_bindings: &'b mut ClassBindings<'a>,
    /// `true` if the use needs static-box / constructor-brand lowering.
    /// Static fields with `uses_weakmap` take the instance-field path instead.
    pub is_static: bool,
    /// `true` if is a private method or accessor property
    pub is_method: bool,
    /// `true` if is a private accessor property or [`PrivateProp::method_kind`] is
    /// `Some(MethodDefinitionKind::Get)` or `Some(MethodDefinitionKind::Set)`
    pub is_accessor: bool,
    /// `true` if class which defines this property is a class declaration
    pub is_declaration: bool,
}

// Shortcut methods to get current class
impl<'a> ClassProperties<'a> {
    /// Get details of current class.
    pub(super) fn current_class(&self) -> &ClassDetails<'a> {
        self.classes_stack.last()
    }

    /// Get details of current class as `&mut` reference.
    pub(super) fn current_class_mut(&mut self) -> &mut ClassDetails<'a> {
        self.classes_stack.last_mut()
    }
}
