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

use std::{
    borrow::Cow,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::{
    cache::{Cache, CacheValue},
    file_system::FileSystemOs,
    package_json::{ExportsField, ExportsKey, MatchObject, PackageJson},
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
        match request.path {
            // 1. If X is a core module,
            //    a. return the core module
            //    b. STOP
            // 2. If X begins with '/'
            //    a. set Y to be the file system root
            RequestPath::Absolute(absolute_path) => {
                if !self.options.prefer_relative && self.options.prefer_absolute {
                    if let Ok(path) = self.require_path(cache_value, absolute_path) {
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
            RequestPath::Hash(hash_path) => self.require_path(cache_value, hash_path),
            //    a. LOAD_PACKAGE_IMPORTS(X, dirname(Y))
            RequestPath::Module(module_path) => self.require_path(cache_value, module_path),
        }
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

    fn require_path(
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
        let cache_value = self.cache.value(&cache_value.path().join(request));
        if let Some(path) = self.load_as_file(&cache_value)? {
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
    fn parse_package_specifier(request: &str) -> (&str, &str) {
        if let Some((module, request)) = request.split_once('/') {
            (module, request)
        } else {
            (request, "")
        }
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
        let Some(exports) = &package_json.exports else { return Ok(None) };
        // 5. let MATCH = PACKAGE_EXPORTS_RESOLVE(pathToFileURL(DIR/NAME), "." + SUBPATH,
        //    `package.json` "exports", ["node", "require"]) defined in the ESM resolver.
        // Note: The subpath is not prepended with a dot on purpose
        if let Some(path) = self.package_exports_resolve(
            cache_value.path(),
            subpath,
            exports,
            &self.options.condition_names,
        )? {
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
        if let Some(exports) = &package_json.exports {
            // 4. If the SCOPE/package.json "name" is not the first segment of X, return.
            if let Some(package_name) = &package_json.name {
                if let Some(subpath) = package_name.strip_prefix(request) {
                    // 5. let MATCH = PACKAGE_EXPORTS_RESOLVE(pathToFileURL(SCOPE),
                    // "." + X.slice("name".length), `package.json` "exports", ["node", "require"])
                    // defined in the ESM resolver.
                    let path = package_json.path.parent().unwrap();
                    // Note: The subpath is not prepended with a dot on purpose
                    if let Some(path) = self.package_exports_resolve(
                        path,
                        subpath,
                        exports,
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
        if let Some(request) = package_json.resolve(path, request)? {
            let request = Request::parse(request).map_err(ResolveError::Request)?;
            debug_assert!(package_json.path.file_name().is_some_and(|x| x == "package.json"));
            // TODO: Do we need to pass query and fragment?
            let cache_value = self.cache.value(package_json.path.parent().unwrap());
            return self.require(&cache_value, &request).map(Some);
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
            return self.require_path(&cache_value, request);
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
    ///
    /// <https://nodejs.org/api/esm.html#resolution-algorithm-specification>
    #[allow(clippy::single_match)]
    fn package_exports_resolve(
        &self,
        package_url: &Path,
        subpath: &str,
        exports: &ExportsField,
        conditions: &[String],
    ) -> ResolveState {
        // 1. If exports is an Object with both a key starting with "." and a key not starting with ".", throw an Invalid Package Configuration error.
        // 2. If subpath is equal to ".", then
        // Note: subpath is not prepended with a dot when passed in.
        if subpath.is_empty() {
            //   1. Let mainExport be undefined.
            //   2. If exports is a String or Array, or an Object containing no keys starting with ".", then
            //     1. Set mainExport to exports.
            //   3. Otherwise if exports is an Object containing a "." property, then
            //     1. Set mainExport to exports["."].
            //   4. If mainExport is not undefined, then
            //     1. Let resolved be the result of PACKAGE_TARGET_RESOLVE( packageURL, mainExport, null, false, conditions).
            //     2. If resolved is not null or undefined, return resolved.
            match exports {
                ExportsField::Map(map) => match map.get(&ExportsKey::Main) {
                    Some(ExportsField::String(value)) => {
                        // TODO: PACKAGE_TARGET_RESOLVE
                        let path = package_url.normalize_with(value);
                        let path = self.cache.value(&path);
                        return Ok(Some(path));
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        // 3. Otherwise, if exports is an Object and all keys of exports start with ".", then
        if let ExportsField::Map(exports) = exports {
            //   1. Let matchKey be the string "./" concatenated with subpath.
            // let match_key =

            //   2. Let resolved be the result of PACKAGE_IMPORTS_EXPORTS_RESOLVE( matchKey, exports, packageURL, false, conditions).
            if let Some(path) =
                self.package_imports_exports_resolve(subpath, exports, package_url, conditions)?
            {
                return Ok(Some(path));
            }
            //   3. If resolved is not null or undefined, return resolved.
        }
        // 4. Throw a Package Path Not Exported error.
        Ok(None)
    }

    /// PACKAGE_IMPORTS_EXPORTS_RESOLVE(matchKey, matchObj, packageURL, isImports, conditions)
    fn package_imports_exports_resolve(
        &self,
        match_key: &str,
        match_obj: &MatchObject,
        package_url: &Path,
        conditions: &[String],
    ) -> ResolveState {
        // 1. If matchKey is a key of matchObj and does not contain "*", then
        //   1. Let target be the value of matchObj[matchKey].
        //   2. Return the result of PACKAGE_TARGET_RESOLVE(packageURL, target, null, isImports, conditions).
        // 2. Let expansionKeys be the list of keys of matchObj containing only a single "*", sorted by the sorting function PATTERN_KEY_COMPARE which orders in descending order of specificity.
        // 3. For each key expansionKey in expansionKeys, do
        for (key, target) in match_obj {
            if let ExportsKey::Pattern(key) = key {
                //   1. Let patternBase be the substring of expansionKey up to but excluding the first "*" character.
                //   2. If matchKey starts with but is not equal to patternBase, then
                if let Some(pattern_match) = match_key.strip_prefix(key) {
                    //     1. Let patternTrailer be the substring of expansionKey from the index after the first "*" character.
                    //     2. If patternTrailer has zero length, or if matchKey ends with patternTrailer and the length of matchKey is greater than or equal to the length of expansionKey, then
                    //       1. Let target be the value of matchObj[expansionKey].
                    //       2. Let patternMatch be the substring of matchKey starting at the index of the length of patternBase up to the length of matchKey minus the length of patternTrailer.
                    //       3. Return the result of PACKAGE_TARGET_RESOLVE(packageURL, target, patternMatch, isImports, conditions).
                    return self.package_target_resolve(
                        package_url,
                        target,
                        pattern_match,
                        conditions,
                    );
                }
            }
        }
        // Return null.
        Ok(None)
    }

    /// PACKAGE_TARGET_RESOLVE(packageURL, target, patternMatch, isImports, conditions)
    fn package_target_resolve(
        &self,
        package_url: &Path,
        target: &ExportsField,
        pattern_match: &str,
        conditions: &[String],
    ) -> ResolveState {
        // 1. If target is a String, then
        match target {
            ExportsField::String(target) => {
                //   1. If target does not start with "./", then
                //     1. If isImports is false, or if target starts with "../" or "/", or if target is a valid URL, then
                //       1. Throw an Invalid Package Target error.
                //     2. If patternMatch is a String, then
                //       1. Return PACKAGE_RESOLVE(target with every instance of "*" replaced by patternMatch, packageURL + "/").
                //     3. Return PACKAGE_RESOLVE(target, packageURL + "/").
                //   2. If target split on "/" or "\" contains any "", ".", "..", or "node_modules" segments after the first "." segment, case insensitive and including percent encoded variants, throw an Invalid Package Target error.
                //   3. Let resolvedTarget be the URL resolution of the concatenation of packageURL and target.
                //   4. Assert: resolvedTarget is contained in packageURL.
                //   5. If patternMatch is null, then
                //     1. Return resolvedTarget.
                //   6. If patternMatch split on "/" or "\" contains any "", ".", "..", or "node_modules" segments, case insensitive and including percent encoded variants, throw an Invalid Module Specifier error.
                //   7. Return the URL resolution of resolvedTarget with every instance of "*" replaced with patternMatch.
                return Ok(Some(self.cache.value(&package_url.join(target).join(pattern_match))));
            }
            // 2. Otherwise, if target is a non-null Object, then
            ExportsField::Map(target) => {
                //   1. If exports contains any index property keys, as defined in ECMA-262 6.1.7 Array Index, throw an Invalid Package Configuration error.
                //   2. For each property p of target, in object insertion order as,
                for (key, target_value) in target {
                    //     1. If p equals "default" or conditions contains an entry for p, then
                    if matches!(key, ExportsKey::CustomCondition(condition) if conditions.contains(condition))
                    {
                        //       1. Let targetValue be the value of the p property in target.
                        //       2. Let resolved be the result of PACKAGE_TARGET_RESOLVE( packageURL, targetValue, patternMatch, isImports, conditions).
                        //       3. If resolved is equal to undefined, continue the loop.
                        //       4. Return resolved.
                        if let Some(path) = self.package_target_resolve(
                            package_url,
                            target_value,
                            pattern_match,
                            conditions,
                        )? {
                            return Ok(Some(path));
                        }
                    }
                    //   3. Return undefined.
                }
            } // 3. Otherwise, if target is an Array, then
            ExportsField::Array(targets) => {
                //   1. If _target.length is zero, return null.
                //   2. For each item targetValue in target, do
                for target_value in targets {
                    //     1. Let resolved be the result of PACKAGE_TARGET_RESOLVE( packageURL, targetValue, patternMatch, isImports, conditions), continuing the loop on any Invalid Package Target error.
                    //     2. If resolved is undefined, continue the loop.
                    if let Some(path) = self.package_target_resolve(
                        package_url,
                        target_value,
                        pattern_match,
                        conditions,
                    )? {
                        return Ok(Some(path));
                    }
                    //     3. Return resolved.
                    //   3. Return or throw the last fallback resolution null return or error.
                }
            }
        }
        // 4. Otherwise, if target is null, return null.
        // 5. Otherwise throw an Invalid Package Target error.
        Ok(None)
    }
}
