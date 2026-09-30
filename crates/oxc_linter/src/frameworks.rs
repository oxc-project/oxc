use std::hash;
#[cfg(not(test))]
use std::path::Path;

use bitflags::bitflags;

use crate::{ModuleRecord, OxlintSettings};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FrameworkFlags: u32 {
        // front-end frameworks

        /// Uses [React](https://reactjs.org/).
        ///
        /// May be part of a meta-framework like Next.js.
        const React = 1 << 0;
        /// Uses [Preact](https://preactjs.com/).
        const Preact = 1 << 1;
        /// Uses [Next.js](https://nextjs.org/).
        const NextOnly = 1 << 2;
        const Next = Self::NextOnly.bits() | Self::React.bits();
        const JsxLike = Self::React.bits() | Self::Preact.bits() | Self::Next.bits();

        const Vue = 1 << 3;
        const NuxtOnly = 1 << 4;
        const Nuxt = Self::NuxtOnly.bits() | Self::Vue.bits();

        const Angular = 1 << 5;

        const Svelte = 1 << 6;
        const SvelteKitOnly = 1 << 7;
        const SvelteKit = Self::SvelteKitOnly.bits() | Self::Svelte.bits();

        const Astro = 1 << 8;

        // Testing frameworks
        const Jest = 1 << 9;
        const Vitest = 1 << 10;
        const OtherTest = 1 << 11;
        /// Flag for if any test frameworks are used, such as Jest or Vitest.
        const Test = Self::Jest.bits() | Self::Vitest.bits() | Self::OtherTest.bits();
    }
}

impl Default for FrameworkFlags {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}
impl hash::Hash for FrameworkFlags {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.bits());
    }
}

impl FrameworkFlags {
    #[inline]
    pub const fn is_test(self) -> bool {
        self.intersects(Self::Test)
    }

    #[inline]
    pub const fn is_vitest(self) -> bool {
        self.contains(Self::Vitest)
    }

    #[inline]
    pub const fn is_jest(self) -> bool {
        self.contains(Self::Jest)
    }
}

/// <https://jestjs.io/docs/configuration#testmatch-arraystring>
#[cfg(not(test))]
pub fn is_jestlike_file(path: &Path) -> bool {
    use std::ffi::OsStr;

    if path.components().any(|c| match c {
        std::path::Component::Normal(p) => p == OsStr::new("__tests__"),
        _ => false,
    }) {
        return true;
    }

    path.file_name() // foo/bar/baz.test.ts -> baz.test.ts
        .and_then(OsStr::to_str)
        .and_then(|filename| filename.split('.').rev().nth(1)) // baz.test.ts -> test
        .is_some_and(|name_or_first_ext| name_or_first_ext == "test" || name_or_first_ext == "spec")
}

pub fn has_vitest_imports(module_record: &ModuleRecord, settings: &OxlintSettings) -> bool {
    module_record
        .import_entries
        .iter()
        .any(|entry| settings.vitest.is_vitest_import_source(entry.module_request.name()))
}

#[cfg(not(test))]
pub fn has_jest_imports(module_record: &ModuleRecord) -> bool {
    module_record.import_entries.iter().any(|entry| entry.module_request.name() == "@jest/globals")
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]

pub enum FrameworkOptions {
    Default,  // default
    VueSetup, // context is inside `<script setup>`
}

#[cfg(test)]
mod test {
    use serde::Deserialize;
    use serde_json::json;

    use oxc_span::Span;

    use super::has_vitest_imports;
    use crate::{
        OxlintSettings,
        module_record::{ImportEntry, ImportImportName, ModuleRecord, NameSpan},
    };

    fn import_entry(module_request: &str) -> ImportEntry {
        let span = Span::new(0, 0);
        ImportEntry {
            statement_span: span,
            module_request: NameSpan::new(module_request.into(), span),
            import_name: ImportImportName::Name(NameSpan::new("test".into(), span)),
            local_name: NameSpan::new("test".into(), span),
            is_type: false,
        }
    }

    fn settings_with_vitest_imports(module: &str) -> OxlintSettings {
        OxlintSettings::deserialize(json!({
            "vitest": { "vitestImports": [module] }
        }))
        .unwrap()
    }

    #[test]
    fn test_has_vitest_imports() {
        let default_settings = OxlintSettings::default();

        // Built-in sources are recognized with default settings.
        for source in ["vitest", "vite-plus/test", "@effect/vitest"] {
            let mut module_record = ModuleRecord::default();
            module_record.import_entries.push(import_entry(source));
            assert!(has_vitest_imports(&module_record, &default_settings));
        }

        // A custom fixture source is only recognized when configured via `vitestImports`.
        let mut custom_module = ModuleRecord::default();
        custom_module.import_entries.push(import_entry("$test/setup/fixtures"));
        assert!(!has_vitest_imports(&custom_module, &default_settings));
        let custom_settings = settings_with_vitest_imports("$test/setup/fixtures");
        assert!(has_vitest_imports(&custom_module, &custom_settings));
    }
}
