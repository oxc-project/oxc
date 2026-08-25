use std::{hash, path::Path};

use bitflags::bitflags;

#[cfg(not(test))]
use crate::ModuleRecord;

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
///
/// Also matches hyphenated markers like NestJS' `app.e2e-spec.ts`, which projects configure
/// through `testRegex` rather than `testMatch`.
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
        .is_some_and(is_test_file_name)
}

fn is_test_file_name(filename: &str) -> bool {
    let mut segments = filename.rsplit('.').skip(1); // baz.e2e-spec.ts -> e2e-spec, baz
    match segments.next() {
        Some("test" | "spec") => true,
        // Mirrors the `(\.|/)` Jest's default `testRegex` requires in front of the marker.
        Some(segment) => {
            (segment.ends_with("-test") || segment.ends_with("-spec")) && segments.next().is_some()
        }
        None => false,
    }
}

#[cfg(not(test))]
pub fn has_vitest_imports(module_record: &ModuleRecord) -> bool {
    module_record.import_entries.iter().any(|entry| {
        let name = entry.module_request.name();
        name == "vitest" || name == "vite-plus/test" || name == "@effect/vitest"
    })
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
    use std::path::Path;

    use super::is_jestlike_file;

    #[test]
    fn test_is_jestlike_file() {
        let cases = [
            ("foo.spec.ts", true),
            ("foo.test.tsx", true),
            ("foo.spec.mjs", true),
            ("foo.test.cts", true),
            ("foo.e2e.spec.ts", true),
            ("src/nested/foo.spec.ts", true),
            ("foo.ts", false),
            ("foo.d.ts", false),
            ("foo", false),
            ("foo.e2e-spec.ts", true),
            ("foo.int-spec.ts", true),
            ("foo.e2e-aws-spec.ts", true),
            ("foo.integration-test.ts", true),
            ("foo.myspec.ts", false),
            ("foo.latest.ts", false),
            ("foo.spec-e2e.ts", false),
            ("spec.ts", true),
            ("test.ts", true),
            ("ab-test.ts", false),
            ("unit-spec.ts", false),
            ("homepage.ab-test.ts", true),
            ("__tests__/foo.ts", true),
            ("src/__tests__/nested/foo.ts", true),
            ("src/nested/foo.ts", false),
        ];

        for (path, expected) in cases {
            assert_eq!(is_jestlike_file(Path::new(path)), expected, "path: {path}");
        }
    }
}
