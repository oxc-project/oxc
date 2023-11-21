//! # Oxc Resolver
//!
//! Node.js Module Resolution.
//!
//! All configuration options are aligned with [enhanced-resolve]
//!
//! ## References:
//!
//! * Tests ported from [enhanced-resolve]
//! * Algorithm adapted from Node.js [CommonJS Module Resolution Algorithm] and [ECMAScript Module Resolution Algorithm]
//! * Some code adapted from [parcel-resolver]
//!
//! [enhanced-resolve]: https://github.com/webpack/enhanced-resolve
//! [CommonJS Module Resolution Algorithm]: https://nodejs.org/api/modules.html#all-together
//! [ECMAScript Module Resolution Algorithm]: https://nodejs.org/api/esm.html#resolution-algorithm-specification
//! [parcel-resolver]: https://github.com/parcel-bundler/parcel/blob/v2/packages/utils/node-resolver-rs

mod builtins;
mod cache;
mod error;
mod file_system;
mod json_comments;
mod options;
mod package_json;
mod path;
mod resolution;
mod specifier;
mod tsconfig;

#[cfg(test)]
mod tests;

use std::{
    borrow::Cow,
    cmp::Ordering,
    ffi::OsStr,
    fmt,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    builtins::BUILTINS,
    cache::{Cache, CachedPath},
    file_system::FileSystemOs,
    package_json::{ExportsField, ExportsKey, MatchObject},
    path::PathUtil,
    specifier::Specifier,
    tsconfig::{ProjectReference, TsConfig},
};
pub use crate::{
    error::{JSONError, ResolveError, SpecifierError},
    file_system::{FileMetadata, FileSystem},
    options::{
        Alias, AliasValue, EnforceExtension, ResolveOptions, Restriction, TsconfigOptions,
        TsconfigReferences,
    },
    package_json::PackageJson,
    resolution::Resolution,
};

/// Resolver with the current operating system as the file system
pub type Resolver = ResolverGeneric<FileSystemOs>;

/// Generic implementation of the resolver, can be configured by the [FileSystem] trait.
pub struct ResolverGeneric<Fs> {
    options: ResolveOptions,
    cache: Arc<Cache<Fs>>,
}

impl<Fs> fmt::Debug for ResolverGeneric<Fs> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.options.fmt(f)
    }
}

type ResolveState = Result<Option<CachedPath>, ResolveError>;

#[derive(Debug, Default, Clone)]
struct ResolveContext(ResolveContextImpl);

