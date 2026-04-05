import { defineConfig } from "../src-js/index.ts";
import type { OxlintConfig, OxlintExtendsEntry, RuleCategoryConfig } from "../src-js/index.ts";

const category: RuleCategoryConfig = "recommended";
void category;

const base = defineConfig({
  rules: {
    "no-console": "warn",
  },
});

const extendsEntries: OxlintConfig["extends"] = ["oxlint-config-svelte", base];
void extendsEntries;

const config: OxlintConfig = defineConfig({
  extends: ["oxlint-config-svelte", base],
  categories: {
    suspicious: "recommended",
  },
});

void config;


const parser = {
  parseForESLint(code: string, options?: Record<string, unknown>) {
    void code;
    void options;
    return { ast: null };
  },
};

const configWithLanguageOptions = defineConfig({
  languageOptions: {
    parser,
    parserOptions: {
      parser,
      svelteConfig: {
        compilerOptions: {
          runes: true,
        },
      },
    },
  },
  overrides: [
    {
      files: ["**/*.svelte"],
      languageOptions: {
        parser,
        parserOptions: {
          projectService: true,
          extraFileExtensions: [".svelte"],
        },
      },
    },
  ],
});

void configWithLanguageOptions;


const nestedTsParser = {
  parseForESLint(code: string, options?: Record<string, unknown>) {
    void code;
    void options;
    return { ast: null };
  },
};

const svelteConfig = {
  compilerOptions: {
    runes: true,
  },
  preprocess() {
    return null;
  },
};

const typeAwareBase = defineConfig({
  languageOptions: {
    parserOptions: {
      parser: nestedTsParser,
      svelteConfig,
      tsFlavor: "base-ts-parser",
    },
  },
});

const typeAwareConfig = defineConfig({
  extends: [typeAwareBase],
  overrides: [
    {
      files: ["**/*.svelte"],
      languageOptions: {
        parser,
        parserOptions: {
          projectService: true,
          extraFileExtensions: [".svelte"],
        },
      },
    },
  ],
});

void typeAwareConfig;

const flatCompatSvelte = [
  {
    name: "svelte:ignores",
    ignores: [".svelte-kit/**", "build/**"],
  },
  {
    name: "svelte:base:setup-plugin",
    plugins: {
      svelte: {
        meta: {
          name: "eslint-plugin-svelte",
        },
        rules: {},
      },
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
      "svelte/no-useless-mustaches": "error",
    },
  },
] satisfies OxlintExtendsEntry[];

const configWithFlatCompatExtends = defineConfig({
  extends: [flatCompatSvelte],
});

void configWithFlatCompatExtends;
