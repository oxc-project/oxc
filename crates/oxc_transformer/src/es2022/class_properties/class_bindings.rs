use oxc_syntax::{
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};
use oxc_traverse::BoundIdentifier;

use crate::context::TraverseCtx;

/// Store for bindings for class.
///
/// 1. Existing binding for class name (if class has a name).
/// 2. Temp var `_Class`, which may or may not be required.
/// 3. Lexical alias for a shadowed class declaration name.
///
/// Temp var is required in the following circumstances:
///
/// * Class expression has static properties.
///   e.g. `C = class { static x = 1; }`
/// * Class declaration has static properties and one of the static prop's initializers contains:
///   a. `this`
///   e.g. `class C { static x = this; }`
///   b. Reference to class name
///   e.g. `class C { static x = C; }`
///   c. A private field referring to one of the class's static private props.
///   e.g. `class C { static #x; static y = obj.#x; }`
///
/// The logic for when transpiled private fields use a reference to class name or class temp var
/// is unfortunately rather complicated.
///
/// Transpiled private fields referring to a static private prop use:
///
/// * Class name when field is within body of class declaration
///   e.g. `class C { static #x; method() { return obj.#x; } }`
///   -> `_assertClassBrand(C, obj, _x)._`
/// * Temp var when field is within body of class expression
///   e.g. `C = class C { static #x; method() { return obj.#x; } }`
///   -> `_assertClassBrand(_C, obj, _x)._`
/// * Temp var when field is within a static prop initializer
///   e.g. `class C { static #x; static y = obj.#x; }`
///   -> `_assertClassBrand(_C, obj, _x)._`
///
/// `static_private_fields_use_temp` is updated as transform moves through the class,
/// to indicate which binding to use.
pub(super) struct ClassBindings<'a> {
    /// Binding for class name, if class has name
    pub name: Option<BoundIdentifier<'a>>,
    /// Temp var for class.
    /// e.g. `_Class` in `_Class = class {}, _Class.x = 1, _Class`
    pub temp: Option<BoundIdentifier<'a>>,
    /// Lexical alias for a class declaration whose name is shadowed inside its body.
    /// Unlike `temp`, this binding must be fresh each time the declaration is evaluated.
    pub shadowed_name: Option<BoundIdentifier<'a>>,
    /// Temp var for WeakSet.
    pub brand: Option<BoundIdentifier<'a>>,
    /// `ScopeId` of hoist scope outside class (which temp `var` binding would be created in)
    pub outer_hoist_scope_id: ScopeId,
    /// `true` if should use temp binding for references to class in transpiled static private fields,
    /// `false` if can use name binding
    pub static_private_fields_use_temp: bool,
    /// `true` if temp var for class has been inserted
    pub temp_var_is_created: bool,
}

impl<'a> ClassBindings<'a> {
    /// Create new `ClassBindings`.
    pub fn new(
        name_binding: Option<BoundIdentifier<'a>>,
        temp_binding: Option<BoundIdentifier<'a>>,
        brand_binding: Option<BoundIdentifier<'a>>,
        outer_scope_id: ScopeId,
        static_private_fields_use_temp: bool,
        temp_var_is_created: bool,
    ) -> Self {
        Self {
            name: name_binding,
            temp: temp_binding,
            shadowed_name: None,
            brand: brand_binding,
            outer_hoist_scope_id: outer_scope_id,
            static_private_fields_use_temp,
            temp_var_is_created,
        }
    }

    /// Create dummy `ClassBindings`.
    ///
    /// Used when class needs no transform, and for dummy entry at top of `ClassesStack`.
    pub fn dummy() -> Self {
        Self::new(None, None, None, ScopeId::new(0), false, false)
    }

    /// Get `SymbolId` of name binding.
    pub fn name_symbol_id(&self) -> Option<SymbolId> {
        self.name.as_ref().map(|binding| binding.symbol_id)
    }

    /// Get [`BoundIdentifier`] for class brand.
    ///
    /// Only use this method when you are sure that [Self::brand] is not `None`,
    /// this will happen when there is a private method in the class.
    ///
    /// # Panics
    /// Panics if [Self::brand] is `None`.
    pub fn brand(&self) -> &BoundIdentifier<'a> {
        self.brand.as_ref().unwrap()
    }

    /// Get binding to use for referring to class in transpiled static private fields.
    ///
    /// e.g. `Class` in `_assertClassBrand(Class, object, _prop)._` (class name)
    /// or `_Class` in `_assertClassBrand(_Class, object, _prop)._` (temp var)
    ///
    /// * In class expressions, this is always be temp binding.
    /// * In class declarations, it's the name binding when code is inside class body,
    ///   or a lexical alias if that name is shadowed, and temp binding outside class body.
    ///
    /// `static_private_fields_use_temp` is set accordingly at the right moments
    /// elsewhere in this transform.
    ///
    /// If a temp binding is required, and one doesn't already exist, a temp binding is created.
    pub fn get_or_init_static_binding(
        &mut self,
        ctx: &mut TraverseCtx<'a>,
    ) -> &BoundIdentifier<'a> {
        if self.static_private_fields_use_temp {
            // Create temp binding if doesn't already exist
            self.temp.get_or_insert_with(|| {
                Self::create_temp_binding(self.name.as_ref(), self.outer_hoist_scope_id, ctx)
            })
        } else {
            // `static_private_fields_use_temp` is always `true` for class expressions.
            // Class declarations always have a name binding if they have any static props.
            // So `unwrap` here cannot panic.
            let name = self.name.as_ref().unwrap();
            if ctx.scoping().find_binding(ctx.current_scope_id(), name.name) == Some(name.symbol_id)
            {
                return name;
            }

            // A function-scoped temp would be shared by class declarations in a loop.
            // Create the alias in the class name's scope so each evaluation keeps its own brand.
            self.shadowed_name.get_or_insert_with(|| {
                let scope_id = ctx.scoping().symbol_scope_id(name.symbol_id);
                ctx.generate_uid(
                    &name.name,
                    scope_id,
                    SymbolFlags::BlockScopedVariable | SymbolFlags::ConstVariable,
                )
            })
        }
    }

    /// Generate binding for temp var.
    pub fn create_temp_binding(
        name_binding: Option<&BoundIdentifier<'a>>,
        outer_hoist_scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        // Base temp binding name on class name, or "Class" if no name.
        // TODO(improve-on-babel): If class name var isn't mutated, no need for temp var for
        // class declaration. Can just use class binding.
        let name = name_binding.map_or("Class", |binding| binding.name.as_str());
        ctx.generate_uid(name, outer_hoist_scope_id, SymbolFlags::FunctionScopedVariable)
    }
}
