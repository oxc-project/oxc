use std::str::FromStr;

/// Represents one of the built-in recommended configurations for the linter.
#[derive(Debug, Clone, Copy)]
pub enum RecommendedConfig {
    Base,       // oxlint:recommended/base
    React,      // oxlint:recommended/react
    Unicorn,    // oxlint:recommended/unicorn
    TypeScript, // oxlint:recommended/typescript
    Oxc,        // oxlint:recommended/oxc
    Import,     // oxlint:recommended/import
    JsDoc,      // oxlint:recommended/jsdoc
    Jest,       // oxlint:recommended/jest
    Vitest,     // oxlint:recommended/vitest
    JsxA11y,    // oxlint:recommended/jsx-a11y
    NextJs,     // oxlint:recommended/nextjs
    ReactPerf,  // oxlint:recommended/react-perf
    Promise,    // oxlint:recommended/promise
    Node,       // oxlint:recommended/node
    Vue,        // oxlint:recommended/vue
}

impl FromStr for RecommendedConfig {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "oxlint:recommended/base" => Ok(RecommendedConfig::Base),
            "oxlint:recommended/react" => Ok(RecommendedConfig::React),
            "oxlint:recommended/unicorn" => Ok(RecommendedConfig::Unicorn),
            "oxlint:recommended/typescript" => Ok(RecommendedConfig::TypeScript),
            "oxlint:recommended/oxc" => Ok(RecommendedConfig::Oxc),
            "oxlint:recommended/import" => Ok(RecommendedConfig::Import),
            "oxlint:recommended/jsdoc" => Ok(RecommendedConfig::JsDoc),
            "oxlint:recommended/jest" => Ok(RecommendedConfig::Jest),
            "oxlint:recommended/vitest" => Ok(RecommendedConfig::Vitest),
            "oxlint:recommended/jsx-a11y" | "oxlint:recommended/jsx_a11y" => {
                Ok(RecommendedConfig::JsxA11y)
            }
            "oxlint:recommended/nextjs" => Ok(RecommendedConfig::NextJs),
            "oxlint:recommended/react-perf" | "oxlint:recommended/react_perf" => {
                Ok(RecommendedConfig::ReactPerf)
            }
            "oxlint:recommended/promise" => Ok(RecommendedConfig::Promise),
            "oxlint:recommended/node" => Ok(RecommendedConfig::Node),
            "oxlint:recommended/vue" => Ok(RecommendedConfig::Vue),
            _ => Err(()),
        }
    }
}
