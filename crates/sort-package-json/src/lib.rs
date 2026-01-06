use serde_json::{Map, Value};

/// Options for controlling JSON formatting when sorting
#[derive(Debug, Clone)]
pub struct SortOptions {
    /// Whether to pretty-print the output JSON
    pub pretty: bool,
    /// Whether to sort the scripts field alphabetically
    pub sort_scripts: bool,
}

impl Default for SortOptions {
    fn default() -> Self {
        Self { pretty: true, sort_scripts: false }
    }
}

/// Sorts a package.json string with custom options
pub fn sort_package_json_with_options(
    input: &str,
    options: &SortOptions,
) -> Result<String, serde_json::Error> {
    let value: Value = serde_json::from_str(input)?;

    let sorted_value = if let Value::Object(obj) = value {
        Value::Object(sort_object_keys(obj, options))
    } else {
        value
    };

    let result = if options.pretty {
        let mut s = serde_json::to_string_pretty(&sorted_value)?;
        s.push('\n');
        s
    } else {
        serde_json::to_string(&sorted_value)?
    };

    Ok(result)
}

/// Sorts a package.json string with default options (pretty-printed)
pub fn sort_package_json(input: &str) -> Result<String, serde_json::Error> {
    sort_package_json_with_options(input, &SortOptions::default())
}

/// Declares package.json field ordering with transformations.
///
/// This macro generates a match statement that handles known package.json fields
/// in a specific order using explicit indices. It supports optional transformation
/// expressions for fields that need special processing.
///
/// # Usage
///
/// ```ignore
/// declare_field_order!(key, value, known, non_private, private; [
///     0 => "$schema",
///     1 => "name",
///     7 => "categories" => transform_array(&value, sort_array_unique),
/// ]);
/// ```
///
/// # Parameters
///
/// - `key`: The field name identifier
/// - `value`: The field value identifier
/// - `known`: The vector to push known fields to
/// - `non_private`: The vector to push non-private unknown fields to
/// - `private`: The vector to push private (underscore-prefixed) fields to
/// - Followed by an array of field declarations in the format:
///   - `index => "field_name"` for fields without transformation
///   - `index => "field_name" => transformation_expr` for fields with transformation
macro_rules! declare_field_order {
    (
        $key:ident, $value:ident, $known:ident, $non_private:ident, $private:ident;
        [
            $( $idx:literal => $field_name:literal $( => $transform:expr )? ),* $(,)?
        ]
    ) => {
        {
            // Compile-time validation: ensure indices are literals
            $( let _ = $idx; )*

            // Generate the match statement
            match $key.as_str() {
                $(
                    $field_name => {
                        $known.push((
                            $idx,
                            $key,
                            declare_field_order!(@value $value $(, $transform)?)
                        ));
                    },
                )*
                _ => {
                    // Unknown field - check if private
                    if $key.starts_with('_') {
                        $private.push(($key, $value));
                    } else {
                        $non_private.push(($key, $value));
                    }
                }
            }
        }
    };

    // Helper: extract value without transformation
    (@value $value:ident) => { $value };

    // Helper: extract value with transformation
    (@value $value:ident, $transform:expr) => { $transform };
}

fn transform_value<F>(value: Value, transform: F) -> Value
where
    F: FnOnce(Map<String, Value>) -> Map<String, Value>,
{
    match value {
        Value::Object(o) => Value::Object(transform(o)),
        _ => value,
    }
}

fn transform_array<F>(value: Value, transform: F) -> Value
where
    F: FnOnce(Vec<Value>) -> Vec<Value>,
{
    match value {
        Value::Array(arr) => Value::Array(transform(arr)),
        _ => value,
    }
}

fn transform_with_key_order(value: Value, key_order: &[&str]) -> Value {
    transform_value(value, |o| sort_object_by_key_order(o, key_order))
}

fn sort_object_alphabetically(obj: Map<String, Value>) -> Map<String, Value> {
    let mut entries: Vec<(String, Value)> = obj.into_iter().collect();
    entries.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
    entries.into_iter().collect()
}

