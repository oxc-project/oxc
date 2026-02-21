//! # React Compiler (Rust Port)
//!
//! A Rust port of Facebook's React Compiler (`babel-plugin-react-compiler`).
//!
//! The React Compiler automatically optimizes React components by analyzing code
//! and inserting memoization to reduce unnecessary re-renders.
//!
//! ## Architecture
//!
//! The compiler operates via a multi-phase pipeline:
//! 1. **Lowering**: AST → HIR (High-level Intermediate Representation)
//! 2. **SSA**: Convert HIR to Static Single Assignment form
//! 3. **Analysis**: Type inference, mutation/aliasing inference
//! 4. **Optimization**: Constant propagation, dead code elimination
//! 5. **Reactive Scopes**: Infer memoization boundaries
//! 6. **Codegen**: Reactive Function → optimized AST

pub mod compiler_error;
pub mod entrypoint;
pub mod hir;
pub mod inference;
pub mod optimization;
pub mod reactive_scopes;
pub mod ssa;
pub mod transform;
pub mod type_inference;
pub mod utils;
pub mod validation;
