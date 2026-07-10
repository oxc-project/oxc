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

/// tsgo `GetLibFileName` over `LibMap` (`internal/tsoptions/enummaps.go`): validate a tsconfig
/// `lib` entry — a lib name (`"es2020"`, `"dom.iterable"`) or an already-canonical lib file
/// name (`"lib.es2020.d.ts"`) — and canonicalize it to its lowercased `lib.*.d.ts` file name.
/// Unknown names yield `None` (tsc diagnoses them and drops the entry).
#[expect(
    clippy::match_same_arms,
    reason = "one arm per tsgo LibMap entry (aliases share targets), then one per file name, \
              so the table diffs 1:1 against enummaps.go"
)]
pub(super) fn get_lib_file_name(lib_name: &str) -> Option<&'static str> {
    match lib_name.cow_to_ascii_lowercase().as_ref() {
        "es5" => Some("lib.es5.d.ts"),
        "es6" => Some("lib.es2015.d.ts"),
        "es2015" => Some("lib.es2015.d.ts"),
        "es7" => Some("lib.es2016.d.ts"),
        "es2016" => Some("lib.es2016.d.ts"),
        "es2017" => Some("lib.es2017.d.ts"),
        "es2018" => Some("lib.es2018.d.ts"),
        "es2019" => Some("lib.es2019.d.ts"),
        "es2020" => Some("lib.es2020.d.ts"),
        "es2021" => Some("lib.es2021.d.ts"),
        "es2022" => Some("lib.es2022.d.ts"),
        "es2023" => Some("lib.es2023.d.ts"),
        "es2024" => Some("lib.es2024.d.ts"),
        "es2025" => Some("lib.es2025.d.ts"),
        "esnext" => Some("lib.esnext.d.ts"),
        "dom" => Some("lib.dom.d.ts"),
        "dom.iterable" => Some("lib.dom.iterable.d.ts"),
        "dom.asynciterable" => Some("lib.dom.asynciterable.d.ts"),
        "webworker" => Some("lib.webworker.d.ts"),
        "webworker.importscripts" => Some("lib.webworker.importscripts.d.ts"),
        "webworker.iterable" => Some("lib.webworker.iterable.d.ts"),
        "webworker.asynciterable" => Some("lib.webworker.asynciterable.d.ts"),
        "scripthost" => Some("lib.scripthost.d.ts"),
        "es2015.core" => Some("lib.es2015.core.d.ts"),
        "es2015.collection" => Some("lib.es2015.collection.d.ts"),
        "es2015.generator" => Some("lib.es2015.generator.d.ts"),
        "es2015.iterable" => Some("lib.es2015.iterable.d.ts"),
        "es2015.promise" => Some("lib.es2015.promise.d.ts"),
        "es2015.proxy" => Some("lib.es2015.proxy.d.ts"),
        "es2015.reflect" => Some("lib.es2015.reflect.d.ts"),
        "es2015.symbol" => Some("lib.es2015.symbol.d.ts"),
        "es2015.symbol.wellknown" => Some("lib.es2015.symbol.wellknown.d.ts"),
        "es2016.array.include" => Some("lib.es2016.array.include.d.ts"),
        "es2016.intl" => Some("lib.es2016.intl.d.ts"),
        "es2017.arraybuffer" => Some("lib.es2017.arraybuffer.d.ts"),
        "es2017.date" => Some("lib.es2017.date.d.ts"),
        "es2017.object" => Some("lib.es2017.object.d.ts"),
        "es2017.sharedmemory" => Some("lib.es2017.sharedmemory.d.ts"),
        "es2017.string" => Some("lib.es2017.string.d.ts"),
        "es2017.intl" => Some("lib.es2017.intl.d.ts"),
        "es2017.typedarrays" => Some("lib.es2017.typedarrays.d.ts"),
        "es2018.asyncgenerator" => Some("lib.es2018.asyncgenerator.d.ts"),
        "es2018.asynciterable" => Some("lib.es2018.asynciterable.d.ts"),
        "es2018.intl" => Some("lib.es2018.intl.d.ts"),
        "es2018.promise" => Some("lib.es2018.promise.d.ts"),
        "es2018.regexp" => Some("lib.es2018.regexp.d.ts"),
        "es2019.array" => Some("lib.es2019.array.d.ts"),
        "es2019.object" => Some("lib.es2019.object.d.ts"),
        "es2019.string" => Some("lib.es2019.string.d.ts"),
        "es2019.symbol" => Some("lib.es2019.symbol.d.ts"),
        "es2019.intl" => Some("lib.es2019.intl.d.ts"),
        "es2020.bigint" => Some("lib.es2020.bigint.d.ts"),
        "es2020.date" => Some("lib.es2020.date.d.ts"),
        "es2020.promise" => Some("lib.es2020.promise.d.ts"),
        "es2020.sharedmemory" => Some("lib.es2020.sharedmemory.d.ts"),
        "es2020.string" => Some("lib.es2020.string.d.ts"),
        "es2020.symbol.wellknown" => Some("lib.es2020.symbol.wellknown.d.ts"),
        "es2020.intl" => Some("lib.es2020.intl.d.ts"),
        "es2020.number" => Some("lib.es2020.number.d.ts"),
        "es2021.promise" => Some("lib.es2021.promise.d.ts"),
        "es2021.string" => Some("lib.es2021.string.d.ts"),
        "es2021.weakref" => Some("lib.es2021.weakref.d.ts"),
        "es2021.intl" => Some("lib.es2021.intl.d.ts"),
        "es2022.array" => Some("lib.es2022.array.d.ts"),
        "es2022.error" => Some("lib.es2022.error.d.ts"),
        "es2022.intl" => Some("lib.es2022.intl.d.ts"),
        "es2022.object" => Some("lib.es2022.object.d.ts"),
        "es2022.string" => Some("lib.es2022.string.d.ts"),
        "es2022.regexp" => Some("lib.es2022.regexp.d.ts"),
        "es2023.array" => Some("lib.es2023.array.d.ts"),
        "es2023.collection" => Some("lib.es2023.collection.d.ts"),
        "es2023.intl" => Some("lib.es2023.intl.d.ts"),
        "es2024.arraybuffer" => Some("lib.es2024.arraybuffer.d.ts"),
        "es2024.collection" => Some("lib.es2024.collection.d.ts"),
        "es2024.object" => Some("lib.es2024.object.d.ts"),
        "es2024.promise" => Some("lib.es2024.promise.d.ts"),
        "es2024.regexp" => Some("lib.es2024.regexp.d.ts"),
        "es2024.sharedmemory" => Some("lib.es2024.sharedmemory.d.ts"),
        "es2024.string" => Some("lib.es2024.string.d.ts"),
        "es2025.collection" => Some("lib.es2025.collection.d.ts"),
        "es2025.float16" => Some("lib.es2025.float16.d.ts"),
        "es2025.intl" => Some("lib.es2025.intl.d.ts"),
        "es2025.iterator" => Some("lib.es2025.iterator.d.ts"),
        "es2025.promise" => Some("lib.es2025.promise.d.ts"),
        "es2025.regexp" => Some("lib.es2025.regexp.d.ts"),
        "esnext.asynciterable" => Some("lib.es2018.asynciterable.d.ts"),
        "esnext.symbol" => Some("lib.es2019.symbol.d.ts"),
        "esnext.bigint" => Some("lib.es2020.bigint.d.ts"),
        "esnext.weakref" => Some("lib.es2021.weakref.d.ts"),
        "esnext.object" => Some("lib.es2024.object.d.ts"),
        "esnext.regexp" => Some("lib.es2024.regexp.d.ts"),
        "esnext.string" => Some("lib.es2024.string.d.ts"),
        "esnext.float16" => Some("lib.es2025.float16.d.ts"),
        "esnext.iterator" => Some("lib.es2025.iterator.d.ts"),
        "esnext.promise" => Some("lib.es2025.promise.d.ts"),
        "esnext.array" => Some("lib.esnext.array.d.ts"),
        "esnext.collection" => Some("lib.esnext.collection.d.ts"),
        "esnext.date" => Some("lib.esnext.date.d.ts"),
        "esnext.decorators" => Some("lib.esnext.decorators.d.ts"),
        "esnext.disposable" => Some("lib.esnext.disposable.d.ts"),
        "esnext.error" => Some("lib.esnext.error.d.ts"),
        "esnext.intl" => Some("lib.esnext.intl.d.ts"),
        "esnext.sharedmemory" => Some("lib.esnext.sharedmemory.d.ts"),
        "esnext.temporal" => Some("lib.esnext.temporal.d.ts"),
        "esnext.typedarrays" => Some("lib.esnext.typedarrays.d.ts"),
        "decorators" => Some("lib.decorators.d.ts"),
        "decorators.legacy" => Some("lib.decorators.legacy.d.ts"),
        "lib.es5.d.ts" => Some("lib.es5.d.ts"),
        "lib.es2015.d.ts" => Some("lib.es2015.d.ts"),
        "lib.es2016.d.ts" => Some("lib.es2016.d.ts"),
        "lib.es2017.d.ts" => Some("lib.es2017.d.ts"),
        "lib.es2018.d.ts" => Some("lib.es2018.d.ts"),
        "lib.es2019.d.ts" => Some("lib.es2019.d.ts"),
        "lib.es2020.d.ts" => Some("lib.es2020.d.ts"),
        "lib.es2021.d.ts" => Some("lib.es2021.d.ts"),
        "lib.es2022.d.ts" => Some("lib.es2022.d.ts"),
        "lib.es2023.d.ts" => Some("lib.es2023.d.ts"),
        "lib.es2024.d.ts" => Some("lib.es2024.d.ts"),
        "lib.es2025.d.ts" => Some("lib.es2025.d.ts"),
        "lib.esnext.d.ts" => Some("lib.esnext.d.ts"),
        "lib.dom.d.ts" => Some("lib.dom.d.ts"),
        "lib.dom.iterable.d.ts" => Some("lib.dom.iterable.d.ts"),
        "lib.dom.asynciterable.d.ts" => Some("lib.dom.asynciterable.d.ts"),
        "lib.webworker.d.ts" => Some("lib.webworker.d.ts"),
        "lib.webworker.importscripts.d.ts" => Some("lib.webworker.importscripts.d.ts"),
        "lib.webworker.iterable.d.ts" => Some("lib.webworker.iterable.d.ts"),
        "lib.webworker.asynciterable.d.ts" => Some("lib.webworker.asynciterable.d.ts"),
        "lib.scripthost.d.ts" => Some("lib.scripthost.d.ts"),
        "lib.es2015.core.d.ts" => Some("lib.es2015.core.d.ts"),
        "lib.es2015.collection.d.ts" => Some("lib.es2015.collection.d.ts"),
        "lib.es2015.generator.d.ts" => Some("lib.es2015.generator.d.ts"),
        "lib.es2015.iterable.d.ts" => Some("lib.es2015.iterable.d.ts"),
        "lib.es2015.promise.d.ts" => Some("lib.es2015.promise.d.ts"),
        "lib.es2015.proxy.d.ts" => Some("lib.es2015.proxy.d.ts"),
        "lib.es2015.reflect.d.ts" => Some("lib.es2015.reflect.d.ts"),
        "lib.es2015.symbol.d.ts" => Some("lib.es2015.symbol.d.ts"),
        "lib.es2015.symbol.wellknown.d.ts" => Some("lib.es2015.symbol.wellknown.d.ts"),
        "lib.es2016.array.include.d.ts" => Some("lib.es2016.array.include.d.ts"),
        "lib.es2016.intl.d.ts" => Some("lib.es2016.intl.d.ts"),
        "lib.es2017.arraybuffer.d.ts" => Some("lib.es2017.arraybuffer.d.ts"),
        "lib.es2017.date.d.ts" => Some("lib.es2017.date.d.ts"),
        "lib.es2017.object.d.ts" => Some("lib.es2017.object.d.ts"),
        "lib.es2017.sharedmemory.d.ts" => Some("lib.es2017.sharedmemory.d.ts"),
        "lib.es2017.string.d.ts" => Some("lib.es2017.string.d.ts"),
        "lib.es2017.intl.d.ts" => Some("lib.es2017.intl.d.ts"),
        "lib.es2017.typedarrays.d.ts" => Some("lib.es2017.typedarrays.d.ts"),
        "lib.es2018.asyncgenerator.d.ts" => Some("lib.es2018.asyncgenerator.d.ts"),
        "lib.es2018.asynciterable.d.ts" => Some("lib.es2018.asynciterable.d.ts"),
        "lib.es2018.intl.d.ts" => Some("lib.es2018.intl.d.ts"),
        "lib.es2018.promise.d.ts" => Some("lib.es2018.promise.d.ts"),
        "lib.es2018.regexp.d.ts" => Some("lib.es2018.regexp.d.ts"),
        "lib.es2019.array.d.ts" => Some("lib.es2019.array.d.ts"),
        "lib.es2019.object.d.ts" => Some("lib.es2019.object.d.ts"),
        "lib.es2019.string.d.ts" => Some("lib.es2019.string.d.ts"),
        "lib.es2019.symbol.d.ts" => Some("lib.es2019.symbol.d.ts"),
        "lib.es2019.intl.d.ts" => Some("lib.es2019.intl.d.ts"),
        "lib.es2020.bigint.d.ts" => Some("lib.es2020.bigint.d.ts"),
        "lib.es2020.date.d.ts" => Some("lib.es2020.date.d.ts"),
        "lib.es2020.promise.d.ts" => Some("lib.es2020.promise.d.ts"),
        "lib.es2020.sharedmemory.d.ts" => Some("lib.es2020.sharedmemory.d.ts"),
        "lib.es2020.string.d.ts" => Some("lib.es2020.string.d.ts"),
        "lib.es2020.symbol.wellknown.d.ts" => Some("lib.es2020.symbol.wellknown.d.ts"),
        "lib.es2020.intl.d.ts" => Some("lib.es2020.intl.d.ts"),
        "lib.es2020.number.d.ts" => Some("lib.es2020.number.d.ts"),
        "lib.es2021.promise.d.ts" => Some("lib.es2021.promise.d.ts"),
        "lib.es2021.string.d.ts" => Some("lib.es2021.string.d.ts"),
        "lib.es2021.weakref.d.ts" => Some("lib.es2021.weakref.d.ts"),
        "lib.es2021.intl.d.ts" => Some("lib.es2021.intl.d.ts"),
        "lib.es2022.array.d.ts" => Some("lib.es2022.array.d.ts"),
        "lib.es2022.error.d.ts" => Some("lib.es2022.error.d.ts"),
        "lib.es2022.intl.d.ts" => Some("lib.es2022.intl.d.ts"),
        "lib.es2022.object.d.ts" => Some("lib.es2022.object.d.ts"),
        "lib.es2022.string.d.ts" => Some("lib.es2022.string.d.ts"),
        "lib.es2022.regexp.d.ts" => Some("lib.es2022.regexp.d.ts"),
        "lib.es2023.array.d.ts" => Some("lib.es2023.array.d.ts"),
        "lib.es2023.collection.d.ts" => Some("lib.es2023.collection.d.ts"),
        "lib.es2023.intl.d.ts" => Some("lib.es2023.intl.d.ts"),
        "lib.es2024.arraybuffer.d.ts" => Some("lib.es2024.arraybuffer.d.ts"),
        "lib.es2024.collection.d.ts" => Some("lib.es2024.collection.d.ts"),
        "lib.es2024.object.d.ts" => Some("lib.es2024.object.d.ts"),
        "lib.es2024.promise.d.ts" => Some("lib.es2024.promise.d.ts"),
        "lib.es2024.regexp.d.ts" => Some("lib.es2024.regexp.d.ts"),
        "lib.es2024.sharedmemory.d.ts" => Some("lib.es2024.sharedmemory.d.ts"),
        "lib.es2024.string.d.ts" => Some("lib.es2024.string.d.ts"),
        "lib.es2025.collection.d.ts" => Some("lib.es2025.collection.d.ts"),
        "lib.es2025.float16.d.ts" => Some("lib.es2025.float16.d.ts"),
        "lib.es2025.intl.d.ts" => Some("lib.es2025.intl.d.ts"),
        "lib.es2025.iterator.d.ts" => Some("lib.es2025.iterator.d.ts"),
        "lib.es2025.promise.d.ts" => Some("lib.es2025.promise.d.ts"),
        "lib.es2025.regexp.d.ts" => Some("lib.es2025.regexp.d.ts"),
        "lib.esnext.array.d.ts" => Some("lib.esnext.array.d.ts"),
        "lib.esnext.collection.d.ts" => Some("lib.esnext.collection.d.ts"),
        "lib.esnext.date.d.ts" => Some("lib.esnext.date.d.ts"),
        "lib.esnext.decorators.d.ts" => Some("lib.esnext.decorators.d.ts"),
        "lib.esnext.disposable.d.ts" => Some("lib.esnext.disposable.d.ts"),
        "lib.esnext.error.d.ts" => Some("lib.esnext.error.d.ts"),
        "lib.esnext.intl.d.ts" => Some("lib.esnext.intl.d.ts"),
        "lib.esnext.sharedmemory.d.ts" => Some("lib.esnext.sharedmemory.d.ts"),
        "lib.esnext.temporal.d.ts" => Some("lib.esnext.temporal.d.ts"),
        "lib.esnext.typedarrays.d.ts" => Some("lib.esnext.typedarrays.d.ts"),
        "lib.decorators.d.ts" => Some("lib.decorators.d.ts"),
        "lib.decorators.legacy.d.ts" => Some("lib.decorators.legacy.d.ts"),
        _ => None,
    }
}
