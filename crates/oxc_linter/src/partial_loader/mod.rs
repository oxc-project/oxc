mod astro;
mod svelte;
mod vue;

use oxc_span::SourceType;

pub use self::{astro::AstroPartialLoader, svelte::SveltePartialLoader, vue::VuePartialLoader};

const SCRIPT_START: &str = "<script";
const SCRIPT_END: &str = "</script>";

pub const LINT_PARTIAL_LOADER_EXT: &[&str] = &["vue", "astro", "svelte"];

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

pub struct PartialLoader;

impl PartialLoader {
    /// Extract js section of specifial files.
    /// Returns `None` if the specifial file does not have a js section.
    pub fn parse<'a>(ext: &str, source_text: &'a str) -> Option<Vec<JavaScriptSource<'a>>> {
        match ext {
            "vue" => Some(VuePartialLoader::new(source_text).parse()),
            "astro" => Some(AstroPartialLoader::new(source_text).parse()),
            "svelte" => Some(SveltePartialLoader::new(source_text).parse()),
            _ => None,
        }
    }
}
