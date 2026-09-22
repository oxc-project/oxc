#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
))]
mod avx2;
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
))]
use avx2::define_find_function;
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
))]
pub(super) use avx2::{bracket_bits, find1, find2, find3, find4};

#[cfg(not(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
)))]
mod generic;
#[cfg(not(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
)))]
use generic::define_find_function;
#[cfg(not(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
)))]
pub(super) use generic::{bracket_bits, find1, find2, find3, find4};

mod common;
pub(super) use common::{
    find_jsx_tag, find_jsx_text, find_line_terminator, find_opener, find_opener_jsx5,
    find_opener_jsx7, find_opener6, find_regex, find_tmpl, unicode_ws_len,
};
