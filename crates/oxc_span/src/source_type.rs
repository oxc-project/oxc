use std::{path::Path, str::Chars};

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::Span;

/// Source Type for JavaScript vs TypeScript / Script vs Module / JSX
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
pub struct SourceType {
    /// JavaScript or TypeScript, default JavaScript
    language: Language,

    /// Script or Module, default Module
    module_kind: ModuleKind,

    /// Support JSX for JavaScript and TypeScript? default without JSX
    variant: LanguageVariant,

    /// Mark strict mode as always strict
    /// See <https://github.com/tc39/test262/blob/main/INTERPRETING.md#strict-mode>
    always_strict: bool,

    /// The span of the JavaScript content in the source file, needed for *.{vue} file
    range: Option<Span>,
}

/// JavaScript or TypeScript
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
pub enum Language {
    JavaScript,
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    TypeScript {
        is_definition_file: bool,
    },
}

/// Script or Module
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
pub enum ModuleKind {
    Script,
    Module,
}

/// JSX for JavaScript and TypeScript
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
pub enum LanguageVariant {
    Standard,
    Jsx,
    Vue,
}

#[derive(Debug)]
pub struct UnknownExtension(pub String);

impl Default for SourceType {
    fn default() -> Self {
        Self {
            language: Language::JavaScript,
            module_kind: ModuleKind::Script,
            variant: LanguageVariant::Standard,
            always_strict: false,
            range: None,
        }
    }
}

/// Valid file extensions
pub const VALID_EXTENSIONS: [&str; 9] =
    ["js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx", "vue"];

impl SourceType {
    pub fn is_script(self) -> bool {
        self.module_kind == ModuleKind::Script
    }

    pub fn is_module(self) -> bool {
        self.module_kind == ModuleKind::Module
    }

    pub fn module_kind(self) -> ModuleKind {
        self.module_kind
    }

    pub fn is_javascript(self) -> bool {
        matches!(self.language, Language::JavaScript)
    }

    pub fn is_typescript(self) -> bool {
        matches!(self.language, Language::TypeScript { .. })
    }

    pub fn is_typescript_definition(self) -> bool {
        matches!(self.language, Language::TypeScript { is_definition_file: true })
    }

    pub fn is_jsx(self) -> bool {
        self.variant == LanguageVariant::Jsx
    }

    pub fn always_strict(self) -> bool {
        self.always_strict
    }

    #[must_use]
    pub fn with_script(mut self, yes: bool) -> Self {
        if yes {
            self.module_kind = ModuleKind::Script;
        }
        self
    }

    #[must_use]
    pub fn with_module(mut self, yes: bool) -> Self {
        if yes {
            self.module_kind = ModuleKind::Module;
        } else {
            self.module_kind = ModuleKind::Script;
        }
        self
    }

    #[must_use]
    pub fn with_typescript(mut self, yes: bool) -> Self {
        if yes {
            self.language = Language::TypeScript { is_definition_file: false };
        }
        self
    }

    #[must_use]
    pub fn with_typescript_definition(mut self, yes: bool) -> Self {
        if yes {
            self.language = Language::TypeScript { is_definition_file: true };
        }
        self
    }

    #[must_use]
    pub fn with_jsx(mut self, yes: bool) -> Self {
        if yes {
            self.variant = LanguageVariant::Jsx;
        }
        self
    }

    #[must_use]
    pub fn with_always_strict(mut self, yes: bool) -> Self {
        self.always_strict = yes;
        self
    }

