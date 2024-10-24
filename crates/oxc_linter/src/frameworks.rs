use std::{hash, path::Path};

use bitflags::bitflags;
use oxc_semantic::ModuleRecord;

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
pub(crate) fn is_jestlike_file(path: &Path) -> bool {
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

pub(crate) fn has_vitest_imports(module_record: &ModuleRecord) -> bool {
    module_record.import_entries.iter().any(|entry| entry.module_request.name() == "vitest")
}

pub(crate) fn has_jest_imports(module_record: &ModuleRecord) -> bool {
    module_record.import_entries.iter().any(|entry| entry.module_request.name() == "@jest/globals")
}
