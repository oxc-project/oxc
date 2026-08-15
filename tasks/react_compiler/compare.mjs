import assert from "node:assert/strict";
import { readdir, readFile, stat } from "node:fs/promises";
import { extname, relative, resolve } from "node:path";

import { transformSync as babelTransformSync } from "@babel/core";
import transformReactJsx from "@babel/plugin-transform-react-jsx";
import transformTypescript from "@babel/plugin-transform-typescript";
import reactCompiler from "babel-plugin-react-compiler";
import { transformSync as oxcCodegenSync } from "oxc-transform";
import { transformSync as oxcReactCompilerSync } from "oxc-transform-react";

const SOURCE_EXTENSIONS = new Set([".jsx", ".tsx"]);
const DEFAULT_ESLINT_SUPPRESSION_RULES_V1 = [
  "react-hooks/exhaustive-deps",
  "react-hooks/rules-of-hooks",
];
const REACT_COMPILER_ENVIRONMENT_OPTIONS_V1 = {
  customMacros: null,
  enableResetCacheOnSourceFileChanges: null,
  enablePreserveExistingMemoizationGuarantees: true,
  validatePreserveExistingMemoizationGuarantees: true,
  validateExhaustiveMemoizationDependencies: false,
  validateExhaustiveEffectDependencies: "off",
  enableOptionalDependencies: true,
  enableNameAnonymousFunctions: false,
  validateHooksUsage: true,
  validateRefAccessDuringRender: true,
  validateNoSetStateInRender: true,
  enableUseKeyedState: false,
  validateNoSetStateInEffects: false,
  validateNoDerivedComputationsInEffects: false,
  validateNoDerivedComputationsInEffectsExperimental: false,
  validateNoJsxInTryStatements: false,
  validateStaticComponents: false,
  validateNoCapitalizedCalls: null,
  validateBlocklistedImports: null,
  validateSourceLocations: false,
  validateNoImpureFunctionsInRender: false,
  validateNoFreezingKnownMutableFunctions: false,
  enableAssumeHooksFollowRulesOfReact: true,
  enableTransitivelyFreezeFunctionExpressions: true,
  enableFunctionOutlining: true,
  enableJsxOutlining: false,
  assertValidMutableRanges: false,
  enableCustomTypeDefinitionForReanimated: false,
  enableTreatRefLikeIdentifiersAsRefs: true,
  enableTreatSetIdentifiersAsStateSetters: false,
  validateNoVoidUseMemo: false,
  enableAllowSetStateFromRefsInEffects: true,
  enableVerboseNoSetStateInEffect: false,
  enableForest: false,
};
const REACT_COMPILER_OPTIONS_V1 = {
  compilationMode: "infer",
  panicThreshold: "none",
  target: "19",
  gating: null,
  dynamicGating: null,
  noEmit: false,
  outputMode: null,
  eslintSuppressionRules: DEFAULT_ESLINT_SUPPRESSION_RULES_V1,
  flowSuppressions: true,
  ignoreUseNoForget: false,
  customOptOutDirectives: null,
};
const BABEL_TYPESCRIPT_OPTIONS = {
  allExtensions: true,
  isTSX: true,
  // Match Oxc's default TypeScript emit for uninitialized class fields.
  allowDeclareFields: true,
};

const args = process.argv.slice(2);
if (args.length !== 1) {
  throw new Error("Usage: pnpm --filter react_compiler compare <directory>");
}

