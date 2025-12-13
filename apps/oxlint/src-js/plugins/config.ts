import { setOptions } from "./options.ts";

/**
 * Populates Rust-resolved configuration options on the JS side.
 * Called from Rust side after all configuration options have been resolved.
 *
 * Note: the name `setupConfigs` is currently incorrect, as we only populate rule options.
 * The intention is for this function to transfer all configurations in a multi-config workspace.
 * The configuration relevant to each file would then be resolved on the JS side.
 *
 * @param optionsJSON - Array of all rule options across all configurations, serialized as JSON
 */
export function setupConfigs(optionsJSON: string): void {
  // TODO: setup settings and globals using this function
  setOptions(optionsJSON);
}
