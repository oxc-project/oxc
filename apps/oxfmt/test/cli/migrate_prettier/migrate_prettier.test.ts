import { join } from "node:path";
import { tmpdir } from "node:os";
import fs from "node:fs/promises";
import { describe, expect, it } from "vitest";
import { runCli } from "../utils";

const realSveltePluginPackageDir = join(
  import.meta.dirname,
  "..",
  "plugin_languages_real_package",
  "fixtures",
  "node_modules",
  "prettier-plugin-svelte",
);

async function installRealSveltePluginForMigration(tempDir: string): Promise<void> {
  const nodeModulesDir = join(tempDir, "node_modules");
  await fs.mkdir(nodeModulesDir, { recursive: true });
  await fs.cp(realSveltePluginPackageDir, join(nodeModulesDir, "prettier-plugin-svelte"), {
    recursive: true,
  });

  await fs.mkdir(join(nodeModulesDir, "prettier", "plugins"), { recursive: true });
  await fs.writeFile(
    join(nodeModulesDir, "prettier", "index.js"),
    `const line = { type: "line" };
const hardline = { type: "line", hard: true };
const softline = { type: "line", soft: true };
const literalline = { type: "line", literal: true };
const identity = (value) => value;

module.exports = {
  doc: {
    builders: {
      join: (_separator, parts) => parts,
      line,
      group: identity,
      indent: identity,
      dedent: identity,
      softline,
      hardline,
      fill: identity,
      breakParent: { type: "break-parent" },
      literalline,
    },
    utils: {
      removeLines: identity,
    },
  },
};
`,
  );
  await fs.writeFile(
    join(nodeModulesDir, "prettier", "plugins", "babel.js"),
    `module.exports = {
  parsers: {
    babel: {
      parse() {
        return { program: { body: [{ expression: null }] } };
      },
    },
    "babel-ts": {
      parse() {
        return { program: { body: [{ expression: null }] } };
      },
    },
  },
};
`,
  );

  await fs.mkdir(join(nodeModulesDir, "svelte"), { recursive: true });
  await fs.writeFile(
    join(nodeModulesDir, "svelte", "compiler.js"),
    `exports.parse = function parse() {
  return { type: "Root", start: 0, end: 0, children: [] };
};
`,
  );
}


async function writeCustomFormatterPluginModule(tempDir: string, moduleFormat: "mjs" | "cjs"): Promise<string> {
  const pluginsDir = join(tempDir, "plugins");
  await fs.mkdir(pluginsDir, { recursive: true });

  const filename = moduleFormat === "mjs"
    ? "prettier-plugin-custom.mjs"
    : "prettier-plugin-custom.cjs";
  const pluginPath = join(pluginsDir, filename);
  const pluginSource = moduleFormat === "mjs"
    ? `export default {
  languages: [{ name: "custom", parsers: ["custom"], extensions: [".custom"] }],
  parsers: { custom: { astFormat: "custom-ast", parse() { return { type: "custom-ast" }; } } },
  printers: { "custom-ast": { print() { return ""; } } }
};
`
    : `module.exports = {
  languages: [{ name: "custom", parsers: ["custom"], extensions: [".custom"] }],
  parsers: { custom: { astFormat: "custom-ast", parse() { return { type: "custom-ast" }; } } },
  printers: { "custom-ast": { print() { return ""; } } }
};
`;

  await fs.writeFile(pluginPath, pluginSource);
  return `./plugins/${filename}`;
}

