//! [Astro](https://astro.build) Parser
//! Astro file parsing support.
//!
//! Astro files have a frontmatter section (TypeScript) delimited by `---` and
//! an HTML body that can contain JSX expressions.
//!
//! ## Structure
//!
//! ```astro
//! ---
//! // TypeScript code (frontmatter)
//! const name = "World";
//! ---
//!
//! <!-- HTML body with JSX expressions -->
//! <h1>Hello {name}!</h1>
//! ```

mod jsx;
mod parse;
mod scripts;

pub use parse::AstroParserReturn;
pub use parse::parse_astro;
pub use scripts::parse_astro_scripts;

use oxc_allocator::{Box, Vec};
use oxc_ast::ast::*;

use crate::{ParserImpl, config::ParserConfig, lexer::Kind};

type NoTypeArgs<'a> = Option<Box<'a, TSTypeParameterInstantiation<'a>>>;
type NoClosingElement<'a> = Option<Box<'a, JSXClosingElement<'a>>>;

impl<'a, C: ParserConfig> ParserImpl<'a, C> {
    /// Parse the HTML body of an Astro file.
    ///
    /// The body is essentially JSX children in an implicit fragment.
    pub(crate) fn parse_astro_body(&mut self) -> Vec<'a, JSXChild<'a>> {
        let mut children = self.ast.vec();

        // Parse JSX children until EOF
        while !self.at(Kind::Eof) && self.fatal_error.is_none() {
            if let Some(child) = self.parse_astro_child() {
                children.push(child);
            } else {
                break;
            }
        }

        children
    }

    /// Parse a single child in the Astro body.
    fn parse_astro_child(&mut self) -> Option<JSXChild<'a>> {
        match self.cur_kind() {
            Kind::LAngle => {
                let span = self.start_span();
                let checkpoint = self.checkpoint();
                self.bump_any(); // bump `<`

                let kind = self.cur_kind();

                // `<>` - fragment
                if kind == Kind::RAngle {
                    return Some(JSXChild::Fragment(self.parse_astro_jsx_fragment(span, true)));
                }

                // `<ident` - element
                if kind == Kind::Ident || kind.is_any_keyword() {
                    // Check if this is a <script> tag - we handle it specially in Astro
                    if self.cur_src().starts_with("script") {
                        // Check it's actually "script" and not "script-something"
                        let after_script =
                            self.source_text.get((self.cur_token().span().start as usize) + 6..);
                        if let Some(rest) = after_script {
                            let next_char: Option<char> = rest.chars().next();
                            if matches!(
                                next_char,
                                Some(' ' | '>' | '/' | '\n' | '\r' | '\t') | None
                            ) {
                                return Some(self.parse_astro_script(span));
                            }
                        }
                    }
                    return Some(JSXChild::Element(self.parse_astro_jsx_element(span, true)));
                }

                // `</` - closing tag (end of parent)
                if kind == Kind::Slash {
                    self.rewind(checkpoint);
                    return None;
                }

                // `<!` - HTML comment or doctype
                if kind == Kind::Bang {
                    if let Some(doctype) = self.parse_html_comment_or_doctype(span) {
                        return Some(doctype);
                    }
                    return self.parse_astro_child();
                }

                self.unexpected()
            }
            Kind::LCurly => {
                // JSX expression container
                let span_start = self.start_span();
                self.bump_any(); // bump `{`

                // `{...expr}` - spread
                if self.eat(Kind::Dot3) {
                    return Some(JSXChild::Spread(self.parse_jsx_spread_child(span_start)));
                }

                // `{expr}` - expression
                Some(JSXChild::ExpressionContainer(self.parse_astro_jsx_expression_container(
                    span_start, /* in_jsx_child */ true,
                )))
            }
            Kind::JSXText => Some(JSXChild::Text(self.parse_jsx_text())),
            Kind::Eof => None,
            _ => {
                // In Astro body, we should be getting JSX tokens
                // If we get something unexpected, try to continue
                self.bump_any();
                self.parse_astro_child()
            }
        }
    }

    /// Parse an HTML comment `<!-- ... -->` or doctype `<!doctype ...>`.
    ///
    /// This handles:
    /// - HTML comments: `<!-- comment -->` - added to trivia, returns `None`
    /// - Doctypes: `<!doctype html>` or `<!DOCTYPE html>` - returns `Some(JSXChild::AstroDoctype)`
    ///
    /// `span` is the start position (at `<`).
    #[expect(clippy::cast_possible_truncation)]
    fn parse_html_comment_or_doctype(&mut self, span: u32) -> Option<JSXChild<'a>> {
        // We're at `!` after `<`, so start position is after `<`
        let start_pos = self.prev_token_end as usize;

        // Check if this is a comment (starts with `<!--`) or a doctype (starts with `<!doctype` or `<!DOCTYPE`)
        if let Some(rest) = self.source_text.get(start_pos..) {
            // Check for HTML comment `<!--`
            if let Some(after_open) = rest.strip_prefix("!--") {
                // Close must come after the open; an overlapping `<!-->` is not a
                // real comment (and naive search would slice backwards and panic).
                if let Some(content_len) = after_open.find("-->") {
                    let comment_end = (start_pos + 3 + content_len + 3) as u32;
                    let comment_start = span; // `<` position

                    let content = &after_open[..content_len];
                    let value = oxc_span::Atom::from(content);

                    // Create the AstroComment node
                    let comment_span = oxc_span::Span::new(comment_start, comment_end);
                    let comment = self.ast.alloc_astro_comment(comment_span, value);

                    // Move lexer position past the comment
                    self.lexer.set_position_for_astro(comment_end);
                    self.token = self.lexer.next_jsx_child();

                    return Some(JSXChild::AstroComment(comment));
                }
            } else {
                // This is likely a doctype or other `<!...>` construct
                // Find the closing `>` (not `-->`)
                if let Some(end_offset) = rest.find('>') {
                    let end_pos = (start_pos + end_offset + 1) as u32;

                    // Extract the doctype value (e.g., "html" from "doctype html" or "DOCTYPE html")
                    // rest starts with `!`, so content is after `!`
                    let content = &rest[1..end_offset];
                    // Skip "doctype" or "DOCTYPE" (case-insensitive) and any whitespace
                    let value = content
                        .strip_prefix("doctype")
                        .or_else(|| content.strip_prefix("DOCTYPE"))
                        .or_else(|| content.strip_prefix("Doctype"))
                        .map_or(content.trim(), str::trim);
                    let value = oxc_span::Atom::from(value);

                    // Create the doctype node
                    let doctype_span = oxc_span::Span::new(span, end_pos);
                    let doctype = self.ast.alloc_astro_doctype(doctype_span, value);

                    // Move lexer position past the doctype/construct
                    self.lexer.set_position_for_astro(end_pos);
                    self.token = self.lexer.next_jsx_child();

                    return Some(JSXChild::AstroDoctype(doctype));
                }
            }
        }

        // Fallback: skip tokens until we find `>` or `-->`
        // This handles malformed comments/doctypes
        self.bump_any(); // skip `!`
        while !self.at(Kind::Eof) {
            if self.at(Kind::RAngle) {
                self.bump_any();
                break;
            } else if self.at(Kind::Minus2) || self.at(Kind::Minus) {
                self.bump_any();
                if self.at(Kind::Minus) {
                    self.bump_any();
                }
                if self.at(Kind::RAngle) {
                    self.bump_any();
                    break;
                }
            } else {
                self.bump_any();
            }
        }
        None // Fallback case, treat as comment (trivia)
    }

    /// Parse an Astro `<script>` element.
    ///
    /// According to the Astro spec and HTML rules:
    /// - `<script>` with no `type` attribute, or with a JS/module MIME `type` →
    ///   parsed as TypeScript/JavaScript (`AstroScript`)
    /// - `<script type="text/css">` (or any other non-JS `type`) →
    ///   raw text (not parsed)
    ///
    /// Non-`type` attributes (e.g. `defer`, `src`, `is:inline`) do **not** affect
    /// whether the content is parsed.
    #[expect(clippy::cast_possible_truncation)]
    fn parse_astro_script(&mut self, span: u32) -> JSXChild<'a> {
        // We're at `script` identifier after `<`
        // Skip the `script` identifier
        self.bump_any();

        // Parse attributes using the Astro JSX attribute parser
        let attributes = self.parse_astro_jsx_attributes();
        let has_attributes = Self::is_raw_text_script(&attributes);

        // Check for self-closing `/>`
        let is_self_closing = if self.at(Kind::Slash) {
            self.bump_any(); // skip `/`
            true
        } else {
            false
        };

        if self.at(Kind::RAngle) {
            self.bump_any(); // skip `>`
        }

        // Self-closing script tag
        if is_self_closing {
            let end = self.prev_token_end;
            let script_span = oxc_span::Span::new(span, end);

            if has_attributes {
                // Script with attributes - return as regular JSX element with no children
                let name =
                    self.ast.jsx_identifier(oxc_span::Span::new(span + 1, span + 7), "script");
                let elem_name = JSXElementName::Identifier(self.alloc(name));
                let no_type_args: NoTypeArgs<'a> = None;
                let opening = self.ast.alloc_jsx_opening_element(
                    script_span,
                    elem_name,
                    no_type_args,
                    attributes,
                );
                let no_closing: NoClosingElement<'a> = None;
                return JSXChild::Element(self.ast.alloc_jsx_element(
                    script_span,
                    opening,
                    self.ast.vec(),
                    no_closing,
                ));
            }

            // Bare self-closing script - wrap empty AstroScript in JSX element
            let program = self.ast.program(
                oxc_span::Span::empty(end),
                self.source_type,
                "",
                self.ast.vec(),
                None,
                self.ast.vec(),
                self.ast.vec(),
            );
            let astro_script =
                JSXChild::AstroScript(self.ast.alloc_astro_script(script_span, program));

            // Wrap in a <script> element for consistency
            let name = self.ast.jsx_identifier(oxc_span::Span::new(span + 1, span + 7), "script");
            let elem_name = JSXElementName::Identifier(self.alloc(name));
            let no_type_args: NoTypeArgs<'a> = None;
            let opening = self.ast.alloc_jsx_opening_element(
                script_span,
                elem_name,
                no_type_args,
                attributes,
            );
            let no_closing: NoClosingElement<'a> = None;
            return JSXChild::Element(self.ast.alloc_jsx_element(
                script_span,
                opening,
                self.ast.vec1(astro_script),
                no_closing,
            ));
        }

        // Find the closing </script> tag.
        // Use `prev_token_end` (right after `>`) instead of `cur_token().span().start` so
        // that leading trivia (e.g. `// eslint-disable-next-line` comments) before the first
        // real token is included in the content span and picked up by the parser.
        let content_start = self.prev_token_end as usize;
        let closing_tag = "</script";

        if let Some(rest) = self.source_text.get(content_start..)
            && let Some(end_offset) = rest.find(closing_tag)
        {
            let content_end = content_start + end_offset;

            // Move lexer to the closing tag
            #[expect(clippy::cast_possible_truncation)]
            self.lexer.set_position_for_astro(content_end as u32);
            self.token = self.lexer.next_jsx_child();

            // Find the `>` that closes `</script>` by scanning source text directly,
            // rather than bumping tokens (which would use the wrong lexer mode).
            let close_end = {
                let close_source = &self.source_text[content_end..];
                // close_source starts with "</script", find the closing ">"
                if let Some(gt) = close_source.find('>') {
                    content_end + gt + 1
                } else {
                    content_end
                }
            };

            #[expect(clippy::cast_possible_truncation)]
            let end = close_end as u32;
            // Position the lexer right after `</script>` and re-enter JSX child
            // mode so the next token is correctly tokenised.
            self.lexer.set_position_for_astro(end);
            self.token = self.lexer.next_jsx_child();
            let full_span = oxc_span::Span::new(span, end);

            if has_attributes {
                // Script with attributes - return as regular JSX element
                // The content is raw text, not parsed
                let opening_name =
                    self.ast.jsx_identifier(oxc_span::Span::new(span + 1, span + 7), "script");
                let opening_elem_name = JSXElementName::Identifier(self.alloc(opening_name));
                let no_type_args: NoTypeArgs<'a> = None;
                let opening = self.ast.alloc_jsx_opening_element(
                    oxc_span::Span::new(span, content_start as u32),
                    opening_elem_name,
                    no_type_args,
                    attributes,
                );

                let closing_name = self
                    .ast
                    .jsx_identifier(oxc_span::Span::new(content_end as u32 + 2, end - 1), "script");
                let closing_elem_name = JSXElementName::Identifier(self.alloc(closing_name));
                let closing = self.ast.alloc_jsx_closing_element(
                    oxc_span::Span::new(content_end as u32, end),
                    closing_elem_name,
                );

                // Create a text child for the raw content
                let text_span = oxc_span::Span::new(content_start as u32, content_end as u32);
                let raw_text = &self.source_text[content_start..content_end];
                let text_node = self.ast.alloc_jsx_text(
                    text_span,
                    oxc_span::Atom::from(raw_text),
                    Some(oxc_span::Atom::from(raw_text)),
                );

                return JSXChild::Element(self.ast.alloc_jsx_element(
                    full_span,
                    opening,
                    self.ast.vec1(JSXChild::Text(text_node)),
                    Some(closing),
                ));
            }

            // Bare script - wrap AstroScript in <script> element
            let script_content_span = oxc_span::Span::new(content_start as u32, content_end as u32);

            let program = self.ast.program(
                script_content_span,
                self.source_type,
                "",
                self.ast.vec(),
                None,
                self.ast.vec(),
                self.ast.vec(),
            );
            let astro_script =
                JSXChild::AstroScript(self.ast.alloc_astro_script(full_span, program));

            // Create <script> opening element
            let opening_name =
                self.ast.jsx_identifier(oxc_span::Span::new(span + 1, span + 7), "script");
            let opening_elem_name = JSXElementName::Identifier(self.alloc(opening_name));
            let no_type_args: NoTypeArgs<'a> = None;
            let opening = self.ast.alloc_jsx_opening_element(
                oxc_span::Span::new(span, content_start as u32),
                opening_elem_name,
                no_type_args,
                attributes,
            );

            // Create </script> closing element
            let closing_name = self
                .ast
                .jsx_identifier(oxc_span::Span::new(content_end as u32 + 2, end - 1), "script");
            let closing_elem_name = JSXElementName::Identifier(self.alloc(closing_name));
            let closing = self.ast.alloc_jsx_closing_element(
                oxc_span::Span::new(content_end as u32, end),
                closing_elem_name,
            );

            return JSXChild::Element(self.ast.alloc_jsx_element(
                full_span,
                opening,
                self.ast.vec1(astro_script),
                Some(closing),
            ));
        }

        // Fallback: couldn't find closing tag, parse as regular element
        JSXChild::Element(self.parse_astro_jsx_element(span, true))
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_ast::ast::{
        Expression, JSXAttributeItem, JSXAttributeName, JSXChild, JSXElementName, JSXExpression,
        Statement,
    };
    use oxc_span::SourceType;

    use crate::Parser;

    #[test]
    fn parse_astro_smoke_test() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        // Frontmatter is always present (synthetic empty one for files without actual frontmatter)
        assert!(ret.root.frontmatter.is_some());
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        // Synthetic frontmatter has 0-length span and empty body
        assert_eq!(frontmatter.span.start, 0);
        assert_eq!(frontmatter.span.end, 0);
        assert!(frontmatter.program.body.is_empty());
        assert!(ret.root.body.is_empty());
        assert!(ret.errors.is_empty());
    }

    #[test]
    fn parse_astro_with_frontmatter() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"---
