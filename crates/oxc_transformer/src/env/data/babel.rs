use std::sync::OnceLock;

use rustc_hash::FxHashMap;

use crate::env::{Targets, Version};

/// Reference: <https://github.com/swc-project/swc/blob/ea14fc8e5996dcd736b8deb4cc99262d07dfff44/crates/swc_ecma_preset_env/src/transform_data.rs#L194-L218>
pub fn features() -> &'static FxHashMap<String, Targets> {
    static FEATURES: OnceLock<FxHashMap<String, Targets>> = OnceLock::new();
    FEATURES.get_or_init(|| {
        let mut map: FxHashMap<String, FxHashMap<String, String>> =
            serde_json::from_str(include_str!("./@babel/compat_data/data/plugins.json")).unwrap();

        map.extend(
            serde_json::from_str::<FxHashMap<String, FxHashMap<String, String>>>(include_str!(
                "./esbuild/features.json"
            ))
            .unwrap(),
        );

        map.into_iter()
            .map(|(feature, mut versions)| {
                (feature, {
                    let version = versions.get("safari");
                    if version.is_some_and(|v| v == "tp") {
                        versions.remove("safari");
                    }

                    Targets::new(
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
pub fn bugfix_features() -> &'static FxHashMap<String, Targets> {
    static BUGFIX_FEATURES: OnceLock<FxHashMap<String, Targets>> = OnceLock::new();
    BUGFIX_FEATURES.get_or_init(|| {
        let map = serde_json::from_str::<FxHashMap<String, Targets>>(include_str!(
            "./@babel/compat_data/data/plugin_bugfixes.json"
        ))
        .unwrap();
        features().clone().into_iter().chain(map).collect()
    })
}
