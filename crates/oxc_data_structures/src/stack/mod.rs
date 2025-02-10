//! Contains the following FILO data structures:
//!
//! * [`Stack`]: A growable stack, equivalent to [`Vec`], but more efficient for stack usage (push/pop).
//! * [`NonEmptyStack`]: A growable stack that can never be empty, allowing for more efficient operations
//!   (very fast `last` / `last_mut`).
//! * [`SparseStack`]: A growable stack of `Option`s, optimized for low memory usage when many entries in
//!   the stack are empty (`None`).

mod capacity;
mod common;
mod non_empty;
mod sparse;
mod standard;

use capacity::StackCapacity;
use common::StackCommon;
pub use non_empty::NonEmptyStack;
pub use sparse::SparseStack;
pub use standard::Stack;
