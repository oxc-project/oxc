pub struct State {
    pub unicode_mode: bool,
    pub unicode_sets_mode: bool,
}

impl State {
    pub fn new(unicode_mode: bool, unicode_sets_mode: bool) -> Self {
        Self { unicode_mode, unicode_sets_mode }
    }
}
