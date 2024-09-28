use napi_derive::napi;

// Aligned with Rollup's sourcemap input.
//
// <https://github.com/rollup/rollup/blob/766dbf90d69268971feaafa1f53f88a0755e8023/src/rollup/types.d.ts#L80-L89>
//
// ```
// export interface ExistingRawSourceMap {
//  file?: string;
//  mappings: string;
//  names: string[];
//  sourceRoot?: string;
//  sources: string[];
//  sourcesContent?: string[];
//  version: number;
//  x_google_ignoreList?: number[];
// }
// ```
#[napi(object)]
pub struct SourceMap {
    pub file: Option<String>,
    pub mappings: String,
    pub names: Vec<String>,
    pub source_root: Option<String>,
    pub sources: Vec<String>,
    pub sources_content: Option<Vec<String>>,
    pub version: u8,
    #[napi(js_name = "x_google_ignoreList")]
    pub x_google_ignorelist: Option<Vec<u32>>,
}

impl From<oxc_sourcemap::SourceMap> for SourceMap {
    fn from(source_map: oxc_sourcemap::SourceMap) -> Self {
        let json = source_map.to_json();
        Self {
            file: json.file,
            mappings: json.mappings,
            names: json.names,
            source_root: json.source_root,
            sources: json.sources,
            sources_content: json.sources_content.map(|content| {
                content.into_iter().map(Option::unwrap_or_default).collect::<Vec<_>>()
            }),
            version: 3,
            x_google_ignorelist: None,
        }
    }
}
