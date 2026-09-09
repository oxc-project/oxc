#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
mod avx2;
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
pub use avx2::{build_spans, compress, lanes_post};

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
mod generic;
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
pub use generic::{build_spans, compress, lanes_post};

mod common;
pub use common::write_sentinels;

#[cfg(test)]
mod tests;
