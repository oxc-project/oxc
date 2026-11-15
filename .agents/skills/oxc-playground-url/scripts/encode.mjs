#!/usr/bin/env node
// Encode source, options, and UI state into an Oxc playground share URL.
// oxlint-disable no-console -- this is a command-line script

import { argv, exit } from "node:process";
import { readFileSync } from "node:fs";

const BASE = "https://playground.oxc.rs/";

function fail(message) {
  console.error(message);
  exit(2);
}

function splitList(value) {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

function parseOptions(value, source) {
  let options;
  try {
    options = JSON.parse(value);
  } catch (error) {
    fail(`invalid options JSON from ${source}: ${error.message}`);
  }
  if (options === null || typeof options !== "object" || Array.isArray(options)) {
    fail(`options from ${source} must be a JSON object`);
  }
  return options;
}

function parseArgs(args) {
  const result = {
    tab: null,
    options: null,
    code: null,
    lintRules: [],
    formatterPanels: [],
  };

  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    const take = () => {
      const value = args[++index];
      if (value === undefined) fail(`${arg} requires a value`);
      return value;
    };

    switch (arg) {
      case "--tab":
        result.tab = take();
        if (!result.tab) fail("--tab must not be empty");
        break;
      case "--options":
        result.options = parseOptions(take(), "--options");
        break;
      case "--options-file": {
        const path = take();
        result.options = parseOptions(readFileSync(path, "utf8"), path);
        break;
      }
      case "--code":
        result.code = take();
        break;
      case "--code-file": {
        const path = take();
        result.code = path === "-" ? readFileSync(0, "utf8") : readFileSync(path, "utf8");
        break;
      }
      case "--lint-rules":
        result.lintRules = splitList(take());
        break;
      case "--formatter-panels":
        result.formatterPanels = splitList(take());
        break;
      case "--help":
      case "-h":
        console.log(
          [
            "usage: encode.mjs (--code <source> | --code-file <path|->) [options]",
            "",
            "  --tab <name>                active tab, such as codegen or minifier",
            "  --options <json>            JSON option object",
            "  --options-file <path>       read the option object from a file",
            "  --lint-rules <a,b>          enabled lint rules, comma-separated",
            "  --formatter-panels <a,b>    visible formatter panels, comma-separated",
          ].join("\n"),
        );
        exit(0);
        break;
      default:
        fail(`unknown argument: ${arg}`);
    }
  }

  if (result.code === null) fail("--code or --code-file is required");
  if (!result.code.trim()) fail("the playground code must not be empty");
  return result;
}

function buildUrl(state) {
  const params = new URLSearchParams();
  if (state.tab) params.set("t", state.tab);
  if (state.options) params.set("options", JSON.stringify(state.options));
  params.set("code", state.code);
  if (state.lintRules.length > 0) params.set("lintRules", state.lintRules.join(","));
  if (state.formatterPanels.length > 0) {
    params.set("formatterPanels", state.formatterPanels.join(","));
  }
  return `${BASE}?${params}`;
}

// Re-parse the URL so a malformed share link is never printed as valid.
function verify(url, state) {
  const params = new URL(url).searchParams;
  if (params.get("code") !== state.code) fail("round-trip check failed: code differs");
  if (params.get("t") !== (state.tab ?? null)) fail("round-trip check failed: tab differs");
  if (params.get("lintRules") !== (state.lintRules.join(",") || null)) {
    fail("round-trip check failed: lint rules differ");
  }
  if (params.get("formatterPanels") !== (state.formatterPanels.join(",") || null)) {
    fail("round-trip check failed: formatter panels differ");
  }
  if (state.options) {
    if (params.get("options") !== JSON.stringify(state.options)) {
      fail("round-trip check failed: options differ");
    }
  }
}

const state = parseArgs(argv.slice(2));
const url = buildUrl(state);
verify(url, state);
console.log(url);
