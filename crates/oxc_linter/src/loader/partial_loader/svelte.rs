use memchr::{memchr2, memmem::Finder};

use oxc_allocator::Allocator;
use oxc_parser::{Kind, Parser, config::TokensParserConfig};
use oxc_span::SourceType;
use oxc_syntax::{
    identifier::{is_identifier_start, is_white_space},
    line_terminator::is_line_terminator,
};

use crate::loader::JavaScriptSource;

use super::{AttributeValue, COMMENT_END, COMMENT_START, SCRIPT_END, SCRIPT_START, find_attribute};

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
        let mut context = MarkupContext::default();
        let Some(result1) = self.parse_script(&mut pointer, &mut context) else {
            return vec![];
        };
        let Some(result2) = self.parse_script(&mut pointer, &mut context) else {
            return vec![result1];
        };
        vec![result1, result2]
    }

    fn parse_script(
        &self,
        pointer: &mut usize,
        context: &mut MarkupContext<'a>,
    ) -> Option<JavaScriptSource<'a>> {
        let script_end_finder = Finder::new(SCRIPT_END);
        *pointer = self.find_script_start(*pointer, context)?;

        let end = find_markup_tag_end(self.source_text, *pointer, context.is_root())?;
        let offset = end - *pointer - 1;

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

    fn find_script_start(
        &self,
        mut pointer: usize,
        context: &mut MarkupContext<'a>,
    ) -> Option<usize> {
        if !self.source_text[pointer..].contains(SCRIPT_START) {
            return None;
        }
        let bytes = self.source_text.as_bytes();
        while let Some(offset) = memchr2(b'<', b'{', &bytes[pointer..]) {
            pointer += offset;
            let rest = &self.source_text[pointer..];
            if rest.starts_with('{') {
                if let Some(end) = skip_markup_expression(self.source_text, pointer) {
                    context.expression(&self.source_text[pointer + 1..end - 1]);
                    pointer = end;
                } else {
                    pointer += 1;
                }
            } else if rest.starts_with(COMMENT_START) {
                pointer += rest.find(COMMENT_END)? + COMMENT_END.len();
            } else {
                let tag = rest.strip_prefix("</").or_else(|| rest.strip_prefix('<'))?;
                if !tag.chars().next().is_some_and(is_identifier_start) {
                    pointer += 1;
                    continue;
                }
                let name_end =
                    tag.find(|c: char| is_svelte_whitespace(c) || matches!(c, '/' | '>'))?;
                let name = &tag[..name_end];
                let closing = rest.starts_with("</");
                let static_text = matches!(name, "script" | "style") && context.is_root();
                let Some(end) = find_markup_tag_end(self.source_text, pointer + 1, static_text)
                else {
                    pointer += 1;
                    continue;
                };
                let self_closing = self.source_text[pointer..end - 1]
                    .trim_end_matches(is_svelte_whitespace)
                    .ends_with('/');
                if !closing && name == "script" && !self_closing {
                    return Some(pointer + SCRIPT_START.len());
                }
                let component_style_attributes =
                    static_text.then_some(&self.source_text[pointer..end - 1]);
                pointer = end;
                if closing {
                    context.close(name);
                } else {
                    match name {
                        "style" if !self_closing => {
                            pointer =
                                skip_style(self.source_text, pointer, component_style_attributes)?;
                        }
                        "textarea" if !self_closing => {
                            pointer = skip_textarea(self.source_text, pointer)?;
                        }
                        _ => context.open(name, self_closing),
                    }
                }
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

#[derive(Default)]
struct MarkupContext<'a> {
    elements: Vec<&'a str>,
    blocks: Vec<usize>,
}

impl<'a> MarkupContext<'a> {
    fn is_root(&self) -> bool {
        self.elements.is_empty() && self.blocks.is_empty()
    }

    fn expression(&mut self, expression: &str) {
        match expression.trim_start_matches(is_svelte_whitespace).as_bytes().first() {
            Some(b'#') => self.blocks.push(self.elements.len()),
            Some(b':') => {
                if let Some(&depth) = self.blocks.last() {
                    self.elements.truncate(depth);
                }
            }
            Some(b'/') => {
                if let Some(depth) = self.blocks.pop() {
                    self.elements.truncate(depth);
                }
            }
            _ => {}
        }
    }

    fn close(&mut self, name: &str) {
        let depth = self.blocks.last().copied().unwrap_or(0);
        if let Some(index) = self.elements[depth..].iter().rposition(|&element| element == name) {
            self.elements.truncate(depth + index);
        }
    }

    fn open(&mut self, name: &'a str, self_closing: bool) {
        let depth = self.blocks.last().copied().unwrap_or(0);
        if self.elements.len() > depth
            && self.elements.last().is_some_and(|&parent| implicitly_closes(parent, name))
        {
            self.elements.pop();
        }
        if !self_closing
            && !matches!(
                name,
                "area"
                    | "base"
                    | "br"
                    | "col"
                    | "command"
                    | "embed"
                    | "hr"
                    | "img"
                    | "input"
                    | "keygen"
                    | "link"
                    | "meta"
                    | "param"
                    | "source"
                    | "track"
                    | "wbr"
            )
        {
            self.elements.push(name);
        }
    }
}

fn implicitly_closes(parent: &str, name: &str) -> bool {
    match parent {
        "li" => name == "li",
        "dt" | "dd" => matches!(name, "dt" | "dd"),
        "p" => matches!(
            name,
            "address"
                | "article"
                | "aside"
                | "blockquote"
                | "div"
                | "dl"
                | "fieldset"
                | "footer"
                | "form"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "header"
                | "hgroup"
                | "hr"
                | "main"
                | "menu"
                | "nav"
                | "ol"
                | "p"
                | "pre"
                | "section"
                | "table"
                | "ul"
        ),
        "rt" | "rp" => matches!(name, "rt" | "rp"),
        "optgroup" => name == "optgroup",
        "option" => matches!(name, "option" | "optgroup"),
        "thead" | "tbody" => matches!(name, "tbody" | "tfoot"),
        "tfoot" => name == "tbody",
        "tr" => matches!(name, "tr" | "tbody"),
        "td" | "th" => matches!(name, "td" | "th" | "tr"),
        _ => false,
    }
}

fn skip_textarea(source_text: &str, mut pointer: usize) -> Option<usize> {
    let bytes = source_text.as_bytes();
    while let Some(offset) = memchr2(b'<', b'{', &bytes[pointer..]) {
        pointer += offset;
        let rest = &source_text[pointer..];
        if rest.starts_with('{') {
            pointer = skip_markup_expression(source_text, pointer).unwrap_or(pointer + 1);
        } else if rest
            .as_bytes()
            .get(..10)
            .is_some_and(|name| name.eq_ignore_ascii_case(b"</textarea"))
            && (rest[10..].starts_with('>') || rest[10..].starts_with(is_svelte_whitespace))
        {
            return Some(pointer + 10 + rest[10..].find('>')? + 1);
        } else {
            pointer += 1;
        }
    }
    None
}

fn find_markup_tag_end(source_text: &str, mut pointer: usize, static_text: bool) -> Option<usize> {
    let bytes = source_text.as_bytes();
    let mut quote = None;
    while let Some(&byte) = bytes.get(pointer) {
        if byte == b'{'
            && (!static_text || quote.is_none())
            && let Some(end) = skip_attribute_expression(source_text, pointer)
        {
            pointer = end;
            continue;
        }
        match byte {
            b'\'' | b'"' if quote == Some(byte) => quote = None,
            b'\'' | b'"' if quote.is_none() => quote = Some(byte),
            b'>' if quote.is_none() => return Some(pointer + 1),
            _ => {}
        }
        pointer += 1;
    }
    None
}

fn skip_style(
    source_text: &str,
    mut pointer: usize,
    component_attributes: Option<&str>,
) -> Option<usize> {
    let Some(attributes) = component_attributes else {
        return Some(pointer + source_text[pointer..].find("</style>")? + "</style>".len());
    };
    if find_attribute(attributes, "lang")
        .is_some_and(|lang| !matches!(lang, AttributeValue::Value("css")))
    {
        let finder = Finder::new("</style");
        loop {
            pointer += finder.find(&source_text.as_bytes()[pointer..])? + "</style".len();
            let rest = source_text[pointer..].trim_start_matches(is_svelte_whitespace);
            if rest.starts_with('>') {
                return Some(source_text.len() - rest.len() + 1);
            }
        }
    }
    let bytes = source_text.as_bytes();
    let mut depth = 0usize;
    let mut quote = None;
    while let Some(&byte) = bytes.get(pointer) {
        if byte == b'\\' {
            pointer = pointer.saturating_add(2);
            continue;
        }
        if let Some(delimiter) = quote {
            if byte == delimiter {
                quote = None;
            }
        } else {
            match byte {
                b'\'' | b'"' => quote = Some(byte),
                b'/' if bytes.get(pointer + 1) == Some(&b'*') => {
                    pointer += source_text[pointer + 2..].find("*/")? + 4;
                    continue;
                }
                b'{' | b'(' | b'[' => depth += 1,
                b'}' | b')' | b']' => depth = depth.saturating_sub(1),
                b'<' if source_text[pointer..].starts_with(COMMENT_START) => {
                    pointer += source_text[pointer + COMMENT_START.len()..].find(COMMENT_END)?
                        + COMMENT_START.len()
                        + COMMENT_END.len();
                    continue;
                }
                b'<' if depth == 0 => {
                    if let Some(rest) = source_text[pointer..].strip_prefix("</style") {
                        let rest = rest.trim_start_matches(is_svelte_whitespace);
                        if rest.starts_with('>') {
                            return Some(source_text.len() - rest.len() + 1);
                        }
                    }
                }
                _ => {}
            }
        }
        pointer += 1;
    }
    None
}

fn is_svelte_whitespace(c: char) -> bool {
    is_white_space(c) || is_line_terminator(c)
}

fn skip_markup_expression(source_text: &str, pointer: usize) -> Option<usize> {
    let mut expression = source_text[pointer + 1..].trim_start_matches(is_svelte_whitespace);
    if let Some(closing_block) = expression.strip_prefix('/')
        && let Some(name_end) = closing_block.find(|c: char| !c.is_ascii_alphabetic())
        && name_end > 0
    {
        let rest = closing_block[name_end..].trim_start_matches(is_svelte_whitespace);
        if rest.starts_with('}') {
            return Some(source_text.len() - rest.len() + 1);
        }
    }
    let is_else = expression.starts_with(":else");
    if expression.starts_with(['@', '#', ':']) {
        expression = expression[1..]
            .trim_start_matches(|c: char| c.is_ascii_alphabetic())
            .trim_start_matches(is_svelte_whitespace);
        if is_else && let Some(condition) = expression.strip_prefix("if") {
            expression = condition.trim_start_matches(is_svelte_whitespace);
        }
        if expression.starts_with('}') {
            return Some(source_text.len() - expression.len() + 1);
        }
    }
    skip_expression(source_text, source_text.len() - expression.len())
}

fn skip_attribute_expression(source_text: &str, pointer: usize) -> Option<usize> {
    let expression = source_text[pointer + 1..].trim_start_matches(is_svelte_whitespace);
    let expression = expression
        .strip_prefix("...")
        .unwrap_or(expression)
        .trim_start_matches(is_svelte_whitespace);
    skip_expression(source_text, source_text.len() - expression.len())
}

fn skip_expression(source_text: &str, mut pointer: usize) -> Option<usize> {
    const INITIAL_CHUNK_LEN: usize = 128;
    let mut depth: usize = 1;
    let mut chunk_len = INITIAL_CHUNK_LEN;
    loop {
        let mut end = pointer.saturating_add(chunk_len).min(source_text.len());
        while !source_text.is_char_boundary(end) {
            end -= 1;
        }
        let allocator = Allocator::default();
        let tokens = Parser::new(&allocator, &source_text[pointer..end], SourceType::ts())
            .with_config(TokensParserConfig)
            .tokenize_expression_prefix();
        let mut next_depth = depth;
        let mut consumed = 0;
        let mut needs_more = false;
        for token in &tokens {
            match token.kind() {
                Kind::Eof | Kind::Undetermined => {
                    needs_more = true;
                    break;
                }
                Kind::LCurly => next_depth += 1,
                Kind::RCurly => {
                    next_depth -= 1;
                    if next_depth == 0 {
                        return Some(pointer + token.end() as usize);
                    }
                }
                _ => {}
            }
            consumed = token.end() as usize;
        }
        if needs_more || consumed == end - pointer {
            if end == source_text.len() {
                return None;
            }
            chunk_len = chunk_len.saturating_mul(2);
        } else {
            if consumed == 0 {
                return None;
            }
            pointer += consumed;
            depth = next_depth;
            chunk_len = INITIAL_CHUNK_LEN;
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
    #[test]
    fn test_script_after_attribute_with_braces_in_string() {
        for attributes in [
            r#"title={"}"}"#,
            r#"title={"{"}"#,
            r#"title={`${"}"}`}"#,
            r#"title="before {"}"} after""#,
            r#"{...{title: "}"}}"#,
            r#"{...(count > 0 ? {title: "}"} : {})}"#,
        ] {
            let source_text =
                format!(r#"<div {attributes}></div><script>const text = `">`; debugger;</script>"#);
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{attributes}");
            assert_eq!(sources[0].source_text, r#"const text = `">`; debugger;"#);
        }
    }

    #[test]
    fn test_script_after_closing_block() {
        for closing in ["{/if}", "{\u{feff}/if}", "{/if\u{feff}}"] {
            let source_text = format!(
                "{{#if true}}<p>hello</p>{closing}<script>const re = /}}/; debugger;</script>"
            );
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{closing}");
            assert_eq!(sources[0].source_text, "const re = /}/; debugger;");
        }
    }

    #[test]
    fn test_script_with_literal_braces_in_component_attributes() {
        for source_text in [
            r#"<script data-x="{">const s="}"; debugger;</script>"#,
            r#"<script data-x="{'">const x = '}'; debugger;</script>"#,
            r#"<style data-x="{">.x { color: red; }</style><script>debugger;</script>"#,
        ] {
            let sources = SveltePartialLoader::new(source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert!(sources[0].source_text.contains("debugger;"));
        }
    }

    #[test]
    fn test_regex_in_attribute_is_not_a_closing_block() {
        for attributes in [
            r#"title={/if}>/.test(value)} data-x="<script>ignored</script>""#,
            r#"title={/if}"/.test('<script>ignored</script>')}"#,
        ] {
            let source_text = format!("<div {attributes}></div><script>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{attributes}");
            assert_eq!(sources[0].source_text, "debugger;");
        }
    }
    #[test]
    fn test_script_after_else_if_header() {
        let source_text = "{#if ok}<p>one</p>{:else if value === '<textarea>'}<p>two</p>{/if}<script>debugger;</script>";
        let sources = SveltePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text, "debugger;");
    }

    #[test]
    fn test_script_after_other_block_headers() {
        for source_text in [
            "{#await value === '<textarea>' ? promise : promise then result}<p>hi</p>{/await}<script lang='ts'>debugger;</script>",
            "{#each items as {x = '<textarea>'}}<p>hi</p>{/each}<script lang='ts'>debugger;</script>",
            "{#await promise then {x = '<textarea>'}}<p>hi</p>{/await}<script lang='ts'>debugger;</script>",
            "{#each items as {x = '<textarea>'}, i ('<textarea>')}<p>hi</p>{/each}<script lang='ts'>debugger;</script>",
            "{#snippet content(x: string = '<textarea>')}<p>hi</p>{/snippet}<script lang='ts'>debugger;</script>",
            "{#each (items as string[]) as {x = '<textarea>'}}<p>hi</p>{/each}<script lang='ts'>debugger;</script>",
            "{#each `a${value as string}` as {x = '<textarea>'}}<p>hi</p>{/each}<script lang='ts'>debugger;</script>",
        ] {
            let sources = SveltePartialLoader::new(source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;");
        }
    }
    #[test]
    fn test_expression_tokenization_chunk_boundaries() {
        for len in [120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 255, 256, 257] {
            for expression in [
                format!("'{}<textarea>'", "a".repeat(len)),
                format!("'{}<textarea>'", "日".repeat(len)),
                format!(r"/{}}}<textarea>/.source", "a".repeat(len)),
                format!("value {} / 2", " ".repeat(len)),
                format!("value {} // <textarea> }}\n", " ".repeat(len)),
                format!("value /* {} <textarea> }} */", "a".repeat(len)),
                format!("`{}${{value as string}}<textarea>`", "a".repeat(len)),
                format!("`a${{value + {} other}}<textarea>`", " ".repeat(len)),
            ] {
                let source_text = format!("{{@html {expression}}}<script>debugger;</script>");
                let sources = SveltePartialLoader::new(&source_text).parse();
                assert_eq!(sources.len(), 1, "{len}: {expression}");
                assert_eq!(sources[0].source_text, "debugger;");
            }
        }
    }
    #[test]
    fn test_component_attribute_context_after_markup() {
        for prefix in [
            "<br>",
            "<command><keygen>",
            "<div></div>",
            "<div\u{feff}></div>",
            "<p>text<hr>",
            "<p><hr/>",
            "<p><div/>",
            "{#if ok}<p>one{:else}<p>two{/if}",
            "<Widget />",
            "<svelte:head><title>x</title></svelte:head>",
        ] {
            let source_text = format!(
                "{prefix}<style title=\"before {{\">.a {{ color: red; }}</style><script>debugger;</script>"
            );
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_textarea_closing_tag_variants() {
        for closing in
            ["</TEXTAREA>", "</TextArea>", "</textarea\u{feff}>", "</textarea title=\">\">"]
        {
            let source_text = format!("<textarea>text{closing}<script>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_textarea_expression_with_closing_tag_text() {
        let source_text = r"<textarea>{'</textarea><script>ignored</script>'}</textarea><script>debugger;</script>";
        let sources = SveltePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text, "debugger;");
    }

    #[test]
    fn test_preprocessed_styles_preserve_scripts() {
        for lang in ["scss", "sass", "less", "stylus"] {
            for comment in ["// {", "// (", "// \"", "// </stylex>"] {
                let source_text = format!(
                    "<style lang=\"{lang}\">{comment}\n.a {{ color: red; }}\n</style ><script>debugger;</script>"
                );
                let sources = SveltePartialLoader::new(&source_text).parse();
                assert_eq!(sources.len(), 1, "{source_text}");
                assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
            }
        }
    }

    #[test]
    fn test_self_closing_scripts_preserve_component_scripts() {
        for element in ["<script/>", "<script />", "<script/\u{feff}>", "<script title={\"}\"}/>"] {
            let source_text =
                format!("<svelte:head>{element}</svelte:head><script>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_component_style_with_closing_tag_text() {
        for css in [
            r#".a::before { content: "</style><script>ignored</script>"; }"#,
            ".a { /* </style><script>ignored</script> */ color: red; }",
            "/* </style><script>ignored</script> */ .a { color: red; }",
            "<!-- { --> .a { color: red; }",
            "<!-- </style><script>ignored</script> --> .a { color: red; }",
        ] {
            let source_text = format!("<style>{css}</style><script>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_style_attribute_contexts_preserve_scripts() {
        for source_text in [
            r#"<style data-x={"}"}>.a { color: red; }</style><script>const text = `">`; debugger;</script>"#,
            r#"<svelte:head><style title={"}"}>.a { color: red; }</style></svelte:head><script>const text = `">`; debugger;</script>"#,
            r#"<div><style title="before {"></style><script>ignored</script>"} after">.a { color: red; }</style></div><script>debugger;</script>"#,
        ] {
            let sources = SveltePartialLoader::new(source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert!(sources[0].source_text.contains("debugger;"), "{source_text}");
        }
    }
}
