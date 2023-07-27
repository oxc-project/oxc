#![allow(clippy::redundant_pub_crate)]
mod adapter;
mod edges;
mod entrypoints;
mod properties;
mod util;
mod vertex;

pub use adapter::{schema, Adapter};

#[cfg(test)]
mod tests;
