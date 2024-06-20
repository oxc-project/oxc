pub struct ParserState {
    strict: bool,
    ecma_version: u32,
}

impl ParserState {
    pub fn new(strict: bool, ecma_version: u32) -> Self {
        Self { strict, ecma_version }
    }
}