const name = "World";
---
<h1>Hello</h1>
"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Check frontmatter - now contains a parsed Program
        assert!(ret.root.frontmatter.is_some());
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        // The program should have one statement (const declaration)
        assert_eq!(frontmatter.program.body.len(), 1);
        assert!(matches!(frontmatter.program.body[0], Statement::VariableDeclaration(_)));

        // Check body has at least one element
        assert!(!ret.root.body.is_empty());
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_without_frontmatter() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<div>Hello</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Frontmatter is always present (synthetic empty one for files without actual frontmatter)
        assert!(ret.root.frontmatter.is_some());
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        // Synthetic frontmatter has 0-length span and empty body
        assert_eq!(frontmatter.span.start, 0);
        assert_eq!(frontmatter.span.end, 0);
        assert!(frontmatter.program.body.is_empty());

        // Body should have one element
        assert_eq!(ret.root.body.len(), 1);
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_with_jsx_expression() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Simpler test case: just an expression in JSX
        let source = r#"---
const name = "World";
---
<div>{name}</div>
"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Check frontmatter - now contains a parsed Program
        assert!(ret.root.frontmatter.is_some());
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        assert_eq!(frontmatter.program.body.len(), 1);

        // Check body
        assert!(!ret.root.body.is_empty());
        // First element should be the <div>
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_fragment() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<><div>1</div><div>2</div></>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one fragment
        assert_eq!(ret.root.body.len(), 1);
        assert!(matches!(ret.root.body[0], JSXChild::Fragment(_)));
    }

    #[test]
    fn parse_astro_script() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<div>Hello</div>\n<script>\nconst x = 1;\nconst y = 2;\nconsole.log(x + y);\n</script>\n<div>World</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have 5 children: div, text(\n), script, text(\n), div
        assert_eq!(ret.root.body.len(), 5, "Expected 5 children, got {}", ret.root.body.len());

        // First should be an element
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));

        // Second should be text (newline)
        assert!(matches!(ret.root.body[1], JSXChild::Text(_)));

        // Third should be a <script> Element containing an AstroScript with parsed TypeScript
        match &ret.root.body[2] {
            JSXChild::Element(el) => {
                // Check it's a <script> element
                if let JSXElementName::Identifier(ident) = &el.opening_element.name {
                    assert_eq!(ident.name.as_str(), "script");
                } else {
                    panic!("Expected script identifier");
                }
                // Should have one child: AstroScript
                assert_eq!(el.children.len(), 1, "Script element should have 1 child");
                if let JSXChild::AstroScript(script) = &el.children[0] {
                    // Should have 3 statements: 2 const declarations + 1 expression statement
                    assert_eq!(
                        script.program.body.len(),
                        3,
                        "Expected 3 statements in script, got {}",
                        script.program.body.len()
                    );
                    assert!(matches!(script.program.body[0], Statement::VariableDeclaration(_)));
                    assert!(matches!(script.program.body[1], Statement::VariableDeclaration(_)));
                    assert!(matches!(script.program.body[2], Statement::ExpressionStatement(_)));
                } else {
                    panic!("Expected AstroScript child, got {:?}", el.children[0]);
                }
            }
            other => panic!("Expected Element, got {other:?}"),
        }

        // Fourth should be text (newline between </script> and <div>)
        assert!(matches!(ret.root.body[3], JSXChild::Text(_)));

        // Fifth should be an element
        assert!(matches!(ret.root.body[4], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_complex() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"---
        import CardSkeleton from "$components/card/CardSkeleton.astro"
        import ReturnHome from "$components/controls/ReturnHome.astro"
        import Layout from "$layouts/Layout.astro"

        /**
         * Catalogue page
         */
        const pageTitle = "Catalogue"
        ---

        <Layout
	title={pageTitle}
	description="Where I keep track of books, movies, songs, video games, and other media I consume."
        >
	<Fragment slot="head">
		<link
			rel="alternate"
			type="text/plain"
			href="/catalogue.llm"
			title="Catalogue (LLM/plain-text)"
		/>
		<!-- Optionally hint crawlers via robots meta; human visitors won't see this. -->
		<meta name="robots" content="index,follow" />
	</Fragment>

	<section>
		<h1>{pageTitle}</h1>
		<p>
			Where I keep track of books, movies, songs, video games, and other media I
			consume. Keep in mind that this is a personal catalogue, incomplete and
			biased.
		</p>
		<p>
			<ReturnHome /> - <a href="/2025/catalogue-astro-turso">Learn more</a> - <a
				href="/catalogue/wrapped"
				class="">Yearly Wrap</a
			>
		</p>
	</section>
	<section>
		<form class="mb-5 grid grid-cols-2 gap-5 text-xs sm:grid-cols-3 sm:text-sm">
			<div class="group col-span-2 flex items-center sm:col-span-3">
				<i class="fas fa-search group-has-focus:text-primary mr-1.5"></i>
				<input
					id="query"
					type="text"
					placeholder="Search by title, author, keywords, etc."
					class="group-has-focus:text-primary w-full truncate overflow-hidden border-b-[0.1rem] border-dotted whitespace-nowrap outline-hidden placeholder:truncate"
				/>
			</div>

			<select id="source-filter" class="border-b-[0.1rem] border-dotted">
				<option value="" selected>⭐ everything</option>
				<option value="IGDB">🎮 video games</option>
				<option value="BGG">🎲 board games</option>
				<option value="TMDB_MOVIE">🎬 movies</option>
				<option value="TMDB_TV">📺 shows</option>
				<option value="SPOTIFY">💿 albums</option>
			</select>

			<select id="emotions-filter" class="border-b-[0.1rem] border-dotted">
				<option value="" selected>🎭 all emotions</option>
			</select>

			<select
				id="sort-filter"
				class="col-span-2 border-b-[0.1rem] border-dotted sm:col-span-1"
			>
				<option value="date" selected>🗓️ by date</option>
				<option value="rating">😍 by rating</option>
			</select>

			<!--
			I'm not sure if I want this filter or not. It's useful, but I don't like the vibe.
			So it's hidden for now, but I can still filter by rating using the URL parameter.
			-->
			<select id="rating-filter" class="hidden border-b-[0.1rem] border-dotted">
				<option value="" selected>⭐ all ratings</option>
				<option value="6">⭐ favorites</option>
				<option value="5">😍 loved it</option>
				<option value="4">😀 liked it</option>
				<option value="3">😐 meh'd it</option>
				<option value="2">🙁 disliked it</option>
				<option value="1">😡 hated it</option>
			</select>
		</form>

		<div id="reviews-container">
			<CardSkeleton />
		</div>
		<button id="load-more" class="button" type="button"> Load more </button>
		<side-note>
			Images and data are fetched from <a href="https://igdb.com/"
				><abbr title="The Internet Game Database">IGDB</abbr></a
			> for video games, <a href="https://boardgamegeek.com"
				><abbr title="Board Game Geek">BGG</abbr></a
			> for board games, <a href="https://themoviedb.org/"
				><abbr title="The Movie DataBase">TMDB</abbr></a
			> for movies and shows, and <a href="https://spotify.com/">Spotify</a> for albums.
			Their licenses apply.
		</side-note>
	</section>

	<!-- Non-JS fallback: show a tiny link so text-only agents and no-JS users can reach LLM version -->
	<noscript>
		<p class="sr-only">
			This page has a text-only version for crawlers and LLMs: <a
				href="/catalogue.llm">/catalogue.llm</a
			>
		</p>
	</noscript>

	<script>
		import type { Emotion } from "./api/catalogue/emotions"
		import type { Review } from "./api/catalogue/reviews"
		import { ReviewCard } from "../components/catalogue/ReviewCard.ts"
		import { CardSkeleton } from "../components/card/CardSkeleton.ts"
		import { ErrorState } from "../components/card/ErrorState.ts"
		import { EmptyState } from "../components/card/EmptyState.ts"

		/**
		 * Sync filter & pagination state to the URL
		 */
		const PAGE_SIZE = 5
		let currentOffset = 0

		// Used so stale requests shouldn't win the race
		let inflight: AbortController | null = null

		function updateUrlNow(filters: ReturnType<typeof getFilterValues>) {
			const params = new URLSearchParams()
			if (filters.query) params.set("query", filters.query)
			if (filters.rating) params.set("rating", filters.rating)
			if (filters.source) params.set("source", filters.source)
			if (filters.emotion) params.set("emotion", filters.emotion)
			if (filters.sort && filters.sort !== "date")
				params.set("sort", filters.sort)

			history.replaceState(
				null,
				"",
				`${location.pathname}${params.toString() ? "?" + params : ""}`,
			)
		}

		// Fetch and display emotions, and populate the map
		const allEmotionsMap = new Map<string | number, Emotion>()
		async function loadEmotions() {
			const select = document.getElementById("emotions-filter")
			if (!select) return

			// Reset to default option while loading
			select.innerHTML = '<option value="" selected>🎭 all emotions</option>'

			try {
				const response = await fetch("/api/catalogue/emotions")
				if (!response.ok)
					throw new Error(
						`HTTP ${response.status} ${response.statusText}:  ${response.text()}`,
					)
				const emotions: Emotion[] = await response.json()
				const sortedEmotions = [...emotions].sort((a, b) =>
					a.name.localeCompare(b.name),
				)
				// Build <option> elements
				sortedEmotions.forEach((emotion: Emotion) => {
					allEmotionsMap.set(emotion.id, emotion)

					const opt = document.createElement("option")
					opt.value = String(emotion.id)
					opt.textContent = `${emotion.emoji} ${emotion.name}`
					select.appendChild(opt)
				})
			} catch (err) {
				console.error("Failed to load emotions:", err)
				const fallback = document.createElement("option")
				fallback.disabled = true
				fallback.textContent = "⚠️ failed to load emotions"
				select.appendChild(fallback)
				throw err
			}
		}

		// Get current filter values from the form
		function getFilterValues() {
			const queryInput = document.getElementById("query") as HTMLInputElement
			const ratingFilter = document.getElementById(
				"rating-filter",
			) as HTMLSelectElement | null
			const sourceFilter = document.getElementById(
				"source-filter",
			) as HTMLSelectElement
			const emotionsFilter = document.getElementById(
				"emotions-filter",
			) as HTMLSelectElement
			const sortFilter = document.getElementById(
				"sort-filter",
			) as HTMLSelectElement

			return {
				query: queryInput?.value || "",
				rating: ratingFilter?.value || "", // Still usable from the URL
				source: sourceFilter?.value || "",
				emotion: emotionsFilter?.value || "",
				sort: sortFilter?.value || "date",
			}
		}

		// Fetch and display reviews, using the populated emotions map
		async function loadReviews({ append = false } = {}) {
			const filters = getFilterValues()
			updateUrlNow(filters)

			const container = document.getElementById("reviews-container")
			if (!container) return

			// Show loading state
			const loadMoreBtn = document.getElementById("load-more")
			if (append) {
				if (loadMoreBtn) {
					loadMoreBtn.classList.add("hidden")
					const skeleton = new CardSkeleton()
					skeleton.id = "load-more-skeleton"
					container.appendChild(skeleton)
				}
			} else {
				container.innerHTML = ""
				container.appendChild(new CardSkeleton())
			}

			// Cancel older fetch (if any)
			inflight?.abort()
			inflight = new AbortController()

			const params = new URLSearchParams()
			if (filters.query) params.append("query", filters.query)
			if (filters.rating) params.append("rating", filters.rating)
			if (filters.source) params.append("source", filters.source)
			if (filters.emotion) params.append("emotion", filters.emotion)
			if (filters.sort && filters.sort !== "date")
				params.append("sort", filters.sort)
			params.set("limit", String(PAGE_SIZE))
			params.set("offset", String(currentOffset))
			const url = `/api/catalogue/reviews${params.toString() ? "?" + params.toString() : ""}`

			try {
				const response = await fetch(url, { signal: inflight.signal })
				if (!response.ok)
					throw new Error(
						`HTTP ${response.status} ${response.statusText}:  ${response.text()}`,
					)
				const { reviews, hasMore } = (await response.json()) as {
					reviews: Review[]
					hasMore: boolean
				}

				if (!append) container.innerHTML = ""
				else {
					const skeleton = document.getElementById("load-more-skeleton")
					if (skeleton) skeleton.remove()
				}

				if (reviews.length === 0 && !append) {
					container.appendChild(new EmptyState())
				} else {
					reviews.forEach((review) => {
						const reviewCard = new ReviewCard()
						reviewCard.setReviewData(review, allEmotionsMap)
						container.appendChild(reviewCard)
					})
				}

				// Show or hide the load‑more button
				if (loadMoreBtn) loadMoreBtn.classList.toggle("hidden", !hasMore)
			} catch (error: any) {
				if (append) {
					const skeleton = document.getElementById("load-more-skeleton")
					if (skeleton) skeleton.remove()
					if (loadMoreBtn) loadMoreBtn.classList.remove("hidden")
				}
				if (error.name === "AbortError") return // out-of-date request
				console.error("Failed to load reviews:", error)
				container.innerHTML = ""
				container.appendChild(new ErrorState())
			}
		}

		// Add event listeners to form controls to trigger filtering
		function setupFilterListeners() {
			const queryInput = document.getElementById("query") as HTMLInputElement
			const ratingFilter = document.getElementById(
				"rating-filter",
			) as HTMLSelectElement | null
			const sourceFilter = document.getElementById(
				"source-filter",
			) as HTMLSelectElement
			const emotionsFilter = document.getElementById(
				"emotions-filter",
			) as HTMLSelectElement
			const sortFilter = document.getElementById(
				"sort-filter",
			) as HTMLSelectElement

			// Use input event for text search with small delay
			let debounceTimer: number | undefined
			queryInput?.addEventListener("input", () => {
				clearTimeout(debounceTimer)
				debounceTimer = setTimeout(() => {
					currentOffset = 0
					loadReviews()
				}, 200) as unknown as number
			})

			// Unified change handling
			;[ratingFilter, sourceFilter, emotionsFilter, sortFilter].forEach((el) =>
				el?.addEventListener("change", () => {
					currentOffset = 0
					loadReviews()
				}),
			)

			// Load‑more button
			document.getElementById("load-more")?.addEventListener("click", () => {
				currentOffset += PAGE_SIZE
				loadReviews({ append: true })
			})
		}

		document.addEventListener("DOMContentLoaded", async () => {
			try {
				await loadEmotions()

				// Prevent form submission (which would reload the page)
				const form = document.querySelector("form")
				form?.addEventListener("submit", (e) => {
					e.preventDefault()
				})

				// Restore state from URL (if any)
				const params = new URLSearchParams(location.search)
				const queryInput = document.getElementById("query") as HTMLInputElement
				const ratingFilter = document.getElementById(
					"rating-filter",
				) as HTMLSelectElement | null
				const sourceFilter = document.getElementById(
					"source-filter",
				) as HTMLSelectElement
				const emotionsFilter = document.getElementById(
					"emotions-filter",
				) as HTMLSelectElement
				const sortFilter = document.getElementById(
					"sort-filter",
				) as HTMLSelectElement
				if (params.has("query")) queryInput.value = params.get("query") ?? ""
				if (params.has("rating") && ratingFilter)
					ratingFilter.value = params.get("rating") ?? ""
				if (params.has("source"))
					sourceFilter.value = params.get("source") ?? ""
				if (params.has("emotion"))
					emotionsFilter.value = params.get("emotion") ?? ""
				if (params.has("sort")) sortFilter.value = params.get("sort") ?? "date"
				currentOffset = 0
				await loadReviews()
				setupFilterListeners()
			} catch (error) {
				console.error("Error initializing catalogue page:", error)
			}
		})

		/**
		 * Hidden function to update existing reviews with IGDB data
		 * <button id="sync-igdb" class="text-primary hover:text-primary/80 text-xs underline">
		 *	🔄 Sync IGDB covers
		 * </button>
		 */
		document
			.getElementById("sync-igdb")
			?.addEventListener("click", async (e) => {
				e.preventDefault()

				const password = prompt("Catalogue password ?")
				if (!password) return

				try {
					const res = await fetch("/api/catalogue/reviews", {
						method: "PATCH",
						headers: { "Content-Type": "application/json" },
						body: JSON.stringify({ password, task: "syncIGDB" }),
					})

					const json = await res.json()
					if (res.ok && json.ok) {
						alert(`✅ ${json.updated} review(s) updated.`)
						await loadReviews()
					} else {
						alert(`❌ ${json.error ?? res.status}`)
					}
				} catch (err) {
					console.error(err)
					alert("❌ Network or server error.")
				}
			})
	</script>
        </Layout>
"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Check frontmatter is parsed as TypeScript
        assert!(ret.root.frontmatter.is_some());
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        // Should have 4 statements: 3 imports + 1 const declaration
        assert_eq!(frontmatter.program.body.len(), 4, "Expected 4 statements in frontmatter");
        // First 3 should be import declarations
        assert!(matches!(frontmatter.program.body[0], Statement::ImportDeclaration(_)));
        assert!(matches!(frontmatter.program.body[1], Statement::ImportDeclaration(_)));
        assert!(matches!(frontmatter.program.body[2], Statement::ImportDeclaration(_)));
        // Last should be a variable declaration
        assert!(matches!(frontmatter.program.body[3], Statement::VariableDeclaration(_)));

        // Check body has the Layout element (may have leading whitespace text)
        assert!(!ret.root.body.is_empty());
        // Find the first non-text element
        let first_element =
            ret.root.body.iter().find(|child| matches!(child, JSXChild::Element(_)));
        assert!(first_element.is_some(), "Expected at least one JSXChild::Element in body");
    }

    #[test]
    fn parse_astro_frontmatter_with_whitespace_before_fence() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Whitespace before opening fence is allowed per spec
        let source = "  ---\nconst x = 1;\n---\n<div>test</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have frontmatter
        assert!(ret.root.frontmatter.is_some());
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        assert_eq!(frontmatter.program.body.len(), 1);
        assert!(matches!(frontmatter.program.body[0], Statement::VariableDeclaration(_)));
    }

    #[test]
    fn parse_astro_frontmatter_content_before_fence() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Content before opening fence is allowed (customarily ignored)
        let source = "ignored content\n---\nconst x = 1;\n---\n<div>test</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have frontmatter
        assert!(ret.root.frontmatter.is_some());
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        assert_eq!(frontmatter.program.body.len(), 1);
        assert!(matches!(frontmatter.program.body[0], Statement::VariableDeclaration(_)));
    }

    #[test]
    fn parse_astro_frontmatter_code_on_opening_line() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Code on same line as opening fence
        let source = "---const x = 1;\n---\n<div>test</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have frontmatter with 1 statement
        assert!(ret.root.frontmatter.is_some());
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        assert_eq!(frontmatter.program.body.len(), 1);
        assert!(matches!(frontmatter.program.body[0], Statement::VariableDeclaration(_)));
    }

    #[test]
    fn parse_astro_frontmatter_code_on_closing_line() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Code on same line as closing fence
        let source = "---\nconst x = 1;---\n<div>test</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have frontmatter with 1 statement
        assert!(ret.root.frontmatter.is_some());
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        assert_eq!(frontmatter.program.body.len(), 1);
        assert!(matches!(frontmatter.program.body[0], Statement::VariableDeclaration(_)));
    }

    #[test]
    fn parse_astro_frontmatter_compact() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Both opening and closing on same "line" with code
        let source = "---const x = 1;---<div>test</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have frontmatter with 1 statement
        assert!(ret.root.frontmatter.is_some());
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        assert_eq!(frontmatter.program.body.len(), 1);
        assert!(matches!(frontmatter.program.body[0], Statement::VariableDeclaration(_)));

        // Body should have the div element
        assert!(!ret.root.body.is_empty());
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_attribute_with_at_sign() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div @click="handler" />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one element
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            // Should have one attribute
            assert_eq!(element.opening_element.attributes.len(), 1);
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_attribute_with_colon_self_closing() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div :class="hey" />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one element
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            // Should have one attribute
            assert_eq!(element.opening_element.attributes.len(), 1);
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_attribute_with_colon_not_self_closing() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div :class="hey"></div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one element
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            // Should have one attribute
            assert_eq!(element.opening_element.attributes.len(), 1);
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_attribute_colon_with_space_two_attrs() {
        // In Astro: `<div : hello>` is two attributes: `:` and `hello`
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"<div : hello></div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one element
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            // Should have TWO attributes: `:` and `hello`
            assert_eq!(
                element.opening_element.attributes.len(),
                2,
                "Expected 2 attributes, got {:?}",
                element.opening_element.attributes
            );
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_attribute_colon_no_space_one_attr() {
        // In Astro: `<div :hello>` is one attribute: `:hello`
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"<div :hello></div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one element
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            // Should have ONE attribute: `:hello`
            assert_eq!(
                element.opening_element.attributes.len(),
                1,
                "Expected 1 attribute, got {:?}",
                element.opening_element.attributes
            );
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_attribute_with_dot() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div x.data="value" />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one element
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            assert_eq!(element.opening_element.attributes.len(), 1);
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_attribute_shorthand() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"<Component {prop} />";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one element
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            // Should have one attribute (the shorthand expanded)
            assert_eq!(element.opening_element.attributes.len(), 1);
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_attribute_shorthand_spans() {
        use oxc_ast::ast::JSXAttributeValue;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"<Component {prop} />";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let JSXChild::Element(element) = &ret.root.body[0] else {
            panic!("Expected JSXChild::Element");
        };
        let JSXAttributeItem::Attribute(attr) = &element.opening_element.attributes[0] else {
            panic!("Expected Attribute, got SpreadAttribute");
        };
        let JSXAttributeName::Identifier(name) = &attr.name else {
            panic!("expected ident name");
        };
        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            panic!("expected expression container value");
        };

        assert_eq!(attr.span.source_text(source), "{prop}");
        assert_eq!(container.span.source_text(source), "{prop}");
        assert_eq!(name.span.source_text(source), "prop");
    }

    /// Shorthand is desugared away, so this span relationship is all consumers have to detect it.
    #[test]
    fn parse_astro_attribute_shorthand_is_distinguishable_by_span() {
        let shorthand =
            ["<C {prop} />", "<C   {prop}   />", "<C {{ a: 1 }} />", "<C {obj?.prop} />"];
        let longhand = [
            "<C class=\"a\" />",
            "<C    class=\"a\" />",
            "<C\n  class=\"a\"\n/>",
            "<C client:load />",
            "<C a:b=\"c\" />",
            "<C disabled />",
            "<C x={y} />",
            "<C x=`y` />",
        ];

        for (source, expected) in
            shorthand.iter().map(|s| (s, true)).chain(longhand.iter().map(|s| (s, false)))
        {
            let allocator = Allocator::default();
            let ret = Parser::new(&allocator, source, SourceType::astro()).parse_astro();
            assert!(!ret.panicked, "{source:?} panicked: {:?}", ret.errors);
            assert!(ret.errors.is_empty(), "{source:?} errors: {:?}", ret.errors);

            let JSXChild::Element(element) = &ret.root.body[0] else {
                panic!("{source:?}: expected JSXChild::Element");
            };
            let JSXAttributeItem::Attribute(attr) = &element.opening_element.attributes[0] else {
                panic!("{source:?}: expected Attribute");
            };
            let JSXAttributeName::Identifier(name) = &attr.name else {
                panic!("{source:?}: expected ident name");
            };

            assert_eq!(
                attr.span.start < name.span.start,
                expected,
                "{source:?}: attribute span {:?} vs name span {:?}",
                attr.span,
                name.span
            );
            if expected {
                assert_eq!(&source[attr.span.start as usize..][..1], "{", "{source:?}");
            }
        }
    }

    // A non-identifier `{expr}` is a shorthand whose name is the expression source text.
    #[test]
    fn parse_astro_attribute_shorthand_object_expression() {
        use oxc_ast::ast::JSXAttributeValue;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"<Debug {{answer: sum(2, 4)}} />";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let JSXChild::Element(element) = &ret.root.body[0] else {
            panic!("Expected JSXChild::Element");
        };
        let attrs = &element.opening_element.attributes;
        assert_eq!(attrs.len(), 1);

        let JSXAttributeItem::Attribute(attr) = &attrs[0] else {
            panic!("Expected Attribute, got SpreadAttribute");
        };
        let JSXAttributeName::Identifier(name) = &attr.name else {
            panic!("expected ident name");
        };
        assert_eq!(name.name.as_str(), "{answer: sum(2, 4)}");

        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            panic!("expected expression container value");
        };
        assert!(matches!(container.expression, JSXExpression::ObjectExpression(_)));
    }

    // `{...spread}` must still parse as a spread attribute, not a shorthand.
    #[test]
    fn parse_astro_attribute_spread() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"<Component {...props} />";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let JSXChild::Element(element) = &ret.root.body[0] else {
            panic!("Expected JSXChild::Element");
        };
        let attrs = &element.opening_element.attributes;
        assert_eq!(attrs.len(), 1);
        assert!(
            matches!(attrs[0], JSXAttributeItem::SpreadAttribute(_)),
            "expected SpreadAttribute, got {:?}",
            attrs[0]
        );
    }

    #[test]
    fn parse_astro_multiple_special_attributes() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div @click="handler" x.bind="value" data-id="123" />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one element with 3 attributes
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            assert_eq!(element.opening_element.attributes.len(), 3);
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_block_comment_between_attributes() {
        // Block comments between attributes are skipped transparently by the lexer.
        // <div
        // /* Hello */
        // class="something"
        // ></div>
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<div\n/* Hello */\nclass=\"something\"\n></div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            assert_eq!(
                element.opening_element.attributes.len(),
                1,
                "expected 1 attribute (class), comment should be skipped"
            );
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_block_comment_between_attributes_multiline() {
        // Multi-line block comment between attributes.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<div\n/*\n * Hello\n */\nclass=\"something\"\n></div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            assert_eq!(element.opening_element.attributes.len(), 1);
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_block_comment_between_multiple_attributes() {
        // Block comment sandwiched between real attributes — all three must survive.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<div id=\"a\"\n/* ignored */\nclass=\"b\"\n></div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            assert_eq!(
                element.opening_element.attributes.len(),
                2,
                "expected 2 attributes (id, class), comment should be skipped"
            );
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_line_comment_on_own_line_between_attributes() {
        // A `//` line comment on its own line between attributes.
        // The newline that terminates the comment acts as the separator,
        // so the next attribute is parsed correctly.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<div\n// Hello\nclass=\"something\"\n></div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            assert_eq!(
                element.opening_element.attributes.len(),
                1,
                "expected 1 attribute (class), line comment should be skipped"
            );
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_line_comment_inline_after_attribute() {
        // A `//` line comment at the end of an attribute line.
        // Everything from `//` to end-of-line is skipped; the next line's
        // attribute is still parsed normally.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<div class=\"a\" // inline comment\nid=\"b\"\n></div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            assert_eq!(
                element.opening_element.attributes.len(),
                2,
                "expected 2 attributes (class, id); inline line comment should be skipped"
            );
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_template_literal_attribute() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<Component attr=`hello world` />";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one element with 1 attribute
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            assert_eq!(element.opening_element.attributes.len(), 1);
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_template_literal_attribute_with_expression() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Template literal with expression interpolation
        let source = r#"---
const value = "test";
---
<Component attr=`hello ${value}` />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have frontmatter
        assert!(ret.root.frontmatter.is_some());

        // Body should have one element with 1 attribute
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            assert_eq!(element.opening_element.attributes.len(), 1);
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_unquoted_numeric_attribute() {
        // Regression test for withastro/compiler-rs#33: unquoted numeric values
        // (`minlength=4`) used to tokenize as `Kind::Number` and trip the JSX
        // attribute-value parser's "Unexpected token" branch.
        use oxc_ast::ast::JSXAttributeValue;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<input type="text" minlength=4 maxlength=255 />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        assert_eq!(ret.root.body.len(), 1);
        let JSXChild::Element(element) = &ret.root.body[0] else {
            panic!("Expected JSXChild::Element");
        };
        let attrs = &element.opening_element.attributes;
        assert_eq!(attrs.len(), 3);

        let unquoted_value = |idx: usize| -> &str {
            let JSXAttributeItem::Attribute(attr) = &attrs[idx] else {
                panic!("Expected Attribute at {idx}");
            };
            let Some(JSXAttributeValue::StringLiteral(str_lit)) = &attr.value else {
                panic!("Expected StringLiteral value at {idx}, got {:?}", attr.value);
            };
            str_lit.value.as_str()
        };

        assert_eq!(unquoted_value(1), "4");
        assert_eq!(unquoted_value(2), "255");
    }

    #[test]
    fn parse_astro_unquoted_attribute_with_dashes_and_hash() {
        // The previous `Kind::Ident` branch only captured the leading identifier,
        // so `data-id=hello-world` ended up as `"hello"` and `color=#abc123`
        // failed to parse. The HTML-style unquoted reader handles both.
        use oxc_ast::ast::JSXAttributeValue;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div data-id=hello-world data-color=#abc123>x</div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let JSXChild::Element(element) = &ret.root.body[0] else {
            panic!("Expected JSXChild::Element");
        };
        let attrs = &element.opening_element.attributes;
        assert_eq!(attrs.len(), 2);

        let unquoted_value = |idx: usize| -> &str {
            let JSXAttributeItem::Attribute(attr) = &attrs[idx] else {
                panic!("Expected Attribute at {idx}");
            };
            let Some(JSXAttributeValue::StringLiteral(str_lit)) = &attr.value else {
                panic!("Expected StringLiteral value at {idx}, got {:?}", attr.value);
            };
            str_lit.value.as_str()
        };

        assert_eq!(unquoted_value(0), "hello-world");
        assert_eq!(unquoted_value(1), "#abc123");
    }

    #[test]
    fn parse_astro_unquoted_attribute_hash_color_with_digit() {
        use oxc_ast::ast::JSXAttributeValue;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"<div color=#18b218 background=#686868>x</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let JSXChild::Element(element) = &ret.root.body[0] else {
            panic!("Expected JSXChild::Element");
        };
        let attrs = &element.opening_element.attributes;
        assert_eq!(attrs.len(), 2);

        let unquoted_value = |idx: usize| -> &str {
            let JSXAttributeItem::Attribute(attr) = &attrs[idx] else {
                panic!("Expected Attribute at {idx}");
            };
            let Some(JSXAttributeValue::StringLiteral(str_lit)) = &attr.value else {
                panic!("Expected StringLiteral value at {idx}, got {:?}", attr.value);
            };
            str_lit.value.as_str()
        };

        assert_eq!(unquoted_value(0), "#18b218");
        assert_eq!(unquoted_value(1), "#686868");
    }

    #[test]
    fn parse_astro_unquoted_attribute_then_self_closing() {
        // The unquoted reader must stop at `/` so `<input value=4/>` still self-closes.
        use oxc_ast::ast::JSXAttributeValue;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<input value=4/>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let JSXChild::Element(element) = &ret.root.body[0] else {
            panic!("Expected JSXChild::Element");
        };
        assert!(element.closing_element.is_none(), "expected self-closing element");
        let attrs = &element.opening_element.attributes;
        assert_eq!(attrs.len(), 1);

        let JSXAttributeItem::Attribute(attr) = &attrs[0] else {
            panic!("Expected Attribute");
        };
        let Some(JSXAttributeValue::StringLiteral(str_lit)) = &attr.value else {
            panic!("Expected StringLiteral value, got {:?}", attr.value);
        };
        assert_eq!(str_lit.value.as_str(), "4");
    }

    #[test]
    fn parse_astro_quoted_and_expression_attributes_still_work() {
        // The new unquoted-value path must not regress string or expression attributes.
        use oxc_ast::ast::JSXAttributeValue;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<input type="text" value={x} other='y' />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let JSXChild::Element(element) = &ret.root.body[0] else {
            panic!("Expected JSXChild::Element");
        };
        let attrs = &element.opening_element.attributes;
        assert_eq!(attrs.len(), 3);

        let JSXAttributeItem::Attribute(type_attr) = &attrs[0] else {
            panic!("Expected Attribute at 0");
        };
        assert!(matches!(type_attr.value, Some(JSXAttributeValue::StringLiteral(_))));

        let JSXAttributeItem::Attribute(value_attr) = &attrs[1] else {
            panic!("Expected Attribute at 1");
        };
        assert!(matches!(value_attr.value, Some(JSXAttributeValue::ExpressionContainer(_))));

        let JSXAttributeItem::Attribute(other_attr) = &attrs[2] else {
            panic!("Expected Attribute at 2");
        };
        assert!(matches!(other_attr.value, Some(JSXAttributeValue::StringLiteral(_))));
    }

    #[test]
    fn parse_astro_unquoted_attribute_in_middle_of_list() {
        // Sibling attributes on either side must still parse across the lexer rewind.
        use oxc_ast::ast::JSXAttributeValue;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<input id=foo class="bar" data-x=42 disabled />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let JSXChild::Element(element) = &ret.root.body[0] else {
            panic!("Expected JSXChild::Element");
        };
        let attrs = &element.opening_element.attributes;
        assert_eq!(attrs.len(), 4);

        let attr = |idx: usize| -> &oxc_ast::ast::JSXAttribute<'_> {
            let JSXAttributeItem::Attribute(a) = &attrs[idx] else {
                panic!("Expected Attribute at {idx}");
            };
            a
        };

        let JSXAttributeName::Identifier(name) = &attr(0).name else {
            panic!("expected ident name");
        };
        assert_eq!(name.name.as_str(), "id");
        assert!(matches!(attr(0).value, Some(JSXAttributeValue::StringLiteral(_))));

        let JSXAttributeName::Identifier(name) = &attr(1).name else {
            panic!("expected ident name");
        };
        assert_eq!(name.name.as_str(), "class");
        let Some(JSXAttributeValue::StringLiteral(class_lit)) = &attr(1).value else {
            panic!("class should be quoted string");
        };
        assert_eq!(class_lit.value.as_str(), "bar");

        let JSXAttributeName::Identifier(name) = &attr(2).name else {
            panic!("expected ident name");
        };
        assert_eq!(name.name.as_str(), "data-x");
        let Some(JSXAttributeValue::StringLiteral(data_lit)) = &attr(2).value else {
            panic!("data-x should produce a string literal");
        };
        assert_eq!(data_lit.value.as_str(), "42");

        let JSXAttributeName::Identifier(name) = &attr(3).name else {
            panic!("expected ident name");
        };
        assert_eq!(name.name.as_str(), "disabled");
        assert!(attr(3).value.is_none(), "boolean attr should have no value");
    }

    #[test]
    fn parse_astro_unquoted_attribute_with_children() {
        // The unquoted reader must stop at `>` so children parse, not get swallowed.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div data-count=3>hello</div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let JSXChild::Element(element) = &ret.root.body[0] else {
            panic!("Expected JSXChild::Element");
        };
        assert!(element.closing_element.is_some(), "expected non-self-closing");
        assert_eq!(element.opening_element.attributes.len(), 1);
        assert_eq!(element.children.len(), 1);
        assert!(matches!(element.children[0], JSXChild::Text(_)));
    }

    #[test]
    fn parse_astro_unquoted_attribute_value_stops_at_newline() {
        // An unquoted value must terminate at whitespace, including newlines.
        // Otherwise multi-line attribute lists would silently swallow the
        // following attribute names.
        use oxc_ast::ast::JSXAttributeValue;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<input\n  data-x=42\n  data-y=99\n/>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let JSXChild::Element(element) = &ret.root.body[0] else {
            panic!("Expected JSXChild::Element");
        };
        let attrs = &element.opening_element.attributes;
        assert_eq!(attrs.len(), 2);

        let attr_value = |idx: usize| -> &str {
            let JSXAttributeItem::Attribute(a) = &attrs[idx] else {
                panic!("Expected Attribute at {idx}");
            };
            let Some(JSXAttributeValue::StringLiteral(lit)) = &a.value else {
                panic!("expected string literal at {idx}");
            };
            lit.value.as_str()
        };
        assert_eq!(attr_value(0), "42");
        assert_eq!(attr_value(1), "99");
    }

    #[test]
    fn parse_astro_unquoted_value_has_correct_span() {
        // The lexer rewind in the unquoted-value path must produce a span that
        // exactly covers the value characters in the source — not the wider
        // span the JS lexer would have produced for a partial token.
        use oxc_ast::ast::JSXAttributeValue;
        use oxc_span::GetSpan;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div data-x=hello-world>x</div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let JSXChild::Element(element) = &ret.root.body[0] else {
            panic!("Expected JSXChild::Element");
        };
        let JSXAttributeItem::Attribute(attr) = &element.opening_element.attributes[0] else {
            panic!("Expected Attribute");
        };
        let Some(JSXAttributeValue::StringLiteral(str_lit)) = &attr.value else {
            panic!("Expected StringLiteral");
        };

        let value_start = source.find("hello-world").unwrap() as u32;
        let value_end = value_start + "hello-world".len() as u32;
        assert_eq!(str_lit.span().start, value_start);
        assert_eq!(str_lit.span().end, value_end);
    }

    #[test]
    fn parse_astro_empty_value_reports_error_at_terminator() {
        // `<Comp value= />` is malformed: `attr=` with no value. The diagnostic
        // must point at the `/`, not silently consume it as the value — that
        // would break self-closing and surface the error far from the cause.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<Comp value= />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.errors.is_empty(), "expected a diagnostic for empty unquoted value");

        let slash_offset = source.find('/').unwrap() as u32;
        assert!(
            ret.errors
                .iter()
                .any(|e| e.labels.as_ref().is_some_and(|labels| labels
                    .iter()
                    .any(|l| l.offset() as u32 == slash_offset))),
            "expected diagnostic at offset {slash_offset}, got {:?}",
            ret.errors
        );
    }

    #[test]
    fn parse_astro_void_element_without_closing() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // HTML void elements don't need to be self-closed
        let source = r#"<input type="text"><br><img src="test.png">"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have 3 elements
        assert_eq!(ret.root.body.len(), 3);
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));
        assert!(matches!(ret.root.body[1], JSXChild::Element(_)));
        assert!(matches!(ret.root.body[2], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_void_element_self_closed() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Self-closing void elements should also work
        let source = r#"<input type="text" /><br /><img src="test.png" />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have 3 elements
        assert_eq!(ret.root.body.len(), 3);
    }

    #[test]
    fn parse_astro_void_elements_mixed_with_content() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div>
    <input type="text">
    <label>Name</label>
    <br>
    <img src="test.png">
</div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have 1 div element
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            // The div should have children (text nodes, input, label, br, img, etc.)
            assert!(!element.children.is_empty());
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_bare_script_parsed_as_typescript() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Bare <script> should be parsed as TypeScript
        let source = r#"<script>
interface User {
    id: number;
    name: string;
}
const user: User = { id: 1, name: "test" };
</script>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one <script> Element containing AstroScript
        assert_eq!(ret.root.body.len(), 1);
        match &ret.root.body[0] {
            JSXChild::Element(el) => {
                // Check it's a <script> element
                if let JSXElementName::Identifier(ident) = &el.opening_element.name {
                    assert_eq!(ident.name.as_str(), "script");
                } else {
                    panic!("Expected script identifier");
                }
                // Should have one child: AstroScript
                assert_eq!(el.children.len(), 1, "Script element should have 1 child");
                if let JSXChild::AstroScript(script) = &el.children[0] {
                    // Should have parsed TypeScript (interface + const)
                    assert_eq!(script.program.body.len(), 2);
                    assert!(matches!(script.program.body[0], Statement::TSInterfaceDeclaration(_)));
                    assert!(matches!(script.program.body[1], Statement::VariableDeclaration(_)));
                } else {
                    panic!("Expected AstroScript child, got {:?}", el.children[0]);
                }
            }
            other => panic!("Expected Element, got {other:?}"),
        }
    }

    #[test]
    fn parse_astro_script_type_module_is_parsed() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // <script type="module"> is a JS MIME type — should be parsed as AstroScript
        let source = r#"<script type="module">
