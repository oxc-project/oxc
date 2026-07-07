//! Browserslist target resolution for the `compat/compat` rule.
//!
//! Port of `determineTargetsFromConfig` and `parseBrowsersListVersion` from
//! eslint-plugin-compat's `src/helpers.ts`, on top of `oxc-browserslist`.

use std::cmp::Ordering;

/// A resolved browser target, e.g. `chrome 50`.
#[derive(Debug, Clone, PartialEq)]
pub struct BrowserTarget {
    /// browserslist target id, e.g. `"chrome"`, `"ios_saf"`, `"op_mini"`.
    pub target: String,
    /// The raw version string, e.g. `"50"`, `"10.0-10.2"`, `"all"`, `"TP"`.
    pub version: String,
    /// Parsed numeric version. `"all"` parses to `0`, ranges parse to the
    /// lower bound (`"10.0-10.2"` -> `10.0`).
    pub parsed_version: f64,
}

/// Resolve browserslist queries into `"name version"` strings.
///
/// Mirrors `determineTargetsFromConfig`: when no queries are configured, the
/// browserslist `defaults` query is used. (Browserslist config-file discovery
/// from `package.json`/`.browserslistrc` is not supported.)
pub fn determine_targets_from_config(
    queries: &[String],
) -> Result<Vec<String>, browserslist::Error> {
    let distribs = if queries.is_empty() {
        browserslist::resolve(&["defaults"], &browserslist::Opts::default())?
    } else {
        browserslist::resolve(queries, &browserslist::Opts::default())?
    };
    Ok(distribs.into_iter().map(|distrib| distrib.to_string()).collect())
}

/// Parse the version part of a browserslist target, mirroring the reference's
/// use of JavaScript's `parseFloat`.
fn parse_version(version: &str) -> f64 {
    if version == "all" {
        return 0.0;
    }
    let version = version.split('-').next().unwrap_or(version);
    parse_float(version)
}

/// A minimal port of JavaScript's `parseFloat`: parses the longest numeric
/// prefix, returning `NaN` if there is none.
fn parse_float(s: &str) -> f64 {
    let s = s.trim_start();
    let mut end = 0;
    let bytes = s.as_bytes();
    if end < bytes.len() && (bytes[end] == b'+' || bytes[end] == b'-') {
        end += 1;
    }
    let mut seen_dot = false;
    while end < bytes.len() {
        let b = bytes[end];
        if b.is_ascii_digit() {
            end += 1;
        } else if b == b'.' && !seen_dot {
            seen_dot = true;
            end += 1;
        } else {
            break;
        }
    }
    s[..end].parse::<f64>().unwrap_or(f64::NAN)
}

/// Parse the versions given by browserslist (`["chrome 50", ...]`) into the
/// lowest version of each target, sorted by target name in descending order.
///
/// Port of `parseBrowsersListVersion` from eslint-plugin-compat.
pub fn parse_browserslist_version(targets: &[String]) -> Vec<BrowserTarget> {
    let mut parsed: Vec<BrowserTarget> = targets
        .iter()
        .map(|entry| {
            let (target, version) = entry.split_once(' ').unwrap_or((entry.as_str(), ""));
            BrowserTarget {
                target: target.to_string(),
                version: version.to_string(),
                parsed_version: parse_version(version),
            }
        })
        .collect();

    // Sort by target name and then version number in descending order.
    parsed.sort_by(|a, b| match b.target.cmp(&a.target) {
        Ordering::Equal => {
            b.parsed_version.partial_cmp(&a.parsed_version).unwrap_or(Ordering::Equal)
        }
        ordering => ordering,
    });

    // Keep only the last entry of each target: the lowest version.
    let mut result = Vec::with_capacity(parsed.len());
    for i in 0..parsed.len() {
        if i + 1 == parsed.len() || parsed[i].target != parsed[i + 1].target {
            result.push(parsed[i].clone());
        }
    }
    result
}

#[cfg(test)]
mod test {
    use super::{BrowserTarget, determine_targets_from_config, parse_browserslist_version};

    fn to_strings(queries: &[&str]) -> Vec<String> {
        queries.iter().map(ToString::to_string).collect()
    }

    fn resolve(queries: &[&str]) -> Vec<BrowserTarget> {
        parse_browserslist_version(&determine_targets_from_config(&to_strings(queries)).unwrap())
    }

