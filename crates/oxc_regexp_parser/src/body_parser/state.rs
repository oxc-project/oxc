pub struct State {
    unicode_mode: bool,
    unicode_sets_mode: bool,
}

impl State {
    pub fn new(unicode_mode: bool, unicode_sets_mode: bool) -> Self {
        Self { unicode_mode, unicode_sets_mode }
    }

    pub fn is_unicode_mode(&self) -> bool {
        self.unicode_mode
    }
    pub fn is_unicode_sets_mode(&self) -> bool {
        self.unicode_sets_mode
    }
}
