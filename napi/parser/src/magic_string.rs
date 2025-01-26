#![allow(clippy::cast_possible_truncation)]
use std::sync::Arc;

use napi::Either;
use napi_derive::napi;
use self_cell::self_cell;
use string_wizard::{Hires, MagicString as MS};

use oxc_data_structures::rope::{get_line_column, Rope};

#[napi]
pub struct MagicString {
    cell: MagicStringImpl,
    rope: Option<Rope>,
}

self_cell!(
    struct MagicStringImpl {
        owner: String,
        #[covariant]
        dependent: MS,
    }
);

impl MagicString {
    pub fn new(source_text: String) -> Self {
        Self {
            cell: MagicStringImpl::new(source_text, |s| string_wizard::MagicString::new(s)),
            rope: None,
        }
    }
}

#[napi(object)]
pub struct LineColumn {
    pub line: u32,
    pub column: u32,
}

#[napi(object)]
pub struct OverwriteOptions {
    pub content_only: bool,
}

#[napi(object)]
pub struct SourceMapOptions {
    pub include_content: Option<bool>,
    pub source: Option<String>,
    pub hires: Option<bool>,
}

#[napi(object)]
pub struct GenerateDecodedMapOptions {
    /// The filename of the file containing the original source.
    pub source: Option<String>,
    /// Whether to include the original content in the map's `sourcesContent` array.
    pub include_content: bool,
    /// Whether the mapping should be high-resolution.
    #[napi(ts_type = "boolean | 'boundary'")]
    pub hires: Either<bool, String>,
}

impl Default for GenerateDecodedMapOptions {
    fn default() -> Self {
        Self { source: None, include_content: false, hires: Either::A(false) }
    }
}

impl From<GenerateDecodedMapOptions> for string_wizard::SourceMapOptions {
    fn from(o: GenerateDecodedMapOptions) -> Self {
        Self {
            source: Arc::from(o.source.unwrap_or_default()),
            include_content: o.include_content,
            hires: match o.hires {
                Either::A(true) => Hires::True,
                Either::A(false) => Hires::False,
                Either::B(s) => {
                    if s == "boundary" {
                        Hires::Boundary
                    } else {
                        Hires::False
                    }
                }
            },
        }
    }
}

#[napi]
impl MagicString {
    /// Get source text from utf8 offset.
    #[napi]
    pub fn get_source_text(&self, start: u32, end: u32) -> &str {
        &self.cell.borrow_owner()[start as usize..end as usize]
    }

    /// Get 0-based line and column number from utf8 offset.
    #[napi]
    pub fn get_line_column_number(&mut self, offset: u32) -> LineColumn {
        let source_text = self.cell.borrow_owner();
        let rope = self.rope.get_or_insert_with(|| Rope::from_str(source_text));
        let (line, column) = get_line_column(rope, offset, source_text);
        LineColumn { line, column }
    }

    /// Get UTF16 byte offset from UTF8 byte offset.
    #[napi]
    pub fn get_utf16_byte_offset(&mut self, offset: u32) -> u32 {
        let source_text = self.cell.borrow_owner();
        // TODO(perf): this is obviously slow ...
        source_text[..offset as usize].encode_utf16().count() as u32
    }

    #[napi]
    pub fn length(&self) -> u32 {
        self.cell.borrow_dependent().len() as u32
    }

    #[napi]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.cell.borrow_dependent().to_string()
    }

    #[napi]
    pub fn has_changed(&self) -> bool {
        self.cell.borrow_dependent().has_changed()
    }

    #[napi]
    pub fn append(&mut self, input: String) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.append(input);
        });
        self
    }

    #[napi]
    pub fn append_left(&mut self, index: u32, input: String) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.append_left(index as usize, input);
        });
        self
    }

    #[napi]
    pub fn append_right(&mut self, index: u32, input: String) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.append_right(index as usize, input);
        });
        self
    }

    #[napi]
    pub fn indent(&mut self) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.indent();
        });
        self
    }

    #[napi]
    pub fn prepend(&mut self, input: String) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.prepend(input);
        });
        self
    }

    #[napi]
    pub fn prepend_left(&mut self, index: u32, input: String) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.prepend_left(index as usize, input);
        });
        self
    }

    #[napi]
    pub fn prepend_right(&mut self, index: u32, input: String) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.prepend_right(index as usize, input);
        });
        self
    }

    #[napi]
    pub fn relocate(&mut self, start: u32, end: u32, to: u32) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.relocate(start as usize, end as usize, to as usize);
        });
        self
    }

    #[napi]
    pub fn remove(&mut self, start: u32, end: u32) -> &Self {
        self.cell.with_dependent_mut(|_, ms| {
            ms.remove(start as usize, end as usize);
        });
        self
    }

    #[napi(
        ts_args_type = "options?: Partial<GenerateDecodedMapOptions>",
        ts_return_type = r"{
    toString: () => string;
    toUrl: () => string;
    toMap: () => {
      file?: string
      mappings: string
      names: Array<string>
      sourceRoot?: string
      sources: Array<string>
      sourcesContent?: Array<string>
      version: number
      x_google_ignoreList?: Array<number>
    }
    }"
    )]
    pub fn generate_map(&self) {
        // only for .d.ts generation
    }

    #[napi(skip_typescript)]
    pub fn to_sourcemap_string(&self, options: Option<GenerateDecodedMapOptions>) -> String {
        self.get_sourcemap(options).to_json_string()
    }

    #[napi(skip_typescript)]
    pub fn to_sourcemap_url(&self, options: Option<GenerateDecodedMapOptions>) -> String {
        self.get_sourcemap(options).to_data_url()
    }

    #[napi(skip_typescript)]
    pub fn to_sourcemap_object(
        &self,
        options: Option<GenerateDecodedMapOptions>,
    ) -> oxc_sourcemap::napi::SourceMap {
        oxc_sourcemap::napi::SourceMap::from(self.get_sourcemap(options))
    }

    fn get_sourcemap(
        &self,
        options: Option<GenerateDecodedMapOptions>,
    ) -> oxc_sourcemap::SourceMap {
        self.cell
            .borrow_dependent()
            .source_map(string_wizard::SourceMapOptions::from(options.unwrap_or_default()))
    }
}
