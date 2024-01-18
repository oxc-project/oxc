use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TypescriptOptions {
    /// When set to true, the transform will only remove [type-only](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-8.html#type-only-imports-exports) imports (introduced in TypeScript 3.8). This should only be used if you are using TypeScript >= 3.8.
    /// defaults to false
    pub only_remove_type_imports: bool,
}
