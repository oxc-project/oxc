import { Linter } from "eslint";

// NOTICE!
// Plugins do not provide their type definitions, and also `@types/*` do not exist!
// Even worse, every plugin has slightly different types, different way of configuration in detail...
// Some Plugins exports the rules and config separately, some only using a default export.
// Rules with a default exports will be destructured after the imports.
//
// So here, we need to list all rules while normalizing recommended and deprecated flags.
// - rule.meta.deprecated
// - rule.meta.docs.recommended
// Some plugins have the recommended flag in rule itself, but some plugins have it in config.

// https://github.com/typescript-eslint/typescript-eslint/blob/v8.9.0/packages/eslint-plugin/src/index.ts
// @ts-ignore
import pluginTypescript from "@typescript-eslint/eslint-plugin";
// https://github.com/eslint-community/eslint-plugin-n/blob/v17.13.2/lib/index.js
// @ts-ignore
import pluginNAll from "eslint-plugin-n";
// https://github.com/sindresorhus/eslint-plugin-unicorn/blob/v57.0.0/index.js
// @ts-ignore
import pluginUnicorn from "eslint-plugin-unicorn";
// https://github.com/gajus/eslint-plugin-jsdoc/blob/v50.5.0/src/index.js
// @ts-ignore
import pluginJSDoc from "eslint-plugin-jsdoc";
// https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/src/index.js
import {
  configs as pluginImportConfigs,
  rules as pluginImportAllRules,
  // @ts-ignore
} from "eslint-plugin-import";
// https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/v6.9.0/src/index.js
// @ts-ignore
import pluginJSXA11y from "eslint-plugin-jsx-a11y";
// https://github.com/jest-community/eslint-plugin-jest/blob/v28.9.0/src/index.ts
// @ts-ignore
import pluginJest from "eslint-plugin-jest";
// https://github.com/jsx-eslint/eslint-plugin-react/blob/v7.37.2/index.js
// @ts-ignore
import pluginReact from "eslint-plugin-react";
// https://github.com/facebook/react/blob/v19.2.0/packages/eslint-plugin-react-hooks/src/index.ts
// @ts-ignore
import pluginReactHooks from "eslint-plugin-react-hooks";
// https://github.com/cvazac/eslint-plugin-react-perf/blob/9bfa930661a23218f5460ebd39d35d76ccdb5724/index.js
// @ts-ignore
import pluginReactPerf from "eslint-plugin-react-perf";
// https://github.com/vercel/next.js/blob/canary/packages/eslint-plugin-next/src/index.ts
// @ts-ignore
import pluginNext from "@next/eslint-plugin-next";
// https://github.com/eslint-community/eslint-plugin-promise/blob/v7.1.0/index.js
// @ts-ignore
import pluginPromise from "eslint-plugin-promise";
// https://github.com/veritem/eslint-plugin-vitest/blob/v1.1.9/src/index.ts
// @ts-ignore
import pluginVitest from "eslint-plugin-vitest";
// https://github.com/vuejs/eslint-plugin-vue
// @ts-ignore
import pluginVue from "eslint-plugin-vue";

type AnyRules = Record<string, any>;
type AnyConfigs = Record<string, any>;

// destructuring default exports (all plugins lack proper TypeScript types)
const pluginTypeScriptConfigs: AnyConfigs = (pluginTypescript as any).configs;
const pluginTypeScriptAllRules: AnyRules = (pluginTypescript as any).rules;
const pluginNAllRules: AnyRules = (pluginNAll as any).rules;
const pluginUnicornConfigs: AnyConfigs = (pluginUnicorn as any).configs;
const pluginUnicornAllRules: AnyRules = (pluginUnicorn as any).rules;
const pluginJSDocAllRules: AnyRules = (pluginJSDoc as any).rules;
const pluginJSDocConfigs: AnyConfigs = (pluginJSDoc as any).configs;
const pluginJSXA11yAllRules: AnyRules = (pluginJSXA11y as any).rules;
const pluginJSXA11yConfigs: AnyConfigs = (pluginJSXA11y as any).configs;
const pluginJestAllRules: AnyRules = (pluginJest as any).rules;
const pluginJestConfigs: AnyConfigs = (pluginJest as any).configs;
const pluginPromiseConfigs: AnyConfigs = (pluginPromise as any).configs;
const pluginPromiseRules: AnyRules = (pluginPromise as any).rules;
const pluginReactAllRules: AnyRules = (pluginReact as any).rules;
const pluginReactHooksAllRules: AnyRules = (pluginReactHooks as any).rules;
const pluginReactPerfAllRules: AnyRules = (pluginReactPerf as any).rules;
const pluginReactPerfConfigs: AnyConfigs = (pluginReactPerf as any).configs;
const pluginNextAllRules: AnyRules = (pluginNext as any).rules;
const pluginVitestConfigs: AnyConfigs = (pluginVitest as any).configs;
const pluginVitestRules: AnyRules = (pluginVitest as any).rules;
const pluginVueConfigs: AnyConfigs = (pluginVue as any).configs;
const pluginVueRules: AnyRules = (pluginVue as any).rules;

