const isObject = (v: unknown) => typeof v === "object" && v !== null && !Array.isArray(v);

// Cache the in-flight `Promise` rather than the resolved module,
// so concurrent callers share one `import("vite-plus")` invocation
// instead of each kicking off their own before the first `await` completes.
// (e.g. oxlint's `Promise.allSettled` over multiple paths)
let vitePlusCache: Promise<typeof import("vite-plus")> | null = null;

/**
 * Resolve a Vite+ config via `vite-plus`'s `resolveConfig` and extract the given field.
 *
 * `vite-plus` is loaded lazily and memoized for the process.
 * Consumers declare it as an optional peer dependency; tsdown leaves the specifier external
 * so the user-installed copy is used at runtime.
 *
 * @param path - Absolute path to the Vite config file
 * @param fieldName - Field name to extract from the resolved config (e.g. `"fmt"`, `"lint"`)
 * @returns The field as an object, or `null` when the field is missing (signals "skip")
 * @throws When the field exists but is not a plain object
 */
export async function loadViteConfigField(
  path: string,
  fieldName: "lint" | "fmt",
): Promise<object | null> {
  vitePlusCache ??= import("vite-plus");
  const { resolveConfig } = await vitePlusCache;

  const config = (await resolveConfig({ configFile: path }, "build")) as Record<string, unknown>;

  // Signal "skip" when the field is missing
  if (!(fieldName in config)) return null;

  const fieldValue = config[fieldName];
  if (!isObject(fieldValue)) {
    throw new Error(`The \`${fieldName}\` field in the default export must be an object.`);
  }

  return fieldValue as object;
}
