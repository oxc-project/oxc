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

mod cache;
mod error;
mod file_system;
mod options;
mod package_json;
mod path;
mod resolution;
mod specifier;

#[cfg(test)]
mod tests;

use std::{
    borrow::Cow,
    cell::RefCell,
    cmp::Ordering,
    ffi::OsStr,
    ops::Deref,
    path::{Path, PathBuf},
};

use crate::{
    cache::{Cache, CachedPath},
    file_system::FileSystemOs,
    package_json::{ExportsField, ExportsKey, MatchObject, PackageJson},
    path::PathUtil,
    specifier::{Specifier, SpecifierPath},
};
pub use crate::{
    error::{JSONError, ResolveError},
    file_system::{FileMetadata, FileSystem},
    options::{Alias, AliasValue, ResolveOptions},
    resolution::Resolution,
};

/// Resolver with the current operating system as the file system
pub type Resolver = ResolverGeneric<FileSystemOs>;

/// Generic implementation of the resolver, can be configured by the [FileSystem] trait.
pub struct ResolverGeneric<Fs> {
    options: ResolveOptions,
    cache: Cache<Fs>,
}

type ResolveState = Result<Option<CachedPath>, ResolveError>;

#[derive(Debug, Default, Clone)]
struct ResolveContext(RefCell<ResolveContextImpl>);

impl Deref for ResolveContext {
    type Target = RefCell<ResolveContextImpl>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ResolveContext {
    fn clone_from(ctx: &Self) -> Self {
        Self(RefCell::new(ResolveContextImpl {
            fully_specified: false,
            query: ctx.borrow().query.clone(),
            fragment: ctx.borrow().fragment.clone(),
        }))
    }

    fn with_fully_specified(&self, yes: bool) -> &Self {
        self.borrow_mut().fully_specified = yes;
        self
    }

    fn with_query_fragment(&self, query: Option<&str>, fragment: Option<&str>) {
        self.borrow_mut().query = query.map(ToString::to_string);
        self.borrow_mut().fragment = fragment.map(ToString::to_string);
    }
}

#[derive(Debug, Default, Clone)]
struct ResolveContextImpl {
    fully_specified: bool,
    query: Option<String>,
    fragment: Option<String>,
}

impl<Fs: FileSystem> Default for ResolverGeneric<Fs> {
    fn default() -> Self {
        Self::new(ResolveOptions::default())
    }
}

impl<Fs: FileSystem> ResolverGeneric<Fs> {
    pub fn new(options: ResolveOptions) -> Self {
        Self { options: options.sanitize(), cache: Cache::default() }
    }

    pub fn new_with_file_system(options: ResolveOptions, file_system: Fs) -> Self {
        Self { cache: Cache::new(file_system), ..Self::new(options) }
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
        self.resolve_impl(path.as_ref(), specifier)
    }

    fn resolve_impl(&self, path: &Path, specifier: &str) -> Result<Resolution, ResolveError> {
        let ctx = ResolveContext(RefCell::new(ResolveContextImpl {
            fully_specified: self.options.fully_specified,
            ..ResolveContextImpl::default()
        }));
        let cached_path = self.cache.value(path);
        let cached_path = self.require(&cached_path, specifier, &ctx).or_else(|err| {
            // enhanced_resolve: try fallback
            self.load_alias(&cached_path, specifier, &self.options.fallback, &ctx)
                .and_then(|value| value.ok_or(err))
        })?;
        let path = self.load_realpath(&cached_path).unwrap_or_else(|| cached_path.to_path_buf());
        let ctx = ctx.borrow();
        Ok(Resolution {
            path,
            query: ctx.query.clone().take(),
            fragment: ctx.fragment.clone().take(),
        })
    }

