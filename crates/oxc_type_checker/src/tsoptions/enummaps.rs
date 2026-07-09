//! Port of typescript-go's `internal/tsoptions/enummaps.go`.
//!
//! The tsconfig string spellings of the enum-valued compiler options. tsgo keeps these maps in
//! `tsoptions` (the parsing layer) rather than on the `core` types; the enums themselves live
//! in [`crate::core`]. Lookups are case-insensitive (tsgo lowercases the input key), and an
//! unknown value yields `None` — tsc diagnoses invalid values and leaves the option unset.

use cow_utils::CowUtils;

use crate::core::{
    JsxEmit, ModuleDetectionKind, ModuleKind, ModuleResolutionKind, NewLineKind, ScriptTarget,
};

impl ModuleKind {
    /// tsgo `moduleOptionMap`. `"none"` and unknown values both yield `None` (tsgo's
    /// `ModuleKindNone` zero value is indistinguishable from an unset option).
    pub fn from_str_ignore_case(value: &str) -> Option<Self> {
        match value.cow_to_ascii_lowercase().as_ref() {
            "commonjs" => Some(Self::CommonJs),
            "amd" => Some(Self::Amd),
            "system" => Some(Self::System),
            "umd" => Some(Self::Umd),
            "es6" | "es2015" => Some(Self::Es2015),
            "es2020" => Some(Self::Es2020),
            "es2022" => Some(Self::Es2022),
            "esnext" => Some(Self::EsNext),
            "node16" => Some(Self::Node16),
            "node18" => Some(Self::Node18),
            "node20" => Some(Self::Node20),
            "nodenext" => Some(Self::NodeNext),
            "preserve" => Some(Self::Preserve),
            _ => None,
        }
    }
}

impl ModuleResolutionKind {
    /// tsgo `moduleResolutionOptionMap`.
    pub fn from_str_ignore_case(value: &str) -> Option<Self> {
        match value.cow_to_ascii_lowercase().as_ref() {
            "node16" => Some(Self::Node16),
            "nodenext" => Some(Self::NodeNext),
            "bundler" => Some(Self::Bundler),
            "classic" => Some(Self::Classic),
            "node" | "node10" => Some(Self::Node10),
            _ => None,
        }
    }
}

impl ScriptTarget {
    /// tsgo `targetOptionMap`.
    pub fn from_str_ignore_case(value: &str) -> Option<Self> {
        match value.cow_to_ascii_lowercase().as_ref() {
            "es5" => Some(Self::Es5),
            "es6" | "es2015" => Some(Self::Es2015),
            "es2016" => Some(Self::Es2016),
            "es2017" => Some(Self::Es2017),
            "es2018" => Some(Self::Es2018),
            "es2019" => Some(Self::Es2019),
            "es2020" => Some(Self::Es2020),
            "es2021" => Some(Self::Es2021),
            "es2022" => Some(Self::Es2022),
            "es2023" => Some(Self::Es2023),
            "es2024" => Some(Self::Es2024),
            "es2025" => Some(Self::Es2025),
            "esnext" => Some(Self::EsNext),
            _ => None,
        }
    }
}

impl ModuleDetectionKind {
    /// tsgo `moduleDetectionOptionMap`.
    pub fn from_str_ignore_case(value: &str) -> Option<Self> {
        match value.cow_to_ascii_lowercase().as_ref() {
            "auto" => Some(Self::Auto),
            "legacy" => Some(Self::Legacy),
            "force" => Some(Self::Force),
            _ => None,
        }
    }
}

impl JsxEmit {
    /// tsgo `jsxOptionMap`.
    pub fn from_str_ignore_case(value: &str) -> Option<Self> {
        match value.cow_to_ascii_lowercase().as_ref() {
            "preserve" => Some(Self::Preserve),
            "react-native" => Some(Self::ReactNative),
            "react-jsx" => Some(Self::ReactJsx),
            "react-jsxdev" => Some(Self::ReactJsxDev),
            "react" => Some(Self::React),
            _ => None,
        }
    }
}

impl NewLineKind {
    /// tsgo `newLineOptionMap`.
    pub fn from_str_ignore_case(value: &str) -> Option<Self> {
        match value.cow_to_ascii_lowercase().as_ref() {
            "crlf" => Some(Self::CarriageReturnLineFeed),
            "lf" => Some(Self::LineFeed),
            _ => None,
        }
    }
}
