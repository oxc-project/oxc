use std::{ptr, str};

use oxc_simd::{Cpu, cpu_multiversion, cpu_version};
use oxc_syntax::identifier::is_identifier_start;

use crate::error::DiagCode;

use crate::pipeline::bytes::{decode_char_at, hexd};

use super::Lanes;

impl Lanes {
    /// Identifier start or digit right after a number (3in), spanning the identifier run after it.
    #[cpu_version(Generic)]
    #[cold]
    #[inline(never)]
    pub(crate) fn push_num_end_diag(&mut self, src: &[u8], number_end: usize, code: DiagCode) {
        let end = ident_start_run_end(src, number_end + 1);
        self.push_diag(number_end as u32, (end - number_end) as u32, code);
    }

    /// Non-ASCII char right after a number: only a Unicode identifier start (1π) is an error.
    #[cpu_version(Generic)]
    #[cold]
    #[inline(never)]
    pub(crate) fn push_num_end_diag_unicode(&mut self, src: &[u8], number_end: usize) {
        let Some(ch) = decode_char_at(src, number_end) else { return };
        if !is_identifier_start(ch) {
            return;
        }
        let end = ident_start_run_end(src, number_end + ch.len_utf8());
        self.push_diag(
            number_end as u32,
            (end - number_end) as u32,
            DiagCode::InvalidNumericLiteral,
        );
    }

    #[cpu_multiversion]
    #[inline]
    pub(crate) fn push_number(&mut self, src: &[u8], s: usize, e: usize, cpu: impl Cpu) {
        let len = e - s;
        if len.wrapping_sub(1) <= 7 && !(len > 1 && src[s] == b'0') {
            let keep = KEEP[len];
            let raw = unsafe { ptr::read_unaligned(src.as_ptr().add(s) as *const u64) };
            let w = raw & keep;
            let f0 = 0xF0F0_F0F0_F0F0_F0F0 & keep;
            let three = 0x3030_3030_3030_3030 & keep;
            let digits =
                (w & f0) == three && (w.wrapping_add(0x0606_0606_0606_0606 & keep) & f0) == three;
            if digits {
                let mut d = (w.wrapping_sub(three)) << ((8 - len) * 8);
                d = ((d & 0x0f00_0f00_0f00_0f00) >> 8) + (d & 0x000f_000f_000f_000f) * 10;
                d = ((d & 0x00ff_0000_00ff_0000) >> 16) + (d & 0x0000_00ff_0000_00ff) * 100;
                d = ((d & 0x0000_ffff_0000_0000) >> 32) + (d & 0x0000_0000_0000_ffff) * 10000;
                self.numbers.push(d as f64);
                return;
            }
        }
        // Only numbers the SWAR path can't parse reach the validation walk.
        let code = validate_number(src, s, e, cpu);
        if code != DiagCode::Ok {
            self.push_diag(s as u32, (e - s) as u32, code);
        }
        self.numbers.push(parse_number(src, s, e, cpu));
    }
}

static KEEP: [u64; 9] = [
    0x0000_0000_0000_0000,
    0x0000_0000_0000_00ff,
    0x0000_0000_0000_ffff,
    0x0000_0000_00ff_ffff,
    0x0000_0000_ffff_ffff,
    0x0000_00ff_ffff_ffff,
    0x0000_ffff_ffff_ffff,
    0x00ff_ffff_ffff_ffff,
    0xffff_ffff_ffff_ffff,
];

/// End of the run of identifier start chars from i, where a digit ends the run.
#[cpu_version(Generic)]
fn ident_start_run_end(s: &[u8], mut i: usize) -> usize {
    while i < s.len() {
        let b = s[i];
        if b < 0x80 {
            if !(b.is_ascii_alphabetic() || b == b'_' || b == b'$') {
                break;
            }
            i += 1;
        } else {
            let Some(c) = decode_char_at(s, i) else { break };
            if !is_identifier_start(c) {
                break;
            }
            i += c.len_utf8();
        }
    }
    i
}

