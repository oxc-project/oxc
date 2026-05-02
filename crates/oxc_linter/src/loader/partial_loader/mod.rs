use memchr::{memmem::Finder, memmem::FinderRev};
use oxc_span::VALID_EXTENSIONS;

use crate::loader::JavaScriptSource;

mod astro;
mod svelte;
mod vue;
pub use astro::AstroPartialLoader;
pub use svelte::SveltePartialLoader;
pub use vue::VuePartialLoader;

const SCRIPT_START: &str = "<script";
const SCRIPT_END: &str = "</script>";
const COMMENT_START: &str = "<!--";
const COMMENT_END: &str = "-->";

/// File extensions that can contain JS/TS code in certain parts, such as in `<script>` tags, and can
/// be loaded using the [`PartialLoader`].
pub const LINT_PARTIAL_LOADER_EXTENSIONS: &[&str] = &["vue", "astro", "svelte"];

/// All valid JavaScript/TypeScript extensions, plus additional framework files that
/// contain JavaScript/TypeScript code in them (e.g., Vue, Astro, Svelte, etc.).
pub const LINTABLE_EXTENSIONS: &[&str] =
    constcat::concat_slices!([&str]: VALID_EXTENSIONS, LINT_PARTIAL_LOADER_EXTENSIONS);

pub struct PartialLoader;

impl PartialLoader {
    /// Extract js section of special files.
    /// Returns `None` if the special file does not have a js section.
    pub fn parse<'a>(ext: &str, source_text: &'a str) -> Option<Vec<JavaScriptSource<'a>>> {
        match ext {
            "vue" => Some(VuePartialLoader::new(source_text).parse()),
            "astro" => Some(AstroPartialLoader::new(source_text).parse()),
            "svelte" => Some(SveltePartialLoader::new(source_text).parse()),
            _ => None,
        }
    }
}

/// Find closing angle for situations where there is another `>` in between.
/// e.g. `<script generic="T extends Record<string, string>">`
/// or `<script attribute="text with > inside">`
/// or `<script onload={() => {}}>`
fn find_script_closing_angle(source_text: &str, pointer: usize) -> Option<usize> {
    let mut open_angle = 0;
    let mut open_brace = 0;
    let mut in_quote: Option<char> = None;

    for (offset, c) in source_text[pointer..].char_indices() {
        match c {
            '"' | '\'' => {
                if let Some(q) = in_quote {
                    if q == c {
                        in_quote = None;
                    }
                } else if open_brace == 0 {
                    in_quote = Some(c);
                }
            }
            '{' if in_quote.is_none() => {
                open_brace += 1;
            }
            '}' if in_quote.is_none() && open_brace > 0 => {
                open_brace -= 1;
            }
            '<' if in_quote.is_none() && open_brace == 0 => {
                open_angle += 1;
            }
            '>' if in_quote.is_none() && open_brace == 0 => {
                if open_angle == 0 {
                    return Some(offset);
                }
                open_angle -= 1;
            }
            _ => {}
        }
    }

    None
}

fn find_script_start(
    source_text: &str,
    pointer: usize,
    script_start_finder: &Finder<'_>,
    comment_start_finder: &FinderRev<'_>,
    comment_end_finder: &Finder<'_>,
) -> Option<usize> {
    let mut new_pointer = pointer;

    loop {
        new_pointer +=
            script_start_finder.find(&source_text.as_bytes()[new_pointer..])? + SCRIPT_START.len();

        if let Some(offset) = comment_start_finder.rfind(&source_text.as_bytes()[..new_pointer]) {
            if comment_end_finder
                .find(&source_text.as_bytes()[offset + COMMENT_START.len()..new_pointer])
                .is_some()
            {
                break;
            }
        } else {
            break;
        }
    }

    Some(new_pointer - pointer)
}

enum AttributeValue<'a> {
    Empty,
    Value(&'a str),
}

fn find_attribute<'a>(content: &'a str, target: &str) -> Option<AttributeValue<'a>> {
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
                    AttributeValue::Value(value)
                }
                Some(_) => {
                    let end = rest
                        .find(|c: char| c.is_whitespace() || matches!(c, '>' | '/'))
                        .unwrap_or(rest.len());
                    let value = &rest[..end];
                    rest = &rest[end..];
                    AttributeValue::Value(value)
                }
                None => return None,
            }
        } else {
            AttributeValue::Empty
        };

        if name.eq_ignore_ascii_case(target) {
            return Some(value);
        }
    }
}
