//! [Astro](https://astro.build) Parser
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

use oxc_allocator::{Box, Vec};
use oxc_ast::ast::*;

use crate::{ParserImpl, lexer::Kind};

type NoTypeArgs<'a> = Option<Box<'a, TSTypeParameterInstantiation<'a>>>;
type NoClosingElement<'a> = Option<Box<'a, JSXClosingElement<'a>>>;

impl<'a> ParserImpl<'a> {
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
                    return Some(JSXChild::Fragment(self.parse_jsx_fragment(span, true)));
                }

                // `<ident` - element
                if kind == Kind::Ident || kind.is_any_keyword() {
                    // Check if this is a <script> tag - we handle it specially in Astro
                    if self.cur_src().starts_with("script") {
                        // Check it's actually "script" and not "script-something"
                        let after_script =
                            self.source_text.get((self.cur_token().span().start as usize) + 6..);
                        if let Some(rest) = after_script {
                            let next_char = rest.chars().next();
                            if matches!(
                                next_char,
                                Some(' ' | '>' | '/' | '\n' | '\r' | '\t') | None
                            ) {
                                return Some(self.parse_astro_script(span));
                            }
                        }
                    }
                    return Some(JSXChild::Element(self.parse_jsx_element(span, true)));
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
                Some(JSXChild::ExpressionContainer(
                    self.parse_jsx_expression_container(span_start, /* in_jsx_child */ true),
                ))
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
            if rest.starts_with("!--") {
                // Find `-->` to close the comment
                if let Some(end_offset) = rest.find("-->") {
                    let comment_end = (start_pos + end_offset + 3) as u32;

                    // Add the HTML comment to trivia
                    let comment_start = (start_pos - 1) as u32; // include the `<`
                    self.lexer.trivia_builder.add_html_comment(
                        comment_start,
                        comment_end,
                        self.source_text,
                    );

                    // Move lexer position past the comment
                    self.lexer.set_position_for_astro(comment_end);
                    self.token = self.lexer.next_jsx_child();
                    return None; // Comments are trivia, not AST nodes
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
    /// According to Astro spec:
    /// - Bare `<script>` (no attributes) = TypeScript, parsed as AstroScript
    /// - `<script>` with any attributes = follows HTML rules, treated as raw text
    #[expect(clippy::cast_possible_truncation)]
    fn parse_astro_script(&mut self, span: u32) -> JSXChild<'a> {
        // We're at `script` identifier after `<`
        // Skip the `script` identifier
        self.bump_any();

        // Check if there are any attributes (anything before `>` or `/>`)
        let mut has_attributes = false;
        let mut is_self_closing = false;

        // Look for attributes or closing
        while !self.at(Kind::Eof) && !self.at(Kind::RAngle) {
            if self.at(Kind::Slash) {
                self.bump_any(); // skip `/`
                if self.at(Kind::RAngle) {
                    is_self_closing = true;
                    break;
                }
            } else if self.at(Kind::Ident) || self.cur_kind().is_any_keyword() {
                // Found an attribute
                has_attributes = true;
            }
            self.bump_any();
        }

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
                    self.ast.vec(),
                );
                let no_closing: NoClosingElement<'a> = None;
                return JSXChild::Element(self.ast.alloc_jsx_element(
                    script_span,
                    opening,
                    self.ast.vec(),
                    no_closing,
                ));
            }

            // Bare script - create empty AstroScript
            let program = self.ast.program(
                oxc_span::Span::empty(end),
                self.source_type,
                "",
                self.ast.vec(),
                None,
                self.ast.vec(),
                self.ast.vec(),
            );
            return JSXChild::AstroScript(self.ast.alloc_astro_script(script_span, program));
        }

        // Find the closing </script> tag
        let content_start = self.cur_token().span().start as usize;
        let closing_tag = "</script";

        if let Some(rest) = self.source_text.get(content_start..)
            && let Some(end_offset) = rest.find(closing_tag)
        {
            let content_end = content_start + end_offset;

            // Move lexer to the closing tag
            #[expect(clippy::cast_possible_truncation)]
            self.lexer.set_position_for_astro(content_end as u32);
            self.token = self.lexer.next_jsx_child();

            // Skip the closing tag </script>
            if self.at(Kind::LAngle) {
                self.bump_any(); // `<`
            }
            if self.at(Kind::Slash) {
                self.bump_any(); // `/`
            }
            // Skip `script`
            while !self.at(Kind::Eof) && !self.at(Kind::RAngle) {
                self.bump_any();
            }
            if self.at(Kind::RAngle) {
                self.bump_any(); // `>`
            }

            let end = self.prev_token_end;
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
                    self.ast.vec(),
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

            // Bare script - create AstroScript for later parsing
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

            return JSXChild::AstroScript(self.ast.alloc_astro_script(full_span, program));
        }

        // Fallback: couldn't find closing tag, parse as regular element
        JSXChild::Element(self.parse_jsx_element(span, true))
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_ast::ast::{JSXChild, JSXElementName, Statement};
    use oxc_span::{GetSpan, SourceType};

    use crate::Parser;

    #[test]
    fn parse_astro_smoke_test() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        let source = "";
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(ret.root.frontmatter.is_none());
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

        // No frontmatter
        assert!(ret.root.frontmatter.is_none());

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

        // Body should have 4 children: div, text (newline), script, div
        // (Text nodes for whitespace between elements)
        assert_eq!(ret.root.body.len(), 4, "Expected 4 children, got {}", ret.root.body.len());

        // First should be an element
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));

        // Second should be text (newline)
        assert!(matches!(ret.root.body[1], JSXChild::Text(_)));

        // Third should be an AstroScript with parsed TypeScript
        match &ret.root.body[2] {
            JSXChild::AstroScript(script) => {
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
            }
            other => panic!("Expected AstroScript, got {other:?}"),
        }

        // Fourth should be an element
        assert!(matches!(ret.root.body[3], JSXChild::Element(_)));
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
			Images and datas are fetched from <a href="https://igdb.com/"
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
        let first_element = ret.root.body.iter().find(|child| matches!(child, JSXChild::Element(_)));
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
        let source = r#"<Component {prop} />"#;
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

        // Body should have one AstroScript
        assert_eq!(ret.root.body.len(), 1);
        match &ret.root.body[0] {
            JSXChild::AstroScript(script) => {
                // Should have parsed TypeScript (interface + const)
                assert_eq!(script.program.body.len(), 2);
                assert!(matches!(script.program.body[0], Statement::TSInterfaceDeclaration(_)));
                assert!(matches!(script.program.body[1], Statement::VariableDeclaration(_)));
            }
            other => panic!("Expected AstroScript, got {other:?}"),
        }
    }

    #[test]
    fn parse_astro_script_with_attributes_not_parsed() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // <script> with attributes should NOT be parsed, just raw text
        let source = r#"<script type="module">
