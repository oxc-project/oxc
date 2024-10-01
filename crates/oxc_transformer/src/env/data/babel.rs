use std::sync::OnceLock;

use rustc_hash::FxHashMap;

use crate::env::{targets::version::Version, Versions};

/// Reference: <https://github.com/swc-project/swc/blob/ea14fc8e5996dcd736b8deb4cc99262d07dfff44/crates/swc_ecma_preset_env/src/transform_data.rs#L194-L218>
fn features() -> &'static FxHashMap<String, Versions> {
    static FEATURES: OnceLock<FxHashMap<String, Versions>> = OnceLock::new();
    FEATURES.get_or_init(|| {
        let mut map: FxHashMap<String, FxHashMap<String, String>> =
            serde_json::from_str(include_str!("./@babel/compat_data/data/plugins.json"))
                .expect("failed to parse json");

        map.extend(
            serde_json::from_str::<FxHashMap<String, FxHashMap<String, String>>>(include_str!(
                "./esbuild/features.json"
            ))
            .expect("failed to parse json"),
        );

        map.into_iter()
            .map(|(feature, mut versions)| {
                (feature, {
                    let version = versions.get("safari");
                    if version.is_some_and(|v| v == "tp") {
                        versions.remove("safari");
                    }

                    Versions(
                        versions
                            .into_iter()
                            .map(|(k, v)| (k, v.parse::<Version>().unwrap()))
                            .collect::<FxHashMap<String, Version>>(),
                    )
                })
            })
            .collect()
    })
}

/// Reference: <https://github.com/swc-project/swc/blob/ea14fc8e5996dcd736b8deb4cc99262d07dfff44/crates/swc_ecma_preset_env/src/transform_data.rs#L220-L237>
fn bugfix_features() -> &'static FxHashMap<String, Versions> {
    static BUGFIX_FEATURES: OnceLock<FxHashMap<String, Versions>> = OnceLock::new();
    BUGFIX_FEATURES.get_or_init(|| {
        let map: FxHashMap<String, Versions> =
            serde_json::from_str(include_str!("./@babel/compat_data/data/plugin_bugfixes.json"))
                .expect("failed to parse json");
        features().clone().into_iter().chain(map).collect()
    })
}

pub fn can_enable_plugin(name: &str, targets: Option<&Versions>, bugfixes: bool) -> bool {
    let versions = if bugfixes {
        bugfix_features().get(name).unwrap_or_else(|| &features()[name])
    } else {
        &features()[name]
    };
    targets.is_some_and(|v| v.should_enable(versions))
}
