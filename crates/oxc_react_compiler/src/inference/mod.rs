pub mod aliasing_effects;
pub mod analyse_functions;
pub mod control_dominators;
pub mod drop_manual_memoization;
pub mod infer_mutation_aliasing_effects;
pub mod infer_mutation_aliasing_ranges;
pub mod infer_reactive_places;
pub mod inline_iife;

// Re-export FunctionSignature for convenience
pub use crate::hir::object_shape::FunctionSignature;