    /// require(X) from module at path Y
    /// X: specifier
    /// Y: path
    fn require(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &ResolveContext,
    ) -> Result<CachedPath, ResolveError> {
        let specifier = Specifier::parse(specifier).map_err(ResolveError::Specifier)?;
        ctx.with_query_fragment(specifier.query, specifier.fragment);

        // enhanced_resolve: try alias
        if let Some(path) =
            self.load_alias(cached_path, specifier.path.as_str(), &self.options.alias, ctx)?
        {
            return Ok(path);
        }

        match specifier.path {
            // 1. If X is a core module,
            //    a. return the core module
            //    b. STOP
            // 2. If X begins with '/'
            //    a. set Y to be the file system root
            SpecifierPath::Absolute(absolute_path) => {
                self.require_absolute(cached_path, absolute_path, ctx)
            }
            // 3. If X begins with './' or '/' or '../'
            SpecifierPath::Relative(relative_path) => {
                self.require_relative(cached_path, relative_path, ctx)
            }
            // 4. If X begins with '#'
            SpecifierPath::Hash(specifier) => {
                // a. LOAD_PACKAGE_IMPORTS(X, dirname(Y))
                self.require_hash(cached_path, specifier, ctx)
            }
            // (ESM) 5. Otherwise,
            // Note: specifier is now a bare specifier.
            // Set resolved the result of PACKAGE_RESOLVE(specifier, parentURL).
            SpecifierPath::Bare(bare_specifier) => {
                self.require_bare(cached_path, bare_specifier, ctx)
            }
        }
    }

