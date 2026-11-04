use crate::cpu::{Cpu, DynCpu, cpu_multiversion, to_dyn};

mod avx2;
mod common;
mod generic;

pub(super) use common::{scan_ident_esc, scan_number, scan_quoted, scan_regex, scan_tmpl_text};

use avx2::{scan_block_comment_avx2, scan_line_comment_avx2};
use generic::{scan_block_comment_generic, scan_line_comment_generic};

#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn scan_block_comment(
    src: *const u8,
    n: usize,
    i: usize,
    cpu: impl Cpu,
) -> (usize, bool, i64) {
    match to_dyn(cpu) {
        DynCpu::Avx2(cpu) => scan_block_comment_avx2(src, n, i, cpu),
        _ => scan_block_comment_generic(src, n, i),
    }
}

/// Scan a `//` line comment to its LineTerminator (LF, CR, or LS/PS),
/// tracking the first `@license` / `@preserve` position for comment metadata.
#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn scan_line_comment(
    src: *const u8,
    n: usize,
    i: usize,
    cpu: impl Cpu,
) -> (usize, i64) {
    match to_dyn(cpu) {
        DynCpu::Avx2(cpu) => scan_line_comment_avx2(src, n, i, cpu),
        _ => scan_line_comment_generic(src, n, i),
    }
}
