//! Registry of JavaScript-hosted formats: files that are JS/TS apart from islands of
//! another language.
//!
//! This is the mirror image of `oxc_linter`'s `PartialLoader`, which extracts JavaScript
//! out of markup hosts like Vue and Svelte. Here the file IS JavaScript and the islands are
//! the foreign part, so a locator reports the islands instead of the script blocks. Each
//! format contributes one locator and one row in [`lookup`]; nothing else in the formatter
//! knows the format exists.
//!
//! Islands are formatted through [`oxc_formatter::OpaqueRegion`]: the caller blanks each
//! one to an equal-length placeholder so the JavaScript parses, then formats against the
//! original text. That mechanism is language-agnostic and lives in `oxc_formatter`; the
//! per-format knowledge, which delimiters bound an island and what language it holds, lives
//! in this directory.

mod ember;

use oxc_formatter::OpaqueRegion;
use oxc_span::SourceType;

/// A format whose files are JavaScript hosting islands of another language.
pub struct HostedFormat {
    /// How the JavaScript around the islands parses.
    pub source_type: SourceType,
    /// Config key that opts this format in, mirroring how `svelte` gates `.svelte`.
    /// Only the napi build gates, so the pure one never reads it.
    #[cfg_attr(not(feature = "napi"), expect(dead_code))]
    pub config_key: &'static str,
    /// Reports every island in the source, in order and non-overlapping.
    ///
    /// A locator works on raw text, so it cannot be exact; the format step verifies each
    /// island against the parsed AST and declines the file when one does not line up.
    pub locate: fn(&str) -> Vec<OpaqueRegion<'static>>,
}

/// The hosted format for this extension, if there is one.
pub fn lookup(extension: &str) -> Option<HostedFormat> {
    match extension {
        // Both are ES modules. `.gts` is TypeScript without JSX: the angle brackets a
        // template tag uses are removed before parsing, and `<T>x` there is a type assertion.
        "gjs" => Some(HostedFormat {
            source_type: SourceType::mjs(),
            config_key: "ember",
            locate: ember::locate,
        }),
        "gts" => Some(HostedFormat {
            source_type: SourceType::ts().with_module(true),
            config_key: "ember",
            locate: ember::locate,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claims_only_registered_extensions() {
        assert!(lookup("gjs").is_some());
        assert!(lookup("gts").is_some());
        for extension in ["js", "ts", "mjs", "vue", "svelte", "hbs", "gj", "gjsx"] {
            assert!(lookup(extension).is_none(), "`{extension}` must not be a hosted format");
        }
    }

    #[test]
    fn every_format_locates_islands_in_its_own_files() {
        // A registry row is only useful if its locator actually reports something for the
        // extension it claims, so exercise each row rather than trusting the table.
        let sources = [
            ("gjs", "<template>x</template>"),
            ("gts", "const a: number = 1;\n<template>x</template>"),
        ];
        for (extension, source) in sources {
            let format = lookup(extension).unwrap();
            let islands = (format.locate)(source);
            assert_eq!(islands.len(), 1, "`{extension}` locator should find one island");
            assert_eq!(islands[0].language, "ember-template-tag");
        }
    }
}
