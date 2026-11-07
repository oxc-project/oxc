use memchr::{memchr2, memmem::Finder};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::loader::JavaScriptSource;

use super::{
    AttributeValue, COMMENT_END, COMMENT_START, SCRIPT_END, SCRIPT_START, find_attribute,
    find_script_closing_angle,
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
        let script_end_finder = Finder::new(SCRIPT_END);
        *pointer = self.find_script_start(*pointer)?;

        // find closing ">"
        let offset = find_script_closing_angle(self.source_text, *pointer)?;

        let content = &self.source_text[*pointer..*pointer + offset];
        let lang = Self::extract_lang_attribute(content);
        let mut source_type =
            SourceType::from_extension(lang).unwrap_or_else(|_| SourceType::mjs());

        // Keep existing behavior for plain `<script>` blocks while detecting `lang="ts"`.
        // In Svelte, module semantics are controlled by `module`/`context="module"`.
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

    fn find_script_start(&self, mut pointer: usize) -> Option<usize> {
        if !self.source_text[pointer..].contains(SCRIPT_START) {
            return None;
        }
        let bytes = self.source_text.as_bytes();
        while let Some(offset) = memchr2(b'<', b'{', &bytes[pointer..]) {
            pointer += offset;
            let rest = &self.source_text[pointer..];
            if rest.starts_with('{') {
                pointer = skip_markup_expression(self.source_text, pointer).unwrap_or(pointer + 1);
            } else if rest.starts_with(COMMENT_START) {
                pointer += rest.find(COMMENT_END)? + COMMENT_END.len();
            } else if let Some(attributes) = rest.strip_prefix(SCRIPT_START)
                && attributes.chars().next().is_some_and(|ch| ch.is_whitespace() || ch == '>')
            {
                return Some(pointer + SCRIPT_START.len());
            } else if rest.as_bytes().get(1).is_some_and(|b| b.is_ascii_alphabetic() || *b == b'/')
            {
                let Some(end) = find_script_closing_angle(self.source_text, pointer + 1) else {
                    pointer += 1;
                    continue;
                };
                pointer += end + 2;
                if self.source_text[..pointer - 1].trim_end().ends_with('/') {
                    continue;
                }
                for tag in ["style", "textarea"] {
                    if rest.strip_prefix('<').and_then(|rest| rest.strip_prefix(tag)).is_some_and(
                        |rest| rest.starts_with('>') || rest.starts_with(char::is_whitespace),
                    ) {
                        let closing = if tag == "style" { "</style" } else { "</textarea" };
                        pointer = skip_raw_text(self.source_text, pointer, closing)?;
                        break;
                    }
                }
            } else {
                pointer += 1;
            }
        }
        None
    }

    fn extract_lang_attribute(content: &str) -> &str {
        match find_attribute(content, "lang") {
            Some(AttributeValue::Value(lang)) if !lang.is_empty() => lang,
            _ => "mjs",
        }
    }

    fn is_module_script(content: &str) -> bool {
        find_attribute(content, "module").is_some()
            || matches!(find_attribute(content, "context"), Some(AttributeValue::Value("module")))
    }
}

fn skip_raw_text(source_text: &str, mut pointer: usize, closing: &str) -> Option<usize> {
    let finder = Finder::new(closing);
    loop {
        pointer += finder.find(&source_text.as_bytes()[pointer..])? + closing.len();
        if source_text[pointer..].starts_with('>')
            || source_text[pointer..].starts_with(char::is_whitespace)
        {
            return Some(pointer + find_script_closing_angle(source_text, pointer)? + 1);
        }
    }
}