describe("--migrate prettier", () => {
  it("should create .oxfmtrc.json when no prettier config exists", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.ignorePatterns).toEqual([]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should abort if .oxfmtrc.json already exists", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create existing config file
      await fs.writeFile(join(tempDir, ".oxfmtrc.json"), "{}");

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(1);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate prettier config to .oxfmtrc.json", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          semi: false,
          singleQuote: true,
          tabWidth: 4,
          printWidth: 120,
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.semi).toBe(false);
      expect(oxfmtrc.singleQuote).toBe(true);
      expect(oxfmtrc.tabWidth).toBe(4);
      expect(oxfmtrc.printWidth).toBe(120);
      expect(oxfmtrc.ignorePatterns).toEqual([]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should default printWidth to 80 when not set in prettier config", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config without printWidth
      await fs.writeFile(join(tempDir, ".prettierrc"), JSON.stringify({ semi: false }));

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      // Prettier default is 80, Oxfmt default is 100
      // So we explicitly set 80 to match Prettier behavior
      expect(oxfmtrc.printWidth).toBe(80);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate .prettierignore to ignorePatterns", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config
      await fs.writeFile(join(tempDir, ".prettierrc"), JSON.stringify({ semi: true }));
      // Create .prettierignore
      await fs.writeFile(
        join(tempDir, ".prettierignore"),
        `# Comment line
dist
node_modules
*.min.js
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.ignorePatterns).toEqual(["dist", "node_modules", "*.min.js"]);
      expect(Object.keys(oxfmtrc).at(-1)).toBe("ignorePatterns"); // `ignorePatterns` should be last
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate prettier-plugin-tailwindcss options to sortTailwindcss", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config with Tailwind plugin options
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          plugins: ["prettier-plugin-tailwindcss"],
          tailwindConfig: "./tailwind.config.js",
          tailwindFunctions: ["clsx", "cn"],
          tailwindAttributes: ["myClass"],
          tailwindPreserveWhitespace: true,
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      // Tailwind options should be migrated to sortTailwindcss
      expect(oxfmtrc.sortTailwindcss).toEqual({
        config: "./tailwind.config.js",
        functions: ["clsx", "cn"],
        attributes: ["myClass"],
        preserveWhitespace: true,
      });
      // Tailwind options should not be at root level
      expect(oxfmtrc.tailwindConfig).toBeUndefined();
      expect(oxfmtrc.tailwindFunctions).toBeUndefined();
      // plugins should not be copied
      expect(oxfmtrc.plugins).toBeUndefined();
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should enable sortTailwindcss when plugin is listed without options", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config with Tailwind plugin but no options
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          plugins: ["prettier-plugin-tailwindcss"],
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      // sortTailwindcss should be enabled (empty object)
      expect(oxfmtrc.sortTailwindcss).toEqual({});
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });


  it("should preserve supported plugin specs while migrating Svelte and Tailwind configs", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          plugins: [
            "prettier-plugin-svelte",
            "prettier-plugin-tailwindcss",
            "./plugins/prettier-plugin-custom.mjs",
            "@scope/prettier-plugin-foo/subpath",
            "prettier-plugin-packagejson",
            "prettier-plugin-svelte",
          ],
          tailwindConfig: "./tailwind.config.js",
          svelteSortOrder: "scripts-markup-styles",
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.plugins).toEqual([
        "prettier-plugin-svelte",
        "./plugins/prettier-plugin-custom.mjs",
        "@scope/prettier-plugin-foo/subpath",
      ]);
      expect(oxfmtrc.svelteSortOrder).toBe("scripts-markup-styles");
      expect(oxfmtrc.sortTailwindcss).toEqual({ config: "./tailwind.config.js" });
      expect(oxfmtrc.sortPackageJson).toEqual({});
      expect(Object.keys(oxfmtrc).at(-1)).toBe("ignorePatterns");
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate JSON-based prettier overrides with Svelte plugins", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          semi: false,
          overrides: [
            {
              files: "*.svelte",
              options: {
                parser: "svelte",
                plugins: [
                  "prettier-plugin-svelte",
                  "prettier-plugin-tailwindcss",
                  "./plugins/prettier-plugin-custom.mjs",
                  "prettier-plugin-packagejson",
                  "prettier-plugin-svelte",
                ],
                svelteSortOrder: "scripts-markup-styles",
                tailwindFunctions: ["cn"],
              },
            },
            {
              files: ["*.json"],
              excludeFiles: "package.json",
              options: {
                plugins: ["prettier-plugin-packagejson"],
                printWidth: 90,
              },
            },
          ],
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.overrides).toEqual([
        {
          files: ["*.svelte"],
          options: {
            parser: "svelte",
            plugins: ["prettier-plugin-svelte", "./plugins/prettier-plugin-custom.mjs"],
            svelteSortOrder: "scripts-markup-styles",
            sortPackageJson: {},
            sortTailwindcss: {
              functions: ["cn"],
            },
          },
        },
        {
          files: ["*.json"],
          excludeFiles: ["package.json"],
          options: {
            printWidth: 90,
            sortPackageJson: {},
          },
        },
      ]);
      expect(Object.keys(oxfmtrc).at(-1)).toBe("ignorePatterns");
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate package.json prettier overrides", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      await fs.writeFile(
        join(tempDir, "package.json"),
        JSON.stringify({
          name: "oxfmt-migrate-test",
          prettier: {
            singleQuote: true,
            overrides: [
              {
                files: ["*.svelte", "*.svx"],
                options: {
                  plugins: ["prettier-plugin-svelte"],
                  svelteSortOrder: "scripts-styles-markup",
                },
              },
            ],
          },
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.singleQuote).toBe(true);
      expect(oxfmtrc.overrides).toEqual([
        {
          files: ["*.svelte", "*.svx"],
          options: {
            plugins: ["prettier-plugin-svelte"],
            svelteSortOrder: "scripts-styles-markup",
          },
        },
      ]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate YAML-based prettier overrides with Svelte plugins from .prettierrc.yaml", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      await fs.writeFile(
        join(tempDir, ".prettierrc.yaml"),
        `# root-level YAML config
semi: false
overrides:
  - files:
      - "*.svelte"
      - "*.svx"
    excludeFiles:
      - "*.generated.svelte"
    options:
      parser: svelte
      plugins:
        - prettier-plugin-svelte
        - prettier-plugin-tailwindcss
        - prettier-plugin-packagejson
      svelteSortOrder: scripts-markup-styles
      tailwindFunctions:
        - cn
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.semi).toBe(false);
      expect(oxfmtrc.overrides).toEqual([
        {
          files: ["*.svelte", "*.svx"],
          excludeFiles: ["*.generated.svelte"],
          options: {
            parser: "svelte",
            plugins: ["prettier-plugin-svelte"],
            svelteSortOrder: "scripts-markup-styles",
            sortPackageJson: {},
            sortTailwindcss: {
              functions: ["cn"],
            },
          },
        },
      ]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate root-level Svelte plugin options from prettier.config.yml", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      await fs.writeFile(
        join(tempDir, "prettier.config.yml"),
        `singleQuote: true
plugins:
  - prettier-plugin-svelte
  - ./plugins/prettier-plugin-custom.mjs
  - prettier-plugin-packagejson
svelteSortOrder: scripts-styles-markup
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.singleQuote).toBe(true);
      expect(oxfmtrc.plugins).toEqual(["prettier-plugin-svelte", "./plugins/prettier-plugin-custom.mjs"]);
      expect(oxfmtrc.svelteSortOrder).toBe("scripts-styles-markup");
      expect(oxfmtrc.sortPackageJson).toEqual({});
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate CommonJS prettier overrides from .prettierrc.cjs", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      await fs.writeFile(
        join(tempDir, ".prettierrc.cjs"),
        `module.exports = {
  semi: false,
  overrides: [
    {
      files: ["*.svelte"],
      options: {
        parser: "svelte",
        plugins: ["prettier-plugin-svelte", "prettier-plugin-tailwindcss"],
        svelteSortOrder: "scripts-markup-styles",
        tailwindFunctions: ["cn"]
      }
    }
  ]
};
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.semi).toBe(false);
      expect(oxfmtrc.overrides).toEqual([
        {
          files: ["*.svelte"],
          options: {
            parser: "svelte",
            plugins: ["prettier-plugin-svelte"],
            svelteSortOrder: "scripts-markup-styles",
            sortTailwindcss: {
              functions: ["cn"],
            },
          },
        },
      ]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate ESM prettier overrides from prettier.config.mjs", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      await fs.writeFile(
        join(tempDir, "prettier.config.mjs"),
        `export default {
  singleQuote: true,
  overrides: [
    {
      files: "*.svelte",
      excludeFiles: ["*.generated.svelte"],
      options: {
        plugins: [
          "prettier-plugin-svelte",
          "prettier-plugin-packagejson",
          "@scope/prettier-plugin-foo/subpath"
        ],
        svelteSortOrder: "scripts-styles-markup"
      }
    }
  ]
};
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.singleQuote).toBe(true);
      expect(oxfmtrc.overrides).toEqual([
        {
          files: ["*.svelte"],
          excludeFiles: ["*.generated.svelte"],
          options: {
            plugins: ["prettier-plugin-svelte", "@scope/prettier-plugin-foo/subpath"],
            svelteSortOrder: "scripts-styles-markup",
            sortPackageJson: {},
          },
        },
      ]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate an imported real prettier-plugin-svelte object from prettier.config.mjs", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      await installRealSveltePluginForMigration(tempDir);
      await fs.writeFile(
        join(tempDir, "prettier.config.mjs"),
        `import sveltePlugin from "prettier-plugin-svelte";

export default {
  singleQuote: true,
  plugins: [sveltePlugin],
  svelteSortOrder: "scripts-markup-styles-options"
};
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);
      expect(result.stderr).toContain("recognized a Svelte formatter plugin object");
      expect(result.stderr).toContain('using "prettier-plugin-svelte"');

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.singleQuote).toBe(true);
      expect(oxfmtrc.plugins).toEqual(["prettier-plugin-svelte"]);
      expect(oxfmtrc.svelteSortOrder).toBe("scripts-markup-styles-options");
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate override-scoped real prettier-plugin-svelte objects from .prettierrc.cjs", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      await installRealSveltePluginForMigration(tempDir);
      await fs.writeFile(
        join(tempDir, ".prettierrc.cjs"),
        `const sveltePlugin = require("prettier-plugin-svelte");

module.exports = {
  overrides: [
    {
      files: ["*.svelte"],
      options: {
        plugins: [sveltePlugin],
        svelteSortOrder: "scripts-markup-styles-options"
      }
    }
  ]
};
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);
      expect(result.stderr).toContain("recognized a Svelte formatter plugin object");

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.overrides).toEqual([
        {
          files: ["*.svelte"],
          options: {
            plugins: ["prettier-plugin-svelte"],
            svelteSortOrder: "scripts-markup-styles-options",
          },
        },
      ]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });


  it("should preserve imported local formatter plugin objects from prettier.config.mjs", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      const customPluginSpecifier = await writeCustomFormatterPluginModule(tempDir, "mjs");
      await fs.writeFile(
        join(tempDir, "prettier.config.mjs"),
        `import customPlugin from "${customPluginSpecifier}";

export default {
  plugins: [customPlugin],
  singleQuote: true,
};
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);
      expect(result.stderr).toContain("recognized an imported formatter plugin object");
      expect(result.stderr).toContain(`using "${customPluginSpecifier}"`);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.singleQuote).toBe(true);
      expect(oxfmtrc.plugins).toEqual([customPluginSpecifier]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should preserve override-scoped CommonJS local formatter plugin objects from .prettierrc.cjs", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      const customPluginSpecifier = await writeCustomFormatterPluginModule(tempDir, "cjs");
      await fs.writeFile(
        join(tempDir, ".prettierrc.cjs"),
        `const customPlugin = require("${customPluginSpecifier}");

module.exports = {
  overrides: [
    {
      files: ["*.custom"],
      options: {
        plugins: [customPlugin],
        semi: false,
      },
    },
  ],
};
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);
      expect(result.stderr).toContain("recognized an imported formatter plugin object");

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.overrides).toEqual([
        {
          files: ["*.custom"],
          options: {
            plugins: [customPluginSpecifier],
            semi: false,
          },
        },
      ]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });


  it("should preserve inline formatter plugin objects with package metadata", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      await fs.writeFile(
        join(tempDir, "prettier.config.mjs"),
        `export default {
  plugins: [
    {
      packageName: "@scope/prettier-plugin-inline",
      languages: [{ name: "inline", parsers: ["inline"], extensions: [".inline"] }],
      parsers: { inline: { astFormat: "inline-ast", parse() { return { type: "inline-ast" }; } } },
      printers: { "inline-ast": { print() { return ""; } } },
    },
  ],
  singleQuote: true,
};
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);
      expect(result.stderr).toContain("recognized a formatter plugin object via metadata");
      expect(result.stderr).toContain('using "@scope/prettier-plugin-inline"');

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.plugins).toEqual(["@scope/prettier-plugin-inline"]);
      expect(oxfmtrc.singleQuote).toBe(true);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should preserve override-scoped formatter plugin objects with meta.packageName metadata", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      await fs.writeFile(
        join(tempDir, ".prettierrc.cjs"),
        `module.exports = {
  overrides: [
    {
      files: ["*.inline"],
      options: {
        plugins: [
          {
            meta: { packageName: "prettier-plugin-inline-cjs" },
            languages: [{ name: "inline", parsers: ["inline"], extensions: [".inline"] }],
            parsers: { inline: { astFormat: "inline-ast", parse() { return { type: "inline-ast" }; } } },
            printers: { "inline-ast": { print() { return ""; } } },
          },
        ],
        semi: false,
      },
    },
  ],
};
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);
      expect(result.stderr).toContain("recognized a formatter plugin object via metadata");

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.overrides).toEqual([
        {
          files: ["*.inline"],
          options: {
            plugins: ["prettier-plugin-inline-cjs"],
            semi: false,
          },
        },
      ]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should warn and skip plugin objects with non-plugin metadata names", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      await fs.writeFile(
        join(tempDir, "prettier.config.mjs"),
        `export default {
  printWidth: 80,
  embeddedLanguageFormatting: "off",
  plugins: [
    {
      name: "totally-random-inline-object",
      languages: [{ name: "inline", parsers: ["inline"], extensions: [".inline"] }],
      parsers: { inline: { astFormat: "inline-ast", parse() { return { type: "inline-ast" }; } } },
      printers: { "inline-ast": { print() { return ""; } } },
    },
  ],
};
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);
      expect(result.stderr).toContain("plugins: custom plugin module is not supported, skipping");
      expect(result.stderr).toContain("Migration completed with 1 warning(s)");

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.plugins).toBeUndefined();
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should print a clean migration summary when no warnings are emitted", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          semi: false,
          printWidth: 80,
          embeddedLanguageFormatting: "off",
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);
      expect(result.stderr).toContain("Migration completed without warnings.");
      expect(result.stderr).toContain("Migration summary:");
      expect(result.stderr).not.toContain("Migration completed with 1 warning(s)");
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should print a migration summary for inferred and skipped plugin objects", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      const customPluginSpecifier = await writeCustomFormatterPluginModule(tempDir, "mjs");
      await fs.writeFile(
        join(tempDir, "prettier.config.mjs"),
        `const preservedPlugin = {
  name: ${JSON.stringify(customPluginSpecifier)},
  languages: [{ name: "custom", parsers: ["custom"], extensions: [".custom"] }],
  parsers: {
    custom: {
      parse() {
        return { type: "Program", body: [] };
      }
    }
  }
};

const skippedPlugin = {
  languages: [{ name: "skipped", parsers: ["skipped"], extensions: [".skipped"] }],
  parsers: {
    skipped: {
      parse() {
        return { type: "Program", body: [] };
      }
    }
  }
};

export default {
  plugins: [preservedPlugin, skippedPlugin, "prettier-plugin-tailwindcss", "prettier-plugin-packagejson"],
  tailwindFunctions: ["clsx"],
  overrides: [
    {
      options: {
        semi: false
      }
    },
    {
      files: ["*.custom"],
      options: {
        plugins: [preservedPlugin]
      }
    }
  ]
};
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);
      expect(result.stderr).toContain("Migration summary:");
      expect(result.stderr).toContain("overrides skipped: 1");
      expect(result.stderr).toContain(`preserved formatter plugins: ${customPluginSpecifier}`);
      expect(result.stderr).toContain(`plugin objects converted to string specs: ${customPluginSpecifier}`);
      expect(result.stderr).toContain("migrated prettier-plugin-tailwindcss in: <root>");
      expect(result.stderr).toContain("migrated prettier-plugin-packagejson in: <root>");
      expect(result.stderr).toContain("unsupported custom plugin objects skipped: 1");

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.plugins).toEqual([customPluginSpecifier]);
      expect(oxfmtrc.sortPackageJson).toEqual({});
      expect(oxfmtrc.sortTailwindcss).toEqual({ functions: ["clsx"] });
      expect(oxfmtrc.overrides).toEqual([
        {
          files: ["*.custom"],
          options: {
            plugins: [customPluginSpecifier],
          },
        },
      ]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should warn about regex values in tailwindFunctions and tailwindAttributes", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config with regex values in Tailwind options
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          plugins: ["prettier-plugin-tailwindcss"],
          tailwindFunctions: ["clsx", "/^tw-/"],
          tailwindAttributes: ["className", "/^data-tw-/"],
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      // Check warnings are printed for regex values
      expect(result.stderr).toContain('Do not support regex in "tailwindFunctions"');
      expect(result.stderr).toContain("/^tw-/");
      expect(result.stderr).toContain('Do not support regex in "tailwindAttributes"');
      expect(result.stderr).toContain("/^data-tw-/");

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      // Non-regex values should still be migrated
      expect(oxfmtrc.sortTailwindcss.functions).toEqual(["clsx", "/^tw-/"]);
      expect(oxfmtrc.sortTailwindcss.attributes).toEqual(["className", "/^data-tw-/"]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should disable sortPackageJson by default", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config without package.json sorting plugin
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          semi: false,
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      // Prettier does not have package.json sorting by default
      expect(oxfmtrc.sortPackageJson).toBe(false);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should enable sortPackageJson when prettier-plugin-packagejson is used", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config with package.json sorting plugin
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          plugins: ["prettier-plugin-packagejson"],
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.sortPackageJson).toBeTruthy();
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should warn clearly for lossy migration cases and keep the surviving config explicit", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      await fs.writeFile(
        join(tempDir, "prettier.config.mjs"),
        `const inlinePlugin = {
  languages: [{ name: "custom-inline", parsers: ["custom-inline"], extensions: [".custom-inline"] }],
  parsers: { "custom-inline": { astFormat: "custom-inline-ast", parse() { return { type: "custom-inline-ast" }; } } },
  printers: { "custom-inline-ast": { print() { return ""; } } },
};

export default {
  semi: false,
  endOfLine: "auto",
  experimentalTernaries: true,
  embeddedLanguageFormatting: "auto",
  plugins: [inlinePlugin],
  overrides: [
    {
      files: ["*.svelte"],
      options: {
        plugins: [inlinePlugin],
        embeddedLanguageFormatting: "auto",
        endOfLine: "auto",
      },
    },
  ],
};
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);
      expect(result.stderr).toContain('plugins: custom plugin module is not supported, skipping...');
      expect(result.stderr).toContain('"endOfLine: auto" is not supported, skipping...');
      expect(result.stderr).toContain('"experimentalTernaries" is not supported in JS/TS files yet');
      expect(result.stderr).toContain(
        '"embeddedLanguageFormatting" in JS/TS files is not fully supported yet',
      );
      expect(result.stderr).toContain(
        '"printWidth" is not set in Prettier config, defaulting to 80 (Oxfmt default: 100)',
      );
      expect(result.stderr).toContain(
        'overrides[0].options: plugins: custom plugin module is not supported, skipping...',
      );
      expect(result.stderr).toContain(
        'overrides[0].options: "endOfLine: auto" is not supported, skipping...',
      );

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.semi).toBe(false);
      expect(oxfmtrc.printWidth).toBe(80);
      expect(oxfmtrc.embeddedLanguageFormatting).toBe("auto");
      expect(oxfmtrc.plugins).toBeUndefined();
      expect(oxfmtrc.endOfLine).toBeUndefined();
      expect(oxfmtrc.experimentalTernaries).toBeUndefined();
      expect(oxfmtrc.overrides).toEqual([
        {
          files: ["*.svelte"],
          options: {
            embeddedLanguageFormatting: "auto",
          },
        },
      ]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

});
