use std::{path::Path, sync::Arc};

use editorconfig_parser::EditorConfig;

#[cfg(feature = "napi")]
use super::js_config::JsConfigLoaderCb;
use super::{
    ConfigResolver, NestedConfigCtx,
    editorconfig::{load_editorconfig, resolve_editorconfig_path},
};

/// Root config and on-demand nested configs for a project rooted at `cwd`.
///
/// Holds the state shared by every file resolved under the same entry point run,
/// so each config file (and `.editorconfig`) is loaded at most once.
///
/// Cloning is shallow, clones share the same caches.
#[derive(Clone)]
pub struct ConfigScopes {
    root: Arc<ConfigResolver>,
    /// Always present, even when nested detection is disabled,
    /// since the walker also uses it to recognize config files.
    nested_ctx: NestedConfigCtx,
    use_nested: bool,
    has_editorconfig: bool,
}

impl ConfigScopes {
    /// Load `.editorconfig` nearest to `cwd` and the root config,
    /// discovered upwards from `cwd` or given by `explicit_config`.
    ///
    /// # Errors
    /// Returns a message ready to print if loading or validation fails.
    pub fn load(
        cwd: &Path,
        explicit_config: Option<&Path>,
        use_nested: bool,
        #[cfg(feature = "napi")] js_config_loader: Option<&JsConfigLoaderCb>,
    ) -> Result<Self, String> {
        let load_err = |err: String| format!("Failed to load configuration file.\n{err}");

        let editorconfig =
            load_editorconfig(resolve_editorconfig_path(cwd).as_deref()).map_err(load_err)?;
        let mut root = ConfigResolver::from_config(
            cwd,
            explicit_config,
            editorconfig.clone(),
            #[cfg(feature = "napi")]
            js_config_loader,
        )
        .map_err(load_err)?;
        root.build_and_validate()
            .map_err(|err| format!("Failed to parse configuration.\n{err}"))?;

        Ok(Self::new(
            root,
            editorconfig,
            use_nested,
            #[cfg(feature = "napi")]
            js_config_loader,
        ))
    }

    /// Same as [`Self::load`], but with the default (empty) root config.
    ///
    /// For LSP, which keeps formatting with defaults when the root config is broken,
    /// while nested configs are still detected.
    /// CLI (Walk / Stdin) does not use this and exits with an error instead.
    #[cfg(feature = "napi")]
    pub fn with_default_root(
        cwd: &Path,
        use_nested: bool,
        js_config_loader: Option<&JsConfigLoaderCb>,
    ) -> Self {
        let mut root = ConfigResolver::from_json_config(None, None)
            .expect("Default ConfigResolver should never fail");
        root.build_and_validate().expect("Default ConfigResolver validation should never fail");

        // Best effort: an unreadable `.editorconfig` is skipped instead of failing again
        let editorconfig =
            load_editorconfig(resolve_editorconfig_path(cwd).as_deref()).ok().flatten();
        Self::new(root, editorconfig, use_nested, js_config_loader)
    }

    fn new(
        root: ConfigResolver,
        editorconfig: Option<EditorConfig>,
        use_nested: bool,
        #[cfg(feature = "napi")] js_config_loader: Option<&JsConfigLoaderCb>,
    ) -> Self {
        let has_editorconfig = editorconfig.is_some();
        let nested_ctx = NestedConfigCtx::new(
            editorconfig,
            #[cfg(feature = "napi")]
            js_config_loader.cloned(),
        );
        Self { root: Arc::new(root), nested_ctx, use_nested, has_editorconfig }
    }

    /// Resolve the config scope for a single file: the nearest nested config, or the root.
    ///
    /// When nested detection is enabled, the ancestor chain of `path` is walked,
    /// short-circuiting on the root's `config_dir()` to avoid re-loading the root via `nested_ctx`.
    /// (which would create a duplicate `Arc` and, with `napi`, re-invoke the JS config loader)
    ///
    /// # Errors
    /// Returns error if a nested config fails to load.
    pub fn resolve(&self, path: &Path) -> Result<Arc<ConfigResolver>, String> {
        if !self.use_nested {
            return Ok(Arc::clone(&self.root));
        }
        let Some(parent) = path.parent() else {
            return Ok(Arc::clone(&self.root));
        };

        let root_config_dir = self.root.config_dir();
        for dir in parent.ancestors() {
            if Some(dir) == root_config_dir {
                return Ok(Arc::clone(&self.root));
            }
            if let Some(r) = self.nested_ctx.probe_dir(dir)? {
                return Ok(r);
            }
        }

        Ok(Arc::clone(&self.root))
    }

    pub fn root(&self) -> &Arc<ConfigResolver> {
        &self.root
    }

    pub fn nested_ctx(&self) -> &NestedConfigCtx {
        &self.nested_ctx
    }

    pub fn use_nested(&self) -> bool {
        self.use_nested
    }

    /// Whether any config file or `.editorconfig` has been found so far.
    ///
    /// Nested configs are detected lazily, call this after resolving files.
    pub fn any_config_found(&self) -> bool {
        self.root.config_dir().is_some() || self.nested_ctx.config_found() || self.has_editorconfig
    }
}
