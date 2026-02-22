/// Reanimated library detection and support.
///
/// Port of `Entrypoint/Reanimated.ts` from the React Compiler.
///
/// The compiler has customized support for react-native-reanimated, intended
/// as a temporary workaround. This module detects the library and activates
/// the support when detected.
///
/// In the TS version, detection works by checking Babel plugin keys for
/// "react-native-reanimated" or by calling `require.resolve`. Since the Rust
/// port does not have access to the Babel plugin system or Node.js module
/// resolution, detection is based on an explicit list of plugin names passed
/// by the build tool integration.
use super::options::PluginOptions;

/// Check if the project uses react-native-reanimated.
///
/// `plugin_names` is a list of plugin name strings provided by the build tool
/// integration (e.g., Babel plugin keys). In the TS version this inspects the
/// Babel plugin array and tries `require.resolve('react-native-reanimated')`.
///
/// Returns `true` if any plugin name contains "react-native-reanimated".
pub fn pipeline_uses_reanimated_plugin(plugin_names: &[String]) -> bool {
    plugin_names.iter().any(|name| name.contains("react-native-reanimated"))
}

/// Inject the reanimated flag into plugin options.
///
/// Sets `enable_custom_type_definition_for_reanimated` on the environment
/// config so the compiler treats reanimated shared values with the correct
/// type signatures.
pub fn inject_reanimated_flag(mut options: PluginOptions) -> PluginOptions {
    options.environment.enable_custom_type_definition_for_reanimated = true;
    options
}
