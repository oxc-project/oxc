mod classify_luts;
mod keywords;
mod operators;
mod pair_luts;
mod punct1;

pub(super) use keywords::KwSet;
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
pub(super) use keywords::{is_kw_init, is_kw_init_ts};

pub(super) use operators::is_op_char;

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
pub(super) use punct1::PUNCT1;
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
pub(super) use punct1::{PH_A, PH_B, PH_T0, PH_T1};

use classify_luts::{MergedLuts, WordLuts};
use keywords::Keywords;
use operators::{OpMap, opch_selfcheck};
use pair_luts::PairLuts;
use punct1::punct1_hash_selfcheck;

#[cfg_attr(
    not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")),
    expect(dead_code, reason = "`merged_luts` and `word_luts` only used in SIMD implementation")
)]
pub(super) struct Tables {
    pub op: OpMap,
    pub keywords: Keywords,
    pub merged_luts: MergedLuts,
    pub word_luts: WordLuts,
    pub pair_luts: PairLuts,
}

impl Tables {
    pub fn new() -> Tables {
        let mut t = Tables {
            op: OpMap::new(),
            keywords: Keywords::new(),
            merged_luts: MergedLuts::new(),
            word_luts: WordLuts::new(),
            pair_luts: PairLuts::new(),
        };

        t.pair_luts.build();
        opch_selfcheck();
        punct1_hash_selfcheck();
        t.keywords.self_check();

        t
    }
}
