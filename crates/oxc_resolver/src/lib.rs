//! # Oxc Resolver
//!
//! Node.js Module Resolution.
//!
//! ## References:
//!
//! * Tests ported from [enhanced-resolve](https://github.com/webpack/enhanced-resolve)
//! * Algorithm adapted from [Node.js Module Resolution Algorithm](https://nodejs.org/api/modules.html#all-together) and [cjs loader](https://github.com/nodejs/node/blob/main/lib/internal/modules/cjs/loader.js)
//! * Some code adapted from [parcel-resolver](https://github.com/parcel-bundler/parcel/blob/v2/packages/utils/node-resolver-rs)

mod cache;
mod error;
mod file_system;
mod options;
mod package_json;
mod path;
mod request;
mod resolution;

#[cfg(test)]
mod tests;

use std::{
    borrow::Cow,
    cmp::Ordering,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::{
    cache::{Cache, CacheValue},
    file_system::FileSystemOs,
    package_json::{ExportsField, MatchObject},
    package_json::{ExportsKey, PackageJson},
    path::PathUtil,
    request::{Request, RequestPath},
};
pub use crate::{
    error::{JSONError, ResolveError},
    file_system::{FileMetadata, FileSystem},
    options::{Alias, AliasValue, ResolveOptions},
    resolution::Resolution,
};

type ResolveState = Result<Option<CacheValue>, ResolveError>;

/// Resolver with the current operating system as the file system
pub type Resolver = ResolverGeneric<FileSystemOs>;

/// Generic implementation of the resolver, backed by a cached file system.
pub struct ResolverGeneric<Fs> {
    options: ResolveOptions,
    cache: Cache<Fs>,
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

    /// Resolve `request` at `path`
    ///
    /// # Errors
    ///
    /// * See [ResolveError]
    pub fn resolve<P: AsRef<Path>>(
        &self,
        path: P,
        request: &str,
    ) -> Result<Resolution, ResolveError> {
        let path = path.as_ref();
        let request = Request::parse(request).map_err(ResolveError::Request)?;
        let cache_value = self.cache.value(path);
        let cache_value = if let Some(path) =
            self.load_alias(&cache_value, request.path.as_str(), &self.options.alias)?
        {
            path
        } else {
            let result = self.require(&cache_value, &request);
            if result.as_ref().is_err_and(ResolveError::is_not_found) {
                if let Some(path) =
                    self.load_alias(&cache_value, request.path.as_str(), &self.options.fallback)?
                {
                    path
                } else {
                    result?
                }
            } else {
                result?
            }
        };
        let path = self.load_symlink(&cache_value).unwrap_or_else(|| cache_value.to_path_buf());
        Ok(Resolution {
            path,
            query: request.query.map(ToString::to_string),
            fragment: request.fragment.map(ToString::to_string),
        })
    }

    /// require(X) from module at path Y
    /// X: request
    /// Y: path
    fn require(
        &self,
        cache_value: &CacheValue,
        request: &Request,
    ) -> Result<CacheValue, ResolveError> {
        let path = match request.path {
            // 1. If X is a core module,
            //    a. return the core module
            //    b. STOP
            // 2. If X begins with '/'
            //    a. set Y to be the file system root
            RequestPath::Absolute(absolute_path) => {
                if !self.options.prefer_relative && self.options.prefer_absolute {
                    if let Ok(path) = self.package_resolve(cache_value, absolute_path) {
                        return Ok(path);
                    }
                }
                self.load_roots(cache_value, absolute_path)
            }
            // 3. If X begins with './' or '/' or '../'
            RequestPath::Relative(relative_path) => {
                self.require_relative(cache_value, relative_path)
            }
            // 4. If X begins with '#'
            RequestPath::Hash(specifier) => {
                // a. LOAD_PACKAGE_IMPORTS(X, dirname(Y))
                self.package_imports_resolve(cache_value, specifier)
            }
            // (ESM) 5. Otherwise,
            // Note: specifier is now a bare specifier.
            // Set resolved the result of PACKAGE_RESOLVE(specifier, parentURL).
            RequestPath::Bare(bare_specifier) => {
                if self.options.prefer_relative {
                    if let Ok(path) = self.require_relative(cache_value, bare_specifier) {
                        return Ok(path);
                    }
                }
                self.package_resolve(cache_value, bare_specifier)
            }
        }?;

        if !path.is_file(&self.cache.fs) {
            // TODO: Throw a Module Not Found error. Or better error message
            return Err(ResolveError::NotFound(path.to_path_buf().into_boxed_path()));
        }

        Ok(path)
    }

    // 3. If X begins with './' or '/' or '../'
    fn require_relative(
        &self,
        cache_value: &CacheValue,
        request: &str,
    ) -> Result<CacheValue, ResolveError> {
        let path = cache_value.path().normalize_with(request);
        let cache_value = self.cache.value(&path);
        // a. LOAD_AS_FILE(Y + X)
        if !request.ends_with('/') {
            if let Some(path) = self.load_as_file(&cache_value)? {
                return Ok(path);
            }
        }
        // b. LOAD_AS_DIRECTORY(Y + X)
        if let Some(path) = self.load_as_directory(&cache_value)? {
            return Ok(path);
        }
        // c. THROW "not found"
        Err(ResolveError::NotFound(path.into_boxed_path()))
    }

    /// PACKAGE_RESOLVE(packageSpecifier, parentURL)
    fn package_resolve(
        &self,
        cache_value: &CacheValue,
        request: &str,
    ) -> Result<CacheValue, ResolveError> {
        let dirname = self.cache.dirname(cache_value);
        // 5. LOAD_PACKAGE_SELF(X, dirname(Y))
        if let Some(path) = self.load_package_self(dirname, request)? {
            return Ok(path);
        }
        // 6. LOAD_NODE_MODULES(X, dirname(Y))
        if let Some(path) = self.load_node_modules(dirname, request)? {
            return Ok(path);
        }
        // 7. THROW "not found"
        Err(ResolveError::NotFound(cache_value.to_path_buf().into_boxed_path()))
    }

    fn load_as_file(&self, cache_value: &CacheValue) -> ResolveState {
        // enhanced-resolve feature: extension_alias
        if let Some(path) = self.load_extension_alias(cache_value)? {
            return Ok(Some(path));
        }
        // 1. If X is a file, load X as its file extension format. STOP
        // let cache_value = self.cache.cache_value(&path);
        if let Some(path) = self.load_alias_or_file(cache_value)? {
            return Ok(Some(path));
        }
        // 2. If X.js is a file, load X.js as JavaScript text. STOP
        // 3. If X.json is a file, parse X.json to a JavaScript Object. STOP
        // 4. If X.node is a file, load X.node as binary addon. STOP
        if let Some(path) = self.load_extensions(cache_value, &self.options.extensions)? {
            return Ok(Some(path));
        }
        Ok(None)
    }

    fn load_extensions(&self, cache_value: &CacheValue, extensions: &[String]) -> ResolveState {
        let mut path_with_extension = cache_value.path().to_path_buf();
        for extension in extensions {
            path_with_extension.set_extension(extension);
            let cache_value = self.cache.value(&path_with_extension);
            if let Some(path) = self.load_alias_or_file(&cache_value)? {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    fn load_symlink(&self, cache_value: &CacheValue) -> Option<PathBuf> {
        if self.options.symlinks {
            cache_value.symlink(&self.cache.fs)
        } else {
            None
        }
    }

    fn load_index(&self, cache_value: &CacheValue) -> ResolveState {
        for main_file in &self.options.main_files {
            let main_path = cache_value.path().join(main_file);
            let cache_value = self.cache.value(&main_path);
            if self.options.enforce_extension == Some(false) {
                if let Some(path) = self.load_alias_or_file(&cache_value)? {
                    return Ok(Some(path));
                }
            }
            // 1. If X/index.js is a file, load X/index.js as JavaScript text. STOP
            // 2. If X/index.json is a file, parse X/index.json to a JavaScript object. STOP
            // 3. If X/index.node is a file, load X/index.node as binary addon. STOP
            if let Some(path) = self.load_extensions(&cache_value, &self.options.extensions)? {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    fn load_alias_or_file(&self, cache_value: &CacheValue) -> ResolveState {
        if let Some(package_json) = cache_value.find_package_json(&self.cache.fs)? {
            let path = cache_value.path();
            if let Some(path) = self.load_browser_field(path, None, &package_json)? {
                return Ok(Some(path));
            }
        }
        if cache_value.is_file(&self.cache.fs) {
            return Ok(Some(cache_value.clone()));
        }
        Ok(None)
    }

    fn load_as_directory(&self, cache_value: &CacheValue) -> ResolveState {
        // TODO: Only package.json is supported, so warn about having other values
        // Checking for empty files is needed for omitting checks on package.json
        // 1. If X/package.json is a file,
        if !self.options.description_files.is_empty() {
            // a. Parse X/package.json, and look for "main" field.
            if let Some(package_json) = cache_value.package_json(&self.cache.fs).transpose()? {
                // b. If "main" is a falsy value, GOTO 2.
                if let Some(main_field) = &package_json.main {
                    // c. let M = X + (json main field)
                    let main_field_path = cache_value.path().normalize_with(main_field);
                    // d. LOAD_AS_FILE(M)
                    let cache_value = self.cache.value(&main_field_path);
                    if let Some(path) = self.load_as_file(&cache_value)? {
                        return Ok(Some(path));
                    }
                    // e. LOAD_INDEX(M)
                    if let Some(path) = self.load_index(&cache_value)? {
                        return Ok(Some(path));
                    }
                    // f. LOAD_INDEX(X) DEPRECATED
                    // g. THROW "not found"
                    return Err(ResolveError::NotFound(main_field_path.into_boxed_path()));
                }

                if let Some(path) = self.load_index(cache_value)? {
                    return Ok(Some(path));
                }
            }
        }
        // 2. LOAD_INDEX(X)
        self.load_index(cache_value)
    }

    fn load_node_modules(&self, cache_value: &CacheValue, request: &str) -> ResolveState {
        // 1. let DIRS = NODE_MODULES_PATHS(START)
        // Use a buffer to reduce total memory allocation.
        let mut node_module_path = cache_value.path().to_path_buf();
        // 2. for each DIR in DIRS:
        loop {
            for module_name in &self.options.modules {
                node_module_path.push(module_name);
                // a. LOAD_PACKAGE_EXPORTS(X, DIR)
                if let Some(path) = self.load_package_exports(&node_module_path, request)? {
                    return Ok(Some(path));
                }

                // Using `join` because `request` can be `/` separated.
                let node_module_file = node_module_path.join(request);
                let cache_value = self.cache.value(&node_module_file);
                // b. LOAD_AS_FILE(DIR/X)
                if !request.ends_with('/') {
                    if let Some(path) = self.load_as_file(&cache_value)? {
                        return Ok(Some(path));
                    }
                }
                // c. LOAD_AS_DIRECTORY(DIR/X)
                if cache_value.is_dir(&self.cache.fs) {
                    if let Some(path) = self.load_as_directory(&cache_value)? {
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

    fn load_package_exports(&self, path: &Path, request: &str) -> ResolveState {
        // 1. Try to interpret X as a combination of NAME and SUBPATH where the name
        //    may have a @scope/ prefix and the subpath begins with a slash (`/`).
        // 2. If X does not match this pattern or DIR/NAME/package.json is not a file,
        //    return.
        let (name, subpath) = Self::parse_package_specifier(request);
        let cache_value = self.cache.value(&path.join(name));
        let Some(package_json) = cache_value.package_json(&self.cache.fs).transpose()? else {
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
        if let Some(path) = self.package_exports_resolve(
            cache_value.path(),
            subpath,
            &package_json.exports,
            &self.options.condition_names,
        )? {
            if let Some(path) = self.load_browser_field(path.path(), None, &package_json)? {
                return Ok(Some(path));
            }
            return Ok(Some(path));
        }
        // 6. RESOLVE_ESM_MATCH(MATCH)
        Ok(None)
    }

    fn load_package_self(&self, cache_value: &CacheValue, request: &str) -> ResolveState {
        // 1. Find the closest package scope SCOPE to DIR.
        // 2. If no scope was found, return.
        let Some(package_json) = cache_value.find_package_json(&self.cache.fs)? else {
            return Ok(None);
        };
        // 3. If the SCOPE/package.json "exports" is null or undefined, return.
        if !package_json.exports.is_none() {
            // 4. If the SCOPE/package.json "name" is not the first segment of X, return.
            if let Some(package_name) = &package_json.name {
                if let Some(subpath) = package_name.strip_prefix(request) {
                    // 5. let MATCH = PACKAGE_EXPORTS_RESOLVE(pathToFileURL(SCOPE),
                    // "." + X.slice("name".length), `package.json` "exports", ["node", "require"])
                    // defined in the ESM resolver.
                    let package_url = package_json.path.parent().unwrap();
                    // Note: The subpath is not prepended with a dot on purpose
                    // because `package_exports_resolve` matches subpath without the leading dot.
                    if let Some(path) = self.package_exports_resolve(
                        package_url,
                        subpath,
                        &package_json.exports,
                        &self.options.condition_names,
                    )? {
                        return Ok(Some(path));
                    }
                }
            }
        }
        // 6. RESOLVE_ESM_MATCH(MATCH)

        // Try non-spec-compliant "browser" field since its another form of export
        self.load_browser_field(cache_value.path(), Some(request), &package_json)
    }

    fn load_browser_field(
        &self,
        path: &Path,
        request: Option<&str>,
        package_json: &PackageJson,
    ) -> ResolveState {
        if self.options.alias_fields.iter().any(|field| field == "browser") {
            if let Some(request) = package_json.resolve(path, request)? {
                let request = Request::parse(request).map_err(ResolveError::Request)?;
                debug_assert!(package_json.path.file_name().is_some_and(|x| x == "package.json"));
                // TODO: Do we need to pass query and fragment?
                let cache_value = self.cache.value(package_json.path.parent().unwrap());
                return self.require(&cache_value, &request).map(Some);
            }
        }
        Ok(None)
    }

    fn load_alias(&self, cache_value: &CacheValue, request: &str, alias: &Alias) -> ResolveState {
        for (alias, requests) in alias {
            let exact_match = alias.strip_prefix(request).is_some_and(|c| c == "$");
            if request.starts_with(alias) || exact_match {
                for r in requests {
                    match r {
                        AliasValue::Path(new_request) => {
                            let new_request = if exact_match {
                                Cow::Borrowed(new_request)
                            } else {
                                Cow::Owned(request.replacen(alias, new_request, 1))
                            };
                            let new_request =
                                Request::parse(&new_request).map_err(ResolveError::Request)?;
                            match self.require(cache_value, &new_request) {
                                Err(ResolveError::NotFound(_)) => { /* noop */ }
                                Ok(path) => return Ok(Some(path)),
                                Err(err) => return Err(err),
                            }
                        }
                        AliasValue::Ignore => {
                            return Err(ResolveError::Ignored(
                                cache_value.path().join(alias).into_boxed_path(),
                            ));
                        }
                    }
                }
                return Err(ResolveError::Alias(alias.clone()));
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
    fn load_extension_alias(&self, cache_value: &CacheValue) -> ResolveState {
        let Some(path_extension) = cache_value.path().extension() else { return Ok(None) };
        let Some((_, extensions)) =
            self.options.extension_alias.iter().find(|(ext, _)| OsStr::new(ext) == path_extension)
        else {
            return Ok(None);
        };
        if let Some(path) = self.load_extensions(cache_value, extensions)? {
            return Ok(Some(path));
        }
        Err(ResolveError::ExtensionAlias)
    }

    fn load_roots(
        &self,
        cache_value: &CacheValue,
        request: &str,
    ) -> Result<CacheValue, ResolveError> {
        debug_assert!(request.starts_with('/'));
        if self.options.roots.is_empty() {
            let cache_value = self.cache.value(Path::new("/"));
            return self.package_resolve(&cache_value, request);
        }
        for root in &self.options.roots {
            let cache_value = self.cache.value(root);
            if let Ok(path) = self.require_relative(&cache_value, request.trim_start_matches('/')) {
                return Ok(path);
            }
        }
        Err(ResolveError::NotFound(cache_value.to_path_buf().into_boxed_path()))
    }

    /// PACKAGE_EXPORTS_RESOLVE(packageURL, subpath, exports, conditions)
    fn package_exports_resolve(
        &self,
        package_url: &Path,
        subpath: &str,
        exports: &ExportsField,
        conditions: &[String],
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
        cache_value: &CacheValue,
        specifier: &str,
    ) -> Result<CacheValue, ResolveError> {
        // 1. Assert: specifier begins with "#".
        debug_assert!(specifier.starts_with('#'), "{specifier}");
        // 2. If specifier is exactly equal to "#" or starts with "#/", then
        if specifier == "#" || specifier.starts_with("#/") {
            // 1. Throw an Invalid Module Specifier error.
            return Err(ResolveError::InvalidModuleSpecifier(specifier.to_string()));
        }
        // 3. Let packageURL be the result of LOOKUP_PACKAGE_SCOPE(parentURL).
        // 4. If packageURL is not null, then
        if let Some(package_json) = cache_value.find_package_json(&self.cache.fs)? {
            // 1. Let pjson be the result of READ_PACKAGE_JSON(packageURL).
            // 2. If pjson.imports is a non-null Object, then
            if !package_json.imports.is_empty() {
                // 1. Let resolved be the result of PACKAGE_IMPORTS_EXPORTS_RESOLVE( specifier, pjson.imports, packageURL, true, conditions).
                let package_url = package_json.path.parent().unwrap();
                if let Some(path) = self.package_imports_exports_resolve(
                    specifier,
                    &package_json.imports,
                    package_url,
                    /* is_imports */ true,
                    &self.options.condition_names,
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
    ) -> ResolveState {
        // enhanced_resolve behaves differently, it throws
        // Error: Resolving to directories is not possible with the exports field (request was ./dist/)
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
            );
        }
        // 4. Return null.
        Ok(None)
    }

    /// PACKAGE_TARGET_RESOLVE(packageURL, target, patternMatch, isImports, conditions)
    fn package_target_resolve(
        &self,
        package_url: &Path,
        target_key: &str,
        target: &ExportsField,
        pattern_match: Option<&str>,
        is_imports: bool,
        conditions: &[String],
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
                    return self.package_resolve(&package_url, &target).map(Some);
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
                return Ok(Some(self.cache.value(&resolved_target)));
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
