/// Flood type system (Flow type inference).
///
/// Port of `Flood/Types.ts`, `Flood/FlowTypes.ts`, `Flood/TypeErrors.ts`,
/// `Flood/TypeUtils.ts` from the React Compiler.
///
/// The Flood module implements a Flow-based type inference system that is
/// used when the `enableForest` environment config flag is enabled.
/// This is an experimental feature for more precise type tracking.
///
/// The full implementation is in `flood_types.rs`, `flood_flow_types.rs`,
/// `flood_type_errors.rs`, and `flood_type_utils.rs`.
// Re-export the key types
pub use super::flood_types::FloodTypeEnv;
