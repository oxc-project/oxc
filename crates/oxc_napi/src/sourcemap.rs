// Source map bindings using the standard ignoreList field.
// See <https://tc39.es/ecma426/#sec-source-map-format>.
//
// `sources` and `sourcesContent` are typed as `Array<string | null>` because
// ECMA-426 permits `null` (unknown source name / content) but not `undefined`,
// which is not representable in JSON. napi-rs typegen widens
// `Vec<Option<String>>` elements to `string | undefined | null`, so the
// emitted TypeScript types are overridden with `ts_type` to match the format.
//
// These types were previously re-exported from `oxc_sourcemap`'s `napi`
// feature, which cannot express `Array<string | null>` for its fields.

use napi_derive::napi;

/// See <https://tc39.es/ecma426/#sec-source-map-format>.
#[napi(object, js_name = "JSONSourceMap")]
pub struct JSONSourceMap {
    /// The version field, must be 3.
    pub version: u32,
    /// An optional name of the generated code that this source map is associated with.
    pub file: Option<String>,
    /// A string with the encoded mapping data.
    pub mappings: String,
    /// An optional source root, useful for relocating source files on a server or removing repeated values in the "sources" entry.
    /// This value is prepended to the individual entries in the "source" field.
    pub source_root: Option<String>,
    /// A list of original sources used by the "mappings" entry, `null` for unknown names.
    #[napi(ts_type = "Array<string | null>")]
    pub sources: Vec<Option<String>>,
    /// An optional list of source content, useful when the "source" can't be hosted.
    /// The contents are listed in the same order as the sources in line 5. "null" may be used if some original sources should be retrieved by name.
    #[napi(ts_type = "Array<string | null>")]
    pub sources_content: Option<Vec<Option<String>>>,
    /// A list of symbol names used by the "mappings" entry.
    pub names: Vec<String>,
    /// An optional field containing the debugId for this sourcemap.
    pub debug_id: Option<String>,
    /// Identifies third-party sources (such as framework code or bundler-generated code), allowing developers to avoid code that they don't want to see or step through, without having to configure this beforehand.
    /// The `ignoreList` field refers to the `sources` array, and lists the indices of all the known third-party sources in that source map.
    /// When parsing the source map, developer tools can use this to determine sections of the code that the browser loads and runs that could be automatically ignore-listed.
    /// JSON input falls back to the deprecated `x_google_ignoreList` when `ignoreList` is missing or null.
    #[napi(js_name = "ignoreList")]
    pub ignore_list: Option<Vec<u32>>,
}

// Source map output using the standard ignoreList field.
// See <https://tc39.es/ecma426/#sec-source-map-format>.
#[napi(object)]
pub struct SourceMap {
    pub file: Option<String>,
    pub mappings: String,
    pub names: Vec<String>,
    pub source_root: Option<String>,
    // `null` for unknown source names; `undefined` is not valid JSON.
    #[napi(ts_type = "Array<string | null>")]
    pub sources: Vec<Option<String>>,
    pub sources_content: Option<Vec<String>>,
    pub version: u8,
    #[napi(js_name = "ignoreList")]
    pub ignore_list: Option<Vec<u32>>,
}

impl From<oxc_sourcemap::SourceMap<'_>> for SourceMap {
    fn from(source_map: oxc_sourcemap::SourceMap<'_>) -> Self {
        let json = source_map.to_json();
        Self {
            file: json.file,
            mappings: json.mappings,
            names: json.names,
            source_root: json.source_root,
            sources: json.sources,
            sources_content: json
                .sources_content
                .map(|content| content.into_iter().map(Option::unwrap_or_default).collect()),
            version: 3,
            ignore_list: json.ignore_list,
        }
    }
}
