import { setOptions } from "./options.ts";
import { getErrorMessage } from "../utils/utils.ts";

/**
 * Populates Rust-resolved configuration options on the JS side.
 * Called from Rust side after all configuration options have been resolved.
 *
 * Note: the name `setupRuleConfigs` is currently incorrect, as we only populate rule options.
 * The intention is for this function to transfer all configurations in a multi-config workspace.
 * The configuration relevant to each file would then be resolved on the JS side.
 *
 * @param optionsJSON - Array of all rule options across all configurations, serialized as JSON
 * @returns `null` if success, or error message string
 */
export function setupRuleConfigs(optionsJSON: string): string | null {
  // TODO: Setup settings and globals using this function
  try {
    setOptions(optionsJSON);
    return null;
  } catch (err) {
    // Return error message to Rust
    return getErrorMessage(err);
  }
}