fn sort_object_recursive(obj: Map<String, Value>) -> Map<String, Value> {
    let mut entries: Vec<(String, Value)> = obj.into_iter().collect();
    entries.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

    entries
        .into_iter()
        .map(|(key, value)| {
            let transformed_value = match value {
                Value::Object(nested) => Value::Object(sort_object_recursive(nested)),
                _ => value,
            };
            (key, transformed_value)
        })
        .collect()
}

fn sort_array_unique(mut arr: Vec<Value>) -> Vec<Value> {
    // Filter non-strings in-place (same behavior as filter_map)
    arr.retain(|v| v.is_string());

    // Sort in-place by comparing string values (zero allocations)
    arr.sort_unstable_by(|a, b| a.as_str().unwrap().cmp(b.as_str().unwrap()));

    // Remove consecutive duplicates in-place
    arr.dedup_by(|a, b| a.as_str() == b.as_str());

    arr
}

fn sort_paths_naturally(mut arr: Vec<Value>) -> Vec<Value> {
    // Filter and deduplicate in-place
    arr.retain(|v| v.is_string());
    arr.sort_unstable_by(|a, b| a.as_str().unwrap().cmp(b.as_str().unwrap()));
    arr.dedup_by(|a, b| a.as_str() == b.as_str());

    // Pre-compute depth and lowercase ONCE per string (not on every comparison)
    // Move Values from arr into tuples (no copying)
    let mut with_keys: Vec<(usize, String, Value)> = arr
        .into_iter()
        .map(|v| {
            let s = v.as_str().unwrap();
            let depth = s.matches('/').count();
            let lowercase = s.to_lowercase();
            (depth, lowercase, v)
        })
        .collect();

    // Sort using pre-computed keys (zero allocations during comparison)
    with_keys.sort_unstable_by(|(depth_a, lower_a, _), (depth_b, lower_b, _)| {
        depth_a.cmp(depth_b).then_with(|| lower_a.cmp(lower_b))
    });

    // Extract Values (move out of tuples, no copying)
    with_keys.into_iter().map(|(_, _, v)| v).collect()
}

fn sort_object_by_key_order(mut obj: Map<String, Value>, key_order: &[&str]) -> Map<String, Value> {
    // Pre-allocate capacity to avoid reallocations
    let mut result = Map::with_capacity(obj.len());

    // Add keys in specified order
    for &key in key_order {
        if let Some(value) = obj.remove(key) {
            result.insert(key.into(), value);
        }
    }

    // Add remaining keys alphabetically
    let mut remaining: Vec<(String, Value)> = obj.into_iter().collect();
    remaining.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

    for (key, value) in remaining {
        result.insert(key, value);
    }

    result
}

fn sort_people_object(obj: Map<String, Value>) -> Map<String, Value> {
    sort_object_by_key_order(obj, &["name", "email", "url"])
}

fn sort_exports(obj: Map<String, Value>) -> Map<String, Value> {
    let mut paths = Vec::new();
    let mut types_conds = Vec::new();
    let mut other_conds = Vec::new();
    let mut default_cond = None;

    for (key, value) in obj {
        if key.starts_with('.') {
            paths.push((key, value));
        } else if key == "default" {
            default_cond = Some((key, value));
        } else if key == "types" || key.starts_with("types@") {
            types_conds.push((key, value));
        } else {
            other_conds.push((key, value));
        }
    }

    let mut result = Map::new();

    // Add in order: paths, types, others, default
    for (key, value) in paths {
        let transformed = match value {
            Value::Object(nested) => Value::Object(sort_exports(nested)),
            _ => value,
        };
        result.insert(key, transformed);
    }

    for (key, value) in types_conds {
        let transformed = match value {
            Value::Object(nested) => Value::Object(sort_exports(nested)),
            _ => value,
        };
        result.insert(key, transformed);
    }

    for (key, value) in other_conds {
        let transformed = match value {
            Value::Object(nested) => Value::Object(sort_exports(nested)),
            _ => value,
        };
        result.insert(key, transformed);
    }

    if let Some((key, value)) = default_cond {
        let transformed = match value {
            Value::Object(nested) => Value::Object(sort_exports(nested)),
            _ => value,
        };
        result.insert(key, transformed);
    }

    result
}

