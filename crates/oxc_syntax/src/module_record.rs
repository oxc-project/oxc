//! [ECMAScript Module Record](https://tc39.es/ecma262/#sec-abstract-module-records)

use rustc_hash::FxHashMap;

use oxc_allocator::{Allocator, Vec};
use oxc_span::{Atom, Span};

/// ESM Module Record
///
/// All data inside this data structure are for ESM, no commonjs data is allowed.
///
/// See
/// * <https://tc39.es/ecma262/#table-additional-fields-of-source-text-module-records>
/// * <https://tc39.es/ecma262/#cyclic-module-record>
#[derive(Debug)]
pub struct ModuleRecord<'a> {
    /// This module has ESM syntax: `import` and `export`.
    pub has_module_syntax: bool,

    /// `[[RequestedModules]]`
    ///
    /// A List of all the ModuleSpecifier strings used by the module represented by this record to request the importation of a module. The List is in source text occurrence order.
    ///
    /// Module requests from:
    ///   import ImportClause FromClause
    ///   import ModuleSpecifier
    ///   export ExportFromClause FromClause
    ///
    /// Keyed by ModuleSpecifier, valued by all node occurrences
    pub requested_modules: FxHashMap<Atom<'a>, Vec<'a, RequestedModule>>,

    /// `[[ImportEntries]]`
    ///
    /// A List of ImportEntry records derived from the code of this module
    pub import_entries: Vec<'a, ImportEntry<'a>>,

    /// `[[LocalExportEntries]]`
    ///
    /// A List of [`ExportEntry`] records derived from the code of this module
    /// that correspond to declarations that occur within the module
    pub local_export_entries: Vec<'a, ExportEntry<'a>>,

    /// `[[IndirectExportEntries]]`
    ///
    /// A List of [`ExportEntry`] records derived from the code of this module
    /// that correspond to reexported imports that occur within the module
    /// or exports from `export * as namespace` declarations.
    pub indirect_export_entries: Vec<'a, ExportEntry<'a>>,

    /// `[[StarExportEntries]]`
    ///
    /// A List of [`ExportEntry`] records derived from the code of this module
    /// that correspond to `export *` declarations that occur within the module,
    /// not including `export * as namespace` declarations.
    pub star_export_entries: Vec<'a, ExportEntry<'a>>,

    /// Local exported bindings
    pub exported_bindings: FxHashMap<Atom<'a>, Span>,

    /// Dynamic import expressions `import(specifier)`.
    pub dynamic_imports: Vec<'a, DynamicImport>,

    /// Span position of `import.meta`.
    pub import_metas: Vec<'a, Span>,
}

impl<'a> ModuleRecord<'a> {
    /// Constructor
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            has_module_syntax: false,
            requested_modules: FxHashMap::default(),
            import_entries: Vec::new_in(allocator),
            local_export_entries: Vec::new_in(allocator),
            indirect_export_entries: Vec::new_in(allocator),
            star_export_entries: Vec::new_in(allocator),
            exported_bindings: FxHashMap::default(),
            dynamic_imports: Vec::new_in(allocator),
            import_metas: Vec::new_in(allocator),
        }
    }
}

/// Name and Span
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameSpan<'a> {
    /// Name
    pub name: Atom<'a>,

    /// Span
    pub span: Span,
}

impl<'a> NameSpan<'a> {
    /// Constructor
    pub fn new(name: Atom<'a>, span: Span) -> Self {
        Self { name, span }
    }
}

