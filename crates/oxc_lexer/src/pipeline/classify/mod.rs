use oxc_simd::{Cpu, DynCpu, cpu_multiversion};

use crate::pipeline::source::Source;

mod avx2;
mod generic;
mod luts;
mod neon;
mod punct;

use avx2::classify_avx2;
use generic::classify_generic;
use neon::classify_neon;

/// The bitmaps and kinds classify fills in, before the later stages share them.
pub(super) struct Planes<'a> {
    pub token_start: &'a mut [u64],
    pub word: &'a mut [u64],
    pub keyword_initial: &'a mut [u64],
    pub op_char: &'a mut [u64],
    pub digit: &'a mut [u64],
    pub dot: &'a mut [u64],
    pub misc: &'a mut [u64],
    pub kinds: &'a mut [[u8; 64]],
}

impl Planes<'_> {
    /// The first count blocks of each plane, checked once.
    #[inline(always)]
    fn first(self, count: usize) -> Self {
        let Self { token_start, word, keyword_initial, op_char, digit, dot, misc, kinds } = self;
        Self {
            token_start: &mut token_start[..count],
            word: &mut word[..count],
            keyword_initial: &mut keyword_initial[..count],
            op_char: &mut op_char[..count],
            digit: &mut digit[..count],
            dot: &mut dot[..count],
            misc: &mut misc[..count],
            kinds: &mut kinds[..count],
        }
    }

    #[inline(always)]
    fn clear_block(&mut self, block_idx: usize) {
        let all = [
            &mut *self.token_start,
            &mut *self.word,
            &mut *self.keyword_initial,
            &mut *self.op_char,
            &mut *self.digit,
            &mut *self.dot,
            &mut *self.misc,
        ];
        for plane in all {
            plane[block_idx] = 0;
        }
    }
}

#[cpu_multiversion]
#[inline(always)]
pub(super) fn classify(src: Source, ts: bool, mut planes: Planes, cpu: impl Cpu) {
    planes.clear_block(src.block_count());
    let (blocks, _) = src.padded().as_chunks::<64>();
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => classify_avx2(ts, blocks, src.len(), planes, cpu),
        DynCpu::Neon(cpu) => classify_neon(ts, blocks, src.len(), planes, cpu),
        DynCpu::Generic(_) => classify_generic(ts, blocks, src.len(), planes),
    }
}
