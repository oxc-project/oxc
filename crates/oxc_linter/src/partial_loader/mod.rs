pub mod astro_partial_loader;
pub mod vue_partial_loader;

use oxc_span::SourceType;

use self::{astro_partial_loader::AstroPartialLoader, vue_partial_loader::VuePartialLoader};

pub const LINT_PARTIAL_LOADER_EXT: &[&str] = &["vue", "astro"];

pub enum PartialLoader {
    Vue,
    Astro,
}

#[derive(Default)]
pub struct PartialLoaderValue<'a> {
    pub source_text: &'a str,
    pub source_type: SourceType,
}

impl<'a> PartialLoaderValue<'a> {
    pub fn new(source_text: &'a str, is_ts: bool, is_jsx: bool) -> Self {
        let source_type =
            SourceType::default().with_typescript(is_ts).with_module(true).with_jsx(is_jsx);
        Self { source_text, source_type }
    }
}

impl PartialLoader {
    pub fn build<'a>(&self, source_text: &'a str) -> Option<PartialLoaderValue<'a>> {
        match self {
            Self::Vue => VuePartialLoader::new(source_text).build(),
            Self::Astro => AstroPartialLoader::new(source_text).build(),
        }
    }
}
