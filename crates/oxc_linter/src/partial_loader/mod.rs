use oxc_span::SourceType;

use self::vue_partial_loader::VuePartialLoader;

pub mod vue_partial_loader;

pub const LINT_PARTIAL_LOADER_EXT: &[&str] = &["vue"];

pub enum PartialLoader {
    Vue,
}

#[derive(Default)]
pub struct PartialLoaderValue {
    pub source_text: String,
    pub source_type: SourceType,
}

impl PartialLoaderValue {
    pub fn from(source_text: String, is_ts: bool, is_jsx: bool) -> Self {
        // `module_kind`  should be `ModuleKind::Module` for allow `import`
        let source_type =
            SourceType::default().with_typescript(is_ts).with_module(true).with_jsx(is_jsx);
        Self { source_text, source_type }
    }
}

impl PartialLoader {
    pub fn parse(&self, source_text: &str) -> PartialLoaderValue {
        if matches!(self, Self::Vue) {
            return VuePartialLoader::from(source_text).build();
        }
        PartialLoaderValue::default()
    }
}
