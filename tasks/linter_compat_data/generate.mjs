// oxlint-disable no-console
// Generates the embedded compat dataset consumed by the `compat/compat` lint rule
// (`crates/oxc_linter/src/utils/compat/compat_data.json`).
//
// The dataset is a Rust-consumable trimming of the data used by
// `eslint-plugin-compat` (https://github.com/amilajack/eslint-plugin-compat):
// - `ast-metadata-inferer` (MDN browser-compat-data derived API metadata)
// - `caniuse-lite` feature support tables (only the features referenced by the
//   CanIUse provider of eslint-plugin-compat)
// - the `globals` package's browser globals (used for case-insensitive member
//   expression lookups of browser globals)
//
// Regenerate with: pnpm install && node generate.mjs
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const astMetadata = require("ast-metadata-inferer");
const lite = require("caniuse-lite");
const globals = require("globals");

const apis = astMetadata.default ?? astMetadata;

// Map of browserslist target name -> MDN browser-compat-data support key.
// Mirrors (the reverse of) `targetIdMappings` in eslint-plugin-compat's
// `src/providers/mdn-provider.ts`. Only mappings whose browserslist name can
// actually be produced by browserslist are relevant; the reference maps the
// remaining MDN keys (chrome_android, ...) to names browserslist never emits,
// so records for them are unreachable and not embedded.
const MDN_TARGETS = {
  chrome: "chrome",
  firefox: "firefox",
  opera: "opera",
  safari: "safari",
  ios_saf: "safari_ios",
  ie: "ie",
  ie_mob: "edge_mobile",
  edge: "edge",
  node: "nodejs",
};

const AST_NODE_TYPE_CHARS = {
  MemberExpression: "M",
  CallExpression: "C",
  NewExpression: "N",
  ExpressionStatement: "E",
  Literal: "L",
};

const mdn = [];
for (const api of apis) {
  if (api.protoChainId !== api.protoChain.join(".")) {
    throw new Error(`protoChainId mismatch for ${api.protoChainId}`);
  }
  const support = {};
  for (const [targetName, mdnKey] of Object.entries(MDN_TARGETS)) {
    let record = api.compat?.support?.[mdnKey];
    if (!record) continue;
    if (Array.isArray(record)) {
      // Mirrors mdn-provider: `compatRecord.find((e) => "version_added" in e)`
      record = record.find((e) => "version_added" in e);
    }
    if (!record || !("version_added" in record)) continue;
    const versionAdded = record.version_added;
    // `true`/`null` mean "supported (version unknown)" -> never fails -> omit.
    // `false` means "never supported" -> encoded as "".
    if (versionAdded === false) {
      support[targetName] = "";
    } else if (typeof versionAdded === "string") {
      support[targetName] = versionAdded;
    }
  }
  // Records with no version-gated or unsupported targets can never produce a
  // failing rule; omit them to keep the embedded dataset small.
  if (Object.keys(support).length === 0) continue;
  const astNodeTypes = api.astNodeTypes
    .map((t) => {
      const c = AST_NODE_TYPE_CHARS[t];
      if (!c) throw new Error(`Unknown astNodeType ${t}`);
      return c;
    })
    .join("");
  mdn.push([api.protoChainId, astNodeTypes, api.kind === "es" ? 1 : 0, support]);
}

// The caniuse feature ids referenced by eslint-plugin-compat's
// `src/providers/caniuse-provider.ts`.
const CANIUSE_FEATURES = [
  "serviceworkers",
  "queryselector",
  "intersectionobserver",
  "resizeobserver",
  "payment-request",
  "promises",
  "fetch",
  "document-currentscript",
  "url",
  "urlsearchparams",
  "high-resolution-time",
  "requestidlecallback",
  "requestanimationframe",
  "typedarrays",
  "js-regexp-lookbehind",
];

const caniuse = {};
for (const featureId of CANIUSE_FEATURES) {
  const packed = lite.features[featureId];
  if (!packed) throw new Error(`caniuse feature not found: ${featureId}`);
  const { stats } = lite.feature(packed);
  const browsers = {};
  for (const [browser, versions] of Object.entries(stats)) {
    // Preserve version key order; store whether the support flags include "y".
    browsers[browser] = Object.entries(versions).map(([version, flags]) => [
      version,
      flags.includes("y") ? 1 : 0,
    ]);
  }
  caniuse[featureId] = browsers;
}

const browserGlobals = Object.keys(globals.browser);

const data = { mdn, caniuse, browserGlobals };
const outPath = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
  "crates",
  "oxc_linter",
  "src",
  "utils",
  "compat",
  "compat_data.json",
);
fs.mkdirSync(path.dirname(outPath), { recursive: true });
fs.writeFileSync(outPath, JSON.stringify(data));
console.log(
  `Wrote ${outPath}: ${mdn.length} MDN records, ${Object.keys(caniuse).length} caniuse features, ${browserGlobals.length} browser globals (${(fs.statSync(outPath).size / 1024).toFixed(1)} KiB)`,
);
