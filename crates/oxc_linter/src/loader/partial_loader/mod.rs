mod astro;
mod svelte;
mod vue;

pub use self::{astro::AstroPartialLoader, svelte::SveltePartialLoader, vue::VuePartialLoader};
use crate::loader::JavaScriptSource;

const SCRIPT_START: &str = "<script";
const SCRIPT_END: &str = "</script>";

pub const LINT_PARTIAL_LOADER_EXT: &[&str] = &["vue", "astro", "svelte"];

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
fn find_script_closing_angle(source_text: &str, pointer: usize) -> Option<usize> {
    let mut numbers_of_open_angle = 0;
    for (offset, c) in source_text[pointer..].char_indices() {
        match c {
            '>' => {
                if numbers_of_open_angle == 0 {
                    return Some(offset);
                }
                numbers_of_open_angle -= 1;
            }
            '<' => {
                numbers_of_open_angle += 1;
            }
            _ => {}
        }
    }
    None
}
