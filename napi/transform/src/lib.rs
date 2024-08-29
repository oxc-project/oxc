// NOTE: the strange order of struct and `mod` statements is to establish the
// desired order in generated `index.d.ts` code. We want options to be on top.
// This is not only for aesthetics, but using declarations before they're parsed
// breaks NAPI typegen.
mod context;
mod options;

use napi_derive::napi;

#[napi(object)]
pub struct SourceMap {
    pub file: Option<String>,
    pub mappings: Option<String>,
    pub source_root: Option<String>,
    pub sources: Option<Vec<Option<String>>>,
    pub sources_content: Option<Vec<Option<String>>>,
    pub names: Option<Vec<String>>,
}

pub use crate::options::*;

mod isolated_declaration;
pub use isolated_declaration::*;

mod transformer;
pub use transformer::*;

impl From<oxc_sourcemap::SourceMap> for SourceMap {
    fn from(source_map: oxc_sourcemap::SourceMap) -> Self {
        let json = source_map.to_json();
        Self {
            file: json.file,
            mappings: json.mappings,
            source_root: json.source_root,
            sources: json.sources,
            sources_content: json.sources_content,
            names: json.names,
        }
    }
}