// This is JavaScript
const x = 1;
</script>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one <script> Element containing an AstroScript (parsed)
        assert_eq!(ret.root.body.len(), 1);
        match &ret.root.body[0] {
            JSXChild::Element(element) => {
                if let JSXElementName::Identifier(ident) = &element.opening_element.name {
                    assert_eq!(ident.name.as_str(), "script");
                } else {
                    panic!("Expected Identifier for script element");
                }
                // Should have one AstroScript child (parsed content)
                assert_eq!(element.children.len(), 1, "Expected 1 child (AstroScript)");
                assert!(
                    matches!(element.children[0], JSXChild::AstroScript(_)),
                    "Expected AstroScript, got {:?}",
                    element.children[0]
                );
            }
            other => panic!("Expected Element, got {other:?}"),
        }
    }

    #[test]
    fn parse_astro_script_non_js_type_is_raw_text() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // <script type="text/css"> is NOT a JS MIME type — should be raw text
        let source = r#"<script type="text/css">
h1 { color: red; }
</script>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one JSX Element with raw text content (not AstroScript)
        assert_eq!(ret.root.body.len(), 1);
        match &ret.root.body[0] {
            JSXChild::Element(element) => {
                if let JSXElementName::Identifier(ident) = &element.opening_element.name {
                    assert_eq!(ident.name.as_str(), "script");
                } else {
                    panic!("Expected Identifier for script element");
                }
                // Should have text child (raw, unparsed content)
                assert_eq!(element.children.len(), 1);
                assert!(matches!(element.children[0], JSXChild::Text(_)));
            }
            other => panic!("Expected Element, got {other:?}"),
        }
    }

    #[test]
    fn parse_astro_script_trailing_whitespace_before_parent_closing_tag() {
        let allocator = Allocator::default();
        let source = "<sl-sidebar-state-persist><script is:inline>code</script>\n</sl-sidebar-state-persist>";
        let ret = Parser::new(&allocator, source, SourceType::astro()).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        let JSXChild::Element(parent) = &ret.root.body[0] else {
            panic!("Expected parent element");
        };
        assert_eq!(parent.children.len(), 2);
        assert!(matches!(parent.children[0], JSXChild::Element(_)));
        let JSXChild::Text(text) = &parent.children[1] else {
            panic!("Expected trailing whitespace text child");
        };
        assert_eq!(text.value.as_str(), "\n");
    }

    #[test]
    fn parse_astro_script_whitespace_before_following_sibling() {
        let allocator = Allocator::default();
        let source = "<div><script is:inline>code</script>\n<span /></div>";
        let ret = Parser::new(&allocator, source, SourceType::astro()).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        let JSXChild::Element(parent) = &ret.root.body[0] else {
            panic!("Expected parent element");
        };
        assert_eq!(parent.children.len(), 3);
        assert!(matches!(parent.children[0], JSXChild::Element(_)));
        let JSXChild::Text(text) = &parent.children[1] else {
            panic!("Expected inter-element whitespace text child");
        };
        assert_eq!(text.value.as_str(), "\n");
        assert!(matches!(parent.children[2], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_script_defer_is_parsed() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // <script defer> has no `type` attribute → should be parsed as AstroScript
        let source = r#"<script defer>
console.log("hello");
</script>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one <script> Element containing an AstroScript (parsed)
        assert_eq!(ret.root.body.len(), 1);
        match &ret.root.body[0] {
            JSXChild::Element(element) => {
                assert_eq!(element.children.len(), 1, "Expected 1 child (AstroScript)");
                assert!(
                    matches!(element.children[0], JSXChild::AstroScript(_)),
                    "Expected AstroScript child, got {:?}",
                    element.children[0]
                );
            }
            other => panic!("Expected Element, got {other:?}"),
        }
    }

    #[test]
    fn parse_astro_html_comment_in_expression() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // HTML comments inside expressions should work in Astro
        let source = r"<div>{
  /* JSX comment */
  <!-- HTML comment -->
}</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one element
        assert_eq!(ret.root.body.len(), 1);
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_html_comment_inline_in_expression() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // HTML comment inline with other code
        let source = r"<div>{ <!-- comment --> true }</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one element
        assert_eq!(ret.root.body.len(), 1);
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_frontmatter_top_level_return() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Top-level return is allowed in Astro frontmatter per spec §2.1
        let source = r"---
const user = null;
if (!user) {
  return;
}
---
<div>Hello</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have frontmatter
        assert!(ret.root.frontmatter.is_some());
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        // Should have 2 statements: const declaration and if statement
        assert_eq!(frontmatter.program.body.len(), 2);
        assert!(matches!(frontmatter.program.body[0], Statement::VariableDeclaration(_)));
        assert!(matches!(frontmatter.program.body[1], Statement::IfStatement(_)));
    }

    #[test]
    fn parse_astro_frontmatter_top_level_return_with_value() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Top-level return with a value (e.g., Astro.redirect)
        let source = r#"---
