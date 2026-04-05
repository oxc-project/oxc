import { afterEach, describe, expect, it } from "vitest";
import { registerLanguageOptions } from "../src-js/js_language_options_registry.ts";
import { lintFile, resetStateAfterError } from "../src-js/plugins/lint.ts";
import { registerPlugin, registeredRules } from "../src-js/plugins/load.ts";
import { allOptions, setOptions } from "../src-js/plugins/options.ts";
import { diagnostics } from "../src-js/plugins/report.ts";
import { tryLoadRealSveltePluginPackages } from "./real_svelte_runtime_packages.ts";

const tryLoadRealSveltePackages = tryLoadRealSveltePluginPackages;

const realSveltePackages = await tryLoadRealSveltePackages();
const REQUIRE_REAL_SVELTE_PACKAGES = process.env.OXLINT_SVELTE_REAL_PACKAGES_CI === "1";

function loadRealSveltePackages() {
  if (realSveltePackages === null) {
    throw new Error(
      "Real Svelte plugin tests require loadable `svelte`, `svelte-eslint-parser`, and `eslint-plugin-svelte` packages.",
    );
  }

  return realSveltePackages;
}

function setSingleRuleOptions(ruleId) {
  setOptions(
    JSON.stringify({
      options: [[]],
      ruleIds: [ruleId],
      cwd: "/workspace",
      workspaceUri: null,
    }),
  );
}

const realPluginIt = realSveltePackages === null && !REQUIRE_REAL_SVELTE_PACKAGES ? it.skip : it;

describe("real eslint-plugin-svelte whole-file integration", () => {
  afterEach(() => {
    registeredRules.length = 0;
    diagnostics.length = 0;
    if (allOptions !== null) allOptions.length = 1;
    resetStateAfterError();
  });

  realPluginIt("runs a real upstream Svelte rule with fixes through the whole-file parser lane", async () => {
    const { parser, plugin } = loadRealSveltePackages();
    const languageOptionsId = registerLanguageOptions({ parser });
    const realRule = plugin.rules["no-useless-mustaches"];

    expect(realRule).toBeDefined();

    const registeredPlugin = registerPlugin(
      {
        meta: plugin.meta,
        rules: {
          "no-useless-mustaches": realRule,
        },
      },
      null,
      false,
      null,
    );

    expect(registeredPlugin.name).toBe("svelte");
    expect(registeredPlugin.ruleNames).toEqual(["no-useless-mustaches"]);

    setSingleRuleOptions(registeredPlugin.offset);

    const sourceText = `<div>{"hello"}</div>`;
    const result = lintFile(
      "/workspace/App.svelte",
      0,
      null,
      [registeredPlugin.offset],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      sourceText,
    );

    expect(result).not.toBeNull();
    if (result === null) throw new Error("Expected lint result");

    const payload = JSON.parse(result);
    expect(payload.Success.comments).toEqual([]);
    expect(payload.Success).not.toHaveProperty("parseError");
    expect(payload.Success.diagnostics).toHaveLength(1);
    expect(payload.Success.diagnostics[0]).toMatchObject({
      message: "Unexpected mustache interpolation with a string literal value.",
      start: 5,
      end: 14,
      ruleIndex: 0,
      messageId: "unexpected",
      fixes: [{ start: 5, end: 14, text: "hello" }],
      suggestions: null,
    });
  });
});
