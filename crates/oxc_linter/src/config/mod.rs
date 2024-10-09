mod categories;
mod env;
mod globals;
mod oxlintrc;
mod rules;
mod settings;

pub use self::{
    env::OxlintEnv,
    globals::OxlintGlobals,
    oxlintrc::Oxlintrc,
    settings::{jsdoc::JSDocPluginSettings, OxlintSettings},
};

#[derive(Debug, Default)]
pub(crate) struct LintConfig {
    pub(crate) settings: OxlintSettings,
    /// Environments enable and disable collections of global variables.
    pub(crate) env: OxlintEnv,
    /// Enabled or disabled specific global variables.
    pub(crate) globals: OxlintGlobals,
}

impl From<Oxlintrc> for LintConfig {
    fn from(config: Oxlintrc) -> Self {
        Self { settings: config.settings, env: config.env, globals: config.globals }
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use oxc_span::CompactStr;
    use rustc_hash::FxHashSet;
    use serde::Deserialize;

    use super::Oxlintrc;
    use crate::rules::RULES;

    #[test]
    fn test_from_file() {
        let fixture_path = env::current_dir().unwrap().join("fixtures/eslint_config.json");
        let config = Oxlintrc::from_file(&fixture_path).unwrap();
        assert!(!config.rules.is_empty());
    }

    #[test]
    fn test_deserialize() {
        let config = Oxlintrc::deserialize(&serde_json::json!({
            "rules": {
                "no-console": "off",
                "no-debugger": 2,
                "no-bitwise": [
                    "error",
                    { "allow": ["~"] }
                ],
                "eqeqeq": [
                    "error",
                    "always", { "null": "ignore" }, "foo"
                ],
                "@typescript-eslint/ban-types": "error",
                "jsx-a11y/alt-text": "warn",
                "@next/next/noop": [1]
            },
            "settings": {
                "jsx-a11y": {
                    "polymorphicPropName": "role",
                    "components": {
                        "Link": "Anchor",
                        "Link2": "Anchor2"
                    }
                },
            },
            "env": { "browser": true, },
            "globals": { "foo": "readonly", }
        }));
        assert!(config.is_ok());

        let Oxlintrc { rules, settings, env, globals, .. } = config.unwrap();
        assert!(!rules.is_empty());
        assert_eq!(
            settings.jsx_a11y.polymorphic_prop_name.as_ref().map(CompactStr::as_str),
            Some("role")
        );
        assert_eq!(env.iter().count(), 1);
        assert!(globals.is_enabled("foo"));
    }

    #[test]
    fn test_vitest_rule_replace() {
        let fixture_path: std::path::PathBuf =
            env::current_dir().unwrap().join("fixtures/eslint_config_vitest_replace.json");
        let config = Oxlintrc::from_file(&fixture_path).unwrap();
        let mut set = FxHashSet::default();
        config.rules.override_rules(&mut set, &RULES);

        let rule = set.into_iter().next().unwrap();
        assert_eq!(rule.name(), "no-disabled-tests");
        assert_eq!(rule.plugin_name(), "jest");
    }
}
