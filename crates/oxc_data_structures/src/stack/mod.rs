//! Contains the following FILO data structures:
//! - [`Stack`]: A growable stack
//! - [`SparseStack`]: A stack that can have empty entries
//! - [`NonEmptyStack`]: A growable stack that can never be empty, allowing for more efficient
//!  operations
mod capacity;
mod common;
mod non_empty;
mod non_null;
mod sparse;
mod standard;

use capacity::StackCapacity;
use common::StackCommon;
pub use non_empty::NonEmptyStack;
use non_null::NonNull;
pub use sparse::SparseStack;
pub use standard::Stack;
