#!/usr/bin/env node
// Decode an Oxc playground URL into source, options, and UI state.
// oxlint-disable no-console -- this is a command-line script

import { argv, exit } from "node:process";
import { Buffer } from "node:buffer";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

function fail(message) {
  console.error(message);
  exit(2);
}

function decode(url) {
  let parsed;
  try {
    parsed = new URL(url);
  } catch (error) {
    fail(`invalid URL: ${error.message}`);
  }

  let query = parsed.search.startsWith("?") ? parsed.search.slice(1) : parsed.search;
  if (!query && parsed.hash) query = parsed.hash.replace(/^#\??/, "");
  const params = new URLSearchParams(query);
  let options = null;
  if (params.has("options")) {
    try {
      options = JSON.parse(params.get("options"));
    } catch (error) {
      options = { _parse_error: error.message, _raw: params.get("options") };
    }
  }

  return {
    has_state: query.length > 0,
    tab: params.get("t") || "(default)",
    code: params.get("code") || "",
    options,
    lint_rules: params.get("lintRules")?.split(",").filter(Boolean) ?? [],
    formatter_panels: params.get("formatterPanels")?.split(",").filter(Boolean) ?? [],
  };
}

function parseArgs(args) {
  const writeIndex = args.indexOf("--write");
  if (writeIndex >= 0 && !args[writeIndex + 1]) fail("--write requires a directory");
  const writeDir = writeIndex >= 0 ? args[writeIndex + 1] : null;
  const url = args.find(
    (arg, index) => arg !== "--write" && (writeIndex < 0 || index !== writeIndex + 1),
  );
  if (!url) fail("usage: decode.mjs <url> [--write <new-dir>]");
  return { url, writeDir };
}

function safeExtension(options) {
  const extension = options?.parser?.extension || "tsx";
  if (typeof extension !== "string" || !/^[a-zA-Z0-9]+$/.test(extension)) {
    fail(`unsafe parser extension: ${JSON.stringify(extension)}`);
  }
  return extension;
}

const { url, writeDir } = parseArgs(argv.slice(2));
const state = decode(url);
console.log(
  JSON.stringify(
    {
      has_state: state.has_state,
      tab: state.tab,
      code_bytes: Buffer.byteLength(state.code, "utf8"),
      lint_rules_count: state.lint_rules.length,
      lint_rules: state.lint_rules,
      formatter_panels: state.formatter_panels,
      options: state.options,
    },
    null,
    2,
  ),
);

if (writeDir) {
  mkdirSync(writeDir, { recursive: false });
  const extension = safeExtension(state.options);
  const output = join(writeDir, `input.${extension}`);
  writeFileSync(output, state.code, { encoding: "utf8", flag: "wx" });
  console.error(
    `\nwrote input.${extension} (${Buffer.byteLength(state.code, "utf8")} bytes) to ${writeDir}`,
  );
}