const loadPluginTypeScriptRules = (linter: Linter) => {
  // We want to list all rules but not support type-checked rules
  const pluginTypeScriptDisableTypeCheckedRules = new Map(
    Object.entries(pluginTypeScriptConfigs["disable-type-checked"].rules),
  );
  for (const [name, rule] of Object.entries(pluginTypeScriptAllRules)) {
    if (pluginTypeScriptDisableTypeCheckedRules.has(`@typescript-eslint/${name}`)) {
      continue;
    }

    const prefixedName = `typescript/${name}`;

    // Recommended can either be
    // - a string describing which configuration it belongs to (recommended, strict, stylistic)
    // - an object with a `recommended` property (ban-ts-comment)
    // - undefined
    let isRecommended = rule.meta.docs.recommended;
    if (typeof isRecommended === "object" && isRecommended !== null) {
      isRecommended = isRecommended.recommended === true;
    } else if (typeof isRecommended === "string") {
      isRecommended = isRecommended === "recommended";
    } else {
      isRecommended = false;
    }
    rule.meta.docs.recommended = isRecommended;

    linter.defineRule(prefixedName, rule);
  }
};

const loadPluginNRules = (linter: Linter) => {
  for (const [name, rule] of Object.entries(pluginNAllRules)) {
    const prefixedName = `n/${name}`;

    linter.defineRule(prefixedName, rule);
  }
};

const loadPluginUnicornRules = (linter: Linter) => {
  const pluginUnicornRecommendedRules = new Map(
    Object.entries(pluginUnicornConfigs.recommended.rules),
  );
  for (const [name, rule] of Object.entries(pluginUnicornAllRules)) {
    const prefixedName = `unicorn/${name}`;

    // If name is presented and value is not "off", it is recommended
    const recommendedValue = pluginUnicornRecommendedRules.get(prefixedName);
    rule.meta.docs.recommended = recommendedValue && recommendedValue !== "off";

    linter.defineRule(prefixedName, rule);
  }
};

const loadPluginJSDocRules = (linter: Linter) => {
  const pluginJSDocRecommendedRules = new Map(Object.entries(pluginJSDocConfigs.recommended.rules));
  for (const [name, rule] of Object.entries(pluginJSDocAllRules)) {
    const prefixedName = `jsdoc/${name}`;

    // If name is presented and value is not "off", it is recommended
    const recommendedValue = pluginJSDocRecommendedRules.get(prefixedName);
    rule.meta.docs.recommended = recommendedValue && recommendedValue !== "off";

    linter.defineRule(prefixedName, rule);
  }
};

const loadPluginImportRules = (linter: Linter) => {
  const pluginImportRecommendedRules = new Map(
    Object.entries((pluginImportConfigs as AnyConfigs).recommended.rules),
  );
  for (const [name, rule] of Object.entries(pluginImportAllRules as AnyRules)) {
    const prefixedName = `import/${name}`;

    rule.meta.docs.recommended = pluginImportRecommendedRules.has(prefixedName);

    linter.defineRule(prefixedName, rule);
  }
};

const loadPluginJSXA11yRules = (linter: Linter) => {
  const pluginJSXA11yRecommendedRules = new Map<string, any>(
    Object.entries(pluginJSXA11yConfigs.recommended.rules),
  );
  for (const [name, rule] of Object.entries(pluginJSXA11yAllRules)) {
    const prefixedName = `jsx-a11y/${name}`;

    const recommendedValue = pluginJSXA11yRecommendedRules.get(prefixedName);
    rule.meta.docs.recommended =
      recommendedValue &&
      // Type is `string | [string, opt]`
      recommendedValue !== "off" &&
      recommendedValue[0] !== "off";

    linter.defineRule(prefixedName, rule);
  }
};

const loadPluginJestRules = (linter: Linter) => {
  const pluginJestRecommendedRules = new Map(Object.entries(pluginJestConfigs.recommended.rules));
  for (const [name, rule] of Object.entries(pluginJestAllRules)) {
    const prefixedName = `jest/${name}`;

    const recommendedValue = pluginJestRecommendedRules.get(prefixedName);
    // Presented but type is `string | undefined`
    rule.meta.docs.recommended = typeof recommendedValue === "string";

    linter.defineRule(prefixedName, rule);
  }
};