const invocationDirectory = process.env.INIT_CWD ?? process.cwd();
const directory = resolve(invocationDirectory, args[0]);
if (!(await stat(directory)).isDirectory()) {
  throw new Error(`Not a directory: ${directory}`);
}
const {
  validateNoDerivedComputationsInEffectsExperimental,
  validateNoJsxInTryStatements,
  ...sharedEnvironmentOptions
} = REACT_COMPILER_ENVIRONMENT_OPTIONS_V1;
const babelReactCompilerOptions = {
  ...REACT_COMPILER_OPTIONS_V1,
  logger: null,
  enableReanimatedCheck: true,
  sources: [directory],
  environment: {
    ...sharedEnvironmentOptions,
    // These two Babel spellings differ from the Oxc NAPI property names.
    validateNoDerivedComputationsInEffects_exp: validateNoDerivedComputationsInEffectsExperimental,
    validateNoJSXInTryStatements: validateNoJsxInTryStatements,
    // Oxc has the same neutral defaults internally, but does not expose these
    // callback/test-only options across the NAPI boundary.
    customHooks: new Map(),
    moduleTypeProvider: null,
    flowTypeProvider: null,
    enableEmitHookGuards: null,
    enableEmitInstrumentForget: null,
    throwUnknownException__testonly: false,
  },
};
const oxcReactCompilerOptions = {
  // napi-rs represents optional object/list fields by omission and does not
  // accept the explicit `null` values used by Babel's resolved configuration.
  ...omitNullOptionsForNapi(REACT_COMPILER_OPTIONS_V1),
  sources: [directory],
  environment: {
    ...omitNullOptionsForNapi(sharedEnvironmentOptions),
    validateNoDerivedComputationsInEffectsExp: validateNoDerivedComputationsInEffectsExperimental,
    validateNoJsxInTryStatements,
  },
};

const filenames = await findSourceFiles(directory);
let differenceCount = 0;
let failureCount = 0;

for (const filename of filenames) {
  const sourceText = await readFile(filename, "utf8");
  const displayPath = relative(directory, filename);

  try {
    const babelOutput = transformWithBabel(filename, sourceText);
    const oxcOutput = transformWithOxc(filename, sourceText);
    if (babelOutput !== oxcOutput) {
      console.log(displayPath);
      differenceCount++;
    }
  } catch (error) {
    console.log(displayPath);
    console.error(`${displayPath}: ${error instanceof Error ? error.message : String(error)}`);
    differenceCount++;
    failureCount++;
  }
}

console.error(
  `Compared ${filenames.length} files: ${differenceCount} different, ${failureCount} failed to transform.`,
);

if (differenceCount > 0) {
  process.exitCode = 1;
}

async function findSourceFiles(root) {
  const entries = await readdir(root, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const path = resolve(root, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await findSourceFiles(path)));
    } else if (entry.isFile() && SOURCE_EXTENSIONS.has(extname(entry.name).toLowerCase())) {
      files.push(path);
    }
  }

  return files.sort();
}

function transformWithBabel(filename, sourceText) {
  const plugins = [[reactCompiler, babelReactCompilerOptions]];
  if (extname(filename).toLowerCase() === ".tsx") {
    plugins.push([transformTypescript, BABEL_TYPESCRIPT_OPTIONS]);
  }
  plugins.push([transformReactJsx, { runtime: "automatic" }]);

  const result = babelTransformSync(sourceText, {
    babelrc: false,
    comments: true,
    configFile: false,
    filename,
    plugins,
    sourceMaps: false,
    sourceType: "unambiguous",
  });

  assert(result?.code, "Babel did not produce code");

  const normalizedResult = oxcCodegenSync(filename, result.code, {
    lang: "js",
    sourceType: "unambiguous",
  });
  assertNoErrors(normalizedResult, "Oxc failed to codegen Babel output");
  assert(normalizedResult.code, "Oxc did not codegen Babel output");
  return normalizedResult.code;
}

function transformWithOxc(filename, sourceText) {
  const result = oxcReactCompilerSync(filename, sourceText, {
    reactCompiler: oxcReactCompilerOptions,
  });
  assertNoErrors(result, "oxc-transform-react failed");
  assert(result.code, "oxc-transform-react did not produce code");
  return result.code;
}

function assertNoErrors(result, message) {
  const errors = result.errors.filter(({ severity }) => severity === "Error");
  assert.equal(errors.length, 0, `${message}\n${errors.map(({ message }) => message).join("\n")}`);
}

function omitNullOptionsForNapi(options) {
  return Object.fromEntries(Object.entries(options).filter(([, value]) => value !== null));
}