/// [`ImportEntry`](https://tc39.es/ecma262/#importentry-record)
///
/// ## Examples
///
/// ```ts
/// //     _ local_name
/// import v from "mod";
/// //             ^^^ module_request
///
/// //     ____ is_type will be `true`
/// import type { foo as bar } from "mod";
/// // import_name^^^    ^^^ local_name
///
/// import * as ns from "mod";
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportEntry<'a> {
    /// Span of the import statement.
    pub statement_span: Span,

    /// String value of the ModuleSpecifier of the ImportDeclaration.
    ///
    /// ## Examples
    ///
    /// ```ts
    /// import { foo } from "mod";
    /// //                   ^^^
    /// ```
    pub module_request: NameSpan<'a>,

    /// The name under which the desired binding is exported by the module identified by `[[ModuleRequest]]`.
    ///
    /// ## Examples
    ///
    /// ```ts
    /// import { foo } from "mod";
    /// //       ^^^
    /// import { foo as bar } from "mod";
    /// //       ^^^
    /// ```
    pub import_name: ImportImportName<'a>,

    /// The name that is used to locally access the imported value from within the importing module.
    ///
    /// ## Examples
    ///
    /// ```ts
    /// import { foo } from "mod";
    /// //       ^^^
    /// import { foo as bar } from "mod";
    /// //              ^^^
    /// ```
    pub local_name: NameSpan<'a>,

    /// Whether this binding is for a TypeScript type-only import. This is a non-standard field.
    /// When creating a [`ModuleRecord`] for a JavaScript file, this will always be false.
    ///
    /// ## Examples
    ///
    /// `is_type` will be `true` for the following imports:
    /// ```ts
    /// import type { foo } from "mod";
    /// import { type foo } from "mod";
    /// ```
    ///
    /// and will be `false` for these imports:
    /// ```ts
    /// import { foo } from "mod";
    /// import { foo as type } from "mod";
    /// ```
    pub is_type: bool,
}

/// `ImportName` For `ImportEntry`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportImportName<'a> {
    /// `import { x } from "mod"`
    Name(NameSpan<'a>),
    /// `import * as ns from "mod"`
    NamespaceObject,
    /// `import defaultExport from "mod"`
    Default(Span),
}

impl ImportImportName<'_> {
    /// Is `default`
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default(_))
    }

    /// Is namespace
    pub fn is_namespace_object(&self) -> bool {
        matches!(self, Self::NamespaceObject)
    }
}

/// [`ExportEntry`](https://tc39.es/ecma262/#exportentry-record)
///
/// Describes a single exported binding from a module. Named export statements that contain more
/// than one binding produce multiple ExportEntry records.
///
/// ## Examples
///
/// ```ts
/// // foo's ExportEntry nas no `module_request` or `import_name.
/// //       ___ local_name
/// export { foo };
/// //       ^^^ export_name. Since there's no alias, it's the same as local_name.
///
/// // re-exports do not produce local bindings, so `local_name` is null.
/// //       ___ import_name    __ module_request
/// export { foo as bar } from "mod";
/// //              ^^^ export_name
///
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ExportEntry<'a> {
    /// Span of the import statement.
    pub statement_span: Span,

    /// Span for the entire export entry
    pub span: Span,

    /// The String value of the ModuleSpecifier of the ExportDeclaration.
    /// null if the ExportDeclaration does not have a ModuleSpecifier.
    pub module_request: Option<NameSpan<'a>>,

    /// The name under which the desired binding is exported by the module identified by `[[ModuleRequest]]`.
    /// null if the ExportDeclaration does not have a ModuleSpecifier.
    /// "all" is used for `export * as ns from "mod"`` declarations.
    /// "all-but-default" is used for `export * from "mod" declarations`.
    pub import_name: ExportImportName<'a>,

    /// The name used to export this binding by this module.
    pub export_name: ExportExportName<'a>,

    /// The name that is used to locally access the exported value from within the importing module.
    /// null if the exported value is not locally accessible from within the module.
    pub local_name: ExportLocalName<'a>,

    /// Whether the export is a TypeScript `export type`.
    ///
    /// Examples:
    ///
    /// ```ts
    /// export type * from 'mod'
    /// export type * as ns from 'mod'
    /// export type { foo }
    /// export { type foo }
    /// export type { foo } from 'mod'
    /// ```
    pub is_type: bool,
}

/// `ImportName` for `ExportEntry`
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ExportImportName<'a> {
    /// Name
    Name(NameSpan<'a>),
    /// all is used for export * as ns from "mod" declarations.
    All,
    /// all-but-default is used for export * from "mod" declarations.
    AllButDefault,
    /// the ExportDeclaration does not have a ModuleSpecifier
    #[default]
    Null,
}

/// Export Import Name
impl ExportImportName<'_> {
    /// Is all
    pub fn is_all(&self) -> bool {
        matches!(self, Self::All)
    }

    /// Is all but default
    pub fn is_all_but_default(&self) -> bool {
        matches!(self, Self::AllButDefault)
    }
}

