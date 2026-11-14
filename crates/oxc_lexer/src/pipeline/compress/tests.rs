use std::iter;

use oxc_span::Span;

use oxc_simd::{BASELINE, DynCpu, test_cpus};

use crate::pipeline::{bitmap::Bitmap, kinds::Kinds};

use crate::token::{TRIVIA_MAX, TRIVIA_MIN, TokenKind, tk};

use super::{build_spans, compress_blocks, init_pair_luts};

#[test_cpus]
fn compress_blocks_matches_scalar_reference(cpu: DynCpu) {
    // Required before calling `compress_blocks`
    init_pair_luts(cpu);

    let mut cases: Vec<Vec<u64>> = Vec::new();
    let all_pairs: Vec<u64> = (0..65536u64)
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|c| c.iter().enumerate().fold(0u64, |w, (i, v)| w | (v << (16 * i))))
        .collect();
    cases.push(all_pairs);
    let mut s = 0x9e37_79b9_7f4a_7c15u64;
    for _ in 0..3 {
        cases.push(iter::repeat_with(|| xorshift(&mut s)).take(512).collect());
    }
    cases.push(
        iter::repeat_with(|| xorshift(&mut s) & xorshift(&mut s) & xorshift(&mut s))
            .take(512)
            .collect(),
    );
    cases.push(
        iter::repeat_with(|| xorshift(&mut s) | xorshift(&mut s) | xorshift(&mut s))
            .take(512)
            .collect(),
    );
    cases.push(vec![!0u64; 64]);
    cases.push({
        let mut v = vec![0u64; 64];
        v[63] = 1u64 << 63;
        v
    });

    for st in &mut cases {
        let nb = st.len();
        let n = nb * 64;
        let mut kind = vec![0u8; n];
        let mut ks = 0xdead_beef_cafe_f00du64;
        for b in kind.iter_mut() {
            let byte = (xorshift(&mut ks) & 0xff) as u8;
            *b = TokenKind::from_u8(byte, BASELINE).map_or(tk!(Ident), |kind| kind as u8);
        }
        let mut starts = vec![0u32; n + 64];
        let mut kinds = vec![0u8; n + 64];
        let m = unsafe {
            compress_blocks(
                Bitmap::new(st),
                Kinds::new(&mut kind),
                0,
                nb,
                starts.as_mut_ptr(),
                kinds.as_mut_ptr(),
                cpu,
            )
        };
        let mut rs: Vec<u32> = Vec::new();
        let mut rk: Vec<u8> = Vec::new();
        for (b, &w0) in st.iter().enumerate() {
            let mut w = w0;
            while w != 0 {
                let bit = w.trailing_zeros() as usize;
                w &= w - 1;
                rs.push((b * 64 + bit) as u32);
                rk.push(kind[b * 64 + bit]);
            }
        }
        assert_eq!(m, rs.len(), "token count mismatch (nb={nb})");
        assert_eq!(&starts[..m], &rs[..], "starts mismatch");
        assert_eq!(&kinds[..m], &rk[..], "kinds mismatch");
    }
}

#[test_cpus]
fn build_spans_matches_scalar_reference(cpu: DynCpu) {
    let mut s = 0x2545_f491_4f6c_dd1du64;
    // From no trivia to all trivia, so the groups of 8 see a wide range of keep masks
    for trivia_per_8 in 0..=8u64 {
        for m in [0, 1, 7, 8, 9, 15, 16, 17, 63, 64, 65, 1000] {
            let kinds: Vec<u8> = iter::repeat_with(|| {
                if xorshift(&mut s) % 8 < trivia_per_8 {
                    TRIVIA_MIN + (xorshift(&mut s) % u64::from(TRIVIA_MAX - TRIVIA_MIN + 1)) as u8
                } else {
                    (xorshift(&mut s) & 0xff) as u8
                }
            })
            .take(m)
            .collect();
            let mut pos = vec![0u32; m + 1];
            for j in 1..=m {
                pos[j] = pos[j - 1] + 1 + (xorshift(&mut s) % 8) as u32;
            }

            let mut spans = vec![Span::default(); m + 16];
            let mut sig_kinds = vec![0u8; m + 16];
            let w = unsafe {
                build_spans(
                    kinds.as_ptr(),
                    pos.as_ptr(),
                    m,
                    spans.as_mut_ptr(),
                    sig_kinds.as_mut_ptr(),
                    cpu,
                )
            };

            let kept: Vec<usize> = (0..m)
                .filter(|&j| {
                    let k = kinds[j];
                    !(TRIVIA_MIN..=TRIVIA_MAX).contains(&k) || k == tk!(Hashbang)
                })
                .collect();
            let expected_spans: Vec<Span> =
                kept.iter().map(|&j| Span::new(pos[j], pos[j + 1])).collect();
            let expected_kinds: Vec<u8> = kept.iter().map(|&j| kinds[j]).collect();
            assert_eq!(w, kept.len(), "kept count (m={m}, trivia_per_8={trivia_per_8})");
            assert_eq!(&spans[..w], &expected_spans[..], "spans (m={m})");
            assert_eq!(&sig_kinds[..w], &expected_kinds[..], "kinds (m={m})");
        }
    }
}

fn xorshift(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}
