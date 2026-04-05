import { createRequire } from "node:module";
import { join as pathJoin } from "node:path";
import { pathToFileURL } from "node:url";
import { TEST_ROOT_PATH } from "../scripts/svelte-real-package-metadata.ts";

const requireFromRealSvelteTestRoot = createRequire(
  pathJoin(TEST_ROOT_PATH, "__oxlint_real_svelte_runtime__.js"),
);

function getDefaultExport<T>(value: unknown): T {
  if (value && typeof value === "object" && "default" in value) {
    return (value as { default: T }).default;
  }

  return value as T;
}

async function importFromRealSvelteTestRoot(specifier: string): Promise<unknown | null> {
  let resolvedSpecifier: string;
  try {
    resolvedSpecifier = requireFromRealSvelteTestRoot.resolve(specifier);
  } catch {
    return null;
  }

  try {
    return requireFromRealSvelteTestRoot(resolvedSpecifier);
  } catch {
    try {
      return await import(pathToFileURL(resolvedSpecifier).href);
    } catch {
      return null;
    }
  }
}

export async function tryLoadRealSvelteParser() {
  try {
    const [parserModule, compilerModule] = await Promise.all([
      importFromRealSvelteTestRoot("svelte-eslint-parser"),
      importFromRealSvelteTestRoot("svelte/compiler"),
    ]);

    if (parserModule === null || compilerModule === null) return null;
    return getDefaultExport(parserModule);
  } catch {
    return null;
  }
}

export async function tryLoadRealSvelteTypeAwarePackages() {
  try {
    const [parserModule, tsParserModule, compilerModule] = await Promise.all([
      importFromRealSvelteTestRoot("svelte-eslint-parser"),
      importFromRealSvelteTestRoot("@typescript-eslint/parser"),
      importFromRealSvelteTestRoot("svelte/compiler"),
    ]);

    if (parserModule === null || tsParserModule === null || compilerModule === null) return null;

    return {
      parser: getDefaultExport(parserModule),
      tsParser: getDefaultExport(tsParserModule),
    };
  } catch {
    return null;
  }
}

export async function tryLoadRealSveltePluginPackages() {
  try {
    const [parserModule, pluginModule, compilerModule] = await Promise.all([
      importFromRealSvelteTestRoot("svelte-eslint-parser"),
      importFromRealSvelteTestRoot("eslint-plugin-svelte"),
      importFromRealSvelteTestRoot("svelte/compiler"),
    ]);

    if (parserModule === null || pluginModule === null || compilerModule === null) return null;

    return {
      parser: getDefaultExport(parserModule),
      plugin: getDefaultExport(pluginModule),
    };
  } catch {
    return null;
  }
}