fn sort_object_keys(obj: Map<String, Value>, options: &SortOptions) -> Map<String, Value> {
    // Storage for categorized keys with their values and ordering information
    let mut known: Vec<(usize, String, Value)> = Vec::new(); // (order_index, key, value)
    let mut non_private: Vec<(String, Value)> = Vec::new();
    let mut private: Vec<(String, Value)> = Vec::new();

    // Single pass through all keys using into_iter()
    for (key, value) in obj {
        declare_field_order!(key, value, known, non_private, private; [
            // Core Package Metadata
            0 => "$schema",
            1 => "name",
            2 => "displayName",
            3 => "version",
            4 => "stableVersion",
            5 => "gitHead",
            6 => "private",
            7 => "description",
            8 => "categories" => transform_array(value, sort_array_unique),
            9 => "keywords" => transform_array(value, sort_array_unique),
            10 => "homepage",
            11 => "bugs" => transform_with_key_order(value, &["url", "email"]),
            // License & People
            12 => "license",
            13 => "author" => transform_value(value, sort_people_object),
            14 => "maintainers",
            15 => "contributors",
            // Repository & Funding
            16 => "repository" => transform_with_key_order(value, &["type", "url"]),
            17 => "funding" => transform_with_key_order(value, &["type", "url"]),
            18 => "donate" => transform_with_key_order(value, &["type", "url"]),
            19 => "sponsor" => transform_with_key_order(value, &["type", "url"]),
            20 => "qna",
            21 => "publisher",
            // Package Content & Distribution
            22 => "man",
            23 => "style",
            24 => "example",
            25 => "examplestyle",
            26 => "assets",
            27 => "bin" => transform_value(value, sort_object_alphabetically),
            28 => "source",
            29 => "directories" => transform_with_key_order(value, &["lib", "bin", "man", "doc", "example", "test"]),
            30 => "workspaces",
            31 => "binary" => transform_with_key_order(value, &["module_name", "module_path", "remote_path", "package_name", "host"]),
            32 => "files" => transform_array(value, sort_paths_naturally),
            33 => "os",
            34 => "cpu",
            35 => "libc" => transform_array(value, sort_array_unique),
            // Package Entry Points
            36 => "type",
            37 => "sideEffects",
            38 => "main",
            39 => "module",
            40 => "browser",
            41 => "types",
            42 => "typings",
            43 => "typesVersions",
            44 => "typeScriptVersion",
            45 => "typesPublisherContentHash",
            46 => "react-native",
            47 => "svelte",
            48 => "unpkg",
            49 => "jsdelivr",
            50 => "jsnext:main",
            51 => "umd",
            52 => "umd:main",
            53 => "es5",
            54 => "esm5",
            55 => "fesm5",
            56 => "es2015",
            57 => "esm2015",
            58 => "fesm2015",
            59 => "es2020",
            60 => "esm2020",
            61 => "fesm2020",
            62 => "esnext",
            63 => "imports",
            64 => "exports" => transform_value(value, sort_exports),
            65 => "publishConfig" => transform_value(value, sort_object_alphabetically),
            // Scripts
            66 => "scripts" => if options.sort_scripts { transform_value(value, sort_object_alphabetically) } else { value },
            67 => "betterScripts" => if options.sort_scripts { transform_value(value, sort_object_alphabetically) } else { value },
            // Dependencies
            68 => "dependencies" => transform_value(value, sort_object_alphabetically),
            69 => "devDependencies" => transform_value(value, sort_object_alphabetically),
            70 => "dependenciesMeta",
            71 => "peerDependencies" => transform_value(value, sort_object_alphabetically),
            72 => "peerDependenciesMeta",
            73 => "optionalDependencies" => transform_value(value, sort_object_alphabetically),
            74 => "bundledDependencies" => transform_array(value, sort_array_unique),
            75 => "bundleDependencies" => transform_array(value, sort_array_unique),
            76 => "resolutions" => transform_value(value, sort_object_alphabetically),
            77 => "overrides" => transform_value(value, sort_object_alphabetically),
            // Git Hooks & Commit Tools
            78 => "husky" => transform_value(value, sort_object_recursive),
            79 => "simple-git-hooks",
            80 => "pre-commit",
            81 => "lint-staged",
            82 => "nano-staged",
            83 => "commitlint" => transform_value(value, sort_object_recursive),
            // VSCode Extension Specific
            84 => "l10n",
            85 => "contributes",
            86 => "activationEvents" => transform_array(value, sort_array_unique),
            87 => "extensionPack" => transform_array(value, sort_array_unique),
            88 => "extensionDependencies" => transform_array(value, sort_array_unique),
            89 => "extensionKind" => transform_array(value, sort_array_unique),
            90 => "icon",
            91 => "badges",
            92 => "galleryBanner",
            93 => "preview",
            94 => "markdown",
            // Build & Tool Configuration
            95 => "napi" => transform_value(value, sort_object_alphabetically),
            96 => "flat",
            97 => "config" => transform_value(value, sort_object_alphabetically),
            98 => "nodemonConfig" => transform_value(value, sort_object_recursive),
            99 => "browserify" => transform_value(value, sort_object_recursive),
            100 => "babel" => transform_value(value, sort_object_recursive),
            101 => "browserslist",
            102 => "xo" => transform_value(value, sort_object_recursive),
            103 => "prettier" => transform_value(value, sort_object_recursive),
            104 => "eslintConfig" => transform_value(value, sort_object_recursive),
            105 => "eslintIgnore",
            106 => "standard" => transform_value(value, sort_object_recursive),
            107 => "npmpkgjsonlint",
            108 => "npmPackageJsonLintConfig",
            109 => "npmpackagejsonlint",
            110 => "release",
            111 => "auto-changelog" => transform_value(value, sort_object_recursive),
            112 => "remarkConfig" => transform_value(value, sort_object_recursive),
            113 => "stylelint" => transform_value(value, sort_object_recursive),
            114 => "typescript" => transform_value(value, sort_object_recursive),
            115 => "typedoc" => transform_value(value, sort_object_recursive),
            116 => "tshy" => transform_value(value, sort_object_recursive),
            117 => "tsdown" => transform_value(value, sort_object_recursive),
            118 => "size-limit",
            // Testing
            119 => "ava" => transform_value(value, sort_object_recursive),
            120 => "jest" => transform_value(value, sort_object_recursive),
            121 => "jest-junit",
            122 => "jest-stare",
            123 => "mocha" => transform_value(value, sort_object_recursive),
            124 => "nyc" => transform_value(value, sort_object_recursive),
            125 => "c8" => transform_value(value, sort_object_recursive),
            126 => "tap",
            127 => "tsd" => transform_value(value, sort_object_recursive),
            128 => "typeCoverage" => transform_value(value, sort_object_recursive),
            129 => "oclif" => transform_value(value, sort_object_recursive),
            // Runtime & Package Manager
            130 => "languageName",
            131 => "preferGlobal",
            132 => "devEngines" => transform_value(value, sort_object_alphabetically),
            133 => "engines" => transform_value(value, sort_object_alphabetically),
            134 => "engineStrict",
            135 => "volta" => transform_value(value, sort_object_recursive),
            136 => "packageManager",
            137 => "pnpm",
        ]);
    }

    // Sort each category (using unstable sort for better performance)
    known.sort_unstable_by_key(|(index, _, _)| *index);
    non_private.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
    private.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

    // Build result map
    let mut result = Map::new();

    // Insert known fields (already transformed)
    for (_index, key, value) in known {
        result.insert(key, value);
    }

    // Insert non-private unknown fields
    for (key, value) in non_private {
        result.insert(key, value);
    }

    // Insert private fields
    for (key, value) in private {
        result.insert(key, value);
    }

    result
}
