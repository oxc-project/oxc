#[derive(Debug, Default)]
pub struct ParserState {
    pub unicode_mode: bool,
    pub unicode_sets_mode: bool,
    pub n_flag: bool,
}

impl ParserState {}
