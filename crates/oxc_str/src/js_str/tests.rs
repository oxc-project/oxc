use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use oxc_allocator::{Allocator, CloneIn, Dummy};
use oxc_data_structures::types::implements;

use crate::{JSChar, JSStr, JSStrBuilder, Str};

fn from_utf16_in<'a>(units: &[u16], allocator: &'a Allocator) -> JSStr<'a> {
    let mut builder = JSStrBuilder::with_capacity_in(units.len(), allocator);
    builder.push_utf16(units);
    builder.into_js_str()
}

/// Decode the input independently with Rust's UTF-16 decoder. Valid scalar
/// values use std's UTF-8 encoder; errors supply the exact unpaired code unit.
fn assert_value(value: JSStr<'_>, units: &[u16]) {
    let mut expected_points = Vec::new();
    let mut expected_bytes = Vec::new();
    let mut has_lone = false;
    for decoded in char::decode_utf16(units.iter().copied()) {
        match decoded {
            Ok(c) => {
                expected_points.push(c as u32);
                expected_bytes.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
            }
            Err(error) => {
                let unit = error.unpaired_surrogate();
                expected_points.push(u32::from(unit));
                // All surrogate encodings have the prefix ED and two
                // continuation bytes carrying the remaining ten bits.
                expected_bytes.extend([
                    0xED,
                    0x80 | ((unit >> 6) & 0x3F) as u8,
                    0x80 | (unit & 0x3F) as u8,
                ]);
                has_lone = true;
            }
        }
    }
    assert_eq!(value.encode_utf16().collect::<Vec<_>>(), units);
    assert_eq!(value.chars().map(JSChar::to_u32).collect::<Vec<_>>(), expected_points);
    assert_eq!(value.as_bytes(), expected_bytes);
    assert_eq!(value.len(), expected_bytes.len());
    assert_eq!(value.len_utf16(), units.len());
    assert_eq!(value.has_lone_surrogate(), has_lone);
    let utf8 = String::from_utf16(units).ok();
    assert_eq!(value.as_str(), utf8.as_deref());
}