    /// Converts file path to `SourceType`
    /// returns `SourceTypeError::UnknownExtension` if:
    ///   * there is no file name
    ///   * the file extension is not one of "js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx"
    /// # Errors
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, UnknownExtension> {
        let file_name = path
            .as_ref()
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or_else(|| UnknownExtension("Please provide a valid file name.".to_string()))?;

        let extension = path
            .as_ref()
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .filter(|s| VALID_EXTENSIONS.contains(s))
            .ok_or_else(|| {
                let path = path.as_ref().to_string_lossy();
                UnknownExtension(
                    format!("Please provide a valid file extension for {path}: .js, .mjs, .jsx, .vue or .cjs for JavaScript, or .ts, .mts, .cts, .vue or .tsx for TypeScript"),
                )
            })?;

        let (is_ts_in_special_file, range) = Self::may_parser_special_extension(&path);

        let is_definition_file = file_name.ends_with(".d.ts")
            || file_name.ends_with(".d.mts")
            || file_name.ends_with(".d.cts");

        let language = match extension {
            "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
            "ts" | "mts" | "cts" | "tsx" => Language::TypeScript { is_definition_file },
            "vue" => {
                if is_ts_in_special_file.unwrap_or(false) {
                    Language::TypeScript { is_definition_file }
                } else {
                    Language::JavaScript
                }
            }
            _ => unreachable!(),
        };

        let variant = match extension {
            "js" | "mjs" | "cjs" | "jsx" | "tsx" => LanguageVariant::Jsx,
            "vue" => LanguageVariant::Vue,
            _ => LanguageVariant::Standard,
        };

        Ok(Self { language, module_kind: ModuleKind::Module, variant, always_strict: false, range })
    }
    pub fn range(&self) -> Option<Span> {
        self.range
    }
    fn may_parser_special_extension<P: AsRef<Path>>(path: P) -> (Option<bool>, Option<Span>) {
        let default_ret = (None, None);
        let Ok(content) = std::fs::read_to_string(path.as_ref()) else { return default_ret };
        if let Some(mut parser) = SpecialLanguageVariantParser::new(path.as_ref(), &content) {
            parser.parse();
            return (Some(parser.is_ts), Some(Span::new(parser.start, parser.end)));
        }

        default_ret
    }
}

/// Used to parse special file extensions that not only contain javascript/typescript,
/// need to read the content to determine the real source type.
struct SpecialLanguageVariantParser<'a> {
    kind: LanguageVariant,
    source_text: &'a str,
    chars: Chars<'a>,
    start: u32,
    end: u32,
    is_ts: bool,
}

impl<'a> SpecialLanguageVariantParser<'a> {
    fn new(path: &Path, source_text: &'a str) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        let kind = match ext {
            "vue" => Some(LanguageVariant::Vue),
            _ => None,
        };

