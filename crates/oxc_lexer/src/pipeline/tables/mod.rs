#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
))]
use crate::pipeline::compress::PairLuts;

pub(super) use keywords::{KwSet, is_kw_init, is_kw_init_ts};

mod keywords;
use keywords::Keywords;

pub(super) struct Tables {
    pub keywords: Keywords,

    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ))]
    pub pair_luts: PairLuts,
}

impl Tables {
    pub fn new() -> Tables {
        Self {
            keywords: Keywords::new(),
            #[cfg(all(
                target_arch = "x86_64",
                target_feature = "avx2",
                target_feature = "bmi2",
                target_feature = "popcnt"
            ))]
            pair_luts: PairLuts::new(),
        }
    }
}
