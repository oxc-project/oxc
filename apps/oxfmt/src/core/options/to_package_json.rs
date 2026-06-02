use super::super::oxfmtrc::{FormatConfig, SortPackageJsonConfig, SortPackageJsonUserConfig};

/// Convert `FormatConfig` into `sort_package_json::SortOptions`.
///
/// `package.json` sorting is opt-out: when `sort_package_json` is unset,
/// it defaults to enabled with default options. Returns `None` only when
/// the user explicitly sets `sort_package_json: false`.
pub fn to_package_json(config: &FormatConfig) -> Option<sort_package_json::SortOptions> {
    let sort_config = config.sort_package_json.clone().map_or_else(
        || Some(SortPackageJsonConfig::default()),
        SortPackageJsonUserConfig::into_config,
    )?;
    Some(sort_package_json::SortOptions {
        sort_scripts: sort_config.sort_scripts.unwrap_or(false),
        // Small optimization: Prettier will reformat anyway
        pretty: false,
    })
}
