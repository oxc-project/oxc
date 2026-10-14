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
use avx2::classify_impl;

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
mod neon;
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
use neon::classify_impl;

#[cfg(not(any(
    all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ),
    all(target_arch = "aarch64", target_feature = "neon")
)))]
mod generic;
#[cfg(not(any(
    all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ),
    all(target_arch = "aarch64", target_feature = "neon")
)))]
use generic::classify_impl;

#[cfg(any(
    test,
    all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ),
    all(target_arch = "aarch64", target_feature = "neon")
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
    all(target_arch = "aarch64", target_feature = "neon")
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
        all(target_arch = "aarch64", target_feature = "neon")
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
