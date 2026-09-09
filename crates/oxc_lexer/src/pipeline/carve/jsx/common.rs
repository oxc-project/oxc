#[inline(always)]
pub(super) unsafe fn wordbit(word: *const u64, p: usize) -> bool {
    (*word.add(p >> 6) >> (p & 63)) & 1 != 0
}
