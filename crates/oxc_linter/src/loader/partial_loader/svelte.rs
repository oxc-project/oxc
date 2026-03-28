use memchr::memmem::{Finder, FinderRev};

use oxc_span::SourceType;

use crate::loader::JavaScriptSource;

use super::{
    COMMENT_END, COMMENT_START, SCRIPT_END, SCRIPT_START, find_script_closing_angle,
    find_script_start,
};

pub struct SveltePartialLoader<'a> {
    source_text: &'a str,
}

impl<'a> SveltePartialLoader<'a> {
    pub fn new(source_text: &'a str) -> Self {
        Self { source_text }
    }

    pub fn parse(self) -> Vec<JavaScriptSource<'a>> {
        self.parse_scripts()
    }

    /// Each *.svelte file can contain at most
    ///  * one `<script>` block
    ///  * one `<script context="module">` or `<script module>` block
    ///    <https://github.com/sveltejs/svelte.dev/blob/ba7ad256f786aa5bc67eac3a58608f3f50b59e91/apps/svelte.dev/content/tutorial/02-advanced-svelte/08-script-module/02-module-exports/index.md>
    fn parse_scripts(&self) -> Vec<JavaScriptSource<'a>> {
        let mut pointer = 0;
        let Some(result1) = self.parse_script(&mut pointer) else {
            return vec![];
        };
        let Some(result2) = self.parse_script(&mut pointer) else {
            return vec![result1];
        };
        vec![result1, result2]
    }

    fn parse_script(&self, pointer: &mut usize) -> Option<JavaScriptSource<'a>> {
        let script_start_finder = Finder::new(SCRIPT_START);
        let script_end_finder = Finder::new(SCRIPT_END);
        let comment_start_finder = FinderRev::new(COMMENT_START);
        let comment_end_finder: Finder<'_> = Finder::new(COMMENT_END);
        // find opening "<script"
        *pointer += find_script_start(
            self.source_text,
            *pointer,
            &script_start_finder,
            &comment_start_finder,
            &comment_end_finder,
        )?;

        // skip `<script-...>` tags and keep searching for a real `<script>` block
        if !self.source_text[*pointer..].starts_with([' ', '>']) {
            return self.parse_script(pointer);
        }

        // find closing ">"
        let offset = find_script_closing_angle(self.source_text, *pointer)?;

        let content = &self.source_text[*pointer..*pointer + offset];
        let lang = Self::extract_lang_attribute(content);
        let Ok(mut source_type) = SourceType::from_extension(lang) else { return None };

        // Svelte script blocks use module semantics. Keep the existing behavior for plain
        // `<script>` blocks while also correctly detecting `lang="ts"`, `module`, and
        // `context="module"`.
        if source_type.is_unambiguous() || Self::is_module_script(content) {
            source_type = source_type.with_module(true);
        }

        *pointer += offset + 1;
        let js_start = *pointer;

        // find "</script>"
        let offset = script_end_finder.find(&self.source_text.as_bytes()[*pointer..])?;
        let js_end = *pointer + offset;
        *pointer += offset + SCRIPT_END.len();

        let source_text = &self.source_text[js_start..js_end];

        // NOTE: loader checked that source_text.len() is less than u32::MAX
        #[expect(clippy::cast_possible_truncation)]
        Some(JavaScriptSource::partial(source_text, source_type, js_start as u32))
    }

    fn extract_lang_attribute(content: &str) -> &str {
        Self::find_attribute(content, "lang")
            .flatten()
            .filter(|lang| !lang.is_empty())
            .unwrap_or("mjs")
    }

    fn is_module_script(content: &str) -> bool {
        Self::find_attribute(content, "module").is_some()
            || matches!(Self::find_attribute(content, "context"), Some(Some("module")))
    }

    fn find_attribute<'b>(content: &'b str, target: &str) -> Option<Option<&'b str>> {
        let mut rest = content.trim();
        if let Some(stripped) = rest.strip_prefix("<script") {
            rest = stripped;
        }

        loop {
            rest = rest.trim_start_matches(|c: char| c.is_whitespace() || c == '/');
            if rest.is_empty() || rest.starts_with('>') {
                return None;
            }

            let name_end = rest
                .find(|c: char| c.is_whitespace() || matches!(c, '=' | '>' | '/'))
                .unwrap_or(rest.len());
            if name_end == 0 {
                return None;
            }

            let name = &rest[..name_end];
            rest = &rest[name_end..];
            rest = rest.trim_start();

            let value = if let Some(stripped) = rest.strip_prefix('=') {
                rest = stripped.trim_start();

                match rest.chars().next() {
                    Some('"' | '\'') => {
                        let quote = rest.chars().next().unwrap();
                        rest = &rest[quote.len_utf8()..];
                        let end = rest.find(quote)?;
                        let value = &rest[..end];
                        rest = &rest[end + quote.len_utf8()..];
                        Some(value)
                    }
                    Some(_) => {
                        let end = rest
                            .find(|c: char| c.is_whitespace() || matches!(c, '>' | '/'))
                            .unwrap_or(rest.len());
                        let value = &rest[..end];
                        rest = &rest[end..];
                        Some(value)
                    }
                    None => return None,
                }
            } else {
                None
            };

            if name == target {
                return Some(value);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{JavaScriptSource, SveltePartialLoader};

    fn parse_svelte(source_text: &str) -> JavaScriptSource<'_> {
        let sources = SveltePartialLoader::new(source_text).parse();
        *sources.first().unwrap()
    }

    #[test]
    fn test_parse_svelte() {
        let source_text = r#"
        <script>
          console.log("hi");
        </script>
        <h1>Hello World</h1>
        "#;

        let result = parse_svelte(source_text);
        assert_eq!(result.source_text.trim(), r#"console.log("hi");"#);
        assert!(result.source_type.is_module());
    }

    #[test]
    fn test_script_inside_code_comment() {
        let source_text = r"
        <!-- <script>a</script> -->
        <!-- <script> -->
        <script>b</script>
        ";

        let result = parse_svelte(source_text);
        assert_eq!(result.source_text, "b");
        assert_eq!(result.start, 79);
    }

    #[test]
    fn test_parse_svelte_ts_with_generic() {
        let source_text = r#"
        <script lang="ts" generics="T extends Record<string, unknown>">
          console.log("hi");
        </script>
        <h1>Hello World</h1>
        "#;

        let result = parse_svelte(source_text);
        assert_eq!(result.source_text.trim(), r#"console.log("hi");"#);
        assert!(result.source_type.is_typescript());
        assert!(result.source_type.is_module());
    }

    #[test]
    fn test_parse_svelte_with_module_script() {
        let source_text = r#"
        <script module>
          export async function load() { /* some loading logic */ }
        </script>
        <script>
          console.log("hi");
        </script>
        <h1>Hello World</h1>
        "#;

        let sources = SveltePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 2);
        assert_eq!(
            sources[0].source_text.trim(),
            "export async function load() { /* some loading logic */ }"
        );
        assert_eq!(sources[1].source_text.trim(), r#"console.log("hi");"#);
        assert!(sources[0].source_type.is_module());
        assert!(sources[1].source_type.is_module());
    }

    #[test]
    fn test_parse_svelte_with_context_module_script() {
        let source_text = r#"
        <script context="module">
          debugger;
        </script>
        <script>
          console.log("hi");
        </script>
        "#;

        let sources = SveltePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source_text.trim(), "debugger;");
        assert_eq!(sources[1].source_text.trim(), r#"console.log("hi");"#);
        assert!(sources[0].source_type.is_module());
        assert!(sources[1].source_type.is_module());
    }

    #[test]
    fn test_parse_svelte_module_script_lang_ts() {
        let source_text = r#"
        <script module lang='ts'>
          debugger;
        </script>
        "#;

        let result = parse_svelte(source_text);
        assert!(result.source_type.is_typescript());
        assert!(result.source_type.is_module());
        assert_eq!(result.source_text.trim(), "debugger;");
    }

    #[test]
    fn test_parse_svelte_context_module_script_lang_ts() {
        let source_text = r#"
        <script context="module" lang="ts">
          debugger;
        </script>
        "#;

        let result = parse_svelte(source_text);
        assert!(result.source_type.is_typescript());
        assert!(result.source_type.is_module());
        assert_eq!(result.source_text.trim(), "debugger;");
    }

    #[test]
    fn test_parse_svelte_does_not_treat_data_language_as_lang() {
        let source_text = r#"
        <script data-language="ts">
          debugger;
        </script>
        "#;

        let result = parse_svelte(source_text);
        assert!(!result.source_type.is_typescript());
        assert!(result.source_type.is_module());
        assert_eq!(result.source_text.trim(), "debugger;");
    }

    #[test]
    fn test_parse_svelte_ignores_script_like_tags() {
        let source_text = r#"
        <script-setup>
          debugger;
        </script-setup>
        <script>
          console.log("hi");
        </script>
        "#;

        let result = parse_svelte(source_text);
        assert_eq!(result.source_text.trim(), r#"console.log("hi");"#);
        assert!(result.source_type.is_module());
    }
}