/// `ExportName` for `ExportEntry`
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ExportExportName<'a> {
    /// Name
    Name(NameSpan<'a>),
    /// Default
    Default(Span),
    /// Null
    #[default]
    Null,
}

impl ExportExportName<'_> {
    /// Returns `true` if this is [`ExportExportName::Default`].
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default(_))
    }

    /// Returns `true` if this is [`ExportExportName::Null`].
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Attempt to get the [`Span`] of this export name.
    pub fn span(&self) -> Option<Span> {
        match self {
            Self::Name(name) => Some(name.span),
            Self::Default(span) => Some(*span),
            Self::Null => None,
        }
    }

    /// Get default export span
    /// `export default foo`
    /// `export { default }`
    pub fn default_export_span(&self) -> Option<Span> {
        match self {
            Self::Default(span) => Some(*span),
            Self::Name(name_span) if name_span.name == "default" => Some(name_span.span),
            _ => None,
        }
    }
}

/// `LocalName` for `ExportEntry`
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ExportLocalName<'a> {
    /// Name
    Name(NameSpan<'a>),
    /// `export default name_span`
    Default(NameSpan<'a>),
    /// Null
    #[default]
    Null,
}

impl<'a> ExportLocalName<'a> {
    /// `true` if this is a [`ExportLocalName::Default`].
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default(_))
    }

    /// `true` if this is a [`ExportLocalName::Null`].
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Get the bound name of this export. [`None`] for [`ExportLocalName::Null`].
    pub fn name(&self) -> Option<Atom<'a>> {
        match self {
            Self::Name(name) | Self::Default(name) => Some(name.name),
            Self::Null => None,
        }
    }
}

/// RequestedModule
#[derive(Debug, Clone, Copy)]
pub struct RequestedModule {
    /// Span of the import statement.
    pub statement_span: Span,

    /// Span
    pub span: Span,

    /// `true` if a `type` modifier was used in the import statement.
    ///
    /// ## Examples
    /// ```ts
    /// import type { foo } from "foo"; // true, `type` is on module request
    /// import { type bar } from "bar"; // false, `type` is on specifier
    /// import { baz } from "baz";      // false, no `type` modifier
    /// ```
    pub is_type: bool,

    /// `true` if the module is requested by an import statement.
    pub is_import: bool,
}

/// Dynamic import expression.
#[derive(Debug, Clone, Copy)]
pub struct DynamicImport {
    /// Span of the import expression.
    pub span: Span,
    /// Span the ModuleSpecifier, which is an expression.
    pub module_request: Span,
}

#[cfg(test)]
mod test {
    use oxc_span::Span;

    use super::{ExportExportName, ExportLocalName, ImportImportName, NameSpan};

    #[test]
    fn import_import_name() {
        let name = NameSpan::new("name".into(), Span::new(0, 0));
        assert!(!ImportImportName::Name(name.clone()).is_default());
        assert!(!ImportImportName::NamespaceObject.is_default());
        assert!(ImportImportName::Default(Span::new(0, 0)).is_default());

        assert!(!ImportImportName::Name(name.clone()).is_namespace_object());
        assert!(ImportImportName::NamespaceObject.is_namespace_object());
        assert!(!ImportImportName::Default(Span::new(0, 0)).is_namespace_object());
    }

    #[test]
    fn export_import_name() {
        let name = NameSpan::new("name".into(), Span::new(0, 0));
        assert!(!ExportExportName::Name(name.clone()).is_default());
        assert!(ExportExportName::Default(Span::new(0, 0)).is_default());
        assert!(!ExportExportName::Null.is_default());

        assert!(!ExportExportName::Name(name).is_null());
        assert!(!ExportExportName::Default(Span::new(0, 0)).is_null());
        assert!(ExportExportName::Null.is_null());
    }

    #[test]
    fn export_local_name() {
        let name = NameSpan::new("name".into(), Span::new(0, 0));
        assert!(!ExportLocalName::Name(name.clone()).is_default());
        assert!(ExportLocalName::Default(name.clone()).is_default());
        assert!(!ExportLocalName::Null.is_default());

        assert!(!ExportLocalName::Name(name.clone()).is_null());
        assert!(!ExportLocalName::Default(name.clone()).is_null());
        assert!(ExportLocalName::Null.is_null());
    }
}
