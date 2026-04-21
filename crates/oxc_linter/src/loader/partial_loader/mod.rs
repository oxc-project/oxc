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
fn find_script_closing_angle(source_text: &str, pointer: usize) -> Option<usize> {
    let mut open_angle = 0;
    let mut in_quote: Option<char> = None;

    for (offset, c) in source_text[pointer..].char_indices() {
        match c {
            '"' | '\'' => {
                if let Some(q) = in_quote {
                    if q == c {
                        in_quote = None;
                    }
                } else {
                    in_quote = Some(c);
                }
            }
            '<' if in_quote.is_none() => {
                open_angle += 1;
            }
            '>' if in_quote.is_none() => {
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
