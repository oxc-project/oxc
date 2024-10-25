use proc_macro::TokenStream;
use syn::{parse_macro_input, Item};

mod ast;

/// This attribute serves two purposes.
/// First, it is a marker for our `ast_tools` to detect AST types.
/// Secondly, it generates the following code:
///
/// * Prepend `#[repr(C)]` to structs
/// * Prepend `#[repr(C, u8)]` to fieldful enums e.g. `enum E { X: u32, Y: u8 }`
/// * Prepend `#[repr(u8)]` to unit (fieldless) enums e.g. `enum E { X, Y, Z, }`
/// * Prepend `#[derive(oxc_ast_macros::Ast)]` to all structs and enums
/// * Add assertions that traits used in `#[generate_derive(...)]` are in scope.
///
/// It also allows the usage of these helper attributes via deriving a "no-op" derive macro.
///
/// # Generator Attributes:
///
/// ## `#[scope(...)]`:
///
/// This attribute can be used in 2 places:
/// ### On `struct`/`enum` items:
/// When this attribute comes before an AST type definition it accepts 3 optional arguments.
/// 1. `flags(expr)`: It accepts an expression that would evaluate to `ScopeFlags`. It is used to annotate scope flags of the AST type.
/// 2. `if(expr)`: It accepts an expression that would evaluate to `bool` used for conditional scope creation.
/// 3. `strict_if(expr)`: It accepts an expression that would evaluate to `bool`, If this value is `true` the created scope would be `strict`.
///
/// NOTE: All these expressions can use `self` to access the current node they are getting executed on via an immutable reference.
///
/// ### On `struct` fields:
/// At this position this attribute can only have one shape: `#[scope(enter_before)]`.
/// It marks where `Visit::enter_scope` events should be fired for this AST type.
///
/// ## `#[visit(...)]`:
///
/// This attribute can only occur on `struct` fields, Or `enum` attributes.
/// It accepts 4 optional arguments.
///     1. `as(ident)`: It accepts an identifier, our generators would treat the type of this field/variant as if they were called as the given identifier.
///     2. `args(arg = expr)`: It accepts an argument name and an expression. Currently it only
///        accepts one argument.
///        a. `args(flags = expr)`: `expr` is an expression that would evaluate to `ScopeFlags`, This argument can only be used at places where the AST type is `Function`.
///     3. `enter_before`: It marks where this AST type should fire `Visit::enter_node` events.
///     4. `ignore`: It would ignore this field/variant in visits.
///
/// ## `#[span]`:
///
/// This attribute can be used to hint to the `ast_tools` which field should be used to obtain the span of this AST type.
///
/// ## `#[generate_derive(...)]`
///
/// This attribute has the same spirit as the `#[derive(...)]` macro, It is used to derive traits for the types.
/// However, Instead of expanding the derive at compile-time, We do this process on PR submits via `ast_tools` code generation.
/// These derived implementations would be output in the `crates/oxc_ast/src/generated` directory.
///
/// # Derive Helper Attributes:
///
/// These are helper attributes that are only meaningful when their respective trait is derived via `generate_derive`.
///
/// ## `#[clone_in(default)]`
///
/// This attribute is only used by `CloneIn` derive.
/// `struct` fields marked with this attribute at cloning will use the `Default::default()` value instead of `CloneIn::clone_in` to initialize.
///
/// # Mocked attributes:
///
/// These are just here to remove the need for boilerplate `#[cfg_attr(...)]`. If their actual trait is derived they would consume these, Otherwise, Our mock attributes will prevent compile errors.
///
/// 1. `serde`
/// 2. `tsify`
#[proc_macro_attribute]
pub fn ast(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);
    let expanded = ast::ast(&input);
    TokenStream::from(expanded)
}

/// Dummy derive macro for a non-existent trait `Ast`.
///
/// Does not generate any code.
/// The only purpose is to allow the occurrence of helper attributes used with the `tasks/ast_tools`.
///
/// Read [`macro@ast`] for further details.
#[proc_macro_derive(Ast, attributes(scope, visit, span, generate_derive, clone_in, estree, tsify))]
pub fn ast_derive(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
