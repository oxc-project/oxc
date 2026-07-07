//! Support lookups for the `compat/compat` rule: ports of the CanIUse and MDN
//! providers from eslint-plugin-compat (`src/providers/*.ts`).

use super::{
    data::{COMPAT_DATA, MdnApi, target_display_name},
    targets::BrowserTarget,
};

/// Formats a target for display, e.g. `IE 8`, `iOS Safari 10.0-10.2`.
pub fn format_target_name(target: &BrowserTarget) -> String {
    format!("{} {}", target_display_name(&target.target), target.version)
}

fn version_is_range(version: &str) -> bool {
    version.contains('-')
}

/// `parseFloat`-based equality used for caniuse range versions.
#[expect(clippy::float_cmp)] // mirrors the reference's `===` on parsed floats
fn are_versions_equal(target_version: f64, stats_version: &str) -> bool {
    let parsed = stats_version.split('-').next().unwrap_or(stats_version);
    // Mirrors `parseFloat(statsVersion)`; caniuse version keys always start
    // with a number (or are `"all"`/`"TP"`, which never equal a numeric
    // target version).
    parsed.parse::<f64>().is_ok_and(|v| v == target_version)
}

/// Check the caniuse database to see if a target is supported.
///
/// If no record could be found, returns `true`: rules might not be found
/// because they belong to another provider.
pub fn caniuse_is_supported(feature_id: &str, target: &BrowserTarget) -> bool {
    let Some(stats) = COMPAT_DATA.caniuse.get(feature_id) else {
        return true;
    };
    let Some(target_stats) = stats.get(target.target.as_str()) else {
        return true;
    };

    if version_is_range(&target.version) {
        return target_stats.iter().any(|(stats_version, supported)| {
            if version_is_range(stats_version)
                && are_versions_equal(target.parsed_version, stats_version)
            {
                !supported
            } else {
                true
            }
        });
    }

    match target_stats.iter().find(|(version, _)| *version == target.version) {
        Some((_, supported)) => *supported,
        // This assumes that all versions are included in the caniuse db; if a
        // version is missing, the target is treated as supported (mirroring
        // the reference implementation).
        None => true,
    }
}

/// Return all unsupported targets for a caniuse feature, formatted for
/// display.
pub fn caniuse_unsupported_targets(feature_id: &str, targets: &[BrowserTarget]) -> Vec<String> {
    targets
        .iter()
        .filter(|target| !caniuse_is_supported(feature_id, target))
        .map(format_target_name)
        .collect()
}

/// A minimal port of `semver.coerce` (plus eslint-plugin-compat's
/// `customCoerce`): extracts the first `major(.minor(.patch)?)?` number
/// sequence found in the string.
fn coerce_version(version: &str) -> Option<[u64; 3]> {
    let bytes = version.as_bytes();
    let start = bytes.iter().position(u8::is_ascii_digit)?;
    let mut components = [0u64; 3];
    let mut component = 0;
    let mut i = start;
    let mut digits = 0;
    while i < bytes.len() && component < 3 {
        let b = bytes[i];
        if b.is_ascii_digit() {
            components[component] =
                components[component].saturating_mul(10).saturating_add(u64::from(b - b'0'));
            digits += 1;
            i += 1;
        } else if b == b'.' && digits > 0 && component < 2 {
            component += 1;
            digits = 0;
            i += 1;
        } else {
            break;
        }
    }
    Some(components)
}

/// Return whether MDN data says the target supports the API.
///
/// Port of `isSupportedByMDN` from `src/providers/mdn-provider.ts`.
pub fn mdn_is_supported(api: &MdnApi, target: &BrowserTarget) -> bool {
    // If no support record could be found for this target, the API is treated
    // as supported: the record might belong to another provider.
    let Some(&version_added) = api.support.get(target.target.as_str()) else {
        return true;
    };
    // Empty string encodes `version_added: false` (never supported).
    if version_added.is_empty() {
        return false;
    }

    // Special case for Safari TP: TP is always gte than any other release.
    if target.target == "safari" {
        if target.version == "TP" {
            return true;
        }
        if version_added == "TP" {
            return false;
        }
    }

    let Some(current) = coerce_version(&target.version) else {
        // Non-semver target version (e.g. `op_mini all`); treat as supported.
        return true;
    };
    let Some(added) = coerce_version(version_added) else {
        return false;
    };

    current >= added
}

/// Return all unsupported targets for an MDN API, formatted for display.
pub fn mdn_unsupported_targets(api: &MdnApi, targets: &[BrowserTarget]) -> Vec<String> {
    targets.iter().filter(|target| !mdn_is_supported(api, target)).map(format_target_name).collect()
}

#[cfg(test)]
mod test {
    use super::{
        super::targets::{
            BrowserTarget, determine_targets_from_config, parse_browserslist_version,
        },
        COMPAT_DATA, MdnApi, caniuse_unsupported_targets, mdn_unsupported_targets,
    };

    fn resolve(queries: &[&str]) -> Vec<BrowserTarget> {
        let queries: Vec<String> = queries.iter().map(ToString::to_string).collect();
        parse_browserslist_version(&determine_targets_from_config(&queries).unwrap())
    }

    fn mdn_api(proto_chain_id: &str) -> &'static MdnApi {
        COMPAT_DATA
            .mdn
            .iter()
            .find(|api| api.proto_chain_id == proto_chain_id)
            .expect("MDN record should exist in the embedded dataset")
    }

    // Port of test/caniuse-provider.spec.ts: "should return unsupported iOS
    // targets with range value for Fetch API" (snapshot: `[]`).
    #[test]
    fn caniuse_fetch_ios_range() {
        let targets = resolve(&["ios 10"]);
        assert_eq!(targets.len(), 1);
        assert!(targets[0].version.contains('-'), "expected a range version for ios 10");
        let result = caniuse_unsupported_targets("fetch", &targets);
        assert_eq!(result, Vec::<String>::new());
    }

    // caniuse: exact version lookups (`y` vs `n` flags).
    #[test]
    fn caniuse_exact_versions() {
        let ie9 = resolve(&["ie 9"]);
        assert_eq!(caniuse_unsupported_targets("fetch", &ie9), vec!["IE 9".to_string()]);
        assert_eq!(caniuse_unsupported_targets("promises", &ie9), vec!["IE 9".to_string()]);
        // Unknown feature ids and browsers missing from the stats are
        // treated as supported (they may belong to another provider).
        assert_eq!(caniuse_unsupported_targets("not-a-feature", &ie9), Vec::<String>::new());
        let node = resolve(&["node 10"]);
        assert_eq!(caniuse_unsupported_targets("promises", &node), Vec::<String>::new());
    }

    // Port of test/mdn-provider.spec.ts: "should support Safari TP".
    #[test]
    fn mdn_safari_tp() {
        let targets = resolve(&["safari tp"]);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].version, "TP");
        let result = mdn_unsupported_targets(mdn_api("AbortController"), &targets);
        assert_eq!(result, Vec::<String>::new());
    }

    // MDN version comparisons: `version_added` string vs target version.
    #[test]
    fn mdn_version_added() {
        let api = mdn_api("AbortController");
        assert_eq!(
            mdn_unsupported_targets(api, &resolve(&["chrome 65", "chrome 66"])),
            vec!["Chrome 65".to_string()]
        );
        // `version_added: false` is never supported.
        assert_eq!(mdn_unsupported_targets(api, &resolve(&["ie 11"])), vec!["IE 11".to_string()]);
        // Targets without a support record are treated as supported.
        assert_eq!(mdn_unsupported_targets(api, &resolve(&["op_mini all"])), Vec::<String>::new());
    }
}
