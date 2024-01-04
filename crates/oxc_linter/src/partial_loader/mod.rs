use oxc_span::SourceType;

use self::vue_partial_loader::VuePartialLoader;

pub mod vue_partial_loader;

pub const LINT_PARTIAL_LOADER_EXT: &[&str] = &["vue"];

pub enum PartialLoader {
    Vue,
}

#[derive(Default)]
pub struct PartialLoaderValue<'a> {
    pub source_text: &'a str,
    pub source_type: SourceType,
}

impl<'a> PartialLoaderValue<'a> {
    pub fn from(source_text: &'a str, is_ts: bool, is_jsx: bool) -> Self {
        let source_type =
            SourceType::default().with_typescript(is_ts).with_module(true).with_jsx(is_jsx);
        Self { source_text, source_type }
    }
}

impl PartialLoader {
    pub fn parse<'a>(&self, source_text: &'a str) -> PartialLoaderValue<'a> {
        if matches!(self, Self::Vue) {
            return VuePartialLoader::from(source_text).build();
        }
        PartialLoaderValue::default()
    }
}