#[cpu_multiversion]
fn parse_number(src: &[u8], s: usize, e: usize, cpu: impl Cpu) -> f64 {
    let mut i = s;
    let mut radix: u64 = 0;
    if e - s >= 2 && src[s] == b'0' {
        let d = src[s + 1];
        radix = match d {
            b'x' | b'X' => 16,
            b'b' | b'B' => 2,
            b'o' | b'O' => 8,
            _ => 0,
        };
        if radix != 0 {
            i = s + 2;
        }
    }
    if radix != 0 {
        let mut v: u64 = 0;
        while i < e {
            let c = src[i];
            i += 1;
            if c == b'_' {
                continue;
            }
            if c == b'n' {
                break;
            }
            match hexd(c, cpu) {
                Some(h) => v = v.wrapping_mul(radix).wrapping_add(h as u64),
                None => break,
            }
        }
        return v as f64;
    }

    let mut flt = false;
    let mut dl = 0usize;
    let mut v: u64 = 0;
    for k in s..e {
        let c = src[k];
        if c == b'.' || c == b'e' || c == b'E' {
            flt = true;
        } else if c.is_ascii_digit() {
            dl += 1;
            v = v.wrapping_mul(10).wrapping_add((c - b'0') as u64);
        }
    }
    if !flt && dl <= 19 {
        return v as f64;
    }

    let mut buf = [0u8; 512];
    let mut bl = 0usize;
    let mut k = s;
    while k < e && bl < buf.len() - 1 {
        let c = src[k];
        k += 1;
        if c == b'_' || c == b'n' {
            continue;
        }
        buf[bl] = c;
        bl += 1;
    }
    str::from_utf8(&buf[..bl]).ok().and_then(|st| st.parse::<f64>().ok()).unwrap_or(0.0)
}

#[cpu_multiversion]
#[inline]
fn radix_digit(c: u8, _cpu: impl Cpu) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// First error in the number src[s..e], or Ok when unsure, so no valid number is ever flagged.
#[cpu_multiversion]
#[inline(never)]
fn validate_number(src: &[u8], s: usize, e: usize, cpu: impl Cpu) -> DiagCode {
    let bytes = &src[s..e];
    let Some((&last, head)) = bytes.split_last() else { return DiagCode::Ok };
    let is_bigint = last == b'n';
    let body = if is_bigint { head } else { bytes };
    if body.is_empty() {
        return DiagCode::Ok;
    }

    // Radix prefix: the scanner stops before an out of range digit, so that arm is a safety net.
    if body.len() >= 2 && body[0] == b'0' {
        let radix = match body[1] | 0x20 {
            b'x' => 16u8,
            b'o' => 8,
            b'b' => 2,
            _ => 0,
        };
        if radix != 0 {
            let mut prev_us = true; // a '_' immediately after the prefix is invalid
            let mut any = false;
            for &c in &body[2..] {
                if c == b'_' {
                    if prev_us {
                        return DiagCode::InvalidNumericSeparator;
                    }
                    prev_us = true;
                } else if matches!(radix_digit(c, cpu), Some(v) if v < radix) {
                    any = true;
                    prev_us = false;
                } else {
                    return DiagCode::InvalidNumericLiteral;
                }
            }
            if any && prev_us {
                return DiagCode::InvalidNumericSeparator; // trailing '_'
            }
            if !any {
                return DiagCode::InvalidNumericLiteral; // empty radix, e.g. 0x
            }
            return DiagCode::Ok;
        }

        // Legacy octal (012, 08): no separators, no bigint suffix, exponent only after an 8 or 9.
        if matches!(body[1], b'0'..=b'9' | b'_') {
            // Only the integer part: separators in a fraction or exponent are checked below.
            if bytes.iter().take_while(|&&c| c.is_ascii_digit() || c == b'_').any(|&c| c == b'_') {
                return DiagCode::InvalidNumericSeparator;
            }
            if is_bigint {
                return DiagCode::InvalidBigint;
            }
            let run = bytes.iter().take_while(|c| c.is_ascii_digit()).count();
            match bytes.get(run) {
                // 08e1 is a valid exponent; its digits are still checked below.
                Some(&b'e' | &b'E') if bytes[..run].iter().any(|&c| c >= b'8') => {}
                Some(&b'e' | &b'E') => return DiagCode::InvalidNumericLiteral,
                _ => {}
            }
            // fall through: the valid shapes still get the generic checks
        }
    }

    // Every '_' must sit between two digits.
    let mut prev_digit = false;
    let mut exp_at: Option<usize> = None;
    for (i, &c) in body.iter().enumerate() {
        if c == b'_' {
            let next_digit = matches!(body.get(i + 1), Some(&d) if d.is_ascii_digit());
            if !prev_digit || !next_digit {
                return DiagCode::InvalidNumericSeparator;
            }
            prev_digit = false;
        } else {
            prev_digit = c.is_ascii_digit();
            if (c | 0x20) == b'e' && exp_at.is_none() {
                exp_at = Some(i); // radix bodies returned above, so this is the marker
            }
        }
    }
    // Empty exponent (1e, 1e+, .5e), never valid.
    if let Some(ep) = exp_at {
        let mut k = ep + 1;
        if matches!(body.get(k), Some(&s) if s == b'+' || s == b'-') {
            k += 1;
        }
        if !matches!(body.get(k), Some(d) if d.is_ascii_digit()) {
            return DiagCode::InvalidNumericLiteral;
        }
    }
    DiagCode::Ok
}
