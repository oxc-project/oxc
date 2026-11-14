// Raw kernels: the unsafe fn boundary is the reviewed surface, not each operation inside.
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]

use oxc_simd::{Cpu, DynCpu, cpu_multiversion};

mod avx2;
mod common;
mod generic;
mod neon;

#[cfg(test)]
mod tests;

pub(super) use common::{
    TemplateEnd, scan_ident_esc, scan_number, scan_quoted, scan_regex, scan_tmpl_text,
};

use avx2::{scan_block_comment_avx2, scan_line_comment_avx2};
use generic::{scan_block_comment_generic, scan_line_comment_generic};
use neon::{scan_block_comment_neon, scan_line_comment_neon};

use crate::pipeline::source::Source;

#[cpu_multiversion]
#[inline(always)]
pub(super) fn scan_block_comment(
    src: Source,
    i: usize,
    cpu: impl Cpu,
) -> (usize, bool, Option<u32>) {
    if i > src.len() {
        return (src.len(), false, None);
    }
    // SAFETY: the scans read at most 9 bytes past src.len(), inside the padding src holds.
    unsafe {
        match DynCpu::from(cpu) {
            DynCpu::Avx2(cpu) => scan_block_comment_avx2(src.as_ptr(), src.len(), i, cpu),
            DynCpu::Neon(cpu) => scan_block_comment_neon(src.as_ptr(), src.len(), i, cpu),
            DynCpu::Generic(_) => scan_block_comment_generic(src.as_ptr(), src.len(), i),
        }
    }
}

/// Scan a `//` line comment to its LineTerminator (LF, CR, or LS/PS),
/// tracking the first `@license` / `@preserve` position for comment metadata.
#[cpu_multiversion]
#[inline(always)]
pub(super) fn scan_line_comment(src: Source, i: usize, cpu: impl Cpu) -> (usize, Option<u32>) {
    if i > src.len() {
        return (src.len(), None);
    }
    // SAFETY: the scans read at most 9 bytes past src.len(), inside the padding src holds.
    unsafe {
        match DynCpu::from(cpu) {
            DynCpu::Avx2(cpu) => scan_line_comment_avx2(src.as_ptr(), src.len(), i, cpu),
            DynCpu::Neon(cpu) => scan_line_comment_neon(src.as_ptr(), src.len(), i, cpu),
            DynCpu::Generic(_) => scan_line_comment_generic(src.as_ptr(), src.len(), i),
        }
    }
}
