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
pub(super) use avx2::{bracket_bits_avx2 as bracket_bits, find_avx2 as find};

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
pub(super) use generic::{bracket_bits_generic as bracket_bits, find_generic as find};

mod common;
pub(super) use common::{
    JSX_TAG, JSX_TEXT, OPENER, OPENER_JSX5, OPENER_JSX7, OPENER6, REGEX, TMPL,
    find_line_terminator, unicode_ws_len,
};
