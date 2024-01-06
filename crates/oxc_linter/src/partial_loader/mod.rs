pub mod astro_partial_loader;
pub mod vue_partial_loader;

use oxc_span::SourceType;

use self::{astro_partial_loader::AstroPartialLoader, vue_partial_loader::VuePartialLoader};

pub const LINT_PARTIAL_LOADER_EXT: &[&str] = &["vue", "astro"];

pub enum PartialLoader {
    Vue,
    Astro,
}

#[derive(Debug, Clone, Copy)]
pub struct JavaScriptSource<'a> {
    pub source_text: &'a str,
    pub source_type: SourceType,
}

impl<'a> JavaScriptSource<'a> {
    pub fn new(source_text: &'a str, source_type: SourceType) -> Self {
        Self { source_text, source_type }
    }
}

impl PartialLoader {
    pub fn build<'a>(&self, source_text: &'a str) -> Vec<JavaScriptSource<'a>> {
        match self {
            Self::Vue => VuePartialLoader::new(source_text).parse(),
            Self::Astro => AstroPartialLoader::new(source_text).parse(),
        }
    }
}
