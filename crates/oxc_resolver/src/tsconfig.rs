use std::{
    hash::BuildHasherDefault,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{json_comments::strip_comments_in_place, PathUtil};
use indexmap::IndexMap;
use rustc_hash::FxHasher;
use serde::Deserialize;

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TsConfig {
    /// Path to `tsconfig.json`. Contains the `tsconfig.json` filename.
    #[serde(skip)]
    path: PathBuf,

    #[serde(default, deserialize_with = "deserialize_extends")]
    extends: Vec<String>,

    #[serde(default)]
    references: Vec<ProjectReference>,

    #[serde(default)]
    compiler_options: CompilerOptions,
}

/// Project Reference
/// <https://www.typescriptlang.org/docs/handbook/project-references.html>
#[derive(Debug, Deserialize)]
pub struct ProjectReference {
    /// The path property of each reference can point to a directory containing a tsconfig.json file,
    /// or to the config file itself (which may have any name).
    pub path: PathBuf,

    /// Reference to the resolved tsconfig
    #[serde(skip)]
    pub tsconfig: Option<Arc<TsConfig>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilerOptions {
    base_url: Option<PathBuf>,
    paths: Option<FxIndexMap<String, Vec<String>>>,
    #[serde(skip)]
    paths_base: PathBuf,
}

fn deserialize_extends<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum StringOrArray {
        String(String),
        Array(Vec<String>),
    }
    Ok(match StringOrArray::deserialize(deserializer)? {
        StringOrArray::String(s) => vec![s],
        StringOrArray::Array(a) => a,
    })
}

impl TsConfig {
    pub fn parse(path: &Path, json: &mut str) -> Result<Self, serde_json::Error> {
        _ = strip_comments_in_place(json);
        let mut tsconfig: Self = serde_json::from_str(json)?;
        tsconfig.path = path.to_path_buf();
        let directory = tsconfig.directory().to_path_buf();
        if let Some(base_url) = tsconfig.compiler_options.base_url {
            tsconfig.compiler_options.base_url = Some(directory.normalize_with(base_url));
        }
        if tsconfig.compiler_options.paths.is_some() {
            tsconfig.compiler_options.paths_base =
                tsconfig.compiler_options.base_url.as_ref().map_or(directory, Clone::clone);
        }
        Ok(tsconfig)
    }

    /// Directory to `package.json`
    ///
    /// # Panics
    ///
    /// * When the package.json path is misconfigured.
    pub fn directory(&self) -> &Path {
        debug_assert!(self.path.file_name().is_some());
        self.path.parent().unwrap()
    }

    pub fn extends(&self) -> &Vec<String> {
        &self.extends
    }

    pub fn references_mut(&mut self) -> &mut Vec<ProjectReference> {
        self.references.as_mut()
    }

    fn base_path(&self) -> &Path {
        self.compiler_options
            .base_url
            .as_ref()
            .map_or_else(|| self.directory(), |path| path.as_ref())
    }

    pub fn extend_tsconfig(&mut self, tsconfig: &Self) {
        let compiler_options = &mut self.compiler_options;
        if compiler_options.base_url.is_none() {
            compiler_options.base_url = tsconfig.compiler_options.base_url.clone();
        }
        if compiler_options.paths.is_none() {
            compiler_options.paths_base = tsconfig.compiler_options.paths_base.clone();
            compiler_options.paths = tsconfig.compiler_options.paths.clone();
        }
    }

    pub fn resolve(&self, path: &Path, specifier: &str) -> Vec<PathBuf> {
        if path.starts_with(self.base_path()) {
            return self.resolve_path_alias(specifier);
        }
        for reference in &self.references {
            if let Some(tsconfig) = &reference.tsconfig {
                if path.starts_with(tsconfig.base_path()) {
                    return tsconfig.resolve_path_alias(specifier);
                }
            }
        }
        vec![]
    }

    // Copied from parcel
    // <https://github.com/parcel-bundler/parcel/blob/b6224fd519f95e68d8b93ba90376fd94c8b76e69/packages/utils/node-resolver-rs/src/tsconfig.rs#L93>
    pub fn resolve_path_alias(&self, specifier: &str) -> Vec<PathBuf> {
        if specifier.starts_with(|s| s == '/' || s == '.') {
            return vec![];
        }

        let base_url_iter = self
            .compiler_options
            .base_url
            .as_ref()
            .map_or_else(Vec::new, |base_url| vec![base_url.normalize_with(specifier)]);

        let Some(paths_map) = &self.compiler_options.paths else { return base_url_iter };

        let paths = paths_map.get(specifier).map_or_else(
            || {
                let mut longest_prefix_length = 0;
                let mut longest_suffix_length = 0;
                let mut best_key: Option<&String> = None;

                for key in paths_map.keys() {
                    if let Some((prefix, suffix)) = key.split_once('*') {
                        if (best_key.is_none() || prefix.len() > longest_prefix_length)
                            && specifier.starts_with(prefix)
                            && specifier.ends_with(suffix)
                        {
                            longest_prefix_length = prefix.len();
                            longest_suffix_length = suffix.len();
                            best_key.replace(key);
                        }
                    }
                }

                best_key.and_then(|key| paths_map.get(key)).map_or_else(Vec::new, |paths| {
                    paths
                        .iter()
                        .map(|path| {
                            path.replace(
                                '*',
                                &specifier[longest_prefix_length
                                    ..specifier.len() - longest_suffix_length],
                            )
                        })
                        .collect::<Vec<_>>()
                })
            },
            Clone::clone,
        );

        paths
            .into_iter()
            .map(|p| self.compiler_options.paths_base.normalize_with(p))
            .chain(base_url_iter)
            .collect()
    }
}
