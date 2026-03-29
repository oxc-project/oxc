use serde::Deserialize;

/// Compiler assumptions
///
/// For producing smaller output.
///
/// See <https://babeljs.io/docs/assumptions>
#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompilerAssumptions {
    /// Assume array-like values are iterable.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub array_like_is_iterable: bool,

    /// Assume re-exported bindings are constant.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub constant_reexports: bool,

    /// Assume `super` property writes do not require runtime checks.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub constant_super: bool,

    /// Treat `import.meta` properties as enumerable.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub enumerable_module_meta: bool,

    /// Ignore `Function#length` when lowering function wrappers.
    #[serde(default)]
    pub ignore_function_length: bool,

    /// Ignore the preferred hint passed to `Symbol.toPrimitive`.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub ignore_to_primitive_hint: bool,

    /// Assume iterable operations only receive arrays.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub iterable_is_array: bool,

    /// Emit mutable template objects.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub mutable_template_object: bool,

    /// Assume class constructors are never called without `new`.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub no_class_calls: bool,

    /// Assume `document.all` is not special-cased.
    #[serde(default)]
    pub no_document_all: bool,

    /// Skip namespace import completeness checks.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub no_incomplete_ns_import_detection: bool,

    /// Assume no new `this` capture is needed for arrows.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub no_new_arrows: bool,

    /// Assume private fields are always initialized before access.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub no_uninitialized_private_field_access: bool,

    /// Assume object rest does not need to copy symbol properties.
    #[serde(default)]
    pub object_rest_no_symbols: bool,

    /// Represent private fields as symbols.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub private_fields_as_symbols: bool,

    /// Represent private fields as string-keyed properties.
    #[serde(default)]
    pub private_fields_as_properties: bool,

    /// Assume property reads can be treated as pure.
    #[serde(default)]
    pub pure_getters: bool,

    /// Assume class methods can be assigned directly.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub set_class_methods: bool,

    /// Assume computed properties can be assigned directly.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub set_computed_properties: bool,

    /// When using public class fields, assume that they don't shadow any getter in the current class,
    /// in its subclasses or in its superclass. Thus, it's safe to assign them rather than using
    /// `Object.defineProperty`.
    ///
    /// For example:
    ///
    /// Input:
    /// ```js
    /// class Test {
    ///  field = 2;
    ///
    ///  static staticField = 3;
    /// }
    /// ```
    ///
    /// When `set_public_class_fields` is `true`, the output will be:
    /// ```js
    /// class Test {
    ///  constructor() {
    ///    this.field = 2;
    ///  }
    /// }
    /// Test.staticField = 3;
    /// ```
    ///
    /// Otherwise, the output will be:
    /// ```js
    /// import _defineProperty from "@oxc-project/runtime/helpers/defineProperty";
    /// class Test {
    ///   constructor() {
    ///     _defineProperty(this, "field", 2);
    ///   }
    /// }
    /// _defineProperty(Test, "staticField", 3);
    /// ```
    ///
    /// NOTE: For TypeScript, if you wanted behavior is equivalent to `useDefineForClassFields: false`, you should
    /// set both `set_public_class_fields` and [`crate::TypeScriptOptions::remove_class_fields_without_initializer`]
    /// to `true`.
    #[serde(default)]
    pub set_public_class_fields: bool,

    /// Assume object spread can use direct assignment semantics.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub set_spread_properties: bool,

    /// Skip `for..of` iterator closing logic.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub skip_for_of_iterator_closing: bool,

    /// Assume `super` can be invoked as a normal callable constructor.
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub super_is_callable_constructor: bool,
}
