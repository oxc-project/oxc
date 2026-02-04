//! [Astro](https://astro.build) AST Definitions
//!
//! Astro files have a frontmatter section (TypeScript) delimited by `---` and
//! an HTML body that can contain JSX expressions.

// NB: `#[span]`, `#[scope(...)]`,`#[visit(...)]` and `#[generate_derive(...)]` do NOT do anything to the code.
// They are purely markers for codegen used in `tasks/ast_tools` and `crates/oxc_traverse/scripts`. See docs in those crates.
// Read [`macro@oxc_ast_macros::ast`] for more information.

use oxc_allocator::{Box, CloneIn, Dummy, TakeIn, UnstableAddress, Vec};
use oxc_ast_macros::ast;
use oxc_estree::ESTree;
use oxc_span::{Atom, ContentEq, GetSpan, GetSpanMut, Span};

use super::js::Program;
use super::jsx::*;

/// Astro Root Node
///
/// The root node of an Astro file, containing optional frontmatter and an HTML body.
///
/// ## Example
///
/// ```astro
/// ---
/// const name = "World";
/// ---
/// <h1>Hello {name}!</h1>
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, TakeIn, GetSpan, GetSpanMut, ContentEq, ESTree, UnstableAddress)]
pub struct AstroRoot<'a> {
    /// Node location in source code
    pub span: Span,
    /// The frontmatter section between `---` delimiters, containing TypeScript code.
    /// This is `None` if the file has no frontmatter.
    pub frontmatter: Option<Box<'a, AstroFrontmatter<'a>>>,
    /// The HTML body of the Astro file, which can contain JSX expressions.
    /// Represented as JSX children since it behaves like an implicit fragment.
    pub body: Vec<'a, JSXChild<'a>>,
}

/// Astro Frontmatter
///
/// The frontmatter section of an Astro file, delimited by `---`.
/// Contains TypeScript code that runs at build time.
///
/// ## Example
///
/// ```astro
/// ---
/// import Component from './Component.astro';
/// const items = ["a", "b", "c"];
/// ---
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, TakeIn, GetSpan, GetSpanMut, ContentEq, ESTree, UnstableAddress)]
pub struct AstroFrontmatter<'a> {
    /// Node location in source code (includes the `---` delimiters)
    pub span: Span,
    /// The parsed TypeScript program from the frontmatter content
    pub program: Program<'a>,
}

/// Astro Script Element
///
/// A `<script>` element in an Astro file containing TypeScript/JavaScript code.
/// Unlike regular HTML script tags, Astro scripts are processed at build time.
///
/// ## Example
///
/// ```astro
/// <script>
///   console.log("Hello from Astro!");
/// </script>
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, TakeIn, GetSpan, GetSpanMut, ContentEq, ESTree, UnstableAddress)]
pub struct AstroScript<'a> {
    /// Node location in source code (includes the `<script>` tags)
    pub span: Span,

    /// The parsed TypeScript/JavaScript program from the script content
    pub program: Program<'a>,
}

/// Astro Doctype Declaration
///
/// Represents an HTML doctype declaration like `<!doctype html>` or `<!DOCTYPE html>`.
/// This is commonly used at the start of Astro pages to declare the document type.
///
/// ## Example
///
/// ```astro
/// <!doctype html>
/// <html>
///   <body>Hello</body>
/// </html>
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, TakeIn, GetSpan, GetSpanMut, ContentEq, ESTree, UnstableAddress)]
pub struct AstroDoctype<'a> {
    /// Node location in source code (includes `<!` and `>`)
    pub span: Span,
    /// The document type value, e.g., "html" from `<!doctype html>` or `<!DOCTYPE html>`
    pub value: Atom<'a>,
}
