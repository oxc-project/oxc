mod classify_luts;
mod keywords;
mod pair_luts;
mod punct1;

pub(super) use keywords::KwSet;
#[cfg(not(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
)))]
pub(super) use keywords::{is_kw_init, is_kw_init_ts};

#[cfg(not(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
)))]
pub(super) use punct1::PUNCT1;
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
))]
pub(super) use punct1::punct1_luts;

use classify_luts::{MergedLuts, WordLuts};
use keywords::Keywords;
use pair_luts::PairLuts;

#[cfg_attr(
    not(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    )),
    expect(dead_code, reason = "`merged_luts` and `word_luts` only used in SIMD implementation")
)]
pub(super) struct Tables {
    pub keywords: Keywords,
    pub merged_luts: MergedLuts,
    pub word_luts: WordLuts,
    pub pair_luts: PairLuts,
}

impl Tables {
    pub fn new() -> Tables {
        Self {
            keywords: Keywords::new(),
            merged_luts: MergedLuts::new(),
            word_luts: WordLuts::new(),
            pair_luts: PairLuts::new(),
        }
    }
}
