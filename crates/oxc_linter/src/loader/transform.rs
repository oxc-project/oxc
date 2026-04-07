use crate::loader::{PossibleParseResult, parse_vue_source};
use oxc_allocator::Allocator;

/// File extensions that has similar syntax or based on JS/TS, (e.g. Vue SFCs)
/// and can be transformed into JS/TS(X) using a specific loader.
pub const LINT_TRANSFORM_LOADER_EXTENSIONS: &[&str] = &["vue"];

pub struct TransformLoader;

impl TransformLoader {
    pub fn parse<'a>(
        allocator: &'a Allocator,
        ext: &str,
        source_text: &'a str,
    ) -> PossibleParseResult<'a> {
        match ext {
            "vue" => Some(parse_vue_source(allocator, source_text)),
            _ => None,
        }
    }
}
