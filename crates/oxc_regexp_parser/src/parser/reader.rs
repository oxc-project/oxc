pub struct Reader<'a> {
    r_impl: Box<dyn ReaderImpl>,
    source: &'a str,
    pub idx: usize,
    end: usize,
    pub c1: Option<char>,
    w1: usize,
    c2: Option<char>,
    w2: usize,
    c3: Option<char>,
    w3: usize,
    c4: Option<char>,
}

impl<'a> Reader<'a> {
    pub fn new() -> Self {
        Self {
            source: "",
            idx: 0,
            end: 0,
            c1: None,
            w1: 0,
            c2: None,
            w2: 0,
            c3: None,
            w3: 0,
            c4: None,
            r_impl: Box::new(LegacyImpl),
        }
    }

    pub fn reset(&mut self, source: &'a str, start: usize, end: usize, u_flag: bool) {
        self.source = source;
        self.end = end;
        if u_flag {
            self.r_impl = Box::new(UnicodeImpl);
        } else {
            self.r_impl = Box::new(LegacyImpl);
        }
        self.rewind(start);
    }

    pub fn rewind(&mut self, index: usize) {
        self.idx = index;
        self.c1 = self.r_impl.at(self.source, self.end, index);
        self.w1 = self.r_impl.width(self.c1);
        self.c2 = self.r_impl.at(self.source, self.end, index + self.w1);
        self.w2 = self.r_impl.width(self.c2);
        self.c3 = self.r_impl.at(self.source, self.end, index + self.w1 + self.w2);
        self.w3 = self.r_impl.width(self.c3);
        self.c4 = self.r_impl.at(self.source, self.end, index + self.w1 + self.w2 + self.w3);
    }

    pub fn advance(&mut self) {
        if self.c1.is_some() {
            self.idx += self.w1;
            self.c1 = self.c2;
            self.w1 = self.c1.map_or(0, |c| c.len_utf8());
            self.c2 = self.c3;
            self.w2 = self.c2.map_or(0, |c| c.len_utf8());
            self.c3 = self.c4;
            self.w3 = self.c3.map_or(0, |c| c.len_utf8());
            self.c4 =
                self.r_impl.at(self.source, self.end, self.idx + self.w1 + self.w2 + self.w3);
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

    pub fn eat2(&mut self, cp1: char, cp2: char) -> bool {
        if self.c1 == Some(cp1) && self.c2 == Some(cp2) {
            self.advance();
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn eat3(&mut self, cp1: char, cp2: char, cp3: char) -> bool {
        if self.c1 == Some(cp1) && self.c2 == Some(cp2) && self.c3 == Some(cp3) {
            self.advance();
            self.advance();
            self.advance();
            true
        } else {
            false
        }
    }

    fn char_at(&self, index: usize) -> Option<char> {
        if index < self.end {
            self.source[index..].chars().next()
        } else {
            None
        }
    }
}

trait ReaderImpl {
    fn at(&self, s: &str, end: usize, i: usize) -> Option<char>;
    fn width(&self, c: Option<char>) -> usize;
}

struct LegacyImpl;
/// Used with `u` flag
struct UnicodeImpl;

impl ReaderImpl for LegacyImpl {
    fn at(&self, s: &str, end: usize, i: usize) -> Option<char> {
        if i < end {
            return s.chars().nth(i);
        }
        None
    }

    fn width(&self, _c: Option<char>) -> usize {
        1
    }
}

impl ReaderImpl for UnicodeImpl {
    fn at(&self, s: &str, end: usize, i: usize) -> Option<char> {
        if i < end {
            return s[i..].chars().next();
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
