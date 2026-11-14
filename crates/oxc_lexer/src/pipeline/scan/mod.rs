use oxc_simd::{Cpu, DynCpu, cpu_multiversion};

mod avx2;
mod common;
mod generic;
mod neon;

#[cfg(test)]
mod tests;

pub(super) use common::{scan_ident_esc, scan_number, scan_quoted, scan_regex, scan_tmpl_text};

use avx2::{scan_block_comment_avx2, scan_line_comment_avx2};
use generic::{scan_block_comment_generic, scan_line_comment_generic};
use neon::{scan_block_comment_neon, scan_line_comment_neon};

use crate::pipeline::source::Source;

#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn scan_block_comment(
    src: Source,
    i: usize,
    cpu: impl Cpu,
) -> (usize, bool, i64) {
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => scan_block_comment_avx2(src.as_ptr(), src.len(), i, cpu),
        DynCpu::Neon(cpu) => scan_block_comment_neon(src.as_ptr(), src.len(), i, cpu),
        DynCpu::Generic(_) => scan_block_comment_generic(src.as_ptr(), src.len(), i),
    }
}

/// Scan a `//` line comment to its LineTerminator (LF, CR, or LS/PS),
/// tracking the first `@license` / `@preserve` position for comment metadata.
#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn scan_line_comment(src: Source, i: usize, cpu: impl Cpu) -> (usize, i64) {
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => scan_line_comment_avx2(src.as_ptr(), src.len(), i, cpu),
        DynCpu::Neon(cpu) => scan_line_comment_neon(src.as_ptr(), src.len(), i, cpu),
        DynCpu::Generic(_) => scan_line_comment_generic(src.as_ptr(), src.len(), i),
    }
}
