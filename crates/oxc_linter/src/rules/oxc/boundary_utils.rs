use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::context::LintContext;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub(super) struct BoundaryElementSetting {
    #[serde(rename = "type")]
    pub(super) element_type: String,
    pub(super) pattern: BoundaryPatterns,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(untagged)]
pub(super) enum BoundaryPatterns {
    Single(String),
    Many(Vec<String>),
    #[default]
    Empty,
}

impl BoundaryPatterns {
    pub(super) fn iter(&self) -> impl Iterator<Item = &str> {
        let patterns: Vec<&str> = match self {
            Self::Single(single) => vec![single.as_str()],
            Self::Many(many) => many.iter().map(String::as_str).collect(),
            Self::Empty => vec![],
        };
        patterns.into_iter()
    }
}

pub(super) fn read_boundary_elements(ctx: &LintContext<'_>) -> Option<Vec<BoundaryElementSetting>> {
    let raw_settings = ctx.settings().json.as_ref()?;
    let raw_elements = raw_settings.get("boundaries/elements")?.clone();
    serde_json::from_value(raw_elements).ok()
}

pub(super) fn classify_path(path: &Path, elements: &[BoundaryElementSetting]) -> Option<String> {
    let suffixes = normalized_path_suffixes(path);
    for element in elements {
        if element
            .pattern
            .iter()
            .any(|pattern| suffixes.iter().any(|suffix| fast_glob::glob_match(pattern, suffix)))
        {
            return Some(element.element_type.clone());
        }
    }
    None
}

pub(super) fn resolve_local_specifier(base_file: &Path, specifier: &str) -> Option<PathBuf> {
    if !specifier.starts_with('.') && !specifier.starts_with('/') {
        return None;
    }

    let candidate = if specifier.starts_with('/') {
        PathBuf::from(specifier)
    } else {
        let parent = base_file.parent()?;
        parent.join(specifier)
    };

    let candidate = normalize_path(candidate);

    resolve_existing_module_path(&candidate)
}

fn resolve_existing_module_path(candidate: &Path) -> Option<PathBuf> {
    if candidate.is_file() {
        return Some(candidate.to_path_buf());
    }

    const EXTENSIONS: [&str; 8] = ["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"];

    if candidate.extension().is_none() {
        for extension in EXTENSIONS {
            let with_extension = candidate.with_extension(extension);
            if with_extension.is_file() {
                return Some(with_extension);
            }
        }
    }

    if candidate.is_dir() {
        for extension in EXTENSIONS {
            let index_path = candidate.join(format!("index.{extension}"));
            if index_path.is_file() {
                return Some(index_path);
            }
        }
    }

    None
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }

    normalized
}

fn normalized_path_suffixes(path: &Path) -> Vec<String> {
    let components = path
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>();

    (0..components.len()).map(|index| components[index..].join("/")).collect()
}