if (!user) {
  return Astro.redirect("/login");
}
---
<div>Hello</div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have frontmatter
        assert!(ret.root.frontmatter.is_some());
    }

    #[test]
    fn parse_astro_frontmatter_top_level_return_bare() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Bare top-level return (not inside any block)
        let source = r"---
return;
---
<div>Hello</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have frontmatter with 1 return statement
        assert!(ret.root.frontmatter.is_some());
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        assert_eq!(frontmatter.program.body.len(), 1);
        assert!(matches!(frontmatter.program.body[0], Statement::ReturnStatement(_)));
    }

    #[test]
    fn test_expression_container_whitespace_spans() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Expression container with whitespace around the inner element.
        // Leading whitespace is skipped by the JS tokeniser (insignificant).
        // Trailing whitespace-only text node before `}` is stripped by the parser.
        let source = "{\n\t<div>Hello</div>\n}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have one expression container
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::ExpressionContainer(container) = &ret.root.body[0] {
            // Container span should cover the entire `{...}`
            let container_text =
                &source[container.span.start as usize..container.span.end as usize];
            assert_eq!(container_text, source, "Container should span entire source");

            // With trailing whitespace stripped, the single <div> element is
            // unwrapped from the implicit fragment — expression is JSXElement.
            assert!(
                matches!(container.expression, JSXExpression::JSXElement(_)),
                "Expression should be a JSXElement (single child after whitespace stripping), got {:?}",
                std::mem::discriminant(&container.expression)
            );
        } else {
            panic!("Expected JSXChild::ExpressionContainer");
        }
    }

    #[test]
    fn parse_astro_multiple_root_elements() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple root elements are allowed in Astro (unlike JSX)
        let source = "<header>Header</header>\n<main>Main</main>\n<footer>Footer</footer>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have 5 children: header, text, main, text, footer
        assert_eq!(ret.root.body.len(), 5);
        assert!(matches!(ret.root.body[0], JSXChild::Element(_))); // header
        assert!(matches!(ret.root.body[1], JSXChild::Text(_))); // newline
        assert!(matches!(ret.root.body[2], JSXChild::Element(_))); // main
        assert!(matches!(ret.root.body[3], JSXChild::Text(_))); // newline
        assert!(matches!(ret.root.body[4], JSXChild::Element(_))); // footer
    }

    #[test]
    fn parse_astro_multiple_elements_in_expression_with_fragment() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple elements inside an expression using a fragment (standard JSX way)
        let source = "{\n  <>\n    <div>1</div>\n    <div>2</div>\n  </>\n}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have one expression container
        assert_eq!(ret.root.body.len(), 1);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_multiple_elements_in_expression_no_fragment() {
        use oxc_ast::ast::JSXExpression;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple elements inside an expression WITHOUT a fragment (Astro-specific)
        let source = "{\n  <div>1</div>\n  <div>2</div>\n  <div>3</div>\n}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have one expression container
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::ExpressionContainer(container) = &ret.root.body[0] {
            // The expression should be an implicit fragment containing the 3 divs
            if let JSXExpression::JSXFragment(fragment) = &container.expression {
                // Fragment should have children (divs and whitespace text nodes)
                // Count the actual div elements
                let div_count =
                    fragment.children.iter().filter(|c| matches!(c, JSXChild::Element(_))).count();
                assert_eq!(div_count, 3, "Expected 3 div elements, got {div_count}");
            } else {
                panic!(
                    "Expected JSXExpression::JSXFragment for multiple elements, got {:?}",
                    container.expression
                );
            }
        } else {
            panic!("Expected JSXChild::ExpressionContainer");
        }
    }

    #[test]
    fn parse_astro_multiple_elements_in_arrow_function() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple elements inside an arrow function body (Astro-specific)
        // This requires multiple JSX roots to work inside any expression context
        let source = "{[1, 2, 3].map((num) => <div>{num}</div><div>{num * 2}</div>)}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have one expression container with a call expression inside
        assert_eq!(ret.root.body.len(), 1);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_multiple_elements_in_ternary() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple elements in ternary expression branches
        let source = "{condition ? <div>a</div><span>b</span> : <p>c</p><em>d</em>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        assert_eq!(ret.root.body.len(), 1);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_multiple_elements_with_fragments() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Mix of elements and fragments
        let source = "{<div>1</div><>fragment</><span>2</span>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        assert_eq!(ret.root.body.len(), 1);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_conditional_rendering_with_comparison() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Realistic pattern: comparison followed by JSX (LHS is NOT a JSX element)
        // This should NOT be confused with multiple JSX elements - the `<` here is
        // a comparison operator, not the start of another JSX element.
        let source = "{items.length < maxItems && <span>Show more</span>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        assert_eq!(ret.root.body.len(), 1);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_style_block() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Style blocks should be treated as raw text (not parsed)
        let source = "<style>\n  h1 { color: red; }\n</style>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have one element
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            if let JSXElementName::Identifier(ident) = &element.opening_element.name {
                assert_eq!(ident.name.as_str(), "style");
            } else {
                panic!("Expected Identifier for style element");
            }
            // Style content is currently skipped, so children may be empty
            // The raw content can be recovered from the span
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_style_with_lang_attribute() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Style with lang attribute for preprocessor
        let source = r#"<style lang="scss">
  $accent: #1d4ed8;
  .card { border-color: $accent; }
