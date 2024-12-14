use oxc_syntax::symbol::SymbolId;
use oxc_traverse::{BoundIdentifier, TraverseCtx};

/// Store for bindings for class.
///
/// 1. Existing binding for class name (if class has a name).
/// 2. Temp var `_Class`, which may or may not be required.
///
/// Temp var is required in the following circumstances:
/// * Class expression has static properties.
///   e.g. `C = class { x = 1; }`
/// * Class declaration has static properties and one of the static prop's initializers contains:
///   a. `this`
///      e.g. `class C { x = this; }`
///   b. Reference to class name
///      e.g. `class C { x = C; }`
///   c. A private field referring to one of the class's static private props.
///      e.g. `class C { static #x; static y = obj.#x; }`
///
/// An instance of `ClassBindings` is stored in main `ClassProperties` transform, and a 2nd is stored
/// in `PrivateProps` for the class, if the class has any private properties.
/// If the class has private props, the instance of `ClassBindings` in `PrivateProps` is the source
/// of truth.
///
/// The logic for when transpiled private fields use a reference to class name or class temp var
/// is unfortunately rather complicated.
///
/// Transpiled private fields referring to a static private prop use:
/// * Class name when field is within class body and class has a name
///   e.g. `class C { static #x; method() { return obj.#x; } }`
/// * Temp var when field is within class body and class has no name
///   e.g. `C = class { static #x; method() { return obj.#x; } }`
/// * Temp var when field is within a static prop initializer.
///   e.g. `class C { static #x; y = obj.#x; }`
///
/// To cover all these cases, the meaning of `temp` binding here changes while traversing the class body.
/// [`ClassProperties::transform_class`] sets `temp` binding to be a copy of the `name` binding before
/// that traversal begins. So the name `temp` is misleading at that point.
///
/// Debug assertions are used to make sure this complex logic is correct.
///
/// [`ClassProperties::transform_class`]: super::ClassProperties::transform_class
#[derive(Default, Clone)]
pub(super) struct ClassBindings<'a> {
    /// Binding for class name, if class has name
    pub name: Option<BoundIdentifier<'a>>,
    /// Temp var for class.
    /// e.g. `_Class` in `_Class = class {}, _Class.x = 1, _Class`
    pub temp: Option<BoundIdentifier<'a>>,
    /// `true` if currently transforming static property initializers.
    /// Only used in debug builds to check logic is correct.
    #[cfg(debug_assertions)]
    pub currently_transforming_static_property_initializers: bool,
}

impl<'a> ClassBindings<'a> {
    /// Create `ClassBindings`.
    pub fn new(
        name_binding: Option<BoundIdentifier<'a>>,
        temp_binding: Option<BoundIdentifier<'a>>,
    ) -> Self {
        Self {
            name: name_binding,
            temp: temp_binding,
            #[cfg(debug_assertions)]
            currently_transforming_static_property_initializers: false,
        }
    }

    /// Get `SymbolId` of name binding.
    pub fn name_symbol_id(&self) -> Option<SymbolId> {
        self.name.as_ref().map(|binding| binding.symbol_id)
    }

    /// Create a binding for temp var, if there isn't one already.
    pub fn get_or_init_temp_binding(&mut self, ctx: &mut TraverseCtx<'a>) -> &BoundIdentifier<'a> {
        if self.temp.is_none() {
            // This should only be possible if we are currently transforming static prop initializers
            #[cfg(debug_assertions)]
            {
                assert!(
                    self.currently_transforming_static_property_initializers,
                    "Should be unreachable"
                );
            }

            self.temp = Some(Self::create_temp_binding(self.name.as_ref(), ctx));
        }
        self.temp.as_ref().unwrap()
    }

    /// Generate a binding for temp var.
    pub fn create_temp_binding(
        name_binding: Option<&BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        // Base temp binding name on class name, or "Class" if no name.
        // TODO(improve-on-babel): If class name var isn't mutated, no need for temp var for
        // class declaration. Can just use class binding.
        let name = name_binding.map_or("Class", |binding| binding.name.as_str());
        ctx.generate_uid_in_current_hoist_scope(name)
    }
}
