// pub struct Reader2<'a> {
//     chars: Box<dyn Iterator<Item = u32> + 'a>,
// }
// impl<'a> Reader2<'a> {
//     pub fn new(source: &'a str, unicode_mode: bool) -> Self {
//         let chars: Box<dyn Iterator<Item = u32>> = if unicode_mode {
//             Box::new(source.chars().map(|c| c as u32))
//         } else {
//             #[allow(clippy::cast_lossless)]
//             Box::new(source.encode_utf16().map(|c| c as u32))
//         };

//         Self { chars }
//     }
// }

pub struct Reader<'a> {
    r_impl: Box<dyn ReaderImpl<'a>>,
    pub idx: usize,
    pub c1: Option<char>,
    w1: usize,
    c2: Option<char>,
    w2: usize,
    c3: Option<char>,
    w3: usize,
    c4: Option<char>,
}

impl<'a> Reader<'a> {
    pub fn new(source: &'a str, unicode_mode: bool) -> Self {
        let mut reader = Self {
            idx: 0,
            c1: None,
            w1: 0,
            c2: None,
            w2: 0,
            c3: None,
            w3: 0,
            c4: None,
            r_impl: if unicode_mode {
                Box::new(UnicodeImpl::new(source))
            } else {
                Box::new(LegacyImpl::new(source))
            },
        };
        reader.rewind(0);
        reader
    }

    pub fn rewind(&mut self, index: usize) {
        self.idx = index;
        self.c1 = self.r_impl.at(index);
        self.w1 = self.r_impl.width(self.c1);
        self.c2 = self.r_impl.at(index + self.w1);
        self.w2 = self.r_impl.width(self.c2);
        self.c3 = self.r_impl.at(index + self.w1 + self.w2);
        self.w3 = self.r_impl.width(self.c3);
        self.c4 = self.r_impl.at(index + self.w1 + self.w2 + self.w3);
    }

    pub fn advance(&mut self) {
        if self.c1.is_some() {
            self.idx += self.w1;
            self.c1 = self.c2;
            self.w1 = self.w2;
            self.c2 = self.c3;
            self.w2 = self.r_impl.width(self.c2);
            self.c3 = self.c4;
            self.w3 = self.r_impl.width(self.c3);
            self.c4 = self.r_impl.at(self.idx + self.w1 + self.w2 + self.w3);
        }
    }

    pub fn eat(&mut self, cp: char) -> bool {
        if self.c1 == Some(cp) {
            self.advance();
            true
        } else {
            false
        }
    }

    // pub fn eat2(&mut self, cp1: char, cp2: char) -> bool {
    //     if self.c1 == Some(cp1) && self.c2 == Some(cp2) {
    //         self.advance();
    //         self.advance();
    //         true
    //     } else {
    //         false
    //     }
    // }

    // pub fn eat3(&mut self, cp1: char, cp2: char, cp3: char) -> bool {
    //     if self.c1 == Some(cp1) && self.c2 == Some(cp2) && self.c3 == Some(cp3) {
    //         self.advance();
    //         self.advance();
    //         self.advance();
    //         true
    //     } else {
    //         false
    //     }
    // }
}

// NOTE: I'm not sure this implementation is required for Rust...
trait ReaderImpl<'a> {
    fn at(&self, i: usize) -> Option<char>;
    fn width(&self, c: Option<char>) -> usize;
}

struct LegacyImpl {
    chars: Vec<u32>,
    end: usize,
}
/// Used when `u` or `v` flag is set
struct UnicodeImpl {
    chars: Vec<u32>,
    end: usize,
}

impl LegacyImpl {
    fn new(source: &str) -> Self {
        #[allow(clippy::cast_lossless)]
        let chars = source.encode_utf16().map(|c| c as u32).collect::<Vec<_>>();
        let end = chars.len();
        Self { chars, end }
    }
}

impl<'a> ReaderImpl<'a> for LegacyImpl {
    fn at(&self, i: usize) -> Option<char> {
        if i < self.end {
            return self.chars.get(i).map(|&c| char::from_u32(c))?;
        }
        None
    }

    fn width(&self, _c: Option<char>) -> usize {
        1
    }
}

impl UnicodeImpl {
    fn new(source: &str) -> Self {
        let chars = source.chars().map(|c| c as u32).collect::<Vec<_>>();
        let end = chars.len();
        Self { chars, end }
    }
}

impl<'a> ReaderImpl<'a> for UnicodeImpl {
    fn at(&self, i: usize) -> Option<char> {
        if i < self.end {
            return self.chars.get(i).map(|&c| char::from_u32(c))?;
        }
        None
    }

    fn width(&self, c: Option<char>) -> usize {
        match c {
            Some(c) if c as u32 > 0xFFFF => 2,
            _ => 1,
        }
    }
}
