import { getErrorMessage } from "./utils/utils.ts";
import { isDefineConfig } from "./package/config.ts";
import { JSONStringify } from "./utils/globals.ts";

interface JsConfigResult {
  path: string;
  config: unknown; // Will be validated as Oxlintrc on Rust side
}

type LoadJsConfigsResult =
  | { Success: JsConfigResult[] }
  | { Failures: { path: string; error: string }[] }
  | { Error: string };

/**
 * Load JavaScript config files in parallel.
 *
 * Uses native Node.js TypeScript support to import the config files.
 * Each config file should have a default export containing the oxlint configuration.
 *
 * @param paths - Array of absolute paths to oxlint.config.ts files
 * @returns JSON-stringified result with all configs or error
 */
export async function loadJsConfigs(paths: string[]): Promise<string> {
  try {
    const results = await Promise.allSettled(
      paths.map(async (path): Promise<JsConfigResult> => {
        const fileUrl = new URL(`file://${path}`);
        const module = await import(fileUrl.href);
        const config = module.default;

        if (config === undefined) {
          throw new Error(`Configuration file has no default export.`);
        }

        if (typeof config !== "object" || config === null || Array.isArray(config)) {
          throw new Error(`Configuration file must have a default export that is an object.`);
        }

        if (!isDefineConfig(config)) {
          throw new Error(
            `Configuration file must wrap its default export with defineConfig() from "oxlint".`,
          );
        }

        return { path, config };
      }),
    );

    const successes: JsConfigResult[] = [];
    const errors: { path: string; error: string }[] = [];

    for (let i = 0; i < results.length; i++) {
      const result = results[i];
      if (result.status === "fulfilled") {
        successes.push(result.value);
      } else {
        errors.push({ path: paths[i], error: getErrorMessage(result.reason) });
      }
    }

    // If any config failed to load, report all errors
    if (errors.length > 0) {
      return JSONStringify({ Failures: errors } satisfies LoadJsConfigsResult);
    }

    return JSONStringify({ Success: successes } satisfies LoadJsConfigsResult);
  } catch (err) {
    return JSONStringify({
      Error: getErrorMessage(err),
    } satisfies LoadJsConfigsResult);
  }
}
