pub struct State {
    pub unicode_mode: bool,
    pub unicode_sets_mode: bool,
    pub named_capture_groups: bool,
}

impl State {
    pub fn new(unicode_mode: bool, unicode_sets_mode: bool, named_capture_groups: bool) -> Self {
        Self { unicode_mode, unicode_sets_mode, named_capture_groups }
    }
}
