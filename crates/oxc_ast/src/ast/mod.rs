//! AST Definitions

mod js;
mod jsx;
mod literal;
mod ts;

pub use self::{js::*, jsx::*, literal::*, ts::*};

// TODO: Make macro implement for multiple types simultaneously.
// e.g. `shared_enum_variants!(MemberExpression, [Expression, SimpleAssignmentTarget, ChainElement], [ ... ])`

/// Macro to allow conversion between 2 enum types where they share some of the same variants.
/// "Parent" enum contains all the "child"'s variants, plus parent contains further other variants.
/// e.g. `Statement` and `Declaration`.
///
/// # SAFETY
/// The discriminants and types of the shared variants must be identical between the 2 enums.
/// Discriminants must be explicitly defined on both types.
/// It is recommended to add the variants to both types using a macro (e.g. `add_declaration_variants!`)
/// to avoid any possibility of the 2 being out of sync.
///
/// # Example
/// NB: For illustration only - `Statement` and `Declaration` in reality share 9 variants, not 2.
///
/// ```
/// shared_enum_variants!(
///     Statement, Declaration,
///     is_declaration,
///     as_declaration, as_declaration_mut,
///     to_declaration, to_declaration_mut,
///     [VariableDeclaration, FunctionDeclaration]
/// )
/// ```
///
/// expands to:
///
/// ```
/// impl<'a> Statement<'a> {
///     /// Return if a `Statement` is a `Declaration`.
///     #[inline]
///     pub fn is_declaration(&self) -> bool {
///         match self {
///             Self::VariableDeclaration(_) | Self::FunctionDeclaration(_) => true,
///             _ => false,
///         }
///     }
///
///     /// Convert `&Statement` to `&Declaration`.
///     #[inline]
///     pub fn as_declaration(&self) -> Option<&Declaration<'a>> {
///         if self.is_declaration() {
///             Some(unsafe { &*(self as *const _ as *const Declaration) })
///         } else {
///             None
///         }
///     }
///
///     /// Convert `&mut Statement` to `&mut Declaration`.
///     #[inline]
///     pub fn as_declaration_mut(&mut self) -> Option<&mut Declaration<'a>> {
///         if self.is_declaration() {
///             Some(unsafe { &mut *(self as *mut _ as *mut Declaration) })
///         } else {
///             None
///         }
///     }
///
///     /// Convert `&Statement` to `&Declaration`.
///     /// # Panic
///     /// Panics if not convertable.
///     #[inline]
///     pub fn to_declaration(&self) -> &Declaration<'a> {
///         self.as_declaration().unwrap()
///     }
///
///     /// Convert `&mut Statement` to `&mut Declaration`.
///     /// # Panic
///     /// Panics if not convertable.
///     #[inline]
///     pub fn to_declaration_mut(&mut self) -> Option<&mut Declaration<'a>> {
///         self.as_declaration_mut().unwrap()
///     }
/// }
///
/// impl<'a> TryFrom<Statement<'a>> for Declaration<'a> {
///     type Error = ();
///
///     /// "Convert `Statement` to `Declaration`.
///     #[inline]
///     fn try_from(value: Statement<'a>) -> Result<Self, Self::Error> {
///         match value {
///             Statement::VariableDeclaration(o) => Ok(Declaration::VariableDeclaration(o)),
///             Statement::FunctionDeclaration(o) => Ok(Declaration::FunctionDeclaration(o)),
///             _ => Err(()),
///         }
///     }
/// }
///
/// impl<'a> From<Declaration<'a>> for Statement<'a> {
///     /// Convert `Declaration` to `Statement`.
///     #[inline]
///     fn from(value: Declaration<'a>) -> Self {
///         match value {
///             Declaration::VariableDeclaration(o) => Statement::VariableDeclaration(o),
///             Declaration::FunctionDeclaration(o) => Statement::FunctionDeclaration(o),
///         }
///     }
/// }
/// ```
macro_rules! shared_enum_variants {
    (
        $parent:ident, $child:ident,
        $is_child:ident,
        $as_child:ident, $as_child_mut:ident,
        $to_child:ident, $to_child_mut:ident,
        [$($variant:ident),+ $(,)?]
    ) => {
        impl<'a> $parent<'a> {
            #[doc = concat!("Return if a `", stringify!($parent), "` is a `", stringify!($child), "`.")]
            #[inline]
            pub fn $is_child(&self) -> bool {
                matches!(
                    self,
                    $(Self::$variant(_))|*
                )
            }

            #[doc = concat!("Convert `&", stringify!($parent), "` to `&", stringify!($child), "`.")]
            #[inline]
            pub fn $as_child(&self) -> Option<&$child<'a>> {
                if self.$is_child() {
                    #[allow(unsafe_code, clippy::ptr_as_ptr)]
                    // SAFETY: Transmute is safe because discriminants + types are identical between
                    // `$parent` and `$child` for $child variants
                    Some(unsafe { &*(self as *const _ as *const $child) })
                } else {
                    None
                }
            }

            #[doc = concat!("Convert `&mut ", stringify!($parent), "` to `&mut ", stringify!($child), "`.")]
            #[inline]
            pub fn $as_child_mut(&mut self) -> Option<&mut $child<'a>> {
                if self.$is_child() {
                    #[allow(unsafe_code, clippy::ptr_as_ptr)]
                    // SAFETY: Transmute is safe because discriminants + types are identical between
                    // `$parent` and `$child` for $child variants
                    Some(unsafe { &mut *(self as *mut _ as *mut $child) })
                } else {
                    None
                }
            }

            #[doc = concat!("Convert `&", stringify!($parent), "` to `&", stringify!($child), "`.")]
            #[doc = "# Panic"]
            #[doc = "Panics if not convertable."]
            #[inline]
            pub fn $to_child(&self) -> &$child<'a> {
                self.$as_child().unwrap()
            }

            #[doc = concat!("Convert `&mut ", stringify!($parent), "` to `&mut ", stringify!($child), "`.")]
            #[doc = "# Panic"]
            #[doc = "Panics if not convertable."]
            #[inline]
            pub fn $to_child_mut(&mut self) -> &mut $child<'a> {
                self.$as_child_mut().unwrap()
            }
        }

        impl<'a> TryFrom<$parent<'a>> for $child<'a> {
            type Error = ();

            #[doc = concat!("Convert `", stringify!($parent), "` to `", stringify!($child), "`.")]
            #[inline]
            fn try_from(value: $parent<'a>) -> Result<Self, Self::Error> {
                // Compiler should implement this as a check of discriminant and then zero-cost transmute,
                // as discriminants for `$parent` and `$child` are aligned
                match value {
                    $($parent::$variant(o) => Ok($child::$variant(o)),)*
                    _ => Err(())
                }
            }
        }

        impl<'a> From<$child<'a>> for $parent<'a> {
            #[doc = concat!("Convert `", stringify!($child), "` to `", stringify!($parent), "`.")]
            #[inline]
            fn from(value: $child<'a>) -> Self {
                // Compiler should implement this as zero-cost transmute as discriminants
                // for `$child` and `$parent` are aligned
                match value {
                    $($child::$variant(o) => $parent::$variant(o),)*
                    $child::Dummy => $parent::Dummy,
                }
            }
        }
    }
}
use shared_enum_variants;
