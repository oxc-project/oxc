use std::{hash, path::Path};

use bitflags::bitflags;

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

pub fn has_vitest_imports(module_record: &ModuleRecord) -> bool {
    module_record.import_entries.iter().any(|entry| entry.module_request.name() == "vitest")
}

pub fn has_jest_imports(module_record: &ModuleRecord) -> bool {
    module_record.import_entries.iter().any(|entry| entry.module_request.name() == "@jest/globals")
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]

pub enum FrameworkOptions {
    Default,          // default
    VueSetup,         // context is inside `<script setup>`
    SvelteModule,     // `<script module>`
    Svelte,           // `<script>`
    AstroFrontmatter, // within `---`-delimited block
}

/// Vue 3 compiler macros available in `<script setup>`
/// Reference: <https://github.com/vuejs/vue-eslint-parser/blob/5ff1a4fda76b07608cc17687a976c2309f5648e2/src/script-setup/scope-analyzer.ts#L86>
static VUE_SETUP_GLOBALS: [&str; 7] = [
    "defineProps",
    "defineEmits",
    "defineExpose",
    "withDefaults",
    "defineOptions",
    "defineSlots",
    "defineModel",
];

/// Svelte runes available in `<script>` context
/// Reference: <https://github.com/sveltejs/svelte/blob/da00abe1162a8e56455e92b79020c4e33290e10e/packages/svelte/src/ambient.d.ts#L23>
static SVELTE_GLOBALS: [&str; 7] =
    ["$state", "$derived", "$effect", "$props", "$bindable", "$inspect", "$host"];

/// A subset of Svelte runes is available in `<script module>` context.
static SVELTE_MODULE_GLOBALS: [&str; 4] = ["$state", "$derived", "$effect", "$inspect"];

impl FrameworkOptions {
    /// Check if a variable is a framework-specific global in this context.
    ///
    /// Returns `true` if the variable is a framework global, `false` otherwise.
    /// Framework globals are always readonly, as they are compiler macros or
    /// special identifiers provided by the framework.
    ///
    /// # Examples
    /// ```
    /// // In a Vue <script setup> context
    /// let options = FrameworkOptions::VueSetup;
    /// assert!(options.has_global("defineProps") == true);
    /// assert!(options.has_global("defineEmits") == true);
    /// assert!(options.has_global("console") == false);
    ///
    /// // No framework globals by default
    /// let options = FrameworkOptions::Default;
    /// assert!(options.has_global("defineProps") == false);
    /// assert!(options.has_global("console") == false);
    /// ```
    pub fn has_global(self, var: &str) -> bool {
        match self {
            Self::Default => false,
            Self::VueSetup => VUE_SETUP_GLOBALS.contains(&var),
            Self::SvelteModule => SVELTE_MODULE_GLOBALS.contains(&var),
            Self::Svelte => SVELTE_GLOBALS.contains(&var),
            // All of Astro's utilities are grouped under the `Astro` namespace.
            Self::AstroFrontmatter => var == "Astro",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vue_setup_globals() {
        // Test all Vue compiler macros
        let options = FrameworkOptions::VueSetup;
        assert!(options.has_global("defineProps"));
        assert!(options.has_global("defineEmits"));
        assert!(options.has_global("defineExpose"));
        assert!(options.has_global("defineOptions"));
        assert!(options.has_global("defineSlots"));
        assert!(options.has_global("defineModel"));
        assert!(options.has_global("withDefaults"));

        // Test that non-Vue globals are not included
        assert!(!options.has_global("console"));
        assert!(!options.has_global("window"));
        assert!(!options.has_global("randomVariable"));
    }

    #[test]
    fn test_svelte_globals() {
        let options = FrameworkOptions::Svelte;
        assert!(options.has_global("$state"));
        assert!(options.has_global("$derived"));
        assert!(options.has_global("$effect"));
        assert!(options.has_global("$props"));
        assert!(options.has_global("$bindable"));
        assert!(options.has_global("$inspect"));
        assert!(options.has_global("$host"));

        let globals = FrameworkOptions::SvelteModule;
        assert!(globals.has_global("$state"));
        assert!(globals.has_global("$derived"));
        assert!(globals.has_global("$effect"));
        assert!(globals.has_global("$inspect"));
        assert!(!globals.has_global("$props"));
        assert!(!globals.has_global("$bindable"));
        assert!(!globals.has_global("$host"));
    }

    #[test]
    fn test_astro_frontmatter_globals() {
        let options = FrameworkOptions::AstroFrontmatter;
        assert!(options.has_global("Astro"));
    }

    #[test]
    fn test_default_no_globals() {
        // Default context has no framework globals
        let options = FrameworkOptions::Default;
        assert!(!options.has_global("defineProps"));
        assert!(!options.has_global("console"));
    }
}
