//! Adversarial property test: JSStrBuilder vs a UTF-16 reference model.

use oxc_allocator::Allocator;
use oxc_str::{JSChar, JSStr, JSStrBuilder};

#[derive(Clone, Copy, Debug)]
enum Op {
    Str(&'static str),
    Char(char),
    Unit(u16),
    Js(&'static [u16]),
}

const OPS: &[Op] = &[
    Op::Str(""),
    Op::Str("a"),
    Op::Str("é😀"),
    Op::Char('b'),
    Op::Char('😀'),
    Op::Unit(0x61),
    Op::Unit(0xD800),
    Op::Unit(0xDBFF),
    Op::Unit(0xDC00),
    Op::Unit(0xDFFF),
    Op::Unit(0xFFFD),
    Op::Js(&[]),
    Op::Js(&[0xD800]),
    Op::Js(&[0xDC00]),
    Op::Js(&[0xDC00, 0xD800]),
    Op::Js(&[0xD83D, 0xDE00]),
    Op::Js(&[0xD800, 0xD800]),
    Op::Js(&[0x61, 0xD800]),
    Op::Js(&[0xDC00, 0x61, 0xD800]),
];

fn is_lead(u: u16) -> bool {
    (0xD800..=0xDBFF).contains(&u)
}
fn is_trail(u: u16) -> bool {
    (0xDC00..=0xDFFF).contains(&u)
}

/// Reference: JS string semantics. Compute code points + lone-surrogate presence
/// from the flat UTF-16 code unit sequence.
fn reference(units: &[u16]) -> (Vec<u32>, bool) {
    let mut points = Vec::new();
    let mut lone = false;
    let mut i = 0;
    while i < units.len() {
        let u = units[i];
        if is_lead(u) && i + 1 < units.len() && is_trail(units[i + 1]) {
            let cp = 0x10000 + ((u32::from(u) - 0xD800) << 10) + (u32::from(units[i + 1]) - 0xDC00);
            points.push(cp);
            i += 2;
        } else {
            if is_lead(u) || is_trail(u) {
                lone = true;
            }
            points.push(u32::from(u));
            i += 1;
        }
    }
    (points, lone)
}

fn apply_ops<'a>(ops: &[Op], allocator: &'a Allocator) -> (JSStr<'a>, Vec<u16>) {
    // Build expected UTF-16 sequence
    let mut units: Vec<u16> = Vec::new();
    for op in ops {
        match op {
            Op::Str(s) => units.extend(s.encode_utf16()),
            Op::Char(c) => {
                let mut buf = [0u16; 2];
                units.extend_from_slice(c.encode_utf16(&mut buf));
            }
            Op::Unit(u) => units.push(*u),
            Op::Js(js) => units.extend_from_slice(js),
        }
    }

    let mut builder = JSStrBuilder::new_in(allocator);
    for op in ops {
        match op {
            Op::Str(s) => builder.push_str(s),
            Op::Char(c) => builder.push_char(*c),
            Op::Unit(u) => builder.push_code_unit(*u),
            Op::Js(js) => builder.push_js_str(JSStr::from_utf16_in(js, &allocator)),
        }
    }
    (builder.finish(), units)
}

fn concat2<'a>(x: JSStr<'_>, y: JSStr<'_>, allocator: &'a Allocator) -> JSStr<'a> {
    let mut builder = JSStrBuilder::new_in(allocator);
    builder.push_js_str(x);
    builder.push_js_str(y);
    builder.finish()
}

fn code_points(value: JSStr<'_>) -> Vec<u32> {
    value.chars().map(JSChar::value).collect()
}

fn check(ops: &[Op]) {
    let allocator = Allocator::new();
    let (result, units) = apply_ops(ops, &allocator);
    let (expected_points, expected_lone) = reference(&units);

    let actual_points = code_points(result);
    assert_eq!(
        actual_points, expected_points,
        "code_points mismatch for ops {ops:?} (units {units:04X?})"
    );
    assert_eq!(
        result.has_lone_surrogate(),
        expected_lone,
        "flag mismatch for ops {ops:?} (units {units:04X?})"
    );
    let actual_units: Vec<u16> = result.encode_utf16().collect();
    assert_eq!(actual_units, units, "utf16 round-trip mismatch for ops {ops:?}");
    assert_eq!(result.utf16_len(), units.len(), "utf16_len mismatch for ops {ops:?}");
    // as_str agrees with flag; from_utf16 canonical reference agrees byte-wise
    let from16 = JSStr::from_utf16_in(&units, &&allocator);
    assert_eq!(result, from16, "byte canonical form mismatch for ops {ops:?}");
    if expected_lone {
        assert!(result.as_str().is_none());
    } else {
        let s = result.as_str().expect("no lone surrogate -> as_str Some");
        assert_eq!(s.encode_utf16().collect::<Vec<_>>(), units);
    }
}

#[test]
fn exhaustive_up_to_len_3() {
    check(&[]);
    for &a in OPS {
        check(&[a]);
        for &b in OPS {
            check(&[a, b]);
            for &c in OPS {
                check(&[a, b, c]);
            }
        }
    }
}

#[test]
fn random_len_6() {
    // Deterministic LCG over op indices
    let mut state: u64 = 0x243F_6A88_85A3_08D3;
    let mut next = move || {
        state =
            state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) as usize
    };
    for _ in 0..20000 {
        let ops: Vec<Op> = std::iter::repeat_with(|| OPS[next() % OPS.len()]).take(6).collect();
        check(&ops);
    }
}

/// Associativity at the JS value level: concat(concat(a,b),c) == concat(a,concat(b,c))
#[test]
fn concat_associativity() {
    let pieces: &[&[u16]] = &[
        &[],
        &[0x61],
        &[0xD800],
        &[0xDC00],
        &[0xD83D],
        &[0xDE00],
        &[0xD800, 0xD800],
        &[0xDC00, 0xDC00],
        &[0xDC00, 0xD800],
        &[0x61, 0xD800],
        &[0xDC00, 0x61],
    ];
    let allocator = Allocator::new();
    for &a in pieces {
        for &b in pieces {
            for &c in pieces {
                let (ja, jb, jc) = (
                    JSStr::from_utf16_in(a, &&allocator),
                    JSStr::from_utf16_in(b, &&allocator),
                    JSStr::from_utf16_in(c, &&allocator),
                );
                let left = concat2(concat2(ja, jb, &allocator), jc, &allocator);
                let right = concat2(ja, concat2(jb, jc, &allocator), &allocator);
                assert_eq!(left, right, "associativity failed: {a:04X?} {b:04X?} {c:04X?}");
                assert_eq!(left.has_lone_surrogate(), right.has_lone_surrogate());
                // And equals the flat reference
                let mut units = a.to_vec();
                units.extend_from_slice(b);
                units.extend_from_slice(c);
                let flat = JSStr::from_utf16_in(&units, &&allocator);
                assert_eq!(left, flat, "flat mismatch: {a:04X?} {b:04X?} {c:04X?}");
            }
        }
    }
}
