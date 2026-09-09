use crate::tables::Tables;

use super::compress;

#[test]
fn compress_matches_scalar_reference() {
    let t = Tables::new();
    let mut cases: Vec<Vec<u64>> = Vec::new();
    let all_pairs: Vec<u64> = (0..65536u64)
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|c| c.iter().enumerate().fold(0u64, |w, (i, v)| w | (v << (16 * i))))
        .collect();
    cases.push(all_pairs);
    let mut s = 0x9e37_79b9_7f4a_7c15u64;
    for _ in 0..3 {
        cases.push(core::iter::repeat_with(|| xorshift(&mut s)).take(512).collect());
    }
    cases.push(
        core::iter::repeat_with(|| xorshift(&mut s) & xorshift(&mut s) & xorshift(&mut s))
            .take(512)
            .collect(),
    );
    cases.push(
        core::iter::repeat_with(|| xorshift(&mut s) | xorshift(&mut s) | xorshift(&mut s))
            .take(512)
            .collect(),
    );
    cases.push(vec![!0u64; 64]);
    cases.push({
        let mut v = vec![0u64; 64];
        v[63] = 1u64 << 63;
        v
    });
    for st in &cases {
        let nb = st.len();
        let n = nb * 64;
        let mut kind = vec![0u8; n];
        let mut ks = 0xdead_beef_cafe_f00du64;
        for b in kind.iter_mut() {
            *b = (xorshift(&mut ks) & 0xff) as u8;
        }
        let mut starts = vec![0u32; n + 64];
        let mut kinds = vec![0u8; n + 64];
        let m = unsafe {
            compress(&t, st.as_ptr(), kind.as_ptr(), 0, nb, starts.as_mut_ptr(), kinds.as_mut_ptr())
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

fn xorshift(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}