</style>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have one element
        assert_eq!(ret.root.body.len(), 1);
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_multiple_style_blocks() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple style blocks are allowed
        let source = "<style>h1 { color: red; }</style>\n<style>h2 { color: blue; }</style>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have 3 children: style, text, style
        assert_eq!(ret.root.body.len(), 3);
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));
        assert!(matches!(ret.root.body[1], JSXChild::Text(_)));
        assert!(matches!(ret.root.body[2], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_nested_script_parsed_as_typescript() {
        // Helper to recursively find AstroScript nodes
        fn count_astro_scripts(children: &oxc_allocator::Vec<JSXChild>) -> usize {
            let mut count = 0;
            for child in children {
                match child {
                    JSXChild::AstroScript(_) => count += 1,
                    JSXChild::Element(el) => {
                        // Check if this element has an AstroScript child
                        count += count_astro_scripts(&el.children);
                    }
                    JSXChild::Fragment(f) => {
                        count += count_astro_scripts(&f.children);
                    }
                    _ => {}
                }
            }
            count
        }

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Bare <script> nested inside other elements should still be parsed as TypeScript
        let source = r#"<!doctype html>
<html>
<head>
  <script>
    const unused = "test";
    interface User { id: number; }
  </script>
</head>
<body>
  <script>
    const x = 1;
  </script>
</body>
</html>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should find 2 AstroScript nodes (one in head, one in body)
        let script_count = count_astro_scripts(&ret.root.body);
        assert_eq!(script_count, 2, "Expected 2 AstroScript nodes, found {script_count}");
    }

    #[test]
    fn parse_astro_nested_script_with_non_type_attributes_is_parsed() {
        // Helper to recursively find AstroScript nodes
        fn count_astro_scripts(children: &oxc_allocator::Vec<JSXChild>) -> usize {
            let mut count = 0;
            for child in children {
                match child {
                    JSXChild::AstroScript(_) => count += 1,
                    JSXChild::Element(el) => {
                        count += count_astro_scripts(&el.children);
                    }
                    JSXChild::Fragment(f) => {
                        count += count_astro_scripts(&f.children);
                    }
                    _ => {}
                }
            }
            count
        }

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // <script is:inline> has no `type` attribute → should be parsed as AstroScript
        let source = r"<html>
<head>
  <script is:inline>
    const x = 1;
  </script>
</head>
</html>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should find 1 AstroScript node (only `type` suppresses parsing)
        let script_count = count_astro_scripts(&ret.root.body);
        assert_eq!(
            script_count, 1,
            "Expected 1 AstroScript node (is:inline does not suppress parsing), found {script_count}"
        );
    }

    #[test]
    fn parse_astro_script_inside_expression() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Bare <script> inside an expression container should be parsed as TypeScript
        let source = r"{
  <script>
    const x = 1;
    const y = 2;
  </script>
}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one expression container
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::ExpressionContainer(container) = &ret.root.body[0] {
            // The expression should be a JSXElement (the script)
            use oxc_ast::ast::JSXExpression;
            if let JSXExpression::JSXElement(element) = &container.expression {
                // The script element should have an AstroScript child with parsed content
                assert_eq!(element.children.len(), 1, "Script element should have 1 child");
                if let JSXChild::AstroScript(script) = &element.children[0] {
                    // Should have 2 statements (const x, const y)
                    assert_eq!(
                        script.program.body.len(),
                        2,
                        "Expected 2 statements in script, got {}",
                        script.program.body.len()
                    );
                    assert!(matches!(script.program.body[0], Statement::VariableDeclaration(_)));
                    assert!(matches!(script.program.body[1], Statement::VariableDeclaration(_)));
                } else {
                    panic!("Expected AstroScript child, got {:?}", element.children[0]);
                }
            } else {
                panic!("Expected JSXElement expression, got {:?}", container.expression);
            }
        } else {
            panic!("Expected ExpressionContainer, got {:?}", ret.root.body[0]);
        }
    }

    #[test]
    fn parse_astro_script_inside_logical_expression() {
        // Helper to find AstroScript nodes in expression containers
        fn find_astro_scripts_in_expression<'a>(
            expr: &'a oxc_ast::ast::JSXExpression<'a>,
        ) -> Vec<&'a oxc_ast::ast::AstroScript<'a>> {
            use oxc_ast::ast::{Expression, JSXExpression};
            let mut scripts = Vec::new();

            match expr {
                JSXExpression::JSXElement(el) => {
                    for child in &el.children {
                        if let JSXChild::AstroScript(script) = child {
                            scripts.push(script.as_ref());
                        }
                    }
                }
                JSXExpression::LogicalExpression(logical) => {
                    if let Expression::JSXElement(el) = &logical.right {
                        for child in &el.children {
                            if let JSXChild::AstroScript(script) = child {
                                scripts.push(script.as_ref());
                            }
                        }
                    }
                }
                _ => {}
            }
            scripts
        }

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Bare <script> inside a logical expression (common Astro pattern)
        let source = r"{condition && <script>const x = 1;</script>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one expression container
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::ExpressionContainer(container) = &ret.root.body[0] {
            let scripts = find_astro_scripts_in_expression(&container.expression);
            assert_eq!(scripts.len(), 1, "Expected 1 AstroScript, found {}", scripts.len());
            // The script should have 1 statement
            assert_eq!(scripts[0].program.body.len(), 1);
            assert!(matches!(scripts[0].program.body[0], Statement::VariableDeclaration(_)));
        } else {
            panic!("Expected ExpressionContainer");
        }
    }

    #[test]
    fn parse_astro_script_inside_ternary_expression() {
        // Helper to count AstroScript nodes in an expression
        fn count_scripts_in_expression(expr: &oxc_ast::ast::Expression) -> usize {
            use oxc_ast::ast::Expression;
            match expr {
                Expression::JSXElement(el) => {
                    el.children.iter().filter(|c| matches!(c, JSXChild::AstroScript(_))).count()
                }
                Expression::ConditionalExpression(cond) => {
                    count_scripts_in_expression(&cond.consequent)
                        + count_scripts_in_expression(&cond.alternate)
                }
                _ => 0,
            }
        }

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Scripts in both branches of a ternary
        let source = r"{condition ? <script>const a = 1;</script> : <script>const b = 2;</script>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one expression container
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::ExpressionContainer(container) = &ret.root.body[0] {
            use oxc_ast::ast::JSXExpression;
            if let JSXExpression::ConditionalExpression(cond) = &container.expression {
                let script_count = count_scripts_in_expression(&cond.consequent)
                    + count_scripts_in_expression(&cond.alternate);
                assert_eq!(
                    script_count, 2,
                    "Expected 2 AstroScripts in ternary, found {script_count}"
                );
            } else {
                panic!("Expected ConditionalExpression");
            }
        } else {
            panic!("Expected ExpressionContainer");
        }
    }

    #[test]
    fn parse_astro_script_inside_map_callback() {
        // Helper to find AstroScript nodes recursively in an expression
        fn find_scripts_in_expr<'a>(
            expr: &'a oxc_ast::ast::Expression<'a>,
        ) -> Vec<&'a oxc_ast::ast::AstroScript<'a>> {
            use oxc_ast::ast::Expression;
            let mut scripts = Vec::new();
            match expr {
                Expression::JSXElement(el) => {
                    for child in &el.children {
                        if let JSXChild::AstroScript(script) = child {
                            scripts.push(script.as_ref());
                        }
                    }
                }
                Expression::CallExpression(call) => {
                    for arg in &call.arguments {
                        if let oxc_ast::ast::Argument::ArrowFunctionExpression(arrow) = arg {
                            // Check all statements in the arrow function body
                            for stmt in &arrow.body.statements {
                                if let Statement::ExpressionStatement(expr_stmt) = stmt {
                                    scripts.extend(find_scripts_in_expr(&expr_stmt.expression));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
            scripts
        }

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Script inside a .map() callback - common Astro pattern
        let source = r"{items.map(item => <script>const x = item;</script>)}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one expression container
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::ExpressionContainer(container) = &ret.root.body[0] {
            use oxc_ast::ast::JSXExpression;
            if let JSXExpression::CallExpression(call) = &container.expression {
                // Traverse into the call to find scripts
                let mut scripts = Vec::new();
                for arg in &call.arguments {
                    if let oxc_ast::ast::Argument::ArrowFunctionExpression(arrow) = arg {
                        // Arrow function with expression body
                        if arrow.expression
                            && let Some(stmt) = arrow.body.statements.first()
                            && let Statement::ExpressionStatement(expr_stmt) = stmt
                        {
                            scripts.extend(find_scripts_in_expr(&expr_stmt.expression));
                        }
                    }
                }
                assert_eq!(
                    scripts.len(),
                    1,
                    "Expected 1 AstroScript in map callback, found {}",
                    scripts.len()
                );
                // Verify the script was actually parsed (has statements)
                assert_eq!(scripts[0].program.body.len(), 1, "Script should have 1 statement");
            } else {
                panic!("Expected CallExpression, got {:?}", container.expression);
            }
        } else {
            panic!("Expected ExpressionContainer");
        }
    }

    #[test]
    fn parse_astro_script_inside_filter_map_chain() {
        // Helper to find all AstroScript nodes recursively in JSX children
        fn find_scripts_in_children<'a>(
            children: &'a [JSXChild<'a>],
        ) -> Vec<&'a oxc_ast::ast::AstroScript<'a>> {
            let mut scripts = Vec::new();
            for child in children {
                match child {
                    JSXChild::AstroScript(script) => {
                        scripts.push(script.as_ref());
                    }
                    JSXChild::Element(el) => {
                        scripts.extend(find_scripts_in_children(&el.children));
                    }
                    JSXChild::Fragment(frag) => {
                        scripts.extend(find_scripts_in_children(&frag.children));
                    }
                    _ => {}
                }
            }
            scripts
        }

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Script inside chained array methods - the outer .map() call returns JSX with a script
        let source =
            r"{items.filter(x => x > 0).map(item => <div><script>const y = 1;</script></div>)}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Verify we have the expression container
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::ExpressionContainer(container) = &ret.root.body[0] {
            use oxc_ast::ast::JSXExpression;
            if let JSXExpression::CallExpression(call) = &container.expression {
                // The outer .map() call should have the script in its callback
                let mut found_parsed_script = false;
                for arg in &call.arguments {
                    if let oxc_ast::ast::Argument::ArrowFunctionExpression(arrow) = arg {
                        for stmt in &arrow.body.statements {
                            if let Statement::ExpressionStatement(expr_stmt) = stmt {
                                // The expression should be a JSXElement containing a nested script
                                if let oxc_ast::ast::Expression::JSXElement(el) =
                                    &expr_stmt.expression
                                {
                                    let scripts = find_scripts_in_children(&el.children);
                                    for script in &scripts {
                                        // Verify the script was parsed (has statements)
                                        if !script.program.body.is_empty() {
                                            found_parsed_script = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                assert!(
                    found_parsed_script,
                    "Expected to find a parsed script in filter-map chain"
                );
            }
        }
    }

    #[test]
    fn parse_astro_script_with_block_body_arrow() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Script inside arrow function with block body (not concise)
        let source = r"{items.map(item => { return <script>const z = 1;</script>; })}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Verify we have the expression container with a script
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::ExpressionContainer(container) = &ret.root.body[0] {
            use oxc_ast::ast::JSXExpression;
            if let JSXExpression::CallExpression(call) = &container.expression {
                let mut found_script = false;
                for arg in &call.arguments {
                    if let oxc_ast::ast::Argument::ArrowFunctionExpression(arrow) = arg {
                        for stmt in &arrow.body.statements {
                            if let Statement::ReturnStatement(ret_stmt) = stmt
                                && let Some(arg) = &ret_stmt.argument
                                && let oxc_ast::ast::Expression::JSXElement(el) = arg
                            {
                                for child in &el.children {
                                    if let JSXChild::AstroScript(script) = child {
                                        // Verify the script was parsed
                                        assert_eq!(
                                            script.program.body.len(),
                                            1,
                                            "Script should have 1 statement"
                                        );
                                        found_script = true;
                                    }
                                }
                            }
                        }
                    }
                }
                assert!(found_script, "Expected to find a parsed script in block body arrow");
            }
        }
    }

    // ==========================================
    // Multiple JSX Roots Tests (Astro-specific)
    // ==========================================
    // These tests cover the key Astro feature: multiple JSX elements
    // without requiring explicit fragment wrappers.

    #[test]
    fn parse_astro_multiple_roots_simple() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Simplest case: two sibling elements at top level
        let source = "<div>1</div><div>2</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 2);
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));
        assert!(matches!(ret.root.body[1], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_multiple_roots_with_text_between() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Elements with text/whitespace between them
        let source = "<div>a</div> text <span>b</span>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        // div, text, span
        assert_eq!(ret.root.body.len(), 3);
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));
        assert!(matches!(ret.root.body[1], JSXChild::Text(_)));
        assert!(matches!(ret.root.body[2], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_multiple_roots_many_elements() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Many sibling elements
        let source = "<a>1</a><b>2</b><c>3</c><d>4</d><e>5</e>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 5);
        for child in &ret.root.body {
            assert!(matches!(child, JSXChild::Element(_)));
        }
    }

    #[test]
    fn parse_astro_multiple_roots_in_expression_simple() {
        use oxc_ast::ast::JSXExpression;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Two elements in expression container (no explicit fragment)
        let source = "{<div>a</div><span>b</span>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::ExpressionContainer(container) = &ret.root.body[0] {
            if let JSXExpression::JSXFragment(fragment) = &container.expression {
                let element_count =
                    fragment.children.iter().filter(|c| matches!(c, JSXChild::Element(_))).count();
                assert_eq!(element_count, 2, "Expected 2 elements in implicit fragment");
            } else {
                panic!("Expected JSXFragment for multiple elements");
            }
        } else {
            panic!("Expected ExpressionContainer");
        }
    }

    #[test]
    fn parse_astro_multiple_roots_in_expression_with_whitespace() {
        use oxc_ast::ast::JSXExpression;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple elements with whitespace in expression
        let source = "{\n  <div>a</div>\n  <span>b</span>\n  <p>c</p>\n}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        if let JSXChild::ExpressionContainer(container) = &ret.root.body[0] {
            if let JSXExpression::JSXFragment(fragment) = &container.expression {
                let element_count =
                    fragment.children.iter().filter(|c| matches!(c, JSXChild::Element(_))).count();
                assert_eq!(element_count, 3, "Expected 3 elements");
            } else {
                panic!("Expected JSXFragment");
            }
        }
    }

    #[test]
    fn parse_astro_single_element_in_expression_no_fragment() {
        use oxc_ast::ast::JSXExpression;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Single element should NOT be wrapped in fragment
        let source = "{<div>only one</div>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        if let JSXChild::ExpressionContainer(container) = &ret.root.body[0] {
            // Single element should be JSXElement, not JSXFragment
            assert!(
                matches!(container.expression, JSXExpression::JSXElement(_)),
                "Single element should be JSXElement, not fragment: {:?}",
                container.expression
            );
        }
    }

    #[test]
    fn parse_astro_map_with_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Common Astro pattern: map returning multiple elements
        let source = "{items.map(item => <dt>{item.term}</dt><dd>{item.def}</dd>)}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 1);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_map_with_index_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Map with index parameter
        let source = "{items.map((item, i) => <span key={i}>{item}</span><br />)}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_filter_map_with_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Chained array methods
        let source = "{items.filter(x => x.visible).map(x => <div>{x.name}</div><hr />)}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_ternary_both_branches_multiple() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Ternary with multiple elements in both branches
        let source = "{cond ? <a>1</a><b>2</b> : <c>3</c><d>4</d>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_ternary_consequent_multiple_only() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Ternary with multiple elements only in consequent
        let source = "{cond ? <a>1</a><b>2</b> : <c>single</c>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_ternary_alternate_multiple_only() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Ternary with multiple elements only in alternate
        let source = "{cond ? <single /> : <a>1</a><b>2</b>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_logical_and_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Logical AND with multiple elements
        let source = "{show && <div>a</div><div>b</div>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_logical_or_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Logical OR with multiple elements as fallback
        let source = "{content || <div>fallback1</div><div>fallback2</div>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_nullish_coalescing_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Nullish coalescing with multiple elements
        let source = "{value ?? <span>default1</span><span>default2</span>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_nested_expression_with_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Nested expressions with multiple roots at different levels
        let source = "<div>{show && <a>1</a><b>2</b>}</div><div>{other && <c>3</c><d>4</d>}</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        // Two root divs
        assert_eq!(ret.root.body.len(), 2);
    }

    #[test]
    fn parse_astro_deeply_nested_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple roots inside nested structure
        let source = "<ul>{items.map(item => <li>{item.name}</li><li>{item.value}</li>)}</ul>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 1);
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_multiple_roots_with_void_elements() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple roots including void elements
        let source = "<input><br><hr><img src=\"test.png\">";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 4);
    }

    #[test]
    fn parse_astro_multiple_roots_mixed_void_and_normal() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Mix of void and normal elements
        let source = "<div>content</div><br><span>more</span><hr>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 4);
    }

    #[test]
    fn parse_astro_multiple_roots_with_self_closing() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple self-closing elements
        let source = "<Component /><Another /><Third prop={value} />";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 3);
    }

    #[test]
    fn parse_astro_multiple_roots_with_fragments_mixed() {
        use oxc_ast::ast::JSXExpression;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Explicit fragments mixed with implicit multiple roots
        let source = "{<>frag1</><div>elem</div><>frag2</>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        if let JSXChild::ExpressionContainer(container) = &ret.root.body[0] {
            if let JSXExpression::JSXFragment(fragment) = &container.expression {
                // Should have 3 children: fragment, element, fragment
                let child_count =
                    fragment.children.iter().filter(|c| !matches!(c, JSXChild::Text(_))).count();
                assert_eq!(child_count, 3, "Expected 3 non-text children");
            } else {
                panic!("Expected implicit JSXFragment wrapper");
            }
        }
    }

    #[test]
    fn parse_astro_multiple_roots_with_expression_children() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple roots with expression children
        let source = "<div>{a}</div><span>{b}</span><p>{c}</p>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 3);
    }

    #[test]
    fn parse_astro_array_literal_with_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Array literal containing multiple JSX roots per element
        let source = "{[<a>1</a><b>2</b>, <c>3</c><d>4</d>]}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_function_call_with_multiple_jsx_args() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Function call with JSX arguments (each could have multiple roots)
        let source = "{render(<div>a</div><span>b</span>)}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_object_property_with_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Object with JSX values (multiple roots)
        let source = "{({ content: <div>a</div><span>b</span> })}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_arrow_block_body_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Arrow function with block body returning multiple roots
        let source = "{items.map(item => { return <div>{item}</div><hr />; })}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_iife_with_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // IIFE returning multiple roots
        let source = "{(() => <div>a</div><div>b</div>)()}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_multiple_roots_complex_attributes() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple roots with complex attributes
        let source = r#"<div class="a" data-x={1}></div><span style={{color: "red"}}></span>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 2);
    }

    #[test]
    fn parse_astro_multiple_roots_with_spread() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple roots with spread attributes
        let source = "<div {...props1}></div><span {...props2}></span>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 2);
    }

    #[test]
    fn parse_astro_multiple_roots_with_namespaced_attrs() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple roots with namespaced attributes (Astro feature)
        let source = r"<div client:load></div><span set:html={content}></span>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 2);
    }

    #[test]
    fn parse_astro_comparison_not_confused_with_jsx() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Make sure comparisons aren't confused with multiple JSX roots
        let source = "{a < b && <div>show</div>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        // Should parse as logical expression, not multiple JSX elements
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_generic_not_confused_with_jsx() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // TypeScript generics shouldn't be confused with JSX
        let source = "{fn<T>(x) && <div>ok</div>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_multiple_scripts_as_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple bare script tags at root level
        let source = "<script>const a = 1;</script><script>const b = 2;</script>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 2);

        // Both should be script elements with AstroScript children
        for child in &ret.root.body {
            if let JSXChild::Element(el) = child {
                if let JSXElementName::Identifier(ident) = &el.opening_element.name {
                    assert_eq!(ident.name.as_str(), "script");
                }
                // Should have AstroScript child
                assert_eq!(el.children.len(), 1);
                assert!(matches!(el.children[0], JSXChild::AstroScript(_)));
            } else {
                panic!("Expected Element");
            }
        }
    }

    #[test]
    fn parse_astro_multiple_styles_as_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Multiple style tags at root level
        let source = "<style>h1 { color: red; }</style><style>h2 { color: blue; }</style>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 2);
    }

    #[test]
    fn parse_astro_realistic_page_layout() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Realistic Astro page layout with multiple root elements
        let source = r"<html>
  <head><title>Page</title></head>
  <body><slot /></body>
