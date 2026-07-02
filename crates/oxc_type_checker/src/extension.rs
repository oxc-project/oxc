//! Ported from typescript-go `internal/tspath/extension.go`.
//!
//! Dotted extension constants and the *grouped* priority arrays used when enumerating
//! project files. Grouping encodes extension precedence: within a group an earlier
//! extension shadows a later one for the same basename (e.g. `.ts` over `.d.ts`/`.js`),
//! while different groups (`.cts` vs `.ts`) are independent.

pub const EXTENSION_TS: &str = ".ts";
pub const EXTENSION_TSX: &str = ".tsx";
pub const EXTENSION_DTS: &str = ".d.ts";
pub const EXTENSION_JS: &str = ".js";
pub const EXTENSION_JSX: &str = ".jsx";
pub const EXTENSION_JSON: &str = ".json";
pub const EXTENSION_MJS: &str = ".mjs";
pub const EXTENSION_MTS: &str = ".mts";
pub const EXTENSION_DMTS: &str = ".d.mts";
pub const EXTENSION_CJS: &str = ".cjs";
pub const EXTENSION_CTS: &str = ".cts";
pub const EXTENSION_DCTS: &str = ".d.cts";

/// `AllSupportedExtensions`: TS + JS, grouped by priority.
pub const ALL_SUPPORTED_EXTENSIONS: &[&[&str]] = &[
    &[EXTENSION_TS, EXTENSION_TSX, EXTENSION_DTS, EXTENSION_JS, EXTENSION_JSX],
    &[EXTENSION_CTS, EXTENSION_DCTS, EXTENSION_CJS],
    &[EXTENSION_MTS, EXTENSION_DMTS, EXTENSION_MJS],
];

/// `SupportedTSExtensions`: TS only, grouped by priority.
pub const SUPPORTED_TS_EXTENSIONS: &[&[&str]] = &[
    &[EXTENSION_TS, EXTENSION_TSX, EXTENSION_DTS],
    &[EXTENSION_CTS, EXTENSION_DCTS],
    &[EXTENSION_MTS, EXTENSION_DMTS],
];

/// The extra priority group appended when `resolveJsonModule` is enabled.
pub const JSON_GROUP: &[&str] = &[EXTENSION_JSON];

/// Flatten grouped extension arrays into a single ordered list.
pub fn flatten(groups: &[&'static [&'static str]]) -> Vec<&'static str> {
    groups.iter().flat_map(|group| group.iter().copied()).collect()
}