    fn require_absolute(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &ResolveContext,
    ) -> Result<CachedPath, ResolveError> {
        debug_assert!(specifier.starts_with('/'));
        if !self.options.prefer_relative && self.options.prefer_absolute {
            if let Ok(path) = self.load_package_self_or_node_modules(cached_path, specifier, ctx) {
                return Ok(path);
            }
        }
        if self.options.roots.is_empty() {
            let cached_path = self.cache.value(Path::new("/"));
            return self.load_package_self_or_node_modules(&cached_path, specifier, ctx);
        }
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

    // 3. If X begins with './' or '/' or '../'
    fn require_relative(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &ResolveContext,
    ) -> Result<CachedPath, ResolveError> {
        let path = cached_path.path().normalize_with(specifier);
        let cached_path = self.cache.value(&path);
        // a. LOAD_AS_FILE(Y + X)
        if !specifier.ends_with('/') {
            if let Some(path) = self.load_as_file(&cached_path, ctx)? {
                return Ok(path);
            }
        }
        // b. LOAD_AS_DIRECTORY(Y + X)
        if let Some(path) = self.load_as_directory(&cached_path, ctx)? {
            return Ok(path);
        }
        // c. THROW "not found"
        Err(ResolveError::NotFound(path))
    }

    fn require_hash(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &ResolveContext,
    ) -> Result<CachedPath, ResolveError> {
        let cached_path = self.cache.dirname(cached_path);
        if let Some(path) = self.load_package_imports(&cached_path, specifier, ctx)? {
            return Ok(path);
        }
        self.load_package_self_or_node_modules(&cached_path, specifier, ctx)
    }

    fn require_bare(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &ResolveContext,
    ) -> Result<CachedPath, ResolveError> {
        if self.options.prefer_relative {
            if let Ok(path) = self.require_relative(cached_path, specifier, ctx) {
                return Ok(path);
            }
        }
        self.load_package_self_or_node_modules(cached_path, specifier, ctx)
    }

    fn load_package_self_or_node_modules(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &ResolveContext,
    ) -> Result<CachedPath, ResolveError> {
        let (_, subpath) = Self::parse_package_specifier(specifier);
        if subpath.is_empty() {
            ctx.with_fully_specified(false);
        }
        let dirname = self.cache.dirname(cached_path);
        // 5. LOAD_PACKAGE_SELF(X, dirname(Y))
        if let Some(path) = self.load_package_self(&dirname, specifier, ctx)? {
            return Ok(path);
        }
        // 6. LOAD_NODE_MODULES(X, dirname(Y))
        if let Some(path) = self.load_node_modules(&dirname, specifier, ctx)? {
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
        ctx: &ResolveContext,
    ) -> ResolveState {
        // 1. Find the closest package scope SCOPE to DIR.
        // 2. If no scope was found, return.
        let Some(package_json) = cached_path.find_package_json(&self.cache.fs)? else {
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

    fn load_as_file(&self, cached_path: &CachedPath, ctx: &ResolveContext) -> ResolveState {
        // enhanced-resolve feature: extension_alias
        if let Some(path) = self.load_extension_alias(cached_path, ctx)? {
            return Ok(Some(path));
        }
        // 1. If X is a file, load X as its file extension format. STOP
        if let Some(path) = self.load_alias_or_file(cached_path, ctx)? {
            return Ok(Some(path));
        }
        // 2. If X.js is a file, load X.js as JavaScript text. STOP
        // 3. If X.json is a file, parse X.json to a JavaScript Object. STOP
        // 4. If X.node is a file, load X.node as binary addon. STOP
        if let Some(path) = self.load_extensions(cached_path, &self.options.extensions, ctx)? {
            return Ok(Some(path));
        }
        Ok(None)
    }

    fn load_extensions(
        &self,
        cached_path: &CachedPath,
        extensions: &[String],
        ctx: &ResolveContext,
    ) -> ResolveState {
        if ctx.borrow().fully_specified {
            return Ok(None);
        }
        let mut path_with_extension = cached_path.path().to_path_buf();
        for extension in extensions {
            path_with_extension.set_extension(extension);
            let cached_path = self.cache.value(&path_with_extension);
            if let Some(path) = self.load_alias_or_file(&cached_path, ctx)? {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    fn load_realpath(&self, cached_path: &CachedPath) -> Option<PathBuf> {
        if self.options.symlinks {
            cached_path.symlink(&self.cache.fs)
        } else {
            None
        }
    }

    fn load_index(&self, cached_path: &CachedPath, ctx: &ResolveContext) -> ResolveState {
        for main_file in &self.options.main_files {
            let main_path = cached_path.path().join(main_file);
            let cached_path = self.cache.value(&main_path);
            if self.options.enforce_extension == Some(false) {
                if let Some(path) = self.load_alias_or_file(&cached_path, ctx)? {
                    return Ok(Some(path));
                }
            }
            // 1. If X/index.js is a file, load X/index.js as JavaScript text. STOP
            // 2. If X/index.json is a file, parse X/index.json to a JavaScript object. STOP
            // 3. If X/index.node is a file, load X/index.node as binary addon. STOP
            if let Some(path) = self.load_extensions(&cached_path, &self.options.extensions, ctx)? {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    fn load_alias_or_file(&self, cached_path: &CachedPath, ctx: &ResolveContext) -> ResolveState {
        if let Some(package_json) = cached_path.find_package_json(&self.cache.fs)? {
            let path = cached_path.path();
            if let Some(path) = self.load_browser_field(path, None, &package_json, ctx)? {
                return Ok(Some(path));
            }
        }
        if cached_path.is_file(&self.cache.fs) {
            return Ok(Some(cached_path.clone()));
        }
        Ok(None)
    }

    fn load_as_directory(&self, cached_path: &CachedPath, ctx: &ResolveContext) -> ResolveState {
        // TODO: Only package.json is supported, so warn about having other values
        // Checking for empty files is needed for omitting checks on package.json
        // 1. If X/package.json is a file,
        if !self.options.description_files.is_empty() {
            // a. Parse X/package.json, and look for "main" field.
            if let Some(package_json) = cached_path.package_json(&self.cache.fs).transpose()? {
                // b. If "main" is a falsy value, GOTO 2.
                if let Some(main_field) = &package_json.main {
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
                    // f. LOAD_INDEX(X) DEPRECATED
                    // g. THROW "not found"
                    return Err(ResolveError::NotFound(main_field_path));
                }
            }
        }
        // 2. LOAD_INDEX(X)
        self.load_index(cached_path, ctx)
    }

    fn load_node_modules(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &ResolveContext,
    ) -> ResolveState {
        // 1. let DIRS = NODE_MODULES_PATHS(START)
        // Use a buffer to reduce total memory allocation.
        let mut node_module_path = cached_path.path().to_path_buf();
        // 2. for each DIR in DIRS:
        loop {
            for module_name in &self.options.modules {
                node_module_path.push(module_name);
                // a. LOAD_PACKAGE_EXPORTS(X, DIR)
                if let Some(path) = self.load_package_exports(&node_module_path, specifier, ctx)? {
                    return Ok(Some(path));
                }

                // Using `join` because `specifier` can be `/` separated.
                let node_module_file = node_module_path.join(specifier);
                let cached_path = self.cache.value(&node_module_file);
                // b. LOAD_AS_FILE(DIR/X)
                if !specifier.ends_with('/') {
                    if let Some(path) = self.load_as_file(&cached_path, ctx)? {
                        return Ok(Some(path));
                    }
                }
                // c. LOAD_AS_DIRECTORY(DIR/X)
                if cached_path.is_dir(&self.cache.fs) {
                    if let Some(path) = self.load_as_directory(&cached_path, ctx)? {
                        return Ok(Some(path));
                    }
                }
                node_module_path.pop();
            }

            if !node_module_path.pop() {
                break;
            }
        }
        Ok(None)
    }

    fn load_package_exports(
        &self,
        path: &Path,
        specifier: &str,
        ctx: &ResolveContext,
    ) -> ResolveState {
        // 1. Try to interpret X as a combination of NAME and SUBPATH where the name
        //    may have a @scope/ prefix and the subpath begins with a slash (`/`).
        // 2. If X does not match this pattern or DIR/NAME/package.json is not a file,
        //    return.
        let (name, subpath) = Self::parse_package_specifier(specifier);
        let cached_path = self.cache.value(&path.join(name));
        let Some(package_json) = cached_path.package_json(&self.cache.fs).transpose()? else {
            return Ok(None);
        };
        // 3. Parse DIR/NAME/package.json, and look for "exports" field.
        // 4. If "exports" is null or undefined, return.
        if package_json.exports.is_none() {
            return Ok(None);
        };
        // 5. let MATCH = PACKAGE_EXPORTS_RESOLVE(pathToFileURL(DIR/NAME), "." + SUBPATH,
        //    `package.json` "exports", ["node", "require"]) defined in the ESM resolver.
        // Note: The subpath is not prepended with a dot on purpose
        let Some(path) = self.package_exports_resolve(
            cached_path.path(),
            subpath,
            &package_json.exports,
            &self.options.condition_names,
            ctx
        )? else {
            return Ok(None)
        };
        // 6. RESOLVE_ESM_MATCH(MATCH)
        self.resolve_esm_match(&path, &package_json, ctx)
    }

    fn load_package_self(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &ResolveContext,
    ) -> ResolveState {
        // 1. Find the closest package scope SCOPE to DIR.
        // 2. If no scope was found, return.
        let Some(package_json) = cached_path.find_package_json(&self.cache.fs)? else {
            return Ok(None);
        };
        // 3. If the SCOPE/package.json "exports" is null or undefined, return.
        if package_json.exports.is_none() {
            return self.load_browser_field(
                cached_path.path(),
                Some(specifier),
                &package_json,
                ctx,
            );
        }
        // 4. If the SCOPE/package.json "name" is not the first segment of X, return.
        let Some(subpath) = package_json.name.as_ref().and_then(|package_json_name| package_json_name.strip_prefix(specifier)) else {
            return Ok(None);
        };
        // 5. let MATCH = PACKAGE_EXPORTS_RESOLVE(pathToFileURL(SCOPE),
        // "." + X.slice("name".length), `package.json` "exports", ["node", "require"])
        // defined in the ESM resolver.
        let package_url = package_json.directory();
        // Note: The subpath is not prepended with a dot on purpose
        // because `package_exports_resolve` matches subpath without the leading dot.
        let Some(cached_path) = self.package_exports_resolve(
            package_url,
            subpath,
            &package_json.exports,
            &self.options.condition_names,
            ctx
        )? else {
            return Ok(None);
        };
        // 6. RESOLVE_ESM_MATCH(MATCH)
        self.resolve_esm_match(&cached_path, &package_json, ctx)
    }

    /// RESOLVE_ESM_MATCH(MATCH)
    fn resolve_esm_match(
        &self,
        cached_path: &CachedPath,
        package_json: &PackageJson,
        ctx: &ResolveContext,
    ) -> ResolveState {
        if let Some(path) = self.load_browser_field(cached_path.path(), None, package_json, ctx)? {
            return Ok(Some(path));
        }
        // 1. let RESOLVED_PATH = fileURLToPath(MATCH)
        // 2. If the file at RESOLVED_PATH exists, load RESOLVED_PATH as its extension
        if let Some(path) = self.load_as_file(cached_path, ctx)? {
            return Ok(Some(path));
        }
        // format. STOP
        // 3. THROW "not found"
        Err(ResolveError::NotFound(cached_path.to_path_buf()))
    }

    fn load_browser_field(
        &self,
        path: &Path,
        specifier: Option<&str>,
        package_json: &PackageJson,
        ctx: &ResolveContext,
    ) -> ResolveState {
        if !self.options.alias_fields.iter().any(|field| field == "browser") {
            return Ok(None);
        }
        let Some(specifier) = package_json.resolve_browser_field(path, specifier)? else {
            return Ok(None);
        };
        let cached_path = self.cache.value(package_json.directory());
        let ctx = ResolveContext::clone_from(ctx);
        self.require(&cached_path, specifier, &ctx).map(Some)
    }

    fn load_alias(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        alias: &Alias,
        ctx: &ResolveContext,
    ) -> ResolveState {
        for (alias, specifiers) in alias {
            let exact_match = alias.strip_prefix(specifier).is_some_and(|c| c == "$");
            if !(specifier.starts_with(alias) || exact_match) {
                continue;
            }
            for r in specifiers {
                match r {
                    AliasValue::Path(new_specifier) => {
                        if new_specifier.starts_with(specifier) {
                            continue;
                        }
                        let new_specifier = if exact_match {
                            Cow::Borrowed(new_specifier)
                        } else {
                            Cow::Owned(specifier.replacen(alias, new_specifier, 1))
                        };
                        let ctx = ResolveContext::clone_from(ctx);
                        match self.require(cached_path, &new_specifier, &ctx) {
                            Err(ResolveError::NotFound(_)) => { /* noop */ }
                            Ok(path) => return Ok(Some(path)),
                            Err(err) => return Err(err),
                        }
                    }
                    AliasValue::Ignore => {
                        return Err(ResolveError::Ignored(cached_path.path().join(alias)));
                    }
                }
            }
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
    fn load_extension_alias(&self, cached_path: &CachedPath, ctx: &ResolveContext) -> ResolveState {
        let Some(path_extension) = cached_path.path().extension() else { return Ok(None) };
        let Some((_, extensions)) =
            self.options.extension_alias.iter().find(|(ext, _)| OsStr::new(ext) == path_extension)
        else {
            return Ok(None);
        };
        if let Some(path) =
            self.load_extensions(cached_path, extensions, &ResolveContext::clone_from(ctx))?
        {
            return Ok(Some(path));
        }
        Err(ResolveError::ExtensionAlias)
    }

    /// PACKAGE_RESOLVE(packageSpecifier, parentURL)
    fn package_resolve(
        &self,
        cached_path: &CachedPath,
        specifier: &str,
        ctx: &ResolveContext,
    ) -> ResolveState {
        let (name, subpath) = Self::parse_package_specifier(specifier);
        // 9. Let selfUrl be the result of PACKAGE_SELF_RESOLVE(packageName, packageSubpath, parentURL).
        if let Some(path) = self.package_self_resolve(name, subpath, cached_path, ctx)? {
            // 10. If selfUrl is not undefined, return selfUrl.
            return Ok(Some(path));
        }
        // 11. While parentURL is not the file system root,
        let mut parent_url = cached_path.path().to_path_buf();
        loop {
            for module_name in &self.options.modules {
                // 1. Let packageURL be the URL resolution of "node_modules/" concatenated with packageSpecifier, relative to parentURL.
                parent_url.push(module_name);
                let package_path = parent_url.join(name);
                // 2. Set parentURL to the parent folder URL of parentURL.
                let cached_path = self.cache.value(&package_path);
                // 3. If the folder at packageURL does not exist, then
                //   1. Continue the next loop iteration.
                if cached_path.is_dir(&self.cache.fs) {
                    // 4. Let pjson be the result of READ_PACKAGE_JSON(packageURL).
                    if let Some(package_json) =
                        cached_path.package_json(&self.cache.fs).transpose()?
                    {
                        // 5. If pjson is not null and pjson.exports is not null or undefined, then
                        if !package_json.exports.is_none() {
                            // 1. Return the result of PACKAGE_EXPORTS_RESOLVE(packageURL, packageSubpath, pjson.exports, defaultConditions).
                            return self.package_exports_resolve(
                                cached_path.path(),
                                subpath,
                                &package_json.exports,
                                &self.options.condition_names,
                                ctx,
                            );
                        }
                        // 6. Otherwise, if packageSubpath is equal to ".", then
                        if subpath == "." {
                            // 1. If pjson.main is a string, then
                            if let Some(main_field) = &package_json.main {
                                // 1. Return the URL resolution of main in packageURL.
                                let path = cached_path.path().normalize_with(main_field);
                                let value = self.cache.value(&path);
                                return Ok(Some(value));
                            }
                        }
                    }
                    let subpath = format!(".{subpath}");
                    return self
                        .require(&cached_path, &subpath, &ResolveContext::clone_from(ctx))
                        .map(Some);
                }
                parent_url.pop();
            }
            if !parent_url.pop() {
                break;
            }
        }

        Err(ResolveError::NotFound(cached_path.to_path_buf()))
    }

    fn package_self_resolve(
        &self,
        package_name: &str,
        package_subpath: &str,
        parent_url: &CachedPath,
        ctx: &ResolveContext,
    ) -> ResolveState {
        let Some(package_json) = parent_url.find_package_json(&self.cache.fs)? else {
            return Ok(None);
        };
        if package_json.exports.is_none() {
            // enhanced_resolve: try browser field
            return self.load_browser_field(
                parent_url.path(),
                Some(package_subpath),
                &package_json,
                ctx,
            );
        }
        if package_json
            .name
            .as_ref()
            .is_some_and(|package_json_name| package_json_name == package_name)
        {
            return self.package_exports_resolve(
                package_json.directory(),
                package_subpath,
                &package_json.exports,
                &self.options.condition_names,
                ctx,
            );
        }
        Ok(None)
    }

    /// PACKAGE_EXPORTS_RESOLVE(packageURL, subpath, exports, conditions)
    fn package_exports_resolve(
        &self,
        package_url: &Path,
        subpath: &str,
        exports: &ExportsField,
        conditions: &[String],
        ctx: &ResolveContext,
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
            let match_key = subpath;
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
        ctx: &ResolveContext,
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
        if let Some(package_json) = cached_path.find_package_json(&self.cache.fs)? {
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
        ctx: &ResolveContext,
    ) -> ResolveState {
        // enhanced_resolve behaves differently, it throws
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
        ctx: &ResolveContext,
    ) -> ResolveState {
        fn normalize_string_target<'a>(
            target_key: &'a str,
            target: &'a str,
            pattern_match: Option<&'a str>,
            package_url: &Path,
        ) -> Result<Cow<'a, str>, ResolveError> {
            let target = if let Some(pattern_match) = pattern_match {
                if !target_key.contains('*') && !target.contains('*') {
                    // enhanced_resolve behaviour
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
                    // Note: node.js does not throw this but enhanced_resolve does.
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
}