</html>
<style>
  body { margin: 0; }
</style>
<script>
  console.log('loaded');
</script>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // html, text, style, text, script
        let element_count =
            ret.root.body.iter().filter(|c| matches!(c, JSXChild::Element(_))).count();
        assert_eq!(element_count, 3, "Expected html, style, script elements");
    }

    #[test]
    fn parse_astro_component_with_multiple_slots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Component-like structure with named slots
        let source = r#"<header><slot name="header" /></header>
<main><slot /></main>
<footer><slot name="footer" /></footer>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let element_count =
            ret.root.body.iter().filter(|c| matches!(c, JSXChild::Element(_))).count();
        assert_eq!(element_count, 3, "Expected header, main, footer");
    }

    #[test]
    fn parse_astro_reduce_with_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Array reduce returning multiple elements
        let source = "{items.reduce((acc, item) => <>{acc}<div>{item}</div></>, <></>)}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_flatmap_with_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // flatMap returning multiple elements per item
        let source = "{items.flatMap(item => [<dt>{item.term}</dt>, <dd>{item.def}</dd>])}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_entries_with_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Object.entries with multiple elements
        let source = "{Object.entries(obj).map(([k, v]) => <dt>{k}</dt><dd>{v}</dd>)}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_async_iteration_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Promise.all with multiple elements (common Astro pattern)
        let source =
            "{await Promise.all(items.map(async (item) => <div>{await fetch(item)}</div><hr />))}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_conditional_chain_multiple_roots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Chained conditionals with multiple roots
        let source = "{a ? <div>a</div><span>a2</span> : b ? <div>b</div><span>b2</span> : <div>c</div><span>c2</span>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    #[test]
    fn parse_astro_switch_like_pattern() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // Switch-like pattern using logical operators
        let source = "{(type === 'a' && <div>A</div><span>A2</span>) || (type === 'b' && <div>B</div><span>B2</span>) || <div>default</div>}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::ExpressionContainer(_)));
    }

    // ==========================================
    // JSX vs Astro Script Behavior Tests
    // ==========================================
    // These tests explicitly demonstrate the difference between how
    // <script> tags are handled in regular JSX vs Astro files.

    #[test]
    fn jsx_script_content_is_text() {
        // In regular JSX/TSX, <script> content is just JSXText (not parsed as code)
        let allocator = Allocator::default();
        let source_type = SourceType::tsx(); // Regular TSX, not Astro
        let source = r#"<div><script>console.log("hello");</script></div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Find the script element in the JSX
        if let Some(Statement::ExpressionStatement(expr_stmt)) = ret.program.body.first()
            && let oxc_ast::ast::Expression::JSXElement(jsx_el) = &expr_stmt.expression
        {
            // First child of <div> is the <script> element
            if let Some(JSXChild::Element(script_el)) = jsx_el.children.first() {
                if let JSXElementName::Identifier(ident) = &script_el.opening_element.name {
                    assert_eq!(ident.name.as_str(), "script");
                }
                // In JSX, script content is JSXText, NOT parsed as code
                assert_eq!(script_el.children.len(), 1, "Script should have 1 child");
                match &script_el.children[0] {
                    JSXChild::Text(text) => {
                        // The content is raw text, not parsed
                        assert_eq!(text.value.as_str(), r#"console.log("hello");"#);
                    }
                    other => panic!("In JSX, script content should be JSXText, got {other:?}"),
                }
            } else {
                panic!("Expected script element");
            }
        }
    }

    #[test]
    fn astro_bare_script_content_is_parsed_program() {
        // In Astro, bare <script> (no attributes) content IS parsed as TypeScript
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<script>console.log("hello");</script>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // In Astro, bare <script> is wrapped in Element with AstroScript child
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(script_el) = &ret.root.body[0] {
            if let JSXElementName::Identifier(ident) = &script_el.opening_element.name {
                assert_eq!(ident.name.as_str(), "script");
            }
            // In Astro, bare script has AstroScript child with PARSED program
            assert_eq!(script_el.children.len(), 1, "Script should have 1 child");
            match &script_el.children[0] {
                JSXChild::AstroScript(astro_script) => {
                    // The content IS parsed as TypeScript code!
                    assert_eq!(
                        astro_script.program.body.len(),
                        1,
                        "Should have 1 parsed statement"
                    );
                    // It's an ExpressionStatement containing console.log call
                    assert!(
                        matches!(astro_script.program.body[0], Statement::ExpressionStatement(_)),
                        "Should be parsed as ExpressionStatement"
                    );
                }
                other => {
                    panic!("In Astro, bare script content should be AstroScript, got {other:?}")
                }
            }
        } else {
            panic!("Expected Element");
        }
    }

    #[test]
    fn astro_script_type_module_content_is_parsed() {
        // <script type="module"> uses a JS MIME type → content should be parsed (AstroScript)
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<script type="module">console.log("hello");</script>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Script with JS MIME type is parsed as AstroScript
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(script_el) = &ret.root.body[0] {
            if let JSXElementName::Identifier(ident) = &script_el.opening_element.name {
                assert_eq!(ident.name.as_str(), "script");
            }
            assert_eq!(script_el.children.len(), 1, "Script should have 1 child");
            assert!(
                matches!(script_el.children[0], JSXChild::AstroScript(_)),
                "Expected AstroScript child, got {:?}",
                script_el.children[0]
            );
        } else {
            panic!("Expected Element");
        }
    }

    #[test]
    fn astro_script_non_js_type_content_is_text() {
        // <script type="application/json"> is a non-JS type → content is raw text
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<script type="application/json">{"key":"value"}</script>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Script with non-JS type is a raw text element
        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(script_el) = &ret.root.body[0] {
            if let JSXElementName::Identifier(ident) = &script_el.opening_element.name {
                assert_eq!(ident.name.as_str(), "script");
            }
            assert_eq!(script_el.children.len(), 1, "Script should have 1 child");
            assert!(
                matches!(script_el.children[0], JSXChild::Text(_)),
                "Expected JSXText child for non-JS type, got {:?}",
                script_el.children[0]
            );
        } else {
            panic!("Expected Element");
        }
    }

    #[test]
    fn astro_bare_script_parses_typescript_syntax() {
        // Bare script in Astro can contain TypeScript-specific syntax
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<script>
const greeting: string = "hello";
interface User { name: string }
</script>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();

        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        if let JSXChild::Element(script_el) = &ret.root.body[0] {
            if let JSXChild::AstroScript(astro_script) = &script_el.children[0] {
                // Should have 2 statements: const declaration + interface
                assert_eq!(astro_script.program.body.len(), 2);
                assert!(matches!(astro_script.program.body[0], Statement::VariableDeclaration(_)));
                assert!(matches!(
                    astro_script.program.body[1],
                    Statement::TSInterfaceDeclaration(_)
                ));
            } else {
                panic!("Expected AstroScript");
            }
        }
    }

    // ==========================================
    // Astro Compiler Compatibility Tests
    // ==========================================
    // These tests are ported from @astrojs/compiler test suite to ensure
    // the oxc Astro parser can handle the same inputs.
    // Source: https://github.com/withastro/compiler/tree/main/packages/compiler/test

    // --- From test/basic/expressions.ts ---
    #[test]
    fn compiler_compat_less_than_inside_jsx_expression() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"<Layout>
   {
      new Array(totalPages).fill(0).map((_, index) => {
        const active = currentPage === index;
        if (
          totalPages > 25 &&
          ( index < currentPage - offset ||
            index > currentPage + offset)
        ) {
          return 'HAAAA';
        }
      })
    }
</Layout>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- From test/basic/lt-gt-text.ts ---
    // In Astro/HTML, `<` followed by whitespace or non-letter is text, not a tag start.
    // The lexer checks if `<` is followed by ASCII letter, `/`, `>`, or `!` to determine
    // if it's a tag. Otherwise, it's treated as text content.
    #[test]
    fn compiler_compat_lt_gt_as_raw_text() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"---
import MainHead from '../components/MainHead.astro';
---

<html lang="en">
    <head>
        <MainHead title="Test" />
    </head>
    <body>
        <small>< header ></small>
    </body>
</html>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- From test/basic/comment.ts ---
    #[test]
    fn compiler_compat_html_comments() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<!-- This is a comment --><div>Hello</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- From test/basic/fragment.ts ---
    #[test]
    fn compiler_compat_fragment_shorthand() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<>Hello</>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(matches!(ret.root.body[0], JSXChild::Fragment(_)));
    }

    #[test]
    fn compiler_compat_fragment_literal() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<Fragment>World</Fragment>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- From test/parse/fragment.ts ---
    #[test]
    fn compiler_compat_both_fragment_types() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<>Hello</><Fragment>World</Fragment>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 2);
    }

    // --- From test/parse/ast.ts ---
    #[test]
    fn compiler_compat_basic_ast_structure() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"