fn skip_markup_expression(source_text: &str, pointer: usize) -> Option<usize> {
    let mut expression = source_text[pointer + 1..].trim_start();
    if expression.starts_with(['@', '#', ':']) {
        expression =
            expression[1..].trim_start_matches(|c: char| c.is_ascii_alphabetic()).trim_start();
        if expression.starts_with('}') {
            return Some(source_text.len() - expression.len() + 1);
        }
    }
    let start = source_text.len() - expression.len();
    let allocator = Allocator::default();
    let (_, next_token) =
        Parser::new(&allocator, expression, SourceType::ts()).parse_expression_prefix().ok()?;
    let end = start + next_token as usize;
    (source_text.as_bytes().get(end) == Some(&b'}')).then_some(end + 1)
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
        let source_text = r"
        <script module lang='ts'>
          debugger;
        </script>
        ";

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

    #[test]
    fn test_parse_svelte_script_tag_allows_newline_after_script_name() {
        let source_text = r#"
        <script
          lang="ts">
          debugger;
        </script>
        "#;

        let result = parse_svelte(source_text);
        assert_eq!(result.source_text.trim(), "debugger;");
        assert!(result.source_type.is_typescript());
        assert!(result.source_type.is_module());
    }

    #[test]
    fn test_parse_svelte_script_tag_allows_tab_after_script_name() {
        let source_text = "<script\tlang=\"ts\">\n  debugger;\n</script>\n";

        let result = parse_svelte(source_text);
        assert_eq!(result.source_text.trim(), "debugger;");
        assert!(result.source_type.is_typescript());
        assert!(result.source_type.is_module());
    }

    #[test]
    fn test_parse_svelte_script_tag_with_expression_lang_and_gt_in_other_attribute() {
        let source_text = r#"
        <script
            lang={"ts"}
            accesskey=">"
            >
            let scoops = 1;
            let flavours = [];
        </script>
        "#;

        let result = parse_svelte(source_text);
        assert!(!result.source_type.is_typescript());
        assert!(result.source_type.is_module());
        assert!(result.source_text.contains("let scoops = 1;"));
    }

    #[test]
    fn test_parse_svelte_script_tag_with_spaced_expression_lang_is_not_dropped() {
        let source_text = r#"
        <script
            lang={ "ts" }
            >
            let scoops = 1;
        </script>
        "#;

        let result = parse_svelte(source_text);
        assert!(!result.source_type.is_typescript());
        assert!(result.source_type.is_module());
        assert!(result.source_text.contains("let scoops = 1;"));
    }

    #[test]
    fn test_parse_svelte_with_many_script_like_tags() {
        let mut source_text = "<script-setup>noop</script-setup>\n".repeat(2_000);
        source_text.push_str("<script>debugger;</script>");

        let result = parse_svelte(&source_text);
        assert_eq!(result.source_text.trim(), "debugger;");
        assert!(result.source_type.is_module());
    }

    #[test]
    fn test_parse_svelte_script_with_callback_attribute() {
        let source_text = r#"<script>
let browser = true;
</script>
{#if browser}
  <script src="/" onload={() => {}}></script>
{/if}"#;

        let sources = SveltePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source_text.trim(), "let browser = true;");
        assert_eq!(sources[1].source_text, "");
    }

    #[test]
    fn test_parse_svelte_script_with_callback_attribute_no_component_script() {
        let source_text = r#"{#if browser}
  <script src="/" onload={() => {}}></script>
{/if}"#;

        let sources = SveltePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text, "");
    }
    #[test]
    fn test_script_in_html_expression() {
        let source_text = r#"<script lang="ts">
let {json}: {json: string} = $props();
</script>
<svelte:head>
    {@html `<script type="application/ld+json">${json}</script>`}
</svelte:head>"#;

        let sources = SveltePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text.trim(), "let {json}: {json: string} = $props();");
        assert!(sources[0].source_type.is_typescript());
    }

    #[test]
    fn test_html_expression_before_component_scripts() {
        let expressions = [
            r"{@html `<script>${json}</script>`}",
            r"{@html '<script>ignored</script>'}",
            r#"{@html "<script>ignored</script>"}"#,
            r#"{@html `<script>${`nested ${"}"}`}</script>`}"#,
            r#"{@html `<script>${({value: "}"}).value}</script>`}"#,
            r#"{@html "escaped \" <script>ignored</script>"}"#,
            r"{@html `escaped \` <script>ignored</script>`}",
            r"{@html /* } <script>ignored</script> */ html}",
            "{@html // } <script>ignored</script>\n html}",
            r"{@html /[{}]/.test(value) ? '<script>ignored</script>' : ''}",
            r"{@html count++ / 2}",
            r"{@html object.in / 2}",
            r"{@html 值 / 2}",
            r"{'{@html '}",
            r#"<style>.x::before { content: "{@html "; }</style>"#,
            r"<style>.x { color: red }</style >",
            "<textarea>hello</textarea\n>",
            r"<textarea />",
            r"<style />",
            r"<textarea>text </textarea-like> {@html </textarea>",
            r#"<div title="{@html ">content</div>"#,
            r#"<div title="{">content</div>"#,
            r"<textarea>{@html </textarea>",
            "{@html value // comment\u{2028}}",
            "{@html value // comment\u{2029}}",
            r"{@html value / count}",
            r"{@html `${/<script>}/.source}` }",
            r"<!-- {@html ` --> {@html '<script>ignored</script>'}",
        ];

        for expression in expressions {
            let source_text = format!(
                "<p>日本語</p>{expression}<script>debugger;</script><script module>export {{}};</script>"
            );
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 2, "{expression}");
            assert_eq!(sources[0].source_text, "debugger;", "{expression}");
            assert_eq!(sources[1].source_text, "export {};", "{expression}");
            assert_eq!(sources[0].start as usize, source_text.find("debugger;").unwrap());
        }
    }

    #[test]
    fn test_html_expression_without_component_scripts() {
        let source_text = r"{@html `<script>${json}</script>`}";
        assert!(SveltePartialLoader::new(source_text).parse().is_empty());
    }
}
