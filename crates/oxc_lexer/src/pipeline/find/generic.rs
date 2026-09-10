use super::super::chunk::{eqm, load8};

#[inline]
pub unsafe fn find1(src: *const u8, n: usize, mut i: usize, a: u8) -> usize {
    while i + 8 <= n {
        let m = eqm(load8(src, i), a);
        if m != 0 {
            return i + (m.trailing_zeros() >> 3) as usize;
        }
        i += 8;
    }
    while i < n {
        if *src.add(i) == a {
            return i;
        }
        i += 1;
    }
    n
}

#[inline]
pub unsafe fn find2(src: *const u8, n: usize, mut i: usize, a: u8, b: u8) -> usize {
    while i + 8 <= n {
        let x = load8(src, i);
        let m = eqm(x, a) | eqm(x, b);
        if m != 0 {
            return i + (m.trailing_zeros() >> 3) as usize;
        }
        i += 8;
    }
    while i < n {
        let c = *src.add(i);
        if c == a || c == b {
            return i;
        }
        i += 1;
    }
    n
}

#[inline]
pub unsafe fn find3(src: *const u8, n: usize, mut i: usize, a: u8, b: u8, c: u8) -> usize {
    while i + 8 <= n {
        let x = load8(src, i);
        let m = eqm(x, a) | eqm(x, b) | eqm(x, c);
        if m != 0 {
            return i + (m.trailing_zeros() >> 3) as usize;
        }
        i += 8;
    }
    while i < n {
        let ch = *src.add(i);
        if ch == a || ch == b || ch == c {
            return i;
        }
        i += 1;
    }
    n
}

#[inline]
pub unsafe fn find4(src: *const u8, n: usize, mut i: usize, a: u8, b: u8, c: u8, d: u8) -> usize {
    while i + 8 <= n {
        let x = load8(src, i);
        let m = eqm(x, a) | eqm(x, b) | eqm(x, c) | eqm(x, d);
        if m != 0 {
            return i + (m.trailing_zeros() >> 3) as usize;
        }
        i += 8;
    }
    while i < n {
        let x = *src.add(i);
        if x == a || x == b || x == c || x == d {
            return i;
        }
        i += 1;
    }
    n
}

macro_rules! finder {
    ($(#[$attr:meta])* $name:ident: $($needle:expr),+ $(,)?) => {
        $(#[$attr])*
        #[inline]
        pub unsafe fn $name(src: *const u8, n: usize, mut i: usize) -> usize {
            use super::super::chunk::{load8, eqm};

            while i + 8 <= n {
                let x = load8(src, i);
                let m = $(eqm(x, $needle))|+;
                if m != 0 {
                    return i + (m.trailing_zeros() >> 3) as usize;
                }
                i += 8;
            }
            while i < n {
                let c = *src.add(i);
                if $(c == $needle)||+ {
                    return i;
                }
                i += 1;
            }
            n
        }
    };
}
pub(super) use finder;