---
let value = 'world';
---

<h1 name="value" empty {shorthand} expression={true} literal=`tags`>Hello {value}</h1>
<div></div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert!(ret.root.frontmatter.is_some());
    }

    // --- From test/parse/literal.ts ---
    #[test]
    fn compiler_compat_style_tag_position_i() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<html><body><h1>Hello world!</h1></body></html>\n<style></style>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_style_tag_position_ii() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<html></html>\n<style></style>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_style_tag_position_iii() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<html lang="en"><head><BaseHead /></head></html>
<style>@use "../styles/global.scss";</style>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- From test/parse/multibyte-characters.ts ---
    #[test]
    fn compiler_compat_multibyte_characters() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<div>こんにちは世界</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_multibyte_in_expressions() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "{items.map(item => <span>日本語: {item}</span>)}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- From test/tsx/basic.ts ---
    #[test]
    fn compiler_compat_at_attributes() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div @click={() => {}} name="value"></div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_attributes_with_dots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div x-on:keyup.shift.enter="alert('Astro')" name="value"></div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // Vue-style shorthand attributes like `:class` are supported in Astro.
    // This works because we disabled namespaced element name parsing in Astro mode,
    // so `:class` is correctly parsed as an attribute starting with `:`.
    #[test]
    fn compiler_compat_attributes_starting_with_colon() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div :class="hey" name="value"></div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_template_literal_attribute() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<div class=`${hello}`></div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_spread_object() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source =
            r#"<DocSearch {...{ lang, labels: { modal, placeholder } }} client:only="preact" />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_spread_object_ii() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<MainLayout {...Astro.props}>\n</MainLayout>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_fragment_with_no_name() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<>+0123456789</>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_preserves_spaces_in_tag() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<Button ></Button>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_preserves_spaces_after_attributes() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<Button a="b" ></Button>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- From test/tsx/complex-generics.ts ---
    #[test]
    fn compiler_compat_complex_generics() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"---
type Props<T extends Record<string, any>> = {
  data: T;
  render: (item: T) => string;
};
const { data, render } = Astro.props as Props<{ name: string }>;
---
<div>{render(data)}</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- From test/tsx/nested-generics.ts ---
    #[test]
    fn compiler_compat_nested_generics() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"---
interface Props {
  items: Array<{ id: number; nested: Map<string, Set<number>> }>;
}
---
<ul>{Astro.props.items.map(i => <li>{i.id}</li>)}</ul>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- From test/tsx/script.ts ---
    #[test]
    fn compiler_compat_script_tag() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<script>console.log('hello');</script>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_script_with_type_module() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<script type="module">import foo from './foo';</script>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- From test/tsx/raw.ts ---
    #[test]
    fn compiler_compat_set_html_directive() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<div set:html={content}></div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_set_text_directive() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<span set:text={text}></span>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- Client Directives ---
    #[test]
    fn compiler_compat_client_load() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<Component client:load />";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_client_idle() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<Component client:idle />";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_client_visible() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<Component client:visible />";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_client_media() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<Component client:media="(max-width: 600px)" />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_client_only() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<Component client:only="react" />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- Styles ---
    #[test]
    fn compiler_compat_style_tag_with_content() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<style>
  .container { color: red; }
  h1 { font-size: 2rem; }
</style>
<div class="container"><h1>Hello</h1></div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_style_is_global() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<style is:global>\n  body { margin: 0; }\n</style>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_style_lang_scss() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<style lang="scss">
  $color: red;
  .foo { color: $color; }
</style>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- Tables ---
    #[test]
    fn compiler_compat_table_structure() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"<table>
  <thead>
    <tr><th>Header</th></tr>
  </thead>
  <tbody>
    <tr><td>Cell</td></tr>
  </tbody>
</table>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_table_with_expressions() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"<table>
  <tbody>
    {items.map(item => <tr><td>{item.name}</td></tr>)}
  </tbody>
</table>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- Slots ---
    #[test]
    fn compiler_compat_default_slot() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<Layout><slot /></Layout>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_named_slots() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<Layout>
  <Fragment slot="header">Header Content</Fragment>
  <slot />
  <div slot="footer">Footer Content</div>
</Layout>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_slot_with_fallback() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<slot>Default content</slot>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- Head and Metadata ---
    #[test]
    fn compiler_compat_head_with_meta() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<html>
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>My Page</title>
  </head>
  <body></body>
</html>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_view_transitions() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"---
import { ViewTransitions } from 'astro:transitions';
---
<head>
  <ViewTransitions />
</head>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- Expressions ---
    #[test]
    fn compiler_compat_ternary_expression() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "{condition ? <True /> : <False />}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_logical_and() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "{show && <Component />}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_logical_or() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "{value || <Fallback />}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_nullish_coalescing() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "{value ?? <Default />}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_map_with_arrow() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "{items.map(item => <li key={item.id}>{item.name}</li>)}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_map_with_block_body() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"{items.map(item => {
      const formatted = format(item);
      return <li>{formatted}</li>;
    })}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_filter_map_chain() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "{items.filter(i => i.active).map(i => <span>{i.name}</span>)}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_await_expression() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "{await fetchData()}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_async_iife() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r"{(async () => {
      const data = await fetch('/api');
      return <div>{data}</div>;
    })()}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- Special Elements ---
    #[test]
    fn compiler_compat_void_elements_without_closing() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<img src="test.png"><br><hr><input type="text">"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_svg_element() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
  <circle cx="50" cy="50" r="40" />
</svg>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- Go Parser Tests (from internal/parser_test.go) ---
    #[test]
    fn compiler_compat_go_end_tag_i() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div id="target"></div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_go_end_tag_ii() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div class="TabBox">
	<div id="target" class="tab-bar">
		<div id="install-npm" class="active toggle"><h5>npm</h5></div>
		<div id="install-yarn" class="toggle"><h5>yarn</h5></div>
	</div>
</div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_go_class_list() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<span id="target" class:list={["link pixel variant", className]} {style}>
	<a {href}>
		<span><slot /></span>
	</a>
