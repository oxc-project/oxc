/**
 * Redirects the `prettier` specifier to the copy bundled into oxfmt, so a
 * user-configured Prettier plugin loads without the user installing Prettier.
 *
 * Plugins declare `prettier` as a peer dependency, so a project that formats
 * with oxfmt and never installed Prettier fails at plugin load with
 * `MODULE_NOT_FOUND`.
 *
 * Identity does not matter here, only resolvability: Prettier's Doc IR is plain
 * data, so a plugin interoperates with a caller holding a different instance.
 *
 * Plugins ship as both CommonJS and ESM, so both module systems are covered.
 */

import Module from "node:module";
import { register } from "node:module";
import { fileURLToPath } from "node:url";
import { HOST_DIR } from "../prettier-host/paths";

// Prettier's public plugin subpaths, re-exported as build entries under
// `prettier-host/` so their `dist` filenames are stable.
const PLUGIN_SUBPATHS = [
  "acorn",
  "angular",
  "babel",
  "estree",
  "flow",
  "glimmer",
  "graphql",
  "html",
  "markdown",
  "meriyah",
  "postcss",
  "typescript",
  "yaml",
] as const;

let installed = false;

/**
 * Install the redirect for this process. Idempotent.
 *
 * Must run before any plugin is loaded. Each pool worker is a separate process,
 * so each one installs its own.
 */
export function installPluginResolution(): void {
  if (installed) return;
  installed = true;

  const urls: Record<string, string> = {
    prettier: new URL("prettier.js", HOST_DIR).href,
    "prettier/doc": new URL("doc.js", HOST_DIR).href,
  };
  for (const name of PLUGIN_SUBPATHS) {
    urls[`prettier/plugins/${name}`] = new URL(`plugins/${name}.js`, HOST_DIR).href;
  }

  installForCommonJs(urls);
  installForEsm(urls);
}

/**
 * CommonJS plugins reach Prettier through `require()`, which never consults the
 * ESM loader hooks, so it needs its own redirect.
 *
 * The targets are ESM. That is fine on every Node version oxfmt supports:
 * `engines` is `^20.19.0 || >=22.12.0`, which is exactly the range where
 * `require()` of an ESM graph works.
 */
function installForCommonJs(urls: Record<string, string>): void {
  const paths: Record<string, string> = {};
  for (const [specifier, url] of Object.entries(urls)) {
    paths[specifier] = fileURLToPath(url);
  }

  // `_resolveFilename` is internal, but it is the only interception point that
  // covers a nested `require()` inside an already-loaded plugin.
  const target = Module as unknown as {
    _resolveFilename: (request: string, ...rest: unknown[]) => string;
  };
  const original = target._resolveFilename;
  target._resolveFilename = function (request: string, ...rest: unknown[]): string {
    const redirect = paths[request];
    if (redirect !== undefined) return redirect;
    return original.call(this, request, ...rest);
  };
}

/** ESM plugins go through the loader, which only `register()` can intercept. */
function installForEsm(urls: Record<string, string>): void {
  register(new URL("hooks.js", HOST_DIR), { data: { map: urls } });
}
