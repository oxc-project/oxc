use std::iter;

use oxc_simd::{DynCpu, test_cpus};

use crate::PAD;

use super::{
    generic::{scan_block_comment_generic, scan_line_comment_generic},
    scan_block_comment, scan_line_comment,
};

/// Bytes the comment scans treat specially, and `@license` / `@preserve` fragments.
const ALPHABET: &[u8] = b"*/\n\r@ ax\xE2\x80\xA8\xA9licenspr";

#[test_cpus]
fn comment_scans_match_generic(cpu: DynCpu) {
    let mut s = 0x9e37_79b9_7f4a_7c15u64;
    for case in 0..4000 {
        let n = 1 + (xorshift(&mut s) % 160) as usize;
        let mut src: Vec<u8> =
            iter::repeat_with(|| ALPHABET[(xorshift(&mut s) % ALPHABET.len() as u64) as usize])
                .take(n)
                .collect();
        // Plant real markers, some straddling a 16 or 32 byte boundary
        if case % 3 == 0 {
            for marker in [&b"@license"[..], b"@preserve", b"*/", b"\xE2\x80\xA8"] {
                let at = (xorshift(&mut s) % n as u64) as usize;
                let end = (at + marker.len()).min(n);
                src[at..end].copy_from_slice(&marker[..end - at]);
            }
        }
        src.resize(n + PAD, 0);
        let i = (xorshift(&mut s) % n as u64) as usize;

        unsafe {
            assert_eq!(
                scan_block_comment(src.as_ptr(), n, i, cpu),
                scan_block_comment_generic(src.as_ptr(), n, i),
                "block comment, case {case}"
            );
            assert_eq!(
                scan_line_comment(src.as_ptr(), n, i, cpu),
                scan_line_comment_generic(src.as_ptr(), n, i),
                "line comment, case {case}"
            );
        }
    }
}

fn xorshift(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}
