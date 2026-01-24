//! Compile-time identifier constants with precomputed hashes.
//!
//! These constants use `Ident::new_const` to compute hashes at compile time,
//! enabling fast hash-based comparisons for commonly used identifier names.

use crate::Ident;

// === General ===
pub const IDENT_EXPORTS: Ident<'static> = Ident::new_const("exports");
pub const IDENT_MODULE: Ident<'static> = Ident::new_const("module");
pub const IDENT_REQUIRE: Ident<'static> = Ident::new_const("require");

// === Global Objects ===
pub const IDENT_GLOBAL_THIS: Ident<'static> = Ident::new_const("globalThis");
pub const IDENT_PROCESS: Ident<'static> = Ident::new_const("process");

// === Built-in Constructors ===
pub const IDENT_ARRAY: Ident<'static> = Ident::new_const("Array");
pub const IDENT_DATE: Ident<'static> = Ident::new_const("Date");
pub const IDENT_FUNCTION: Ident<'static> = Ident::new_const("Function");
pub const IDENT_MATH: Ident<'static> = Ident::new_const("Math");
pub const IDENT_NUMBER: Ident<'static> = Ident::new_const("Number");
pub const IDENT_REGEXP: Ident<'static> = Ident::new_const("RegExp");
pub const IDENT_STRING: Ident<'static> = Ident::new_const("String");

// === Error Types ===
pub const IDENT_AGGREGATE_ERROR: Ident<'static> = Ident::new_const("AggregateError");
pub const IDENT_ERROR: Ident<'static> = Ident::new_const("Error");
pub const IDENT_TYPE_ERROR: Ident<'static> = Ident::new_const("TypeError");
