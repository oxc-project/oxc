use oxc_syntax::{
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};
use oxc_traverse::BoundIdentifier;

use crate::context::TraverseCtx;

use super::class_details::ClassTemps;

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
    /// What the class's own bindings refer to once the class is defined.
    pub identity: ClassIdentity<'a>,
    /// Effective private-storage mode selected when the owning class is entered.
    private_fields_as_properties: bool,
    /// `true` if should use temp binding for references to class in transpiled static private fields,
    /// `false` if can use name binding
    pub static_private_fields_use_temp: bool,
    /// `true` if temp var for class has been inserted
    pub temp_var_is_created: bool,
    /// Where this class declares its temporaries, including a lazily created temp var.
    pub temps: ClassTemps,
}

/// What the class's own bindings refer to once the class is defined, which decides how transpiled
/// static private members find the original constructor.
///
/// Static private members belong to the original constructor. Transpiled code checks it as the
/// brand, and `Class.#x` can skip the check when `Class` is known to be that constructor.
pub(super) enum ClassIdentity<'a> {
    /// The class name and temp var always refer to the original constructor.
    Stable,
    /// Legacy decorators reassign the class name after the class is defined:
    /// `Foo = _decorate([dec], Foo)`. Static private members use the temp var, which keeps the
    /// original constructor.
    NameReassigned,
    /// A standard class decorator may replace the class, so its name can refer to the replacement.
    /// `Class.#x` never skips the brand check. Static private methods and accessors are checked
    /// against `static_brand`, a WeakSet holding the original constructor. It is `None` when the
    /// class has none. Static private fields are stored in WeakMaps instead
    /// ([`PrivateProp::uses_weakmap`](super::PrivateProp::uses_weakmap)).
    Decorated { static_brand: Option<BoundIdentifier<'a>> },
}

impl<'a> ClassIdentity<'a> {
    /// WeakSet holding the original constructor of a decorated class with static private methods.
    pub fn static_brand(&self) -> Option<&BoundIdentifier<'a>> {
        match self {
            Self::Decorated { static_brand } => static_brand.as_ref(),
            Self::Stable | Self::NameReassigned => None,
        }
    }
}

impl<'a> ClassBindings<'a> {
    /// Binding which refers to the class from outside its body, e.g. for static initializers and
    /// decorator application.
    ///
    /// A class declaration uses its name (or a temp for `export default class {}`). A class
    /// expression always uses the temp var.
    pub fn outer_binding(&self, is_declaration: bool) -> &BoundIdentifier<'a> {
        if is_declaration { self.name.as_ref().or(self.temp.as_ref()) } else { self.temp.as_ref() }
            .expect("class referenced from outside its body always has a binding")
    }

    /// Create new `ClassBindings`.
    pub fn new(
        name_binding: Option<BoundIdentifier<'a>>,
        temp_binding: Option<BoundIdentifier<'a>>,
        brand_binding: Option<BoundIdentifier<'a>>,
        temps: ClassTemps,
        static_private_fields_use_temp: bool,
        temp_var_is_created: bool,
        private_fields_as_properties: bool,
    ) -> Self {
        Self {
            name: name_binding,
            temp: temp_binding,
            shadowed_name: None,
            brand: brand_binding,
            identity: ClassIdentity::Stable,
            private_fields_as_properties,
            static_private_fields_use_temp,
            temp_var_is_created,
            temps,
        }
    }

    /// Create dummy `ClassBindings`.
    ///
    /// Used when class needs no transform, and for dummy entry at top of `ClassesStack`.
    pub fn dummy(private_fields_as_properties: bool) -> Self {
        Self::new(
            None,
            None,
            None,
            ClassTemps::Hoisted(ScopeId::new(0)),
            false,
            false,
            private_fields_as_properties,
        )
    }

    /// Get `SymbolId` of name binding.
    pub fn name_symbol_id(&self) -> Option<SymbolId> {
        self.name.as_ref().map(|binding| binding.symbol_id)
    }

    /// Class name whose `Class.#x` can skip the brand check of a static private member.
    ///
    /// Only a class declaration's name is known to refer to the class inside its body, and only
    /// while no class decorator can replace the class.
    pub fn shortcut_class_symbol_id(&self, is_declaration: bool) -> Option<SymbolId> {
        if is_declaration && !matches!(self.identity, ClassIdentity::Decorated { .. }) {
            self.name_symbol_id()
        } else {
            None
        }
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
    /// * In class declarations whose name legacy decorators reassign, it's always temp binding,
    ///   which keeps referring to the original class after `Foo = _decorate([dec], Foo)`.
    ///
    /// `static_private_fields_use_temp` is set accordingly at the right moments
    /// elsewhere in this transform.
    ///
    /// If a temp binding is required, and one doesn't already exist, a temp binding is created.
    pub fn get_or_init_static_binding(
        &mut self,
        ctx: &mut TraverseCtx<'a>,
    ) -> &BoundIdentifier<'a> {
        if self.static_private_fields_use_temp
            || matches!(self.identity, ClassIdentity::NameReassigned)
        {
            // Create temp binding if doesn't already exist
            self.temp
                .get_or_insert_with(|| self.temps.create_class_binding(self.name.as_ref(), ctx))
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
                ctx.generate_uid(&name.name, scope_id, SymbolFlags::BlockScopedVariable)
            })
        }
    }

    /// Whether private names owned by this class use loose property storage.
    pub fn private_fields_as_properties(&self) -> bool {
        self.private_fields_as_properties
    }

    /// Brand for static private uses. Decorated methods retain the original constructor;
    /// decorated fields use WeakMaps and do not call this selector.
    pub fn get_or_init_static_private_binding(
        &mut self,
        ctx: &mut TraverseCtx<'a>,
    ) -> &BoundIdentifier<'a> {
        if self.identity.static_brand().is_none() {
            return self.get_or_init_static_binding(ctx);
        }
        self.identity.static_brand().unwrap()
    }
}