    fn find<'t>(targets: &'t [BrowserTarget], name: &str) -> Option<&'t BrowserTarget> {
        targets.iter().find(|t| t.target == name)
    }

    fn assert_unique_targets(targets: &[BrowserTarget]) {
        let mut names: Vec<&str> = targets.iter().map(|t| t.target.as_str()).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(len_before, names.len(), "each target should appear only once");
    }

    // Port of "should support multi env config in browserslist package.json"
    // from eslint-plugin-compat's test/helpers.spec.ts (multi-config.package.json).
    #[test]
    fn multi_env_config() {
        let targets = resolve(&[
            // production
            ">1%",
            "last 4 versions",
            "Firefox ESR",
            "not ie < 9",
            // development
            "last 1 chrome version",
        ]);
        assert_unique_targets(&targets);
        // `last 4 versions` keeps ie 8..11 and `not ie < 9` drops ie 8,
        // leaving ie 9 as the lowest version.
        let ie = find(&targets, "ie").unwrap();
        assert_eq!(ie.version, "9");
        assert!((ie.parsed_version - 9.0).abs() < f64::EPSILON);
        // The only version of op_mini is `all`, which parses to 0.
        let op_mini = find(&targets, "op_mini").unwrap();
        assert_eq!(op_mini.version, "all");
        assert!(op_mini.parsed_version.abs() < f64::EPSILON);
        // Targets are sorted by name in descending order.
        let mut sorted = targets.clone();
        sorted.sort_by(|a, b| b.target.cmp(&a.target));
        assert_eq!(
            targets.iter().map(|t| t.target.clone()).collect::<Vec<_>>(),
            sorted.iter().map(|t| t.target.clone()).collect::<Vec<_>>()
        );
    }

    // Port of "should support single array config in browserslist package.json"
    // (single-array-config.package.json).
    #[test]
    fn single_array_config() {
        let targets = resolve(&[
            ">1%",
            "last 4 versions",
            "last 10 chrome version",
            "Firefox ESR",
            "not ie < 9",
        ]);
        assert_unique_targets(&targets);
        assert_eq!(find(&targets, "ie").unwrap().version, "9");
        // `last 10 chrome version` widens the chrome range at least as far as
        // `last 4 versions` (usage queries like `>1%` may widen both).
        let chrome = find(&targets, "chrome").unwrap();
        let multi = resolve(&[">1%", "last 4 versions", "Firefox ESR", "not ie < 9"]);
        assert!(chrome.parsed_version <= find(&multi, "chrome").unwrap().parsed_version);
    }

    // Port of "should support single version config in browserslist package.json"
    // (single-version-config.package.json + its inline snapshot).
    #[test]
    fn single_version_config() {
        let targets = resolve(&["chrome 32", "firefox 20", "safari 8", "ie 9"]);
        let expected: Vec<(&str, &str, f64)> = vec![
            ("safari", "8", 8.0),
            ("ie", "9", 9.0),
            ("firefox", "20", 20.0),
            ("chrome", "32", 32.0),
        ];
        assert_eq!(targets.len(), expected.len());
        for (target, (name, version, parsed)) in targets.iter().zip(expected) {
            assert_eq!(target.target, name);
            assert_eq!(target.version, version);
            assert!((target.parsed_version - parsed).abs() < f64::EPSILON);
        }
    }

    // Port of "should get lowest target versions" and its snapshot.
    #[test]
    fn lowest_target_versions() {
        let targets = parse_browserslist_version(&to_strings(&[
            "chrome 20",
            "chrome 30",
            "node 7",
            "chrome 30.5",
            "firefox 50.5",
        ]));
        let expected: Vec<(&str, &str, f64)> =
            vec![("node", "7", 7.0), ("firefox", "50.5", 50.5), ("chrome", "20", 20.0)];
        assert_eq!(targets.len(), expected.len());
        for (target, (name, version, parsed)) in targets.iter().zip(expected) {
            assert_eq!(target.target, name);
            assert_eq!(target.version, version);
            assert!((target.parsed_version - parsed).abs() < f64::EPSILON);
        }
    }

    // Port of "should support string config in rule option".
    #[test]
    fn string_config() {
        let targets = resolve(&["defaults, not ie < 9"]);
        assert!(!targets.is_empty());
        assert_unique_targets(&targets);
        if let Some(ie) = find(&targets, "ie") {
            assert!(ie.parsed_version >= 9.0);
        }
    }

    // Port of "should fail on incorrect browserslist target version".
    // Note: oxc-browserslist words the error as
    // "unknown version '100000' of browser 'edge'" instead of the JS
    // implementation's "Unknown version 100000 of edge".
    #[test]
    fn unknown_browser_version() {
        let error = determine_targets_from_config(&to_strings(&["edge 100000"])).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("100000"), "unexpected error message: {message}");
        assert!(message.contains("edge"), "unexpected error message: {message}");
    }

    // No configured targets resolve to the browserslist `defaults` query.
    #[test]
    fn empty_config_uses_defaults() {
        let targets = resolve(&[]);
        assert!(!targets.is_empty());
        assert_unique_targets(&targets);
    }

    // Range versions (e.g. iOS "10.0-10.2") parse to their lower bound.
    #[test]
    fn range_versions() {
        let targets = resolve(&["ios 10"]);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].target, "ios_saf");
        assert!(targets[0].version.contains('-'), "expected a range version");
        assert!((targets[0].parsed_version - 10.0).abs() < f64::EPSILON);
    }
}