const loadPluginReactRules = (linter: Linter) => {
  for (const [name, rule] of Object.entries(pluginReactAllRules)) {
    const prefixedName = `react/${name}`;

    linter.defineRule(prefixedName, rule);
  }

  // `react-hooks` plugin is available along with `react` plugin
  for (const [name, rule] of Object.entries(pluginReactHooksAllRules)) {
    // This may be conflict with `react` plugin
    // (but `react-hooks` plugin has only 2 rules, so it's fine...!)
    const prefixedName = `react/${name}`;

    linter.defineRule(prefixedName, rule);
  }
};

const loadPluginReactPerfRules = (linter: Linter) => {
  const pluginReactPerfRecommendedRules = new Map(
    Object.entries(pluginReactPerfConfigs.recommended.rules),
  );
  for (const [name, rule] of Object.entries(pluginReactPerfAllRules)) {
    const prefixedName = `react-perf/${name}`;

    rule.meta.docs.recommended = pluginReactPerfRecommendedRules.has(prefixedName);

    linter.defineRule(prefixedName, rule);
  }
};

const loadPluginNextRules = (linter: Linter) => {
  for (const [name, rule] of Object.entries(pluginNextAllRules)) {
    const prefixedName = `nextjs/${name}`;

    linter.defineRule(prefixedName, rule);
  }
};

const loadPluginPromiseRules = (linter: Linter) => {
  const pluginPromiseRecommendedRules = new Map(
    Object.entries(pluginPromiseConfigs.recommended.rules),
  );
  for (const [name, rule] of Object.entries(pluginPromiseRules)) {
    const prefixedName = `promise/${name}`;

    if (rule.meta && rule.meta.docs) {
      rule.meta.docs.recommended = pluginPromiseRecommendedRules.has(prefixedName);
    }

    linter.defineRule(prefixedName, rule);
  }
};

const loadPluginVitestRules = (linter: Linter) => {
  const pluginVitestRecommendedRules = new Map(
    Object.entries(pluginVitestConfigs.recommended.rules),
  );
  for (const [name, rule] of Object.entries(pluginVitestRules)) {
    const prefixedName = `vitest/${name}`;

    if (rule.meta?.docs) {
      rule.meta.docs.recommended = pluginVitestRecommendedRules.has(prefixedName);
    }

    linter.defineRule(prefixedName, rule);
  }
};

const loadPluginVueRules = (linter: Linter) => {
  const pluginVueRecommendedRules = new Map(
    Object.entries(pluginVueConfigs.recommended.rules || {}),
  );
  for (const [name, rule] of Object.entries(pluginVueRules)) {
    const prefixedName = `vue/${name}`;

    rule.meta.docs.recommended = pluginVueRecommendedRules.has(prefixedName);

    linter.defineRule(prefixedName, rule);
  }
};

export type TargetPluginMeta = {
  npm: string[];
  issueNo: number;
};

export const ALL_TARGET_PLUGINS = new Map<string, TargetPluginMeta>([
  ["eslint", { npm: ["eslint"], issueNo: 479 }],
  ["typescript", { npm: ["@typescript-eslint/eslint-plugin"], issueNo: 2180 }],
  ["n", { npm: ["eslint-plugin-n"], issueNo: 493 }],
  ["unicorn", { npm: ["eslint-plugin-unicorn"], issueNo: 684 }],
  ["jsdoc", { npm: ["eslint-plugin-jsdoc"], issueNo: 1170 }],
  ["import", { npm: ["eslint-plugin-import"], issueNo: 1117 }],
  ["jsx-a11y", { npm: ["eslint-plugin-jsx-a11y"], issueNo: 1141 }],
  ["jest", { npm: ["eslint-plugin-jest"], issueNo: 492 }],
  [
    "react",
    {
      npm: ["eslint-plugin-react", "eslint-plugin-react-hooks"],
      issueNo: 1022,
    },
  ],
  ["react-perf", { npm: ["eslint-plugin-react-perf"], issueNo: 2041 }],
  ["nextjs", { npm: ["@next/eslint-plugin-next"], issueNo: 1929 }],
  ["promise", { npm: ["eslint-plugin-promise"], issueNo: 4655 }],
  ["vitest", { npm: ["eslint-plugin-vitest"], issueNo: 4656 }],
  ["vue", { npm: ["eslint-plugin-vue"], issueNo: 11440 }],
]);

// All rules(including deprecated, recommended) are loaded initially.
export const createESLintLinter = () =>
  new Linter({
    configType: "eslintrc",
  });

export const loadTargetPluginRules = (linter: Linter) => {
  loadPluginTypeScriptRules(linter);
  loadPluginNRules(linter);
  loadPluginUnicornRules(linter);
  loadPluginJSDocRules(linter);
  loadPluginImportRules(linter);
  loadPluginJSXA11yRules(linter);
  loadPluginJestRules(linter);
  loadPluginReactRules(linter);
  loadPluginReactPerfRules(linter);
  loadPluginNextRules(linter);
  loadPluginPromiseRules(linter);
  loadPluginVitestRules(linter);
  loadPluginVueRules(linter);
};
