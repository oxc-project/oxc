// Post-build script for oxfmt npm package.
// Copies dist files from apps/oxfmt/dist to npm/oxfmt/dist.
// Native package generation and optionalDependencies are handled by NAPI commands.

// oxlint-disable no-console

import * as fs from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** `<REPO ROOT>/npm/oxfmt` */
const OXFMT_ROOT = resolve(fileURLToPath(import.meta.url), "../..");
/** `<REPO ROOT>` */
const REPO_ROOT = resolve(OXFMT_ROOT, "../..");
/** `<REPO ROOT>/apps/oxfmt/dist` */
const OXFMT_DIST_SRC = resolve(REPO_ROOT, "apps/oxfmt/dist");
/** `<REPO ROOT>/npm/oxfmt/dist` */
const OXFMT_DIST_DEST = resolve(OXFMT_ROOT, "dist");

/**
 * Copy `dist` directory from `apps/oxfmt/dist` to `npm/oxfmt/dist`.
 * `apps/oxfmt/scripts/build.js` must be run before this script.
 */
function copyDistFiles() {
  console.log(`Copying dist files from ${OXFMT_DIST_SRC} to ${OXFMT_DIST_DEST}`);
  fs.cpSync(OXFMT_DIST_SRC, OXFMT_DIST_DEST, { recursive: true });
}

copyDistFiles();