// This is JavaScript, not TypeScript
const x = 1;
</script>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one JSX Element (not AstroScript)
        assert_eq!(ret.root.body.len(), 1);
        match &ret.root.body[0] {
            JSXChild::Element(element) => {
                // Should be a script element with text content
                if let JSXElementName::Identifier(ident) = &element.opening_element.name {
                    assert_eq!(ident.name.as_str(), "script");
                } else {
                    panic!("Expected Identifier for script element");
                }
                // Should have text child
                assert_eq!(element.children.len(), 1);
                assert!(matches!(element.children[0], JSXChild::Text(_)));
            }
            other => panic!("Expected Element, got {other:?}"),
        }
    }

    #[test]
    fn parse_astro_script_defer_not_parsed() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // <script defer> should NOT be parsed as TypeScript
        let source = r#"<script defer>
console.log("hello");
</script>"#;
        let ret = Parser::new(&allocator, source, source_type).parse_astro();
        assert!(!ret.panicked, "parser panicked: {:?}", ret.errors);
        assert!(ret.errors.is_empty(), "errors: {:?}", ret.errors);

        // Body should have one JSX Element (not AstroScript)
        assert_eq!(ret.root.body.len(), 1);
        assert!(matches!(ret.root.body[0], JSXChild::Element(_)));
    }

    #[test]
    fn parse_astro_html_comment_in_expression() {
        let allocator = Allocator::default();
        let source_type = SourceType::astro();
        // HTML comments inside expressions should work in Astro
        let source = r#"<div>{
  /* JSX comment */
  <!-- HTML comment -->
}</div>"#;
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
        let source = r#"<div>{ <!-- comment --> true }</div>"#;
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
        let source = r#"---
const user = null;
if (!user) {
  return;
}
---
<div>Hello</div>"#;
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
        let source = r#"---
return;
---
<div>Hello</div>"#;
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
        // Expression container with whitespace around the inner element
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

            // Get the expression span
            let expr_span = container.expression.span();

            // Calculate leading/trailing whitespace from spans
            let leading_ws = &source[(container.span.start + 1) as usize..expr_span.start as usize];
            let trailing_ws = &source[expr_span.end as usize..(container.span.end - 1) as usize];

            // The leading whitespace should be "\n\t" (newline + tab)
            assert_eq!(leading_ws, "\n\t", "Leading whitespace should be preserved in span");

            // The trailing whitespace should be "\n" (newline before closing brace)
            assert_eq!(trailing_ws, "\n", "Trailing whitespace should be preserved in span");
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
                assert_eq!(div_count, 3, "Expected 3 div elements, got {}", div_count);
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
}