impl Deref for ResolveContext {
    type Target = ResolveContextImpl;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ResolveContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ResolveContext {
    fn with_fully_specified(&mut self, yes: bool) {
        self.fully_specified = yes;
    }

    fn with_query_fragment(&mut self, query: Option<&str>, fragment: Option<&str>) {
        if let Some(query) = query {
            self.query.replace(query.to_string());
        }
        if let Some(fragment) = fragment {
            self.fragment.replace(fragment.to_string());
        }
    }

    fn with_resolving_alias(&mut self, alias: String) {
        self.resolving_alias = Some(alias);
    }

    fn test_for_infinite_recursion(&mut self) -> Result<(), ResolveError> {
        self.depth += 1;
        // 64 should be more than enough for detecting infinite recursion.
        if self.depth > 64 {
            return Err(ResolveError::Recursion);
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
struct ResolveContextImpl {
    fully_specified: bool,
    query: Option<String>,
    fragment: Option<String>,
    /// The current resolving alias for bailing recursion alias.
    resolving_alias: Option<String>,
    /// For avoiding infinite recursion, which will cause stack overflow.
    depth: u8,
}

impl<Fs: FileSystem + Default> Default for ResolverGeneric<Fs> {
    fn default() -> Self {
        Self::new(ResolveOptions::default())
    }
}

impl<Fs: FileSystem + Default> ResolverGeneric<Fs> {
    pub fn new(options: ResolveOptions) -> Self {
        Self { options: options.sanitize(), cache: Arc::new(Cache::default()) }
    }

    pub fn new_with_file_system(file_system: Fs, options: ResolveOptions) -> Self {
        Self { cache: Arc::new(Cache::new(file_system)), ..Self::new(options) }
    }

    #[must_use]
    pub fn clone_with_options(&self, options: ResolveOptions) -> Self {
        Self { options: options.sanitize(), cache: Arc::clone(&self.cache) }
    }

    pub fn options(&self) -> &ResolveOptions {
        &self.options
    }

    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Resolve `specifier` at `path`
    ///
    /// # Errors
    ///
    /// * See [ResolveError]
    pub fn resolve<P: AsRef<Path>>(
        &self,
        path: P,
        specifier: &str,
    ) -> Result<Resolution, ResolveError> {
        let path = path.as_ref();
        let span = tracing::debug_span!("resolve", path = ?path, specifier = specifier);
        let _enter = span.enter();
        tracing::trace!(options = ?self.options, "resolve_options");
        let r = self.resolve_impl(path, specifier);
        match &r {
            Ok(r) => tracing::debug!(path = ?path, specifier = specifier, ret = ?r.path),
            Err(err) => tracing::debug!(path = ?path, specifier = specifier, err = ?err),
        };
        r
    }

    fn resolve_impl(&self, path: &Path, specifier: &str) -> Result<Resolution, ResolveError> {
        let mut ctx = ResolveContext(ResolveContextImpl {
            fully_specified: self.options.fully_specified,
            ..ResolveContextImpl::default()
        });
        let specifier = Specifier::parse(specifier).map_err(ResolveError::Specifier)?;
        ctx.with_query_fragment(specifier.query, specifier.fragment);
        let cached_path = self.cache.value(path);
        let cached_path =
            self.require(&cached_path, specifier.path(), &mut ctx).or_else(|err| {
                if err.is_ignore() {
                    return Err(err);
                }
                // enhanced-resolve: try fallback
                self.load_alias(&cached_path, specifier.path(), &self.options.fallback, &mut ctx)
                    .and_then(|value| value.ok_or(err))
            })?;
        let path = self.load_realpath(&cached_path)?;
        // enhanced-resolve: restrictions
        self.check_restrictions(&path)?;
        let package_json = cached_path.find_package_json(&self.cache.fs, &self.options)?;
        if let Some(package_json) = &package_json {
            // path must be inside the package.
            debug_assert!(path.starts_with(package_json.directory()));
        }
        Ok(Resolution {
            path,
            query: ctx.query.take(),
            fragment: ctx.fragment.take(),
            package_json,
        })
    }

    /// require(X) from module at path Y
    ///
    /// X: specifier
    /// Y: path
    ///
    /// <https://nodejs.org/api/modules.html#all-together>
    fn require(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &mut ResolveContext,
    ) -> Result<CachedPath, ResolveError> {
        ctx.test_for_infinite_recursion()?;

        // enhanced-resolve: try fragment as path
        if let Some(path) = self.try_fragment_as_path(cached_path, specifier, ctx) {
            return Ok(path);
        }

        // tsconfig-paths
        if let Some(path) =
            self.load_tsconfig_paths(cached_path, specifier, &mut ResolveContext::default())?
        {
            return Ok(path);
        }

        // enhanced-resolve: try alias
        if let Some(path) = self.load_alias(cached_path, specifier, &self.options.alias, ctx)? {
            return Ok(path);
        }

        match specifier.as_bytes()[0] {
            // 3. If X begins with './' or '/' or '../'
            b'/' => self.require_absolute(cached_path, specifier, ctx),
            // 3. If X begins with './' or '/' or '../'
            b'.' => self.require_relative(cached_path, specifier, ctx),
            // 4. If X begins with '#'
            b'#' => self.require_hash(cached_path, specifier, ctx),
            _ => {
                // 1. If X is a core module,
                //   a. return the core module
                //   b. STOP
                self.require_core(specifier)?;

                // (ESM) 5. Otherwise,
                // Note: specifier is now a bare specifier.
                // Set resolved the result of PACKAGE_RESOLVE(specifier, parentURL).
                self.require_bare(cached_path, specifier, ctx)
            }
        }
    }

    fn require_core(&self, specifier: &str) -> Result<(), ResolveError> {
        if self.options.builtin_modules
            && (specifier.starts_with("node:") || BUILTINS.binary_search(&specifier).is_ok())
        {
            return Err(ResolveError::Builtin(specifier.to_string()));
        }
        Ok(())
    }

    fn require_absolute(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &mut ResolveContext,
    ) -> Result<CachedPath, ResolveError> {
        debug_assert!(specifier.starts_with('/'));
        if !self.options.prefer_relative && self.options.prefer_absolute {
            if let Ok(path) = self.load_package_self_or_node_modules(cached_path, specifier, ctx) {
                return Ok(path);
            }
        }
        if self.options.roots.is_empty() {
            // 2. If X begins with '/'
            //   a. set Y to be the file system root
            let path = self.cache.value(Path::new(specifier));
            if let Some(path) = self.load_as_file_or_directory(&path, specifier, ctx)? {
                return Ok(path);
            }
            Err(ResolveError::NotFound(cached_path.to_path_buf()))
        } else {
            for root in &self.options.roots {
                let cached_path = self.cache.value(root);
                if let Ok(path) =
                    self.require_relative(&cached_path, specifier.trim_start_matches('/'), ctx)
                {
                    return Ok(path);
                }
            }
            Err(ResolveError::NotFound(cached_path.to_path_buf()))
        }
    }

    // 3. If X begins with './' or '/' or '../'
    fn require_relative(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &mut ResolveContext,
    ) -> Result<CachedPath, ResolveError> {
        let path = cached_path.path().normalize_with(specifier);
        let cached_path = self.cache.value(&path);
        // a. LOAD_AS_FILE(Y + X)
        // b. LOAD_AS_DIRECTORY(Y + X)
        if let Some(path) = self.load_as_file_or_directory(&cached_path, specifier, ctx)? {
            return Ok(path);
        }
        // c. THROW "not found"
        Err(ResolveError::NotFound(path))
    }

    fn require_hash(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &mut ResolveContext,
    ) -> Result<CachedPath, ResolveError> {
        // a. LOAD_PACKAGE_IMPORTS(X, dirname(Y))
        if let Some(path) = self.load_package_imports(cached_path, specifier, ctx)? {
            return Ok(path);
        }
        self.load_package_self_or_node_modules(cached_path, specifier, ctx)
    }

    fn require_bare(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &mut ResolveContext,
    ) -> Result<CachedPath, ResolveError> {
        if self.options.prefer_relative {
            if let Ok(path) = self.require_relative(cached_path, specifier, ctx) {
                return Ok(path);
            }
        }
        self.load_package_self_or_node_modules(cached_path, specifier, ctx)
    }

    /// Try fragment as part of the path
    ///
    /// It's allowed to escape # as \0# to avoid parsing it as fragment.
    /// enhanced-resolve will try to resolve requests containing `#` as path and as fragment,
    /// so it will automatically figure out if `./some#thing` means `.../some.js#thing` or `.../some#thing.js`.
    /// When a # is resolved as path it will be escaped in the result. Here: `.../some\0#thing.js`.
    ///
    /// <https://github.com/webpack/enhanced-resolve#escaping>
    fn try_fragment_as_path(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &mut ResolveContext,
    ) -> Option<CachedPath> {
        if ctx.fragment.is_some() && ctx.query.is_none() {
            let fragment = ctx.fragment.take().unwrap();
            let path = format!("{specifier}{fragment}");
            if let Ok(path) = self.require(cached_path, &path, ctx) {
                return Some(path);
            }
            ctx.fragment.replace(fragment);
        }
        None
    }

    fn load_package_self_or_node_modules(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &mut ResolveContext,
    ) -> Result<CachedPath, ResolveError> {
        let (_, subpath) = Self::parse_package_specifier(specifier);
        if subpath.is_empty() {
            ctx.with_fully_specified(false);
        }
        // 5. LOAD_PACKAGE_SELF(X, dirname(Y))
        if let Some(path) = self.load_package_self(cached_path, specifier, ctx)? {
            return Ok(path);
        }
        // 6. LOAD_NODE_MODULES(X, dirname(Y))
        if let Some(path) = self.load_node_modules(cached_path, specifier, ctx)? {
            return Ok(path);
        }
        // 7. THROW "not found"
        Err(ResolveError::NotFound(cached_path.to_path_buf()))
    }

    /// LOAD_PACKAGE_IMPORTS(X, DIR)
    fn load_package_imports(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        // 1. Find the closest package scope SCOPE to DIR.
        // 2. If no scope was found, return.
        let Some(package_json) = cached_path.find_package_json(&self.cache.fs, &self.options)?
        else {
            return Ok(None);
        };
        // 3. If the SCOPE/package.json "imports" is null or undefined, return.
        if package_json.imports.is_empty() {
            return Ok(None);
        }
        // 4. let MATCH = PACKAGE_IMPORTS_RESOLVE(X, pathToFileURL(SCOPE), ["node", "require"]) defined in the ESM resolver.
        let package_url = self.cache.value(package_json.directory());
        let path = self.package_imports_resolve(&package_url, specifier, ctx)?;
        // 5. RESOLVE_ESM_MATCH(MATCH).
        self.resolve_esm_match(&path, &package_json, ctx)
    }

    fn load_as_file(&self, cached_path: &CachedPath, ctx: &mut ResolveContext) -> ResolveState {
        // enhanced-resolve feature: extension_alias
        if let Some(path) = self.load_extension_alias(cached_path, ctx)? {
            return Ok(Some(path));
        }
        if self.options.enforce_extension.is_disabled() {
            // 1. If X is a file, load X as its file extension format. STOP
            if let Some(path) = self.load_alias_or_file(cached_path, ctx)? {
                return Ok(Some(path));
            }
        }
        // 2. If X.js is a file, load X.js as JavaScript text. STOP
        // 3. If X.json is a file, parse X.json to a JavaScript Object. STOP
        // 4. If X.node is a file, load X.node as binary addon. STOP
        if let Some(path) =
            self.load_extensions(cached_path.path(), &self.options.extensions, ctx)?
        {
            return Ok(Some(path));
        }
        Ok(None)
    }

    fn load_as_directory(
        &self,
        cached_path: &CachedPath,
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        if !cached_path.is_dir(&self.cache.fs) {
            return Ok(None);
        }
        // TODO: Only package.json is supported, so warn about having other values
        // Checking for empty files is needed for omitting checks on package.json
        // 1. If X/package.json is a file,
        if !self.options.description_files.is_empty() {
            // a. Parse X/package.json, and look for "main" field.
            if let Some(package_json) = cached_path.package_json(&self.cache.fs, &self.options)? {
                // b. If "main" is a falsy value, GOTO 2.
                for main_field in &package_json.main_fields {
                    // c. let M = X + (json main field)
                    let main_field_path = cached_path.path().normalize_with(main_field);
                    // d. LOAD_AS_FILE(M)
                    let cached_path = self.cache.value(&main_field_path);
                    if let Some(path) = self.load_as_file(&cached_path, ctx)? {
                        return Ok(Some(path));
                    }
                    // e. LOAD_INDEX(M)
                    if let Some(path) = self.load_index(&cached_path, ctx)? {
                        return Ok(Some(path));
                    }
                }
                // f. LOAD_INDEX(X) DEPRECATED
                // g. THROW "not found"
            }
        }
        // 2. LOAD_INDEX(X)
        self.load_index(cached_path, ctx)
    }

    fn load_as_file_or_directory(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        if self.options.resolve_to_context {
            return Ok(cached_path.is_dir(&self.cache.fs).then(|| cached_path.clone()));
        }
        if !specifier.ends_with('/') {
            if let Some(path) = self.load_as_file(cached_path, ctx)? {
                return Ok(Some(path));
            }
        }
        if let Some(path) = self.load_as_directory(cached_path, ctx)? {
            return Ok(Some(path));
        }
        Ok(None)
    }

    fn load_extensions(
        &self,
        path: &Path,
        extensions: &[String],
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        if ctx.fully_specified {
            return Ok(None);
        }
        for extension in extensions {
            let mut path_with_extension = path.to_path_buf().into_os_string();
            path_with_extension.reserve_exact(extension.len());
            path_with_extension.push(extension);
            let path_with_extension = PathBuf::from(path_with_extension);
            let cached_path = self.cache.value(&path_with_extension);
            if let Some(path) = self.load_alias_or_file(&cached_path, ctx)? {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    fn load_realpath(&self, cached_path: &CachedPath) -> Result<PathBuf, ResolveError> {
        if self.options.symlinks {
            cached_path.realpath(&self.cache.fs).map_err(ResolveError::from)
        } else {
            Ok(cached_path.to_path_buf())
        }
    }

    fn check_restrictions(&self, path: &Path) -> Result<(), ResolveError> {
        // https://github.com/webpack/enhanced-resolve/blob/a998c7d218b7a9ec2461fc4fddd1ad5dd7687485/lib/RestrictionsPlugin.js#L19-L24
        fn is_inside(path: &Path, parent: &Path) -> bool {
            if !path.starts_with(parent) {
                return false;
            }
            if path.as_os_str().len() == parent.as_os_str().len() {
                return true;
            }
            path.strip_prefix(parent).is_ok_and(|p| p == Path::new("./"))
        }
        for restriction in &self.options.restrictions {
            match restriction {
                Restriction::Path(restricted_path) => {
                    if !is_inside(path, restricted_path) {
                        return Err(ResolveError::Restriction(path.to_path_buf()));
                    }
                }
                Restriction::RegExp(_) => {
                    return Err(ResolveError::Unimplemented("Restriction with regex"))
                }
            }
        }
        Ok(())
    }

    fn load_index(&self, cached_path: &CachedPath, ctx: &mut ResolveContext) -> ResolveState {
        for main_file in &self.options.main_files {
            let main_path = cached_path.path().join(main_file);
            let cached_path = self.cache.value(&main_path);
            if self.options.enforce_extension.is_disabled() {
                if let Some(path) = self.load_alias_or_file(&cached_path, ctx)? {
                    return Ok(Some(path));
                }
            }
            // 1. If X/index.js is a file, load X/index.js as JavaScript text. STOP
            // 2. If X/index.json is a file, parse X/index.json to a JavaScript object. STOP
            // 3. If X/index.node is a file, load X/index.node as binary addon. STOP
            if let Some(path) =
                self.load_extensions(cached_path.path(), &self.options.extensions, ctx)?
            {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    fn load_alias_or_file(
        &self,
        cached_path: &CachedPath,
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        if let Some(package_json) = cached_path.find_package_json(&self.cache.fs, &self.options)? {
            let path = cached_path.path();
            if let Some(path) = self.load_browser_field(path, None, &package_json, ctx)? {
                return Ok(Some(path));
            }
        }
        // enhanced-resolve: try file as alias
        let alias_specifier = cached_path.path().to_string_lossy();
        if let Some(path) =
            self.load_alias(cached_path, &alias_specifier, &self.options.alias, ctx)?
        {
            return Ok(Some(path));
        }
        if cached_path.is_file(&self.cache.fs) {
            return Ok(Some(cached_path.clone()));
        }
        tracing::trace!(path = ?cached_path, "is_not_file");
        Ok(None)
    }

    fn load_node_modules(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        let (package_name, subpath) = Self::parse_package_specifier(specifier);
        tracing::trace!(path = ?cached_path, package_name, subpath, "load_node_modules");
        // 1. let DIRS = NODE_MODULES_PATHS(START)
        // 2. for each DIR in DIRS:
        for module_name in &self.options.modules {
            for cached_path in std::iter::successors(Some(cached_path), |p| p.parent()) {
                let Some(cached_path) = self.get_module_directory(cached_path, module_name) else {
                    continue;
                };
                // Optimize node_modules lookup by inspecting whether the package exists
                // From LOAD_PACKAGE_EXPORTS(X, DIR)
                // 1. Try to interpret X as a combination of NAME and SUBPATH where the name
                //    may have a @scope/ prefix and the subpath begins with a slash (`/`).
                if !package_name.is_empty() {
                    let package_path = cached_path.path().join(package_name);
                    let cached_path = self.cache.value(&package_path);
                    // Try foo/node_modules/package_name
                    if cached_path.is_dir(&self.cache.fs) {
                        // a. LOAD_PACKAGE_EXPORTS(X, DIR)
                        if let Some(path) = self.load_package_exports(subpath, &cached_path, ctx)? {
                            return Ok(Some(path));
                        }
                    } else {
                        // foo/node_modules/package_name is not a directory, so useless to check inside it
                        if !subpath.is_empty() {
                            continue;
                        }
                        // Skip if the directory lead to the scope package does not exist
                        // i.e. `foo/node_modules/@scope` is not a directory for `foo/node_modules/@scope/package`
                        if package_name.starts_with('@') {
                            if let Some(path) = cached_path.parent() {
                                if !path.is_dir(&self.cache.fs) {
                                    continue;
                                }
                            }
                        }
                    }
                }

                // Try as file or directory for all other cases
                // b. LOAD_AS_FILE(DIR/X)
                // c. LOAD_AS_DIRECTORY(DIR/X)
                let node_module_file = cached_path.path().normalize_with(specifier);
                let cached_path = self.cache.value(&node_module_file);
                if let Some(path) = self.load_as_file_or_directory(&cached_path, specifier, ctx)? {
                    return Ok(Some(path));
                }
            }
        }
        Ok(None)
    }

    fn get_module_directory(
        &self,
        cached_path: &CachedPath,
        module_name: &str,
    ) -> Option<CachedPath> {
        if cached_path.path().ends_with(module_name) {
            Some(cached_path.clone())
        } else if module_name == "node_modules" {
            cached_path.cached_node_modules(&self.cache)
        } else {
            cached_path.module_directory(module_name, &self.cache)
        }
    }

    fn load_package_exports(
        &self,
        subpath: &str,
        cached_path: &CachedPath,
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        // 2. If X does not match this pattern or DIR/NAME/package.json is not a file,
        //    return.
        let Some(package_json) = cached_path.package_json(&self.cache.fs, &self.options)? else {
            return Ok(None);
        };
        // 3. Parse DIR/NAME/package.json, and look for "exports" field.
        // 4. If "exports" is null or undefined, return.
        if package_json.exports.is_empty() {
            return Ok(None);
        };
        tracing::trace!(path = ?cached_path, exports = ?package_json.exports, "load_package_exports");
        // 5. let MATCH = PACKAGE_EXPORTS_RESOLVE(pathToFileURL(DIR/NAME), "." + SUBPATH,
        //    `package.json` "exports", ["node", "require"]) defined in the ESM resolver.
        // Note: The subpath is not prepended with a dot on purpose
        for exports in &package_json.exports {
            if let Some(path) = self.package_exports_resolve(
                cached_path.path(),
                subpath,
                exports,
                &self.options.condition_names,
                ctx,
            )? {
                // 6. RESOLVE_ESM_MATCH(MATCH)
                return self.resolve_esm_match(&path, &package_json, ctx);
            };
        }
        Ok(None)
    }

    fn load_package_self(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        // 1. Find the closest package scope SCOPE to DIR.
        // 2. If no scope was found, return.
        let Some(package_json) = cached_path.find_package_json(&self.cache.fs, &self.options)?
        else {
            return Ok(None);
        };
        // 3. If the SCOPE/package.json "exports" is null or undefined, return.
        if package_json.exports.is_empty() {
            return self.load_browser_field(
                cached_path.path(),
                Some(specifier),
                &package_json,
                ctx,
            );
        }
        // 4. If the SCOPE/package.json "name" is not the first segment of X, return.
        let Some(subpath) = package_json
            .name
            .as_ref()
            .and_then(|package_name| Self::strip_package_name(specifier, package_name))
        else {
            return Ok(None);
        };
        // 5. let MATCH = PACKAGE_EXPORTS_RESOLVE(pathToFileURL(SCOPE),
        // "." + X.slice("name".length), `package.json` "exports", ["node", "require"])
        // defined in the ESM resolver.
        let package_url = package_json.directory();
        tracing::trace!(package = ?package_url, exports = ?package_json.exports, "load_package_self");
        // Note: The subpath is not prepended with a dot on purpose
        // because `package_exports_resolve` matches subpath without the leading dot.
        for exports in &package_json.exports {
            if let Some(cached_path) = self.package_exports_resolve(
                package_url,
                subpath,
                exports,
                &self.options.condition_names,
                ctx,
            )? {
                // 6. RESOLVE_ESM_MATCH(MATCH)
                return self.resolve_esm_match(&cached_path, &package_json, ctx);
            }
        }
        Ok(None)
    }

    /// RESOLVE_ESM_MATCH(MATCH)
    fn resolve_esm_match(
        &self,
        cached_path: &CachedPath,
        package_json: &PackageJson,
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        if let Some(path) = self.load_browser_field(cached_path.path(), None, package_json, ctx)? {
            return Ok(Some(path));
        }

        // 1. let RESOLVED_PATH = fileURLToPath(MATCH)
        // 2. If the file at RESOLVED_PATH exists, load RESOLVED_PATH as its extension format. STOP
        //
        // Non-compliant ESM can result in a directory, so directory is tried as well.
        if let Some(path) = self.load_as_file_or_directory(cached_path, "", ctx)? {
            return Ok(Some(path));
        }

        // 3. THROW "not found"
        Err(ResolveError::NotFound(cached_path.to_path_buf()))
    }

    /// enhanced-resolve: AliasFieldPlugin for [ResolveOptions::alias_fields]
    fn load_browser_field(
        &self,
        path: &Path,
        specifier: Option<&str>,
        package_json: &PackageJson,
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        let Some(specifier) = package_json.resolve_browser_field(path, specifier)? else {
            return Ok(None);
        };
        if ctx.resolving_alias.as_ref().is_some_and(|s| s == specifier) {
            return Ok(None);
        }
        let specifier = Specifier::parse(specifier).map_err(ResolveError::Specifier)?;
        ctx.with_query_fragment(specifier.query, specifier.fragment);
        ctx.with_resolving_alias(specifier.path().to_string());
        ctx.with_fully_specified(false);
        let cached_path = self.cache.value(package_json.directory());
        self.require(&cached_path, specifier.path(), ctx).map(Some)
    }

    /// enhanced-resolve: AliasPlugin for [ResolveOptions::alias] and [ResolveOptions::fallback].
    fn load_alias(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        aliases: &Alias,
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        for (alias_key_raw, specifiers) in aliases {
            let from = alias_key_raw.strip_suffix('$');
            let alias_key = from.unwrap_or(alias_key_raw);
            let exact_match = from.is_some() && specifier == alias_key;
            if !(exact_match || Self::strip_package_name(specifier, alias_key).is_some()) {
                continue;
            }
            for r in specifiers {
                match r {
                    AliasValue::Path(alias_value) => {
                        let new_specifier =
                            Specifier::parse(alias_value).map_err(ResolveError::Specifier)?;

                        // `#` can be a fragment or a path, try fragment as path first
                        if new_specifier.query.is_none() && new_specifier.fragment.is_some() {
                            if let Some(path) = self.load_alias_value(
                                cached_path,
                                alias_key,
                                alias_value, // pass in original alias value, not parsed
                                specifier,
                                ctx,
                            )? {
                                return Ok(Some(path));
                            }
                        }

                        // Then try path without query and fragment
                        let old_query = ctx.query.clone();
                        let old_fragment = ctx.fragment.clone();
                        ctx.with_query_fragment(new_specifier.query, new_specifier.fragment);
                        if let Some(path) = self.load_alias_value(
                            cached_path,
                            alias_key,
                            new_specifier.path(), // pass in passed alias value
                            specifier,
                            ctx,
                        )? {
                            return Ok(Some(path));
                        }
                        ctx.with_query_fragment(old_query.as_deref(), old_fragment.as_deref());
                    }
                    AliasValue::Ignore => {
                        let path = cached_path.path().normalize_with(alias_key);
                        return Err(ResolveError::Ignored(path));
                    }
                }
            }
        }
        Ok(None)
    }

    fn load_alias_value(
        &self,
        cached_path: &CachedPath,
        alias_key: &str,
        alias_value: &str,
        request: &str,
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        if request != alias_value
            && !request.strip_prefix(alias_value).is_some_and(|prefix| prefix.starts_with('/'))
        {
            let new_specifier = format!("{alias_value}{}", &request[alias_key.len()..]);
            ctx.with_fully_specified(false);
            return match self.require(cached_path, &new_specifier, ctx) {
                Err(ResolveError::NotFound(_)) => Ok(None),
                Ok(path) => return Ok(Some(path)),
                Err(err) => return Err(err),
            };
        }
        Ok(None)
    }

    /// Given an extension alias map `{".js": [".ts", "js"]}`,
    /// load the mapping instead of the provided extension
    ///
    /// This is an enhanced-resolve feature
    ///
    /// # Errors
    ///
    /// * [ResolveError::ExtensionAlias]: When all of the aliased extensions are not found
    fn load_extension_alias(
        &self,
        cached_path: &CachedPath,
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        let Some(path_extension) = cached_path.path().extension() else { return Ok(None) };
        let Some((_, extensions)) = self
            .options
            .extension_alias
            .iter()
            .find(|(ext, _)| OsStr::new(ext.trim_start_matches('.')) == path_extension)
        else {
            return Ok(None);
        };
        let path = cached_path.path().with_extension("");
        ctx.with_fully_specified(false);
        if let Some(path) = self.load_extensions(&path, extensions, ctx)? {
            return Ok(Some(path));
        }
        Err(ResolveError::ExtensionAlias)
    }

    fn load_tsconfig_paths(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        let Some(tsconfig_options) = &self.options.tsconfig else { return Ok(None) };
        let tsconfig =
            self.load_tsconfig(&tsconfig_options.config_file, &tsconfig_options.references)?;
        let paths = tsconfig.resolve(cached_path.path(), specifier);
        for path in paths {
            tracing::trace!(path = ?cached_path, tsconfig_path = ?path, "load_tsconfig_paths");
            let cached_path = self.cache.value(&path);
            if let Ok(path) = self.require_relative(&cached_path, ".", ctx) {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    fn load_tsconfig(
        &self,
        path: &Path,
        references: &TsconfigReferences,
    ) -> Result<Arc<TsConfig>, ResolveError> {
        self.cache.tsconfig(path, |tsconfig| {
            let directory = self.cache.value(tsconfig.directory());
            tracing::trace!(tsconfig = ?tsconfig, "load_tsconfig");
            // Extend tsconfig
            let mut extended_tsconfig_paths = vec![];
            for tsconfig_extend_specifier in &tsconfig.extends {
                let extended_tsconfig_path = match tsconfig_extend_specifier.as_bytes().first() {
                    None => return Err(ResolveError::Specifier(SpecifierError::Empty)),
                    Some(b'/') => PathBuf::from(tsconfig_extend_specifier),
                    Some(b'.') => tsconfig.directory().normalize_with(tsconfig_extend_specifier),
                    _ => self
                        .clone_with_options(ResolveOptions {
                            description_files: vec![],
                            extensions: vec![".json".into()],
                            main_files: vec!["tsconfig.json".into()],
                            ..ResolveOptions::default()
                        })
                        .load_package_self_or_node_modules(
                            &directory,
                            tsconfig_extend_specifier,
                            &mut ResolveContext::default(),
                        )
                        .map_err(|err| match err {
                            ResolveError::NotFound(_) => ResolveError::TsconfigNotFound(
                                PathBuf::from(tsconfig_extend_specifier),
                            ),
                            _ => err,
                        })?
                        .to_path_buf(),
                };
                extended_tsconfig_paths.push(extended_tsconfig_path);
            }
            for extended_tsconfig_path in extended_tsconfig_paths {
                let extended_tsconfig =
                    self.load_tsconfig(&extended_tsconfig_path, &TsconfigReferences::Disabled)?;
                tsconfig.extend_tsconfig(&extended_tsconfig);
            }
            // Load project references
            match references {
                TsconfigReferences::Disabled => {
                    tsconfig.references.drain(..);
                }
                TsconfigReferences::Auto => {}
                TsconfigReferences::Paths(paths) => {
                    tsconfig.references = paths
                        .iter()
                        .map(|path| ProjectReference { path: path.clone(), tsconfig: None })
                        .collect();
                }
            }
            if !tsconfig.references.is_empty() {
                let directory = tsconfig.directory().to_path_buf();
                for reference in &mut tsconfig.references {
                    let reference_tsconfig_path = directory.normalize_with(&reference.path);
                    let tsconfig = self.cache.tsconfig(&reference_tsconfig_path, |_| Ok(()))?;
                    reference.tsconfig.replace(tsconfig);
                }
            }
            Ok(())
        })
    }

    /// PACKAGE_RESOLVE(packageSpecifier, parentURL)
    fn package_resolve(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        let (package_name, subpath) = Self::parse_package_specifier(specifier);
        // 11. While parentURL is not the file system root,
        for module_name in &self.options.modules {
            for cached_path in std::iter::successors(Some(cached_path), |p| p.parent()) {
                // 1. Let packageURL be the URL resolution of "node_modules/" concatenated with packageSpecifier, relative to parentURL.
                let Some(cached_path) = self.get_module_directory(cached_path, module_name) else {
                    continue;
                };
                // 2. Set parentURL to the parent folder URL of parentURL.
                let package_path = cached_path.path().join(package_name);
                let cached_path = self.cache.value(&package_path);
                // 3. If the folder at packageURL does not exist, then
                //   1. Continue the next loop iteration.
                if cached_path.is_dir(&self.cache.fs) {
                    // 4. Let pjson be the result of READ_PACKAGE_JSON(packageURL).
                    if let Some(package_json) =
                        cached_path.package_json(&self.cache.fs, &self.options)?
                    {
                        // 5. If pjson is not null and pjson.exports is not null or undefined, then
                        if !package_json.exports.is_empty() {
                            // 1. Return the result of PACKAGE_EXPORTS_RESOLVE(packageURL, packageSubpath, pjson.exports, defaultConditions).
                            for exports in &package_json.exports {
                                if let Some(path) = self.package_exports_resolve(
                                    cached_path.path(),
                                    subpath,
                                    exports,
                                    &self.options.condition_names,
                                    ctx,
                                )? {
                                    return Ok(Some(path));
                                }
                            }
                        }
                        // 6. Otherwise, if packageSubpath is equal to ".", then
                        if subpath == "." {
                            // 1. If pjson.main is a string, then
                            for main_field in &package_json.main_fields {
                                // 1. Return the URL resolution of main in packageURL.
                                let path = cached_path.path().normalize_with(main_field);
                                let cached_path = self.cache.value(&path);
                                if cached_path.is_file(&self.cache.fs) {
                                    return Ok(Some(cached_path));
                                }
                            }
                        }
                    }
                    let subpath = format!(".{subpath}");
                    let specifier = Specifier::parse(&subpath).map_err(ResolveError::Specifier)?;
                    ctx.with_fully_specified(false);
                    ctx.with_query_fragment(specifier.query, specifier.fragment);
                    return self.require(&cached_path, specifier.path(), ctx).map(Some);
                }
            }
        }

        Err(ResolveError::NotFound(cached_path.to_path_buf()))
    }

    /// PACKAGE_EXPORTS_RESOLVE(packageURL, subpath, exports, conditions)
    fn package_exports_resolve(
        &self,
        package_url: &Path,
        subpath: &str,
        exports: &ExportsField,
        conditions: &[String],
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        // 1. If exports is an Object with both a key starting with "." and a key not starting with ".", throw an Invalid Package Configuration error.
        if let ExportsField::Map(map) = exports {
            let mut has_dot = false;
            let mut without_dot = false;
            for key in map.keys() {
                has_dot = has_dot || matches!(key, ExportsKey::Main | ExportsKey::Pattern(_));
                without_dot = without_dot || matches!(key, ExportsKey::CustomCondition(_));
                if has_dot && without_dot {
                    return Err(ResolveError::InvalidPackageConfig(
                        package_url.join("package.json"),
                    ));
                }
            }
        }
        // 2. If subpath is equal to ".", then
        // Note: subpath is not prepended with a dot when passed in.
        if subpath.is_empty() {
            // enhanced-resolve appends query and fragment when resolving exports field
            // https://github.com/webpack/enhanced-resolve/blob/a998c7d218b7a9ec2461fc4fddd1ad5dd7687485/lib/ExportsFieldPlugin.js#L57-L62
            // This is only need when querying the main export, otherwise ctx is passed through.
            if ctx.query.is_some() || ctx.fragment.is_some() {
                let query = ctx.query.clone().unwrap_or_default();
                let fragment = ctx.fragment.clone().unwrap_or_default();
                return Err(ResolveError::PackagePathNotExported(format!(
                    "./{subpath}{query}{fragment}"
                )));
            }
            // 1. Let mainExport be undefined.
            let main_export = match exports {
                ExportsField::None => None,
                // 2. If exports is a String or Array, or an Object containing no keys starting with ".", then
                ExportsField::String(_) | ExportsField::Array(_) => {
                    // 1. Set mainExport to exports.
                    Some(exports)
                }
                // 3. Otherwise if exports is an Object containing a "." property, then
                ExportsField::Map(map) => {
                    // 1. Set mainExport to exports["."].
                    map.get(&ExportsKey::Main).map_or_else(
                        || {
                            if map.keys().any(|key| matches!(key, ExportsKey::Pattern(_))) {
                                None
                            } else {
                                Some(exports)
                            }
                        },
                        Some,
                    )
                }
            };
            // 4. If mainExport is not undefined, then
            if let Some(main_export) = main_export {
                // 1. Let resolved be the result of PACKAGE_TARGET_RESOLVE( packageURL, mainExport, null, false, conditions).
                let resolved = self.package_target_resolve(
                    package_url,
                    ".",
                    main_export,
                    None,
                    /* is_imports */ false,
                    conditions,
                    ctx,
                )?;
                // 2. If resolved is not null or undefined, return resolved.
                if let Some(path) = resolved {
                    return Ok(Some(path));
                }
            }
        }
        // 3. Otherwise, if exports is an Object and all keys of exports start with ".", then
        if let ExportsField::Map(exports) = exports {
            // 1. Let matchKey be the string "./" concatenated with subpath.
            // Note: `package_imports_exports_resolve` does not require the leading dot.
            let match_key = &subpath;
            // 2. Let resolved be the result of PACKAGE_IMPORTS_EXPORTS_RESOLVE( matchKey, exports, packageURL, false, conditions).
            if let Some(path) = self.package_imports_exports_resolve(
                match_key,
                exports,
                package_url,
                /* is_imports */ false,
                conditions,
                ctx,
            )? {
                // 3. If resolved is not null or undefined, return resolved.
                return Ok(Some(path));
            }
        }
        // 4. Throw a Package Path Not Exported error.
        Err(ResolveError::PackagePathNotExported(format!(".{subpath}")))
    }

    /// PACKAGE_IMPORTS_RESOLVE(specifier, parentURL, conditions)
    fn package_imports_resolve(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &mut ResolveContext,
    ) -> Result<CachedPath, ResolveError> {
        // 1. Assert: specifier begins with "#".
        debug_assert!(specifier.starts_with('#'), "{specifier}");
        // 2. If specifier is exactly equal to "#" or starts with "#/", then
        if specifier == "#" || specifier.starts_with("#/") {
            // 1. Throw an Invalid Module Specifier error.
            return Err(ResolveError::InvalidModuleSpecifier(specifier.to_string()));
        }
        // 3. Let packageURL be the result of LOOKUP_PACKAGE_SCOPE(parentURL).
        // 4. If packageURL is not null, then
        if let Some(package_json) = cached_path.find_package_json(&self.cache.fs, &self.options)? {
            // 1. Let pjson be the result of READ_PACKAGE_JSON(packageURL).
            // 2. If pjson.imports is a non-null Object, then
            if !package_json.imports.is_empty() {
                // 1. Let resolved be the result of PACKAGE_IMPORTS_EXPORTS_RESOLVE( specifier, pjson.imports, packageURL, true, conditions).
                let package_url = package_json.directory();
                if let Some(path) = self.package_imports_exports_resolve(
                    specifier,
                    &package_json.imports,
                    package_url,
                    /* is_imports */ true,
                    &self.options.condition_names,
                    ctx,
                )? {
                    // 2. If resolved is not null or undefined, return resolved.
                    return Ok(path);
                }
            }
        }
        // 5. Throw a Package Import Not Defined error.
        Err(ResolveError::PackageImportNotDefined(specifier.to_string()))
    }

    /// PACKAGE_IMPORTS_EXPORTS_RESOLVE(matchKey, matchObj, packageURL, isImports, conditions)
    fn package_imports_exports_resolve(
        &self,
        match_key: &str,
        match_obj: &MatchObject,
        package_url: &Path,
        is_imports: bool,
        conditions: &[String],
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        // enhanced-resolve behaves differently, it throws
        // Error: CachedPath to directories is not possible with the exports field (specifier was ./dist/)
        if match_key.ends_with('/') {
            return Ok(None);
        }
        // 1. If matchKey is a key of matchObj and does not contain "*", then
        if !match_key.contains('*') {
            // 1. Let target be the value of matchObj[matchKey].
            if let Some(target) = match_obj.get(&ExportsKey::Pattern(match_key.to_string())) {
                // 2. Return the result of PACKAGE_TARGET_RESOLVE(packageURL, target, null, isImports, conditions).
                return self.package_target_resolve(
                    package_url,
                    match_key,
                    target,
                    None,
                    is_imports,
                    conditions,
                    ctx,
                );
            }
        }

        let mut best_target = None;
        let mut best_match = "";
        let mut best_key = "";
        // 2. Let expansionKeys be the list of keys of matchObj containing only a single "*", sorted by the sorting function PATTERN_KEY_COMPARE which orders in descending order of specificity.
        // 3. For each key expansionKey in expansionKeys, do
        for (expansion_key, target) in match_obj {
            if let ExportsKey::Pattern(expansion_key) = expansion_key {
                // 1. Let patternBase be the substring of expansionKey up to but excluding the first "*" character.
                if let Some((pattern_base, pattern_trailer)) = expansion_key.split_once('*') {
                    // 2. If matchKey starts with but is not equal to patternBase, then
                    if match_key.starts_with(pattern_base)
                        // 1. Let patternTrailer be the substring of expansionKey from the index after the first "*" character.
                        && !pattern_trailer.contains('*')
                        // 2. If patternTrailer has zero length, or if matchKey ends with patternTrailer and the length of matchKey is greater than or equal to the length of expansionKey, then
                        && (pattern_trailer.is_empty()
                            || (match_key.len() >= expansion_key.len()
                                && match_key.ends_with(pattern_trailer)))
                        && Self::pattern_key_compare(best_key, expansion_key).is_gt()
                    {
                        // 1. Let target be the value of matchObj[expansionKey].
                        best_target = Some(target);
                        // 2. Let patternMatch be the substring of matchKey starting at the index of the length of patternBase up to the length of matchKey minus the length of patternTrailer.
                        best_match =
                            &match_key[pattern_base.len()..match_key.len() - pattern_trailer.len()];
                        best_key = expansion_key;
                    }
                } else if expansion_key.ends_with('/')
                    && match_key.starts_with(expansion_key)
                    && Self::pattern_key_compare(best_key, expansion_key).is_gt()
                {
                    // TODO: [DEP0148] DeprecationWarning: Use of deprecated folder mapping "./dist/" in the "exports" field module resolution of the package at xxx/package.json.
                    best_target = Some(target);
                    best_match = &match_key[expansion_key.len()..];
                    best_key = expansion_key;
                }
            }
        }
        if let Some(best_target) = best_target {
            // 3. Return the result of PACKAGE_TARGET_RESOLVE(packageURL, target, patternMatch, isImports, conditions).
            return self.package_target_resolve(
                package_url,
                best_key,
                best_target,
                Some(best_match),
                is_imports,
                conditions,
                ctx,
            );
        }
        // 4. Return null.
        Ok(None)
    }

    /// PACKAGE_TARGET_RESOLVE(packageURL, target, patternMatch, isImports, conditions)
    #[allow(clippy::too_many_arguments)]
    fn package_target_resolve(
        &self,
        package_url: &Path,
        target_key: &str,
        target: &ExportsField,
        pattern_match: Option<&str>,
        is_imports: bool,
        conditions: &[String],
        ctx: &mut ResolveContext,
    ) -> ResolveState {
        fn normalize_string_target<'a>(
            target_key: &'a str,
            target: &'a str,
            pattern_match: Option<&'a str>,
            package_url: &Path,
        ) -> Result<Cow<'a, str>, ResolveError> {
            let target = if let Some(pattern_match) = pattern_match {
                if !target_key.contains('*') && !target.contains('*') {
                    // enhanced-resolve behaviour
                    // TODO: [DEP0148] DeprecationWarning: Use of deprecated folder mapping "./dist/" in the "exports" field module resolution of the package at xxx/package.json.
                    if target_key.ends_with('/') && target.ends_with('/') {
                        Cow::Owned(format!("{target}{pattern_match}"))
                    } else {
                        return Err(ResolveError::InvalidPackageConfigDirectory(
                            package_url.join("package.json"),
                        ));
                    }
                } else {
                    Cow::Owned(target.replace('*', pattern_match))
                }
            } else {
                Cow::Borrowed(target)
            };
            Ok(target)
        }

        match target {
            ExportsField::None => {}
            // 1. If target is a String, then
            ExportsField::String(target) => {
                // 1. If target does not start with "./", then
                if !target.starts_with("./") {
                    // 1. If isImports is false, or if target starts with "../" or "/", or if target is a valid URL, then
                    if !is_imports || target.starts_with("../") || target.starts_with('/') {
                        // 1. Throw an Invalid Package Target error.
                        // TODO:
                        // Error [ERR_INVALID_PACKAGE_TARGET]: Invalid "exports" target "/a/" defined for './utils/*' in the package config /Users/bytedance/github/test-resolver/node_modules/foo/package.json; targets must start with "./"
                        return Err(ResolveError::InvalidPackageTarget(target.to_string()));
                    }
                    // 2. If patternMatch is a String, then
                    //   1. Return PACKAGE_RESOLVE(target with every instance of "*" replaced by patternMatch, packageURL + "/").
                    let target =
                        normalize_string_target(target_key, target, pattern_match, package_url)?;
                    let package_url = self.cache.value(package_url);
                    // // 3. Return PACKAGE_RESOLVE(target, packageURL + "/").
                    return self.package_resolve(&package_url, &target, ctx);
                }

                // 2. If target split on "/" or "\" contains any "", ".", "..", or "node_modules" segments after the first "." segment, case insensitive and including percent encoded variants, throw an Invalid Package Target error.
                // 3. Let resolvedTarget be the URL resolution of the concatenation of packageURL and target.
                // 4. Assert: resolvedTarget is contained in packageURL.
                // 5. If patternMatch is null, then
                let target =
                    normalize_string_target(target_key, target, pattern_match, package_url)?;
                if Path::new(target.as_ref()).is_invalid_exports_target() {
                    return Err(ResolveError::InvalidPackageTarget(target.to_string()));
                }
                let resolved_target = package_url.join(target.as_ref()).normalize();
                // 6. If patternMatch split on "/" or "\" contains any "", ".", "..", or "node_modules" segments, case insensitive and including percent encoded variants, throw an Invalid Module Specifier error.
                // 7. Return the URL resolution of resolvedTarget with every instance of "*" replaced with patternMatch.
                let value = self.cache.value(&resolved_target);
                return Ok(Some(value));
            }
            // 2. Otherwise, if target is a non-null Object, then
            ExportsField::Map(target) => {
                // 1. If exports contains any index property keys, as defined in ECMA-262 6.1.7 Array Index, throw an Invalid Package Configuration error.
                // 2. For each property p of target, in object insertion order as,
                for (i, (key, target_value)) in target.iter().enumerate() {
                    // https://nodejs.org/api/packages.html#conditional-exports
                    // "default" - the generic fallback that always matches. Can be a CommonJS or ES module file. This condition should always come last.
                    // Note: node.js does not throw this but enhanced-resolve does.
                    let is_default = matches!(key, ExportsKey::CustomCondition(condition) if condition == "default");
                    if i < target.len() - 1 && is_default {
                        return Err(ResolveError::InvalidPackageConfigDefault(
                            package_url.join("package.json"),
                        ));
                    }

                    // 1. If p equals "default" or conditions contains an entry for p, then
                    if is_default
                        || matches!(key, ExportsKey::CustomCondition(condition) if conditions.contains(condition))
                    {
                        // 1. Let targetValue be the value of the p property in target.
                        // 2. Let resolved be the result of PACKAGE_TARGET_RESOLVE( packageURL, targetValue, patternMatch, isImports, conditions).
                        let resolved = self.package_target_resolve(
                            package_url,
                            target_key,
                            target_value,
                            pattern_match,
                            is_imports,
                            conditions,
                            ctx,
                        );
                        // 3. If resolved is equal to undefined, continue the loop.
                        if let Some(path) = resolved? {
                            // 4. Return resolved.
                            return Ok(Some(path));
                        }
                    }
                }
                // 3. Return undefined.
                return Ok(None);
            }
            // 3. Otherwise, if target is an Array, then
            ExportsField::Array(targets) => {
                // 1. If _target.length is zero, return null.
                if targets.is_empty() {
                    // Note: return PackagePathNotExported has the same effect as return because there are no matches.
                    return Err(ResolveError::PackagePathNotExported(format!(
                        ".{}",
                        pattern_match.unwrap_or(".")
                    )));
                }
                // 2. For each item targetValue in target, do
                for (i, target_value) in targets.iter().enumerate() {
                    // 1. Let resolved be the result of PACKAGE_TARGET_RESOLVE( packageURL, targetValue, patternMatch, isImports, conditions), continuing the loop on any Invalid Package Target error.
                    let resolved = self.package_target_resolve(
                        package_url,
                        target_key,
                        target_value,
                        pattern_match,
                        is_imports,
                        conditions,
                        ctx,
                    );

                    if resolved.is_err() && i == targets.len() {
                        return resolved;
                    }

                    // 2. If resolved is undefined, continue the loop.
                    if let Ok(Some(path)) = resolved {
                        // 3. Return resolved.
                        return Ok(Some(path));
                    }
                }
                // 3. Return or throw the last fallback resolution null return or error.
                // Note: see `resolved.is_err() && i == targets.len()`
            }
        }
        // 4. Otherwise, if target is null, return null.
        Ok(None)
        // 5. Otherwise throw an Invalid Package Target error.
    }

    // Returns (module, subpath)
    // https://github.com/nodejs/node/blob/8f0f17e1e3b6c4e58ce748e06343c5304062c491/lib/internal/modules/esm/resolve.js#L688
    fn parse_package_specifier(specifier: &str) -> (&str, &str) {
        let mut separator_index = specifier.as_bytes().iter().position(|b| *b == b'/');
        // let mut valid_package_name = true;
        // let mut is_scoped = false;
        if specifier.starts_with('@') {
            // is_scoped = true;
            if separator_index.is_none() || specifier.is_empty() {
                // valid_package_name = false;
            } else if let Some(index) = &separator_index {
                separator_index = specifier[*index + 1..]
                    .as_bytes()
                    .iter()
                    .position(|b| *b == b'/')
                    .map(|i| i + *index + 1);
            }
        }
        let package_name =
            separator_index.map_or(specifier, |separator_index| &specifier[..separator_index]);

        // TODO: https://github.com/nodejs/node/blob/8f0f17e1e3b6c4e58ce748e06343c5304062c491/lib/internal/modules/esm/resolve.js#L705C1-L714C1
        // Package name cannot have leading . and cannot have percent-encoding or
        // \\ separators.
        // if (RegExpPrototypeExec(invalidPackageNameRegEx, packageName) !== null)
        // validPackageName = false;

        // if (!validPackageName) {
        // throw new ERR_INVALID_MODULE_SPECIFIER(
        // specifier, 'is not a valid package name', fileURLToPath(base));
        // }
        let package_subpath =
            separator_index.map_or("", |separator_index| &specifier[separator_index..]);
        (package_name, package_subpath)
    }

    /// PATTERN_KEY_COMPARE(keyA, keyB)
    fn pattern_key_compare(key_a: &str, key_b: &str) -> Ordering {
        if key_a.is_empty() {
            return Ordering::Greater;
        }
        // 1. Assert: keyA ends with "/" or contains only a single "*".
        debug_assert!(key_a.ends_with('/') || key_a.match_indices('*').count() == 1, "{key_a}");
        // 2. Assert: keyB ends with "/" or contains only a single "*".
        debug_assert!(key_b.ends_with('/') || key_b.match_indices('*').count() == 1, "{key_b}");
        // 3. Let baseLengthA be the index of "*" in keyA plus one, if keyA contains "*", or the length of keyA otherwise.
        let a_pos = key_a.chars().position(|c| c == '*');
        let base_length_a = a_pos.map_or(key_a.len(), |p| p + 1);
        // 4. Let baseLengthB be the index of "*" in keyB plus one, if keyB contains "*", or the length of keyB otherwise.
        let b_pos = key_b.chars().position(|c| c == '*');
        let base_length_b = b_pos.map_or(key_b.len(), |p| p + 1);
        // 5. If baseLengthA is greater than baseLengthB, return -1.
        if base_length_a > base_length_b {
            return Ordering::Less;
        }
        // 6. If baseLengthB is greater than baseLengthA, return 1.
        if base_length_b > base_length_a {
            return Ordering::Greater;
        }
        // 7. If keyA does not contain "*", return 1.
        if !key_a.contains('*') {
            return Ordering::Greater;
        }
        // 8. If keyB does not contain "*", return -1.
        if !key_b.contains('*') {
            return Ordering::Less;
        }
        // 9. If the length of keyA is greater than the length of keyB, return -1.
        if key_a.len() > key_b.len() {
            return Ordering::Less;
        }
        // 10. If the length of keyB is greater than the length of keyA, return 1.
        if key_b.len() > key_a.len() {
            return Ordering::Greater;
        }
        // 11. Return 0.
        Ordering::Equal
    }

    fn strip_package_name<'a>(specifier: &'a str, package_name: &'a str) -> Option<&'a str> {
        specifier
            .strip_prefix(package_name)
            .filter(|tail| tail.is_empty() || tail.starts_with('/') || tail.starts_with('\\'))
    }
}
