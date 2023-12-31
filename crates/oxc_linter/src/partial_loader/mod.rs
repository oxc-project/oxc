use oxc_span::SourceType;

pub const LINT_PARTIAL_LOADER_EXT: &[&str] = &["vue"];

pub enum PartialLoader {
    Vue,
}

pub struct PartialLoaderValue {
    pub source_text: String,
    pub source_type: SourceType,
}

impl PartialLoader {
    // TODO: add parser
    pub fn parse(&self, _source_text: &str) -> PartialLoaderValue {
        PartialLoaderValue {
            source_text: String::from("const a = 1"), // for not report "empty file error"
            source_type: SourceType::default(),
        }
    }
}