</span>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_go_complex_component() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<HeadingWrapper id="target">
		<h2 class="heading"><UIString key="rightSidebar.community" /></h2>
		{
			hideOnLargerScreens && (
				<svg
					class="chevron"
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 1 16 16"
					width="16"
					height="16"
					aria-hidden="true"
				>
					<path
						fill-rule="evenodd"
						d="M6.22 3.22a.75.75 0 011.06 0l4.25 4.25a.75.75 0 010 1.06l-4.25 4.25a.75.75 0 01-1.06-1.06L9.94 8 6.22 4.28a.75.75 0 010-1.06z"
					/>
				</svg>
			)
		}
	</HeadingWrapper>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // --- Edge Cases ---
    #[test]
    fn compiler_compat_empty_file() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_only_frontmatter() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "---\nconst x = 1;\n---";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_only_whitespace() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "   \n\n   ";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_deeply_nested() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<a><b><c><d><e><f><g><h><i><j>deep</j></i></h></g></f></e></d></c></b></a>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_many_siblings() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<a/><b/><c/><d/><e/><f/><g/><h/><i/><j/><k/><l/><m/><n/><o/><p/>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
        assert_eq!(ret.root.body.len(), 16);
    }

    #[test]
    fn compiler_compat_mixed_content() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "Text before <span>inline</span> text after {expression} more text";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_unicode_not_tag_start_simple() {
        // Per HTML spec, only ASCII letters can start a tag name.
        // `<日本語>` is treated as text, not a tag.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<日本語>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should be parsed as text content, not as a JSX element
        assert_eq!(ret.root.body.len(), 1);
        assert!(
            matches!(&ret.root.body[0], JSXChild::Text(_)),
            "Expected JSXText, got {:?}",
            ret.root.body[0]
        );
    }

    #[test]
    fn compiler_compat_unicode_not_tag_start_with_expression() {
        // `<日本語 {expr}>` - the `<日本語 ` part is text, then `{expr}` is expression, then `>` is text
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<日本語 {expr}>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn compiler_compat_emoji_in_content() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<div>Hello 👋 World 🌍</div>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // Test attribute with `>` in value - should work fine
    #[test]
    fn compiler_compat_attr_with_gt() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div data=">" />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // Test attribute with `<` in value
    #[test]
    fn compiler_compat_attr_with_lt() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div data="<" />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // Test special characters in attribute strings.
    // In Astro/HTML, `\"` does NOT escape the quote - backslash is literal.
    // So `"<>&\"` is a string containing `<>&\`, followed by the closing `"`.
    // Then `'"` becomes a separate attribute name (HTML allows quotes in attr names).
    #[test]
    fn compiler_compat_special_chars_in_strings() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div data-special="<>&\"'" />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    // Test that `---` inside strings in frontmatter doesn't close the frontmatter.
    // This is a regression test for incorrect fence detection.
    #[test]
    fn parse_astro_frontmatter_dashes_in_string() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // The string '--- message ---' contains `---` but should NOT end the frontmatter
        let source = r"---
export async function getStaticPaths() {
  console.log('--- built product pages ---')
  return [];
}
---
<div>Hello</div>
";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Check frontmatter has the function
        assert!(ret.root.frontmatter.is_some());
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        assert_eq!(frontmatter.program.body.len(), 1);
        assert!(matches!(frontmatter.program.body[0], Statement::ExportNamedDeclaration(_)));

        // Check body has the div
        assert!(!ret.root.body.is_empty());
    }

    // Test that HTML comments are preserved as AstroComment nodes
    #[test]
    fn parse_astro_html_comments_preserved() {
        use oxc_ast::ast::JSXChild;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<!-- Global Metadata -->
<meta charset="utf-8">
<!-- Another comment -->
<link rel="icon" href="/favicon.ico" />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have at least: comment, text, meta, text, comment, text, link
        // That's 7 children (comments + elements + whitespace text nodes)
        let comment_count =
            ret.root.body.iter().filter(|c| matches!(c, JSXChild::AstroComment(_))).count();

        assert_eq!(comment_count, 2, "Expected 2 HTML comments in the output");
    }

    #[test]
    fn parse_astro_math_foreign_content() {
        // {2x} inside <math> should be literal text, not an expression
        let source = r#"<math xmlns="http://www.w3.org/1998/Math/MathML"><msup><mi>R</mi><mrow><mn>2</mn><mi>x</mi></mrow></msup><annotation encoding="application/x-tex">R^{2x}</annotation></math>"#;
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // The math element should be parsed successfully with {2x} as text
        assert!(!ret.root.body.is_empty(), "should have parsed content");
    }

    #[test]
    fn parse_astro_math_simple_braces() {
        // Simple {test} inside <math> should be text
        let source = "<math>{test}</math>";
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have a <math> element
        let math_el = ret.root.body.iter().find(|c| matches!(c, JSXChild::Element(_)));
        assert!(math_el.is_some(), "should have a <math> element");

        // The content should be text children, not expression containers
        if let Some(JSXChild::Element(el)) = math_el {
            let has_expression =
                el.children.iter().any(|c| matches!(c, JSXChild::ExpressionContainer(_)));
            assert!(!has_expression, "{{test}} inside <math> should be text, not an expression");
        }
    }

    #[test]
    fn parse_astro_math_nested_in_span() {
        // <math> inside a <span> should still disable expressions
        let source = r#"<span><math xmlns="http://www.w3.org/1998/Math/MathML"><annotation encoding="application/x-tex">\sqrt {x}</annotation></math></span>"#;
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn parse_astro_math_expression_attributes_still_work() {
        // Expression attributes on <math> itself should still work
        let source = r"<math set:html={test} />";
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn parse_astro_svg_still_has_expressions() {
        // SVG should still support expressions (unlike math)
        let source = "<svg>{expr}</svg>";
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Should have an SVG element with an expression container child
        if let Some(JSXChild::Element(el)) =
            ret.root.body.iter().find(|c| matches!(c, JSXChild::Element(_)))
        {
            let has_expression =
                el.children.iter().any(|c| matches!(c, JSXChild::ExpressionContainer(_)));
            assert!(has_expression, "{{expr}} inside <svg> should be an expression, not text");
        }
    }

    #[test]
    fn parse_astro_empty_attribute_expression() {
        // Empty expression in attribute value should be allowed in Astro (no TS17000 error)
        let source = r"<body attr={}></body>";
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(
            ret.errors.is_empty(),
            "empty attribute expression should not produce errors in Astro: {:?}",
            ret.errors
        );
    }

    #[test]
    fn parse_astro_multiple_jsx_children_in_expression() {
        // Multiple JSX elements inside a child expression container should be
        // wrapped into a synthetic fragment.
        let source = "<div>{<span>A</span><span>B</span>}</div>";
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // The div should have one child: an expression container holding a fragment
        if let JSXChild::Element(div) = &ret.root.body[0] {
            assert_eq!(div.children.len(), 1, "div should have one expression container child");
            if let JSXChild::ExpressionContainer(container) = &div.children[0] {
                assert!(
                    matches!(container.expression, JSXExpression::JSXFragment(_)),
                    "expression should be a synthetic fragment wrapping multiple children"
                );
                if let JSXExpression::JSXFragment(ref frag) = container.expression {
                    assert_eq!(frag.children.len(), 2, "fragment should have 2 children");
                }
            } else {
                panic!("expected ExpressionContainer child");
            }
        } else {
            panic!("expected Element at root");
        }
    }

    #[test]
    fn parse_astro_single_jsx_child_in_expression() {
        // A single JSX element inside a child expression container should be
        // unwrapped (no synthetic fragment).
        let source = "<div>{<span>A</span>}</div>";
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        if let JSXChild::Element(div) = &ret.root.body[0] {
            assert_eq!(div.children.len(), 1);
            if let JSXChild::ExpressionContainer(container) = &div.children[0] {
                assert!(
                    matches!(container.expression, JSXExpression::JSXElement(_)),
                    "single JSX child should be unwrapped as JSXElement, not wrapped in fragment"
                );
            } else {
                panic!("expected ExpressionContainer child");
            }
        } else {
            panic!("expected Element at root");
        }
    }

    #[test]
    fn parse_astro_jsx_children_with_text_in_expression() {
        // Mixed JSX elements and text inside a child expression container
        let source = "<div>{<span>A</span> text <span>B</span>}</div>";
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        if let JSXChild::Element(div) = &ret.root.body[0] {
            if let JSXChild::ExpressionContainer(container) = &div.children[0] {
                if let JSXExpression::JSXFragment(ref frag) = container.expression {
                    // element, text, element = 3 children
                    assert_eq!(
                        frag.children.len(),
                        3,
                        "fragment should have 3 children (element, text, element)"
                    );
                    assert!(matches!(frag.children[0], JSXChild::Element(_)));
                    assert!(matches!(frag.children[1], JSXChild::Text(_)));
                    assert!(matches!(frag.children[2], JSXChild::Element(_)));
                } else {
                    panic!("expected fragment for multiple children");
                }
            } else {
                panic!("expected ExpressionContainer child");
            }
        } else {
            panic!("expected Element at root");
        }
    }

    #[test]
    fn parse_astro_fragment_child_in_expression() {
        // A fragment inside a child expression container
        let source = "<div>{<>A<span>B</span></>}</div>";
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        if let JSXChild::Element(div) = &ret.root.body[0] {
            if let JSXChild::ExpressionContainer(container) = &div.children[0] {
                assert!(
                    matches!(container.expression, JSXExpression::JSXFragment(_)),
                    "single fragment child should be unwrapped as JSXFragment"
                );
            } else {
                panic!("expected ExpressionContainer child");
            }
        } else {
            panic!("expected Element at root");
        }
    }

    #[test]
    fn parse_astro_nested_expressions_in_jsx_children() {
        // Nested expression containers inside multi-child JSX expressions
        let source = "<div>{<span>{value}</span><p>{other}</p>}</div>";
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        if let JSXChild::Element(div) = &ret.root.body[0] {
            if let JSXChild::ExpressionContainer(container) = &div.children[0] {
                if let JSXExpression::JSXFragment(ref frag) = container.expression {
                    assert_eq!(frag.children.len(), 2, "fragment should have 2 element children");
                    // Verify each child element has its own expression container
                    for child in &frag.children {
                        if let JSXChild::Element(el) = child {
                            let has_expr = el
                                .children
                                .iter()
                                .any(|c| matches!(c, JSXChild::ExpressionContainer(_)));
                            assert!(has_expr, "each child element should contain an expression");
                        } else {
                            panic!("expected Element children in fragment");
                        }
                    }
                } else {
                    panic!("expected fragment");
                }
            } else {
                panic!("expected ExpressionContainer child");
            }
        } else {
            panic!("expected Element at root");
        }
    }

    #[test]
    fn parse_astro_html_comment_containing_script_tags() {
        // HTML comments containing <script> tags should be preserved as a single
        // comment, not parsed as real script elements.
        use oxc_ast::ast::JSXChild;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<div>before</div>
<!-- <script async src="https://example.com/gtag.js"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());
</script> -->
<div>after</div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let comment_count =
            ret.root.body.iter().filter(|c| matches!(c, JSXChild::AstroComment(_))).count();
        assert_eq!(comment_count, 1, "Expected 1 HTML comment wrapping the script tags");

        // There should be no AstroScript nodes — the <script> is inside a comment.
        let script_count =
            ret.root.body.iter().filter(|c| matches!(c, JSXChild::AstroScript(_))).count();
        assert_eq!(script_count, 0, "Script tags inside comments should not be parsed");

        // There should be exactly 2 real elements (the two <div>s).
        let element_count =
            ret.root.body.iter().filter(|c| matches!(c, JSXChild::Element(_))).count();
        assert_eq!(element_count, 2, "Expected 2 div elements around the comment");
    }

    #[test]
    fn parse_astro_script_followed_by_comment_with_script() {
        // A real <script is:inline> followed by comments containing <script> tags.
        // The <script> inside comments must NOT be parsed as a real script element.
        use oxc_ast::ast::JSXChild;

        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<script is:inline>
  const root = document.documentElement;
</script>

<!-- Global site tag (gtag.js) - Google Analytics -->
<!-- <script async src="https://www.googletagmanager.com/gtag/js?id=G-TEL60V1WM9"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());
  gtag('config', 'G-TEL60V1WM9');
</script> -->"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        // Parse errors from script content parsing are acceptable — we're checking
        // the AST structure, not whether the content is valid JS.

        let comment_count =
            ret.root.body.iter().filter(|c| matches!(c, JSXChild::AstroComment(_))).count();
        let script_count = ret
            .root
            .body
            .iter()
            .filter(|c| {
                matches!(c, JSXChild::Element(el) if {
                    match &el.opening_element.name {
                        JSXElementName::Identifier(ident) => ident.name.as_str() == "script",
                        _ => false,
                    }
                })
            })
            .count();

        assert_eq!(
            comment_count,
            2,
            "Expected 2 HTML comments, got {comment_count}. Children: {:?}",
            ret.root
                .body
                .iter()
                .map(|c| match c {
                    JSXChild::AstroComment(_) => "Comment".to_string(),
                    JSXChild::Element(el) => {
                        let name = match &el.opening_element.name {
                            JSXElementName::Identifier(ident) => ident.name.to_string(),
                            _ => "?".to_string(),
                        };
                        format!("Element({name})")
                    }
                    JSXChild::Text(t) =>
                        format!("Text({:?})", &t.value.as_str()[..t.value.len().min(20)]),
                    JSXChild::AstroScript(_) => "AstroScript".to_string(),
                    _ => "Other".to_string(),
                })
                .collect::<Vec<_>>()
        );
        // Only the real <script is:inline> should be a script element, not the commented-out ones.
        assert_eq!(script_count, 1, "Expected 1 real script element, got {script_count}");
    }

    // === Edge case tests for `---` appearing in non-frontmatter contexts ===

    #[test]
    fn parse_astro_dashes_in_css_comment_no_frontmatter() {
        // `---` inside a CSS comment should NOT be treated as a frontmatter fence
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<style>
/* --- horizontal rule style --- */
hr { border: 1px solid red; }
</style>
<div>Hello</div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        // The key assertion: this should NOT have any frontmatter content
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        assert!(
            frontmatter.program.body.is_empty(),
            "Expected empty frontmatter, but got {} statements - the CSS comment `---` was wrongly treated as a fence",
            frontmatter.program.body.len()
        );
        // Body should contain the style and div elements
        assert!(!ret.root.body.is_empty(), "Body should not be empty");
    }

    #[test]
    fn parse_astro_dashes_in_script_string_no_frontmatter() {
        // `---` inside a script string should NOT be treated as a frontmatter fence
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<script>
const msg = "--- separator ---";
console.log(msg);
</script>
<div>Hello</div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        assert!(
            frontmatter.program.body.is_empty(),
            "Expected empty frontmatter, but got {} statements - the script string `---` was wrongly treated as a fence",
            frontmatter.program.body.len()
        );
        assert!(!ret.root.body.is_empty(), "Body should not be empty");
    }

    #[test]
    fn parse_astro_dashes_in_css_comment_with_frontmatter() {
        // `---` inside a CSS comment in the body should not interfere with parsing
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"---
const x = 1;
---
<style>
/* --- horizontal rule style --- */
hr { border: 1px solid red; }
</style>
<div>Hello</div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        assert_eq!(frontmatter.program.body.len(), 1, "Frontmatter should have 1 statement");
        assert!(!ret.root.body.is_empty(), "Body should not be empty");
    }

    #[test]
    fn parse_astro_dashes_in_script_string_with_frontmatter() {
        // `---` inside a script string constant in the body should not interfere
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"---
const x = 1;
---
<script>
const msg = "--- separator ---";
console.log(msg);
</script>
<div>Hello</div>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let frontmatter = ret.root.frontmatter.as_ref().unwrap();
        assert_eq!(frontmatter.program.body.len(), 1, "Frontmatter should have 1 statement");
        assert!(!ret.root.body.is_empty(), "Body should not be empty");
    }

    #[test]
    fn parse_astro_extra_closing_brace_no_panic() {
        // `content={Astro.generator}}` has an extra `}` — the parser should not panic.
        // The stray `}` should be consumed as a (bogus) attribute name with no value.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"---

---

<html lang="en">
	<head>
		<meta charset="utf-8" />
		<link rel="icon" type="image/svg+xml" href="/favicon.svg" />
		<link rel="icon" href="/favicon.ico" />
		<meta name="viewport" content="width=device-width" />
		<meta name="generator" content={Astro.generator}} />
		<title>Astro</title>
	</head>
	<body>
		<h1>Astro</h1>
	</body>
</html>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        // We expect parse errors but no panic
    }

    #[test]
    fn parse_astro_extra_closing_brace_becomes_attribute() {
        // Minimal repro: the stray `}` after the expression should be parsed as
        // a valueless attribute named `}`, not cause an infinite loop.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = r#"<meta name="generator" content={Astro.generator}} />"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);

        assert_eq!(ret.root.body.len(), 1);
        if let JSXChild::Element(element) = &ret.root.body[0] {
            let attrs = &element.opening_element.attributes;
            // name, content={Astro.generator}, }
            assert_eq!(attrs.len(), 3, "expected 3 attributes, got {attrs:?}");
            // The third attribute should be the stray `}`
            if let JSXAttributeItem::Attribute(attr) = &attrs[2] {
                if let JSXAttributeName::Identifier(ident) = &attr.name {
                    assert_eq!(ident.name.as_str(), "}");
                } else {
                    panic!("Expected Identifier attribute name for stray `}}`");
                }
                assert!(attr.value.is_none(), "stray `}}` attribute should have no value");
            } else {
                panic!("Expected Attribute, got SpreadAttribute");
            }
        } else {
            panic!("Expected JSXChild::Element");
        }
    }

    #[test]
    fn parse_astro_stray_closing_tag_reports_unmatched() {
        // An extra `</div>` with no matching open tag should be reported at its
        // own location, not blamed on an unrelated ancestor element.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<section><div>hi</div></div></section>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);

        let msg = ret.errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
        assert_eq!(ret.errors.len(), 1, "expected exactly one error, got: {msg}");
        assert!(
            msg.contains("Closing tag '</div>' has no matching opening tag."),
            "unexpected error message: {msg}"
        );

        // Recovery: <section> still pairs with its own </section>.
        assert_eq!(ret.root.body.len(), 1);
        let JSXChild::Element(section) = &ret.root.body[0] else {
            panic!("expected <section> element");
        };
        assert!(section.closing_element.is_some(), "<section> should be closed");
    }

    #[test]
    fn parse_astro_stray_closing_tag_recovers_following_siblings() {
        // After skipping the stray `</div>`, the following `<p>` sibling must
        // still be parsed as a child of `<main>`.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "<main><div>a</div></div><p>after</p></main>";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);

        let msg = ret.errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
        assert_eq!(ret.errors.len(), 1, "expected exactly one error, got: {msg}");
        assert!(
            msg.contains("Closing tag '</div>' has no matching opening tag."),
            "unexpected error message: {msg}"
        );

        assert_eq!(ret.root.body.len(), 1);
        let JSXChild::Element(main) = &ret.root.body[0] else {
            panic!("expected <main> element");
        };
        assert!(main.closing_element.is_some(), "<main> should be closed");
        let element_children =
            main.children.iter().filter(|c| matches!(c, JSXChild::Element(_))).count();
        assert_eq!(element_children, 2, "expected <div> and <p> as children of <main>");
    }

    #[test]
    fn parse_astro_second_script_in_expression_is_raw_text() {
        // A non-first `<script>` sibling in a `{ ... }` expression must still parse
        // its body as raw text, or a close tag in a template literal becomes a
        // stray closing-tag error.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "{enabled && (\n  <script src=\"x.js\"></script>\n  <script>var a = `</article>`;</script>\n)}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);
    }

    #[test]
    fn parse_astro_bare_siblings_in_expression_keep_whitespace() {
        // Whitespace between bare JSX siblings in a `{ ... }` expression must
        // survive as a JSXText child (the JS lexer skips it otherwise).
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "{x && (<em>a</em>\n  <span>b</span>)}";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        let JSXChild::ExpressionContainer(container) = &ret.root.body[0] else {
            panic!("expected expression container");
        };
        let Some(Expression::LogicalExpression(logical)) = container.expression.as_expression()
        else {
            panic!("expected `&&` logical expression");
        };
        let right = match &logical.right {
            Expression::ParenthesizedExpression(paren) => &paren.expression,
            other => other,
        };
        let Expression::JSXFragment(fragment) = right else {
            panic!("expected implicit fragment for bare siblings, got {right:?}");
        };
        let text_children = fragment
            .children
            .iter()
            .filter(|c| matches!(c, JSXChild::Text(_)))
            .count();
        assert_eq!(text_children, 1, "inter-sibling whitespace should be a JSXText node");
    }

    #[test]
    fn parse_astro_overlapping_comment_marker_does_not_panic() {
        // `<!-->` overlaps the `-->` with the `<!--` open; the comment parsers must
        // look for the close at offset >= 3, not slice `rest[3..1]` and panic.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        for source in ["{<a/><!-->}", "<!-->", "<div><!--></div>", "{x && (<a/><!-->)}"] {
            // The contract is "no panic"; some of these are still errors.
            let _ = Parser::new(&allocator, source, source_type).parse_astro();
        }
    }

    #[test]
    fn parse_astro_comment_between_bare_siblings() {
        // An HTML comment in a run of bare JSX siblings is kept as an AstroComment
        // child (matching Go), including the common case of the comment on its own
        // line, which only works because the lexer no longer line-comments `<!--`.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        for source in [
            "{x && (<a/><!--c--><b/>)}",
            "{x && (\n  <a/>\n  <!-- c -->\n  <b/>\n)}",
            "{x && (<a/><!--c-->)}",
        ] {
            let ret = Parser::new(&allocator, source, source_type).parse_astro();
            assert!(ret.errors.is_empty(), "{source:?} errors: {:?}", ret.errors);

            let JSXChild::ExpressionContainer(container) = &ret.root.body[0] else {
                panic!("expected expression container for {source:?}");
            };
            let Some(Expression::LogicalExpression(logical)) =
                container.expression.as_expression()
            else {
                panic!("expected `&&` logical expression for {source:?}");
            };
            let right = match &logical.right {
                Expression::ParenthesizedExpression(paren) => &paren.expression,
                other => other,
            };
            let Expression::JSXFragment(fragment) = right else {
                panic!("expected implicit fragment for {source:?}, got {right:?}");
            };
            let comments =
                fragment.children.iter().filter(|c| matches!(c, JSXChild::AstroComment(_))).count();
            assert_eq!(comments, 1, "{source:?} should keep the comment as a child");
        }
    }

    #[test]
    fn parse_astro_jsx_lt_comparison_still_parses() {
        // The comment heuristic only diverts when the left side is a JSX element, so
        // an ordinary JS `< !--` (less-than, negated pre-decrement) is untouched even
        // when a `-->` appears later. Must not be misread as a comment.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        for source in ["{(3 < !--n, m-->q)}", "{x && y < !--count}"] {
            let ret = Parser::new(&allocator, source, source_type).parse_astro();
            assert!(ret.errors.is_empty(), "{source:?} should parse: {:?}", ret.errors);
        }
    }

    #[test]
    fn parse_astro_shorthand_attribute_fatal_expr_does_not_panic() {
        // Guards the inverted name-span panic when a shorthand-attribute `{…}`
        // expression fails to parse without advancing the lexer.
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        for source in ["<div><div{\n)</div>", "{x && (<a/>\n<div{\n)}", "<Comp {(\n} />"] {
            // The contract is "no panic"; these are still errors.
            let _ = Parser::new(&allocator, source, source_type).parse_astro();
        }
    }
}
