#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
mod avx2;
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
pub(super) use avx2::{scan_block_comment, scan_line_comment};

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
mod generic;
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
pub(super) use generic::{scan_block_comment, scan_line_comment};

mod common;
pub(super) use common::{scan_ident_esc, scan_number, scan_quoted, scan_regex, scan_tmpl_text};
