#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
mod avx2;
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
pub use avx2::classify;

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
mod generic;
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
pub use generic::classify;

mod common;
pub use common::{misc_post, misc_pre};