#[test]
fn layouts_and_traits() {
    assert_eq!(size_of::<JSChar>(), 4);
    assert_eq!(size_of::<JSStr<'_>>(), if cfg!(target_pointer_width = "64") { 16 } else { 12 });
    assert_eq!(size_of::<Option<JSStr<'_>>>(), size_of::<JSStr<'_>>());
    assert_eq!(align_of::<JSStr<'_>>(), align_of::<usize>());
    assert!(implements!(JSStr: Send));
    assert!(implements!(JSStr: Sync));
    assert!(implements!(JSStr: Copy));
    assert!(!implements!(JSStr: std::fmt::Display));
    assert!(!implements!(JSStr: AsRef<str>));
    assert!(!implements!(JSStr: std::ops::Deref));
    assert!(!implements!(JSStr: Ord));
    assert!(!implements!(JSStrBuilder: Send));
}

#[test]
fn js_char_range_and_encoding() {
    assert_eq!(JSChar::from_u32(0x11_0000), None);
    assert_eq!(JSChar::from_u32(u32::MAX), None);
    let boundaries = [
        0, 0x7F, 0x80, 0x7FF, 0x800, 0xD7FF, 0xD800, 0xDBFF, 0xDC00, 0xDFFF, 0xE000, 0xFFFF,
        0x10000, 0x10_FFFF,
    ];
    let values: Box<dyn Iterator<Item = u32>> =
        if cfg!(miri) { Box::new(boundaries.into_iter()) } else { Box::new(0..=0x10_FFFF) };
    let mut allocator = Allocator::new();
    for value in values {
        allocator.reset();
        let js_char = JSChar::from_u32(value).unwrap();
        assert_eq!(js_char.to_u32(), value);
        assert_eq!(js_char.to_char(), char::from_u32(value));
        let mut builder = JSStrBuilder::new_in(&allocator);
        builder.push_js_char(js_char);
        if let Some(c) = char::from_u32(value) {
            assert_eq!(builder.into_js_str().as_str(), Some(c.encode_utf8(&mut [0; 4]) as &str));
        } else {
            assert_value(builder.into_js_str(), &[u16::try_from(value).unwrap()]);
        }
    }
}

#[test]
fn borrowing_and_arena_conversions() {
    const EMPTY: JSStr<'_> = JSStr::empty();
    assert_value(EMPTY, &[]);
    assert_value(JSStr::from("a𐀀�"), &[0x61, 0xD800, 0xDC00, 0xFFFD]);
    let source = String::from("hello\0é😀");
    let value = JSStr::from(source.as_str());
    assert_eq!(value.as_str().unwrap().as_ptr(), source.as_ptr());
    assert_eq!(JSStr::from(Str::from(source.as_str())), value);
    assert_eq!(value, source.as_str());
    assert_eq!(source.as_str(), value);
    assert_eq!(value, *source.as_str());
    assert_eq!(*source.as_str(), value);
    let allocator = Allocator::new();
    let copied = JSStr::from_str_in(&source, &&allocator);
    assert_eq!(copied, value);
    assert_ne!(copied.as_bytes().as_ptr(), source.as_ptr());
    assert_eq!(JSStr::dummy(&allocator), JSStr::empty());
    assert!(JSStr::empty().is_empty());
}

#[test]
fn clone_survives_original_arena() {
    let destination = Allocator::new();
    let units = [0xD800, 0x61, 0xDC00, 0xD83D, 0xDE00];
    let cloned = {
        let source = Allocator::new();
        let value = from_utf16_in(&units, &source);
        let cloned = value.clone_in(&destination);
        assert_ne!(value.as_bytes().as_ptr(), cloned.as_bytes().as_ptr());
        cloned
    };
    assert_value(cloned, &units);
}

#[test]
fn empty_builders_and_into_js_str_do_not_allocate() {
    let allocator = Allocator::new();
    let before = allocator.used_bytes();
    assert_eq!(JSStrBuilder::new_in(&allocator).into_js_str(), JSStr::empty());
    assert_eq!(JSStrBuilder::with_capacity_in(0, &allocator).into_js_str(), JSStr::empty());
    assert_eq!(allocator.used_bytes(), before);

    let mut builder = JSStrBuilder::new_in(&allocator);
    builder.push_code_unit(0xD800);
    let before = allocator.used_bytes();
    assert_value(builder.into_js_str(), &[0xD800]);
    assert_eq!(allocator.used_bytes(), before);

    let mut builder = JSStrBuilder::with_capacity_in(64, &allocator);
    builder.push_str("hello");
    builder.push_code_unit(0xD800);
    let before = allocator.used_bytes();
    assert_value(builder.into_js_str(), &[0x68, 0x65, 0x6C, 0x6C, 0x6F, 0xD800]);
    assert_eq!(allocator.used_bytes(), before);
}

#[test]
fn boundary_pairing_and_empty_appends() {
    let allocator = Allocator::new();
    let lead = from_utf16_in(&[0xD800], &allocator);
    let trail = from_utf16_in(&[0xDC00], &allocator);
    let mut builder = JSStrBuilder::new_in(&allocator);
    builder.push_js_str(lead);
    builder.push_str("");
    builder.push_utf16(&[]);
    builder.push_js_str(JSStr::empty());
    builder.push_js_str(trail);
    assert_eq!(builder.into_js_str(), JSStr::from("𐀀"));

    for units in [
        vec![0xD800, 0xD800],
        vec![0xDC00, 0xD800],
        vec![0xD800, 0x61],
        vec![0xD800, 0xDC00],
        vec![0xD800, 0xDC00, 0xD800],
    ] {
        let mut builder = JSStrBuilder::new_in(&allocator);
        for &unit in &units {
            builder.push_js_str(from_utf16_in(&[unit], &allocator));
        }
        assert_value(builder.into_js_str(), &units);
    }
}

#[test]
fn every_short_partition() {
    let alphabet: &[u16] = if cfg!(miri) {
        &[0x61, 0xD800, 0xDC00]
    } else {
        &[
            0, 0x7F, 0x80, 0x7FF, 0x800, 0xD7FF, 0xD800, 0xDBFF, 0xDC00, 0xDFFF, 0xE000, 0xFFFD,
            0xFFFF,
        ]
    };
    let mut allocator = Allocator::new();
    for &a in alphabet {
        for &b in alphabet {
            for &c in alphabet {
                allocator.reset();
                let units = [a, b, c];
                for first in 0..=3 {
                    for second in first..=3 {
                        let mut builder = JSStrBuilder::new_in(&allocator);
                        for part in [&units[..first], &units[first..second], &units[second..]] {
                            builder.push_js_str(from_utf16_in(part, &allocator));
                        }
                        assert_value(builder.into_js_str(), &units);
                    }
                }
            }
        }
    }
}

fn concat<'a>(allocator: &'a Allocator, a: JSStr<'_>, b: JSStr<'_>) -> JSStr<'a> {
    let mut builder = JSStrBuilder::new_in(allocator);
    builder.push_js_str(a);
    builder.push_js_str(b);
    builder.into_js_str()
}

