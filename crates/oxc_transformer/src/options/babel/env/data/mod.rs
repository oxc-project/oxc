use std::sync::OnceLock;

use rustc_hash::FxHashMap;

use super::EngineTargets;

/// Reference: <https://github.com/swc-project/swc/blob/ea14fc8e5996dcd736b8deb4cc99262d07dfff44/crates/swc_ecma_preset_env/src/transform_data.rs#L194-L218>
pub fn features() -> &'static FxHashMap<String, EngineTargets> {
    static FEATURES: OnceLock<FxHashMap<String, EngineTargets>> = OnceLock::new();
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
                let version = versions.get("safari");
                if version.is_some_and(|v| v == "tp") {
                    versions.remove("safari");
                }
                let versions = versions.into_iter().collect::<Vec<_>>();
                (feature, EngineTargets::parse_versions(versions))
            })
            .collect()
    })
}

/// Reference: <https://github.com/swc-project/swc/blob/ea14fc8e5996dcd736b8deb4cc99262d07dfff44/crates/swc_ecma_preset_env/src/transform_data.rs#L220-L237>
pub fn bugfix_features() -> &'static FxHashMap<String, EngineTargets> {
    static BUGFIX_FEATURES: OnceLock<FxHashMap<String, EngineTargets>> = OnceLock::new();
    BUGFIX_FEATURES.get_or_init(|| {
        let map = serde_json::from_str::<FxHashMap<String, EngineTargets>>(include_str!(
            "./@babel/compat_data/data/plugin_bugfixes.json"
        ))
        .unwrap();
        features().clone().into_iter().chain(map).collect()
    })
}