        kind.map(|kind| Self {
            kind,
            source_text,
            chars: source_text.chars(),
            start: 0,
            end: 0,
            is_ts: false,
        })
    }
    fn parse(&mut self) {
        if self.kind == LanguageVariant::Vue {
            self.parse_vue();
        }
    }

    fn parse_vue(&mut self) {
        let mut in_script = false;
        let mut is_ts = false;
        let mut start = 0_u32;
        let mut end = 0_u32;

        while let Some(c) = self.chars.next() {
            match c {
                '<' => {
                    // we are in script tag, so we don't need to parse it, or Javascript text may contains '<script' too.
                    if in_script && self.consume("/script>") {
                        let end_tag_len = 9; // 9 is the length of '</script>'
                        end = self.offset() - end_tag_len;
                        break;
                    }

                    if in_script {
                        continue;
                    }

                    if self.consume("script") {
                        let open_tag_start = self.offset();
                        self.consume_to('>');
                        let open_tag_end = self.offset();
                        start = open_tag_end;
                        let attributes_text =
                            Span::new(open_tag_start, open_tag_end).source_text(self.source_text);
                        is_ts = attributes_text.contains(r#"lang="ts""#)
                            || attributes_text.contains("lang='ts'");
                        in_script = true;
                    }
                }
                // JS plain text may contains string contains '</script>', we want to skip it.
                '\'' | '"' | '`' => {
                    if in_script {
                        // if encounter a escape string, we need to skip it.
                        self.skip_until_next_delimiter(c);
                    }
                }
                '/' => {
                    if !in_script {
                        continue;
                    }
                    match self.peek() {
                        // single line comment
                        Some('/') => {
                            self.consume_to('\n');
                        }
                        // multi line comment
                        Some('*') => {
                            self.chars.next();
                            while let Some(c) = self.chars.next() {
                                if c == '*' && self.peek() == Some('/') {
                                    self.chars.next();
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        self.is_ts = is_ts;
        self.start = start;
        self.end = end;
    }
    fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }
    fn skip_until_next_delimiter(&mut self, delimiter: char) {
        let mut last_is_escape = false;
        for c in self.chars.by_ref() {
            if last_is_escape {
                last_is_escape = false;
                continue;
            }
            match c {
                '\\' => last_is_escape = true,
                '\'' | '"' | '`' => {
                    if c == delimiter {
                        break;
                    }
                }
                _ => last_is_escape = false,
            }
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    fn offset(&self) -> u32 {
        (self.source_text.len() - self.chars.as_str().len()) as u32
    }
    fn consume(&mut self, target: &str) -> bool {
        let mut chars = self.chars.clone();
        for ch in target.chars() {
            if let Some(c) = chars.next() {
                if c != ch {
                    return false;
                }
            } else {
                return false;
            }
        }

        self.chars = chars;
        true
    }
    fn consume_to(&mut self, target: char) -> bool {
        for ch in self.chars.by_ref() {
            if target == ch {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::Span;

    use super::SpecialLanguageVariantParser;

    fn get_js_content<'a>(parser: &'a SpecialLanguageVariantParser) -> &'a str {
        let start = parser.start;
        let end = parser.end;
        Span::new(start, end).source_text(parser.source_text)
    }

    #[test]
    fn test_parse_vue_one_line() {
        let source_text = r#"
        <template>
            <div>
                <h1>hello world</h1>
            </div>
        </template>
        <script> console.log("hi") </script>
        "#;

        let mut parser =
            SpecialLanguageVariantParser::new(Path::new("test.vue"), source_text).unwrap();
        parser.parse();
        assert_eq!(get_js_content(&parser), r#" console.log("hi") "#);
    }

    #[test]
    fn test_parse_vue_multi_line() {
        let source_text = r#"
        <template>
            <div>
                <h1>hello world</h1>
            </div>
        </template>
        <script>
            console.log("hi")
            console.log("I am multi line")
            console.log("<script></script>")
            console.log(`<script></script>`)
            console.log('<script></script>')
        </script>
        "#;

        let mut parser =
            SpecialLanguageVariantParser::new(Path::new("test.vue"), source_text).unwrap();
        parser.parse();
        assert!(!parser.is_ts);
        assert_eq!(
            get_js_content(&parser),
            r#"
            console.log("hi")
            console.log("I am multi line")
            console.log("<script></script>")
            console.log(`<script></script>`)
            console.log('<script></script>')
        "#
        );
    }

    #[test]
    fn test_parse_vue_with_ts_flag() {
        let source_text = r#"
        <script lang="ts" setup>
            1/1
        </script>

        <template>
        </template>
        "#;

        let mut parser =
            SpecialLanguageVariantParser::new(Path::new("test.vue"), source_text).unwrap();
        parser.parse();
        assert!(parser.is_ts);
        assert_eq!(
            get_js_content(&parser),
            "
            1/1
        "
        );
    }

    #[test]
    fn test_parse_vue_with_escape_string() {
        let source_text = r#"
        <script setup lang="ts">
            a.replace(/&#39;/g, '\''))
        </script>
        <template> </template>
        "#;

        let mut parser =
            SpecialLanguageVariantParser::new(Path::new("test.vue"), source_text).unwrap();
        parser.parse();
        assert!(parser.is_ts);
        assert_eq!(
            get_js_content(&parser),
            r"
            a.replace(/&#39;/g, '\''))
        "
        );
    }
}
