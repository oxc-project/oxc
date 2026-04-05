import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { join as pathJoin } from "node:path";
import { tmpdir } from "node:os";
import { afterEach, describe, expect, it } from "vitest";
import { loadJsConfigs } from "../src-js/js_config.ts";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true })));
});

async function writeConfigFile(source: string): Promise<string> {
  const dir = await mkdtemp(pathJoin(tmpdir(), "oxlint-js-config-flat-compat-"));
  tempDirs.push(dir);

  const configPath = pathJoin(dir, "oxlint.config.mjs");
  await writeFile(configPath, source);
  return configPath;
}

describe("JS config flat-config compatibility", () => {
  it("normalizes real-package-shaped flat config fragments in extends", async () => {
    const configPath = await writeConfigFile(`
const parser = {
  parseForESLint(code) {
    return {
      ast: {
        type: "Program",
        sourceType: "module",
        body: [],
        range: [0, code.length],
        loc: {
          start: { line: 1, column: 0 },
          end: { line: 1, column: code.length },
        },
      },
    };
  },
};

const svelte = {
  meta: {
    name: "eslint-plugin-svelte",
  },
  rules: {},
};

export default {
  extends: [[
    {
      name: "svelte:base:setup-plugin",
      plugins: {
        svelte,
      },
    },
    {
      name: "svelte:base:setup-for-svelte",
      files: ["**/*.svelte"],
      languageOptions: {
        parser,
      },
      processor: "svelte/svelte",
      rules: {
        "svelte/comment-directive": "error",
      },
    },
    {
      name: "svelte:recommended:rules",
      rules: {
        "svelte/no-useless-mustaches": "error",
      },
    },
  ]],
};
`);

    const result = await loadJsConfigs([configPath]);
    const payload = JSON.parse(result);

    expect(payload).toHaveProperty("Success");
    expect(payload.Success).toHaveLength(1);

    const normalizedConfig = payload.Success[0].config;
    // This is the JS loader boundary, so severities still use ESLint-style aliases
    // like "error". Rust canonicalizes those aliases to Oxlint's "deny" form
    // after deserializing the config payload.
    expect(normalizedConfig.extends).toHaveLength(3);
    expect(normalizedConfig.extends[0]).toEqual({
      jsPlugins: [{ name: "svelte", specifier: "eslint-plugin-svelte" }],
    });
    expect(normalizedConfig.extends[1]).toMatchObject({
      overrides: [
        {
          files: ["**/*.svelte"],
          _languageOptionsId: expect.any(Number),
          _languageOptionsHasParser: true,
          rules: {
            "svelte/comment-directive": "error",
          },
        },
      ],
    });
    expect(normalizedConfig.extends[1]).not.toHaveProperty("processor");
    expect(normalizedConfig.extends[2]).toEqual({
      rules: {
        "svelte/no-useless-mustaches": "error",
      },
    });
  });

  it("maps root-level flat-config ignores to ignorePatterns", async () => {
    const configPath = await writeConfigFile(`
export default {
  extends: [[
    {
      name: "svelte:ignores",
      ignores: [".svelte-kit/**", "build/**"],
    },
  ]],
};
`);

    const result = await loadJsConfigs([configPath]);
    const payload = JSON.parse(result);

    expect(payload).toHaveProperty("Success");
    expect(payload.Success).toHaveLength(1);
    expect(payload.Success[0].config.extends).toEqual([
      {
        ignorePatterns: [".svelte-kit/**", "build/**"],
      },
    ]);
  });

  it("rejects flat-config ignores on override-like fragments with files", async () => {
    const configPath = await writeConfigFile(`
export default {
  extends: [[
    {
      files: ["**/*.svelte"],
      ignores: ["generated/**"],
    },
  ]],
};
`);

    const result = await loadJsConfigs([configPath]);
    const payload = JSON.parse(result);

    expect(payload).toHaveProperty("Failures");
    expect(payload.Failures[0].path).toBe(configPath);
    expect(payload.Failures[0].error).toContain("ignores");
    expect(payload.Failures[0].error).toContain("files");
  });

  it("rejects unsupported flat-config processors instead of silently dropping them", async () => {
    const configPath = await writeConfigFile(`
export default {
  extends: [[
    {
      files: ["**/*.svelte"],
      processor: "custom/processor",
    },
  ]],
};
`);

    const result = await loadJsConfigs([configPath]);
    const payload = JSON.parse(result);

    expect(payload).toHaveProperty("Failures");
    expect(payload.Failures[0].path).toBe(configPath);
    expect(payload.Failures[0].error).toContain("processor");
    expect(payload.Failures[0].error).toContain("custom/processor");
  });
});
