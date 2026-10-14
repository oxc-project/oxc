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
use avx2::classify_avx2 as classify_impl;

#[cfg(target_arch = "aarch64")]
mod neon;
#[cfg(target_arch = "aarch64")]
use neon::classify_neon as classify_impl;

#[cfg(not(any(
    all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ),
    target_arch = "aarch64"
)))]
mod generic;
#[cfg(not(any(
    all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ),
    target_arch = "aarch64"
)))]
use generic::classify_generic as classify_impl;

#[cfg(any(
    test,
    all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ),
    target_arch = "aarch64"
))]
mod luts;
#[cfg(any(
    test,
    all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ),
    target_arch = "aarch64"
))]
mod punct1_luts;

#[cfg(any(
    test,
    not(any(
        all(
            target_arch = "x86_64",
            target_feature = "avx2",
            target_feature = "bmi2",
            target_feature = "popcnt"
        ),
        target_arch = "aarch64"
    ))
))]
mod punct1;

#[inline(never)]
pub(super) unsafe fn classify(
    ts: bool,
    src: *const u8,
    n: usize,
    nb: usize,
    word: *mut u64,
    st: *mut u64,
    kwinit: *mut u64,
    opch: *mut u64,
    digit: *mut u64,
    dot: *mut u64,
    misc: *mut u64,
    kind: *mut u8,
) {
    classify_impl(ts, src, n, word, st, kwinit, opch, digit, dot, misc, kind);

    *word.add(nb) = 0;
    *st.add(nb) = 0;
    *kwinit.add(nb) = 0;
    *opch.add(nb) = 0;
    *digit.add(nb) = 0;
    *dot.add(nb) = 0;
    *misc.add(nb) = 0;
}