#[test]
fn randomized_appends_and_associativity() {
    let mut seed = 0xA930_154Bu32;
    let mut next = || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        seed
    };
    let mut allocator = Allocator::new();
    for _ in 0..if cfg!(miri) { 12 } else { 2000 } {
        allocator.reset();
        let mut expected = Vec::new();
        let mut builder = JSStrBuilder::new_in(&allocator);
        for _ in 0..if cfg!(miri) { 6 } else { 32 } {
            let len = next() as usize % 9;
            let units = std::iter::repeat_with(|| {
                let value = next();
                if value.is_multiple_of(4) {
                    0xD800 | u16::try_from((value >> 8) & 0x7FF).unwrap()
                } else {
                    u16::try_from((value >> 8) & 0xFFFF).unwrap()
                }
            })
            .take(len)
            .collect::<Vec<_>>();
            match next() % 3 {
                0 => builder.push_utf16(&units),
                1 => builder.push_js_str(from_utf16_in(&units, &allocator)),
                _ => {
                    for decoded in char::decode_utf16(units.iter().copied()) {
                        match decoded {
                            Ok(c) => builder.push(c),
                            Err(error) => builder.push_code_unit(error.unpaired_surrogate()),
                        }
                    }
                }
            }
            expected.extend_from_slice(&units);
        }
        let value = builder.into_js_str();
        assert_value(value, &expected);
        let n = expected.len();
        let a = from_utf16_in(&expected[..n / 3], &allocator);
        let b = from_utf16_in(&expected[n / 3..2 * n / 3], &allocator);
        let c = from_utf16_in(&expected[2 * n / 3..], &allocator);
        assert_eq!(concat(&allocator, concat(&allocator, a, b), c), value);
        assert_eq!(concat(&allocator, a, concat(&allocator, b, c)), value);
    }
}

#[test]
fn growth_with_interleaved_allocations() {
    let allocator = Allocator::new();
    let mut builder = JSStrBuilder::with_capacity_in(1, &allocator);
    let mut expected = Vec::new();
    for _ in 0..64 {
        let units = [0xD800, 0xDC00, 0x61, 0xDFFF, 0xD800];
        builder.push_utf16(&units);
        expected.extend(units);
        allocator.alloc([42u8; 128]);
    }
    assert_value(builder.into_js_str(), &expected);
}

#[test]
fn equality_hash_and_debug() {
    let allocator = Allocator::new();
    let units = [0xD800, 0xDC00, 0xD800, 0x61, 0xFFFD];
    let a = from_utf16_in(&units, &allocator);
    let b = concat(
        &allocator,
        from_utf16_in(&units[..1], &allocator),
        from_utf16_in(&units[1..], &allocator),
    );
    let hash = |value: JSStr<'_>| {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    };
    assert_eq!(a, b);
    assert_eq!(hash(a), hash(b));
    assert_ne!(a, JSStr::from("𐀀�d800a�"));
    assert_ne!(a, "𐀀�d800a�");
    assert_eq!(format!("{a:?}"), "\"𐀀\\ud800a�\"");
    assert_eq!(format!("{:?}", JSStr::from("\"\n\\")), "\"\\\"\\n\\\\\"");
}

#[test]
fn cloned_iterators_and_fused_end() {
    let allocator = Allocator::new();
    let value = from_utf16_in(&[0xD800, 0xDC00, 0xDFFF], &allocator);
    let mut chars = value.chars();
    assert_eq!(chars.size_hint(), (2, Some(7)));
    assert_eq!(chars.next().unwrap().to_u32(), 0x10000);
    assert_eq!(chars.clone().next().unwrap().to_u32(), 0xDFFF);
    assert_eq!(chars.next().unwrap().to_u32(), 0xDFFF);
    assert_eq!(chars.next(), None);
    assert_eq!(chars.next(), None);
    let mut units = value.encode_utf16();
    assert_eq!(units.next(), Some(0xD800));
    assert_eq!(units.clone().collect::<Vec<_>>(), [0xDC00, 0xDFFF]);
    assert_eq!(units.next(), Some(0xDC00));
    assert_eq!(units.next(), Some(0xDFFF));
    assert_eq!(units.next(), None);
    assert_eq!(units.next(), None);
}

#[test]
#[should_panic(expected = "capacity overflow")]
fn reject_capacity_overflow() {
    let allocator = Allocator::new();
    let _ = JSStrBuilder::with_capacity_in(usize::MAX, &allocator);
}
