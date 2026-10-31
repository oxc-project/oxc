use memchr::memmem::{Finder, FinderRev};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, ObjectPropertyKind,
    PropertyKey,
};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use oxc_str::CompactStr;

use crate::loader::JavaScriptSource;

use super::{
    AttributeValue, COMMENT_END, COMMENT_START, SCRIPT_END, SCRIPT_START, find_attribute,
    find_script_closing_angle, find_script_start,
};

const ASTRO_SPLIT: &str = "---";

pub struct AstroPartialLoader<'a> {
    source_text: &'a str,
}

impl<'a> AstroPartialLoader<'a> {
    pub fn new(source_text: &'a str) -> Self {
        Self { source_text }
    }

    pub fn parse(self) -> Vec<JavaScriptSource<'a>> {
        let mut results = vec![];
        let mut start = 0;
        if let Some((frontmatter, frontmatter_end)) = self.parse_frontmatter() {
            start = frontmatter_end;
            results.push(frontmatter);
        }
        results.extend(self.parse_scripts(start));
        results
    }

    /// Parse `---` frontmatter block
    #[expect(clippy::cast_possible_truncation)]
    fn parse_frontmatter(&self) -> Option<(JavaScriptSource<'a>, usize)> {
        let split_finder = Finder::new(ASTRO_SPLIT);
        let mut offsets = split_finder
            .find_iter(self.source_text.as_bytes())
            .filter(|offset| self.is_fence(*offset));

        let start = offsets.next()?;
        if !self.source_text[..start].chars().all(char::is_whitespace) {
            return None;
        }
        let end = offsets.next()?;

        let Ok(start) = u32::try_from(start) else {
            return None;
        };
        let Ok(end) = u32::try_from(end) else {
            return None;
        };

        // move start to the end of the ASTRO_SPLIT
        let start = start + ASTRO_SPLIT.len() as u32;
        let js_code = Span::new(start, end).source_text(self.source_text);
        let frontmatter = JavaScriptSource::partial(js_code, SourceType::ts(), start);

        Some((frontmatter, end as usize + ASTRO_SPLIT.len()))
    }

    fn is_fence(&self, offset: usize) -> bool {
        let line_start = self.source_text[..offset].rfind('\n').map_or(0, |index| index + 1);
        if !self.source_text[line_start..offset].chars().all(char::is_whitespace) {
            return false;
        }

        let fence_end = offset + ASTRO_SPLIT.len();
        let line_end = self.source_text[fence_end..]
            .find('\n')
            .map_or(self.source_text.len(), |index| fence_end + index);

        self.source_text[fence_end..line_end].chars().all(char::is_whitespace)
    }

    /// In .astro files, you can add client-side JavaScript by adding one (or more) `<script>` tags.
    /// <https://docs.astro.build/en/guides/client-side-scripts/#using-script-in-astro>
    fn parse_scripts(&self, start: usize) -> Vec<JavaScriptSource<'a>> {
        let script_start_finder = Finder::new(SCRIPT_START);
        let script_end_finder = Finder::new(SCRIPT_END);
        let comment_start_finder = FinderRev::new(COMMENT_START);
        let comment_end_finder = Finder::new(COMMENT_END);

        let mut results = vec![];
        let mut pointer = start;

        loop {
            let js_end;
            // find opening "<script"
            if let Some(offset) = find_script_start(
                self.source_text,
                pointer,
                &script_start_finder,
                &comment_start_finder,
                &comment_end_finder,
            ) {
                pointer += offset;
            } else {
                break;
            }
            // find closing ">"
            let Some(offset) = find_script_closing_angle(self.source_text, pointer) else {
                break;
            };
            let script_opening_tag = &self.source_text[pointer..pointer + offset];
            let is_javascript_script = Self::is_javascript_script(script_opening_tag);
            let define_vars = if is_javascript_script {
                Self::parse_define_vars(script_opening_tag)
            } else {
                Vec::new()
            };

            pointer += offset + 1;
            let js_start = pointer;

            // check for the / of a self closing script tag
            if js_start.checked_sub(2).is_some_and(|slash_offset| {
                self.source_text.as_bytes().get(slash_offset) == Some(&b'/')
            }) {
                js_end = pointer;
            // find "</script>" if no self closing tag was found
            } else if let Some(offset) =
                script_end_finder.find(&self.source_text.as_bytes()[pointer..])
            {
                js_end = pointer + offset;
                pointer += offset + SCRIPT_END.len();
            } else {
                break;
            }

            if !is_javascript_script {
                continue;
            }

            // NOTE: loader checked that source_text.len() is less than u32::MAX
            #[expect(clippy::cast_possible_truncation)]
            results.push(
                JavaScriptSource::partial(
                    &self.source_text[js_start..js_end],
                    SourceType::ts(),
                    js_start as u32,
                )
                .with_globals(define_vars),
            );
        }
        results
    }

    /// Returns the variables that `define:vars={{ ... }}` declares in a `<script>`.
    /// Astro declares each non-computed key of the object as a variable in the script.
    /// <https://docs.astro.build/en/reference/directives-reference/#definevars>
    ///
    /// Astro's attribute syntax is close to JSX, so the opening tag is parsed as a JSX element.
    /// Forms that JSX rejects, such as unquoted attribute values, declare nothing.
    fn parse_define_vars(script_attributes: &str) -> Vec<CompactStr> {
        if !script_attributes.contains("define:vars") {
            return Vec::new();
        }
        let attributes = script_attributes.trim_end();
        let attributes = attributes.strip_suffix('/').unwrap_or(attributes);
        let allocator = Allocator::default();
        let source_text = allocator.alloc_str(&format!("<script{attributes} />"));
        let Ok(Expression::JSXElement(element)) =
            Parser::new(&allocator, source_text, SourceType::tsx()).parse_expression()
        else {
            return Vec::new();
        };

        let Some((name_span, value)) =
            element.opening_element.attributes.iter().find_map(|attribute| {
                let JSXAttributeItem::Attribute(attribute) = attribute else {
                    return None;
                };
                let JSXAttributeName::NamespacedName(name) = &attribute.name else {
                    return None;
                };
                // JSX also accepts whitespace and comments around the colon.
                if name.span.source_text(source_text) != "define:vars" {
                    return None;
                }
                // Astro uses the first directive with a string or expression value.
                attribute
                    .value
                    .as_ref()
                    .filter(|value| {
                        matches!(
                            value,
                            JSXAttributeValue::StringLiteral(_)
                                | JSXAttributeValue::ExpressionContainer(_)
                        )
                    })
                    .map(|value| (name.span, value))
            })
        else {
            return Vec::new();
        };
        let JSXAttributeValue::ExpressionContainer(container) = value else {
            return Vec::new();
        };
        // A comment after `=` is an unquoted Astro value, not JavaScript trivia.
        // Conservatively reject comments anywhere in the name/value gap.
        let value_prefix = Span::new(name_span.end, container.span.start).source_text(source_text);
        if value_prefix.trim_matches([' ', '\t', '\r', '\n']) != "=" {
            return Vec::new();
        }
        let Some(value) = container.expression.as_expression() else {
            return Vec::new();
        };
        let Expression::ObjectExpression(object) = value.without_parentheses() else {
            return Vec::new();
        };

        object
            .properties
            .iter()
            .filter_map(|property| {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    return None;
                };
                if property.computed {
                    return None;
                }
                match &property.key {
                    PropertyKey::StaticIdentifier(ident) => {
                        Some(CompactStr::from(ident.name.as_str()))
                    }
                    // `None` for strings with lone surrogates, which cannot be variable names.
                    PropertyKey::StringLiteral(literal) => {
                        literal.value.as_str().map(CompactStr::from)
                    }
                    _ => None,
                }
            })
            .collect()
    }

    fn is_javascript_script(content: &str) -> bool {
        match find_attribute(content, "type") {
            Some(AttributeValue::Empty) | None => true,
            Some(AttributeValue::Value(value)) => {
                let script_type = value.trim().split(';').next().unwrap_or("").trim();
                script_type.is_empty()
                    || [
                        "module",
                        "text/javascript",
                        "application/javascript",
                        "text/ecmascript",
                        "application/ecmascript",
                    ]
                    .iter()
                    .any(|allowed| script_type.eq_ignore_ascii_case(allowed))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use oxc_str::CompactStr;

    use super::{AstroPartialLoader, JavaScriptSource};

    fn parse_astro(source_text: &str) -> Vec<JavaScriptSource<'_>> {
        AstroPartialLoader::new(source_text).parse()
    }

    #[test]
    fn test_parse_astro() {
        let source_text = r#"
        <h1>Welcome, world!</h1>

        <script>
            console.log("Hi");
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text.trim(), r#"console.log("Hi");"#);
        assert_eq!(sources[0].start, 51);
    }

    #[test]
    fn test_parse_astro_with_fontmatter() {
        let source_text = r#"
        ---
            const { message = 'Welcome, world!' } = Astro.props;
        ---

        <h1>Welcome, world!</h1>

        <script>
            console.log("Hi");
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 2);
        assert_eq!(
            sources[0].source_text.trim(),
            "const { message = 'Welcome, world!' } = Astro.props;"
        );
        assert_eq!(sources[0].start, 12);
        assert_eq!(sources[1].source_text.trim(), r#"console.log("Hi");"#);
        assert_eq!(sources[1].start, 141);
    }

    #[test]
    fn test_parse_astro_with_inline_script() {
        let source_text = r#"
        <h1>Welcome, world!</h1>

        <script is:inline src="https://my-analytics.com/script.js"></script>

        <script>
            console.log("Hi");
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source_text, "");
        assert_eq!(sources[0].start, 102);
        assert_eq!(sources[1].source_text.trim(), r#"console.log("Hi");"#);
        assert_eq!(sources[1].start, 129);
    }

    #[test]
    fn test_script_inside_code_comment() {
        let source_text = r"
        <!-- <script>a</script> -->
        <!-- <script> -->
        <script>b</script>
        ";

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text, "b");
        assert_eq!(sources[0].start, 79);
    }

    #[test]
    fn test_parse_astro_with_inline_script_self_closing() {
        let source_text = r#"
        <h1>Welcome, world!</h1>

        <script is:inline src="https://my-analytics.com/script.js" />

        <script>
            console.log("Hi");
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source_text, "");
        assert_eq!(sources[0].start, 104);
        assert_eq!(sources[1].source_text.trim(), r#"console.log("Hi");"#);
        assert_eq!(sources[1].start, 122);
    }

    #[test]
    fn test_parse_astro_with_inline_script_self_closing_after_unicode() {
        let source_text = r#"
        <h1>日历</h1>

        <script is:inline src="https://my-analytics.com/script.js" />

        <script>
            console.log("Hi");
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source_text, "");
        assert_eq!(sources[1].source_text.trim(), r#"console.log("Hi");"#);
    }

    #[test]
    fn test_parse_astro_skips_non_javascript_script_type() {
        let source_text = r#"
        <script TYPE="importmap">
            {
                "imports": {
                    "swiper": "https://cdn.jsdelivr.net/npm/swiper@12/swiper-bundle.min.mjs"
                }
            }
        </script>

        <script type="application/ld+json">
            { "@context": "https://schema.org" }
        </script>

        <script type="module">
            console.log("Hi");
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text.trim(), r#"console.log("Hi");"#);
    }

    #[test]
    fn test_parse_astro_keeps_javascript_script_type() {
        let source_text = r#"
        <script type="text/javascript">
            console.log("text/javascript");
        </script>

        <script type="application/ecmascript; charset=utf-8">
            console.log("application/ecmascript");
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source_text.trim(), r#"console.log("text/javascript");"#);
        assert_eq!(sources[1].source_text.trim(), r#"console.log("application/ecmascript");"#);
    }

    #[test]
    fn test_parse_astro_frontmatter_with_later_js_separator() {
        let source_text = r#"
        ---
        const title = "hello";
        ---

        <div>{"---"}</div>

        <script>
            const marker = "---";
            console.log(marker);
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source_text.trim(), "const title = \"hello\";");
        assert_eq!(
            sources[1].source_text.trim(),
            "const marker = \"---\";\n            console.log(marker);"
        );
    }

    fn globals<'a>(source: &'a JavaScriptSource<'_>) -> Vec<&'a str> {
        source.globals.iter().map(CompactStr::as_str).collect()
    }

    #[test]
    fn test_parse_astro_define_vars() {
        let source_text = r#"
        ---
        const foo = 1;
        ---
        <script is:inline define:vars={{ foo, bar: foo + 1, "baz": "}", [qux]: 1, ...rest }}>
            console.log(foo, bar, baz);
        </script>

        <script>
            console.log(foo);
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 3);
        assert_eq!(globals(&sources[0]), Vec::<&str>::new());
        assert_eq!(globals(&sources[1]), vec!["foo", "bar", "baz"]);
        assert_eq!(sources[1].source_text.trim(), "console.log(foo, bar, baz);");
        assert_eq!(globals(&sources[2]), Vec::<&str>::new());
    }

    #[test]
    fn test_parse_astro_define_vars_skips_braces_in_tokens() {
        for source_text in [
            r"<script define:vars={{ foo: /}/ }}>foo;</script>",
            r"<script define:vars={{ foo: 1 /* } */ }}>foo;</script>",
            r"<script define:vars={{ foo: `x${`}`}` }}>foo;</script>",
            r"<script data-x={{ a: '}' }} define:vars={{ foo }}>foo;</script>",
        ] {
            let sources = parse_astro(source_text);
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(globals(&sources[0]), vec!["foo"], "{source_text}");
        }
    }

    #[test]
    fn test_parse_astro_define_vars_decodes_keys() {
        let sources = parse_astro(
            r#"<script define:vars={{ \u0066oo: 1, "b\u0061r": 2 }}>foo; bar;</script>"#,
        );
        assert_eq!(sources.len(), 1);
        assert_eq!(globals(&sources[0]), vec!["foo", "bar"]);
    }

    #[test]
    fn test_parse_astro_define_vars_with_parentheses() {
        let sources = parse_astro(r"<script define:vars={({ foo: 1 })}>foo;</script>");
        assert_eq!(sources.len(), 1);
        assert_eq!(globals(&sources[0]), vec!["foo"]);
    }

    #[test]
    fn test_parse_astro_define_vars_ignored() {
        for source_text in [
            // Not an object, or not an expression value.
            r"<script define:vars={vars}>foo;</script>",
            r#"<script define:vars="{{foo}}">foo;</script>"#,
            // Astro matches the directive name exactly.
            r"<script DEFINE:VARS={{ foo }}>foo;</script>",
            // Invalid expression.
            r"<script define:vars={{ foo: }}>foo;</script>",
        ] {
            let sources = parse_astro(source_text);
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "foo;", "{source_text}");
            assert_eq!(globals(&sources[0]), Vec::<&str>::new(), "{source_text}");
        }
    }

    #[test]
    fn test_parse_astro_skips_json_script_after_expression_attribute() {
        let source_text =
            r#"<script data-x={{ foo: 1 /* ' */ }} type="application/json">{"foo":1}</script>"#;
        assert!(parse_astro(source_text).is_empty());
    }

    #[test]
    fn test_parse_astro_define_vars_after_spread_attribute() {
        let sources = parse_astro(r"<script {...attrs} define:vars={{ foo }}>foo;</script>");
        assert_eq!(sources.len(), 1);
        assert_eq!(globals(&sources[0]), vec!["foo"]);
    }

    #[test]
    fn test_parse_astro_define_vars_text_inside_other_values() {
        for source_text in [
            // Inside a comment of another attribute's expression.
            "<script data-x={value // } define:vars={{ foo }}\n}>foo;</script>",
            // A comment hides the brace that would end the object.
            "<script define:vars={{ foo: 1 } // }\n.bar}>foo;</script>",
            // Inside a template literal or unquoted value of another attribute.
            r"<script data-x=`abc define:vars={{ foo }}`>foo;</script>",
            r"<script data-x=abc/define:vars={{ foo }}>foo;</script>",
            "<script data-x=abc\u{a0}define:vars={{ foo }}>foo;</script>",
        ] {
            let sources = parse_astro(source_text);
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "foo;", "{source_text}");
            assert_eq!(globals(&sources[0]), Vec::<&str>::new(), "{source_text}");
        }
    }

    #[test]
    fn test_parse_astro_define_vars_self_closing() {
        let sources = parse_astro(r"<script define:vars={{ foo }} /><script>foo;</script>");
        assert_eq!(sources.len(), 2);
        assert_eq!(globals(&sources[0]), vec!["foo"]);
        assert_eq!(globals(&sources[1]), Vec::<&str>::new());
    }

    #[test]
    fn test_parse_astro_define_vars_name_must_be_contiguous() {
        for source_text in [
            r"<script define :vars={{foo}}>foo;</script>",
            r"<script define: vars={{foo}}>foo;</script>",
            r"<script define : vars={{foo}}>foo;</script>",
            "<script define:\nvars={{foo}}>foo;</script>",
            r"<script define/*x*/:vars={{foo}}>foo;</script>",
            r"<script define:/*x*/vars={{foo}}>foo;</script>",
            "<script define:\u{a0}vars={{foo}}>foo;</script>",
            // A textual absence check alone cannot handle this case.
            r#"<script data-x="define:vars" define : vars={{foo}}>foo;</script>"#,
        ] {
            let sources = parse_astro(source_text);
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "foo;", "{source_text}");
            assert_eq!(globals(&sources[0]), Vec::<&str>::new(), "{source_text}");
        }
    }

    #[test]
    fn test_parse_astro_define_vars_uses_first_directive_with_value() {
        for source_text in [
            r#"<script define:vars="" define:vars={{foo: 1}}>foo;</script>"#,
            r#"<script define:vars="x" define:vars={{foo: 1}}>foo;</script>"#,
            r"<script define:vars={other} define:vars={{foo: 1}}>foo;</script>",
        ] {
            let sources = parse_astro(source_text);
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "foo;", "{source_text}");
            assert_eq!(globals(&sources[0]), Vec::<&str>::new(), "{source_text}");
        }
        for source_text in [
            r"<script define:vars={{foo: 1}} define:vars={{bar: 2}}>foo;</script>",
            r#"<script define:vars={{foo: 1}} define:vars="">foo;</script>"#,
            r"<script define:vars define:vars={{foo: 1}}>foo;</script>",
            r"<script define:vars=<x/> define:vars={{foo: 1}}>foo;</script>",
        ] {
            let sources = parse_astro(source_text);
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(globals(&sources[0]), vec!["foo"], "{source_text}");
        }
    }

    #[test]
    fn test_parse_astro_define_vars_value_must_follow_equals() {
        for source_text in [
            r"<script define:vars=/*comment*/{{foo}}>foo;</script>",
            r"<script define:vars= /*comment*/ {{foo}}>foo;</script>",
            "<script define:vars=//comment\n{{foo}}>foo;</script>",
            "<script define:vars=//=\n{{foo}}>foo;</script>",
            "<script define:vars=\u{a0}{{foo}}>foo;</script>",
            "<script define:vars=\x0c{{foo}}>foo;</script>",
            "<script define:vars\u{a0}={{foo}}>foo;</script>",
        ] {
            let sources = parse_astro(source_text);
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "foo;", "{source_text}");
            assert_eq!(globals(&sources[0]), Vec::<&str>::new(), "{source_text}");
        }
        let sources = parse_astro("<script define:vars \t=\n {{foo}}>foo;</script>");
        assert_eq!(globals(&sources[0]), vec!["foo"]);
    }
}
