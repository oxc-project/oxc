use std::{
    hash::BuildHasherDefault,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use oxc_resolver::Resolver;
use rustc_hash::FxHasher;
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum PackageJsonError {
    Read,
    Parse(String),
}

type PackageJsonResult = Result<Arc<Value>, PackageJsonError>;

#[derive(Clone, Copy)]
pub enum ImportKind {
    Import,
    Require,
    Type,
}

/// Import-rule state shared across files. Manifest contents are cached for one lint run.
#[derive(Debug)]
pub struct ImportContext {
    resolver: Arc<Resolver>,
    import_resolver: OnceLock<Resolver>,
    require_resolver: OnceLock<Resolver>,
    type_resolver: OnceLock<Resolver>,
    packages: papaya::HashMap<PathBuf, OnceLock<PackageJsonResult>, BuildHasherDefault<FxHasher>>,
}

impl ImportContext {
    pub fn new(resolver: Arc<Resolver>) -> Self {
        Self {
            resolver,
            import_resolver: OnceLock::new(),
            require_resolver: OnceLock::new(),
            type_resolver: OnceLock::new(),
            packages: papaya::HashMap::default(),
        }
    }

    pub(crate) fn resolver(&self, kind: ImportKind) -> &Resolver {
        let resolver = match kind {
            ImportKind::Import => &self.import_resolver,
            ImportKind::Require => &self.require_resolver,
            ImportKind::Type => &self.type_resolver,
        };
        resolver.get_or_init(|| {
            let mut options = self.resolver.options().clone();
            if matches!(kind, ImportKind::Require) {
                options.condition_names = vec!["require".into(), "node".into()];
            } else if !options.condition_names.iter().any(|condition| condition == "node") {
                options.condition_names.push("node".into());
            }
            if matches!(kind, ImportKind::Type) {
                options.condition_names.push("types".into());
            }
            self.resolver.clone_with_options(options)
        })
    }

    pub fn package_json(&self, path: &Path) -> PackageJsonResult {
        let packages = self.packages.pin();
        let package = packages
            .get(path)
            .unwrap_or_else(|| packages.get_or_insert_with(path.to_path_buf(), OnceLock::new));
        package
            .get_or_init(|| {
                let source =
                    crate::utils::read_to_string(path).map_err(|_| PackageJsonError::Read)?;
                serde_json::from_str(&source)
                    .map(Arc::new)
                    .map_err(|error| PackageJsonError::Parse(error.to_string()))
            })
            .clone()
    }

    pub fn clear_package_json_cache(&self) {
        self.packages.pin().clear();
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsStr, fs, sync::Arc, time::SystemTime};

    use rustc_hash::FxHashMap;

    use crate::{
        AllowWarnDeny, ConfigStore, ConfigStoreBuilder, ExternalPluginStore, LintOptions,
        LintPlugins, LintService, LintServiceOptions, Linter, OsFileSystem,
        rules::{ImportNoExtraneousDependencies, RuleEnum},
    };

    #[test]
    fn manifest_changes_between_runs() {
        let root = std::env::temp_dir().join(format!(
            "oxc-dependency-cache-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(),
        ));
        fs::create_dir_all(root.join("node_modules/example")).unwrap();
        let root = root.canonicalize().unwrap();
        fs::write(
            root.join("node_modules/example/package.json"),
            r#"{"name":"example","main":"index.js"}"#,
        )
        .unwrap();
        fs::write(root.join("node_modules/example/index.js"), "export const value = 1;").unwrap();

        let mut external_plugins = ExternalPluginStore::new(false);
        let config = ConfigStoreBuilder::empty()
            .with_builtin_plugins(LintPlugins::IMPORT)
            .with_rule(
                RuleEnum::ImportNoExtraneousDependencies(ImportNoExtraneousDependencies::default()),
                AllowWarnDeny::Warn,
            )
            .build(&mut external_plugins)
            .unwrap();
        let linter = Linter::new(
            LintOptions::default(),
            ConfigStore::new(config, FxHashMap::default(), external_plugins),
            None,
        );
        let service =
            LintService::new(linter, LintServiceOptions::new(root.clone()).with_cross_module(true));
        for (run, package, expected) in [
            (0, r#"{"name":"consumer"}"#, 2),
            (1, r#"{"name":"consumer","dependencies":{"example":"*"}}"#, 0),
            (2, r#"{"name":"consumer"}"#, 2),
        ] {
            fs::write(root.join("package.json"), package).unwrap();
            let paths = (0..2)
                .map(|file| {
                    let path = root.join(format!("input-{run}-{file}.js"));
                    fs::write(&path, "import 'example';").unwrap();
                    Arc::<OsStr>::from(path.as_os_str())
                })
                .collect();
            let messages = service.run_source(&OsFileSystem, paths);
            assert_eq!(messages.len(), expected, "run {run}");
            assert!(
                messages.iter().all(|message| message.error.message.contains("Package `example`"))
            );
        }
        fs::remove_dir_all(root).unwrap();
    }
}
