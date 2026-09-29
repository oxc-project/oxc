mod keywords;
mod pair_luts;

pub(super) use keywords::{KwSet, is_kw_init, is_kw_init_ts};

use keywords::Keywords;
use pair_luts::PairLuts;

pub(super) struct Tables {
    pub keywords: Keywords,

    #[cfg_attr(
        not(all(
            target_arch = "x86_64",
            target_feature = "avx2",
            target_feature = "bmi2",
            target_feature = "popcnt"
        )),
        expect(dead_code, reason = "only used in SIMD implementation")
    )]
    pub pair_luts: PairLuts,
}

impl Tables {
    pub fn new() -> Tables {
        Self { keywords: Keywords::new(), pair_luts: PairLuts::new() }
    }
}
