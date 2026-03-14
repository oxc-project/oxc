#!/usr/bin/env node

/**
 * compare-tests.mjs
 *
 * Compares upstream ESLint test cases (evaluated via Node.js) against the
 * statically-extracted test cases in oxlint's Rust rule files.
 *
 * Usage:
 *   node tasks/rulegen/compare-tests.mjs <rule-name> <plugin>
 *
 * Examples:
 *   node tasks/rulegen/compare-tests.mjs no-empty-function eslint
 *   node tasks/rulegen/compare-tests.mjs no-unnecessary-condition typescript
 *   node tasks/rulegen/compare-tests.mjs no-empty-function eslint --json
 */

import { execSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PROJECT_ROOT = path.resolve(__dirname, "../..");

// ── Plugin configuration ────────────────────────────────────────────────
// Maps plugin names to { testUrl, extension, rustDir }

const PLUGINS = {
  eslint: {
    testUrl: "https://raw.githubusercontent.com/eslint/eslint/main/tests/lib/rules",
    ext: ".js",
    rustDir: "eslint",
  },
  typescript: {
    testUrl:
      "https://raw.githubusercontent.com/typescript-eslint/typescript-eslint/main/packages/eslint-plugin/tests/rules",
    ext: ".test.ts",
    rustDir: "typescript",
  },
  jest: {
    testUrl:
      "https://raw.githubusercontent.com/jest-community/eslint-plugin-jest/main/src/rules/__tests__",
    ext: ".test.ts",
    rustDir: "jest",
  },
  unicorn: {
    testUrl: "https://raw.githubusercontent.com/sindresorhus/eslint-plugin-unicorn/main/test",
    ext: ".js",
    rustDir: "unicorn",
  },
  import: {
    testUrl:
      "https://raw.githubusercontent.com/import-js/eslint-plugin-import/main/tests/src/rules",
    ext: ".js",
    rustDir: "import",
  },
  react: {
    testUrl:
      "https://raw.githubusercontent.com/jsx-eslint/eslint-plugin-react/master/tests/lib/rules",
    ext: ".js",
    rustDir: "react",
  },
  "react-perf": {
    testUrl:
      "https://raw.githubusercontent.com/cvazac/eslint-plugin-react-perf/master/tests/lib/rules",
    ext: ".js",
    rustDir: "react_perf",
  },
  "jsx-a11y": {
    testUrl:
      "https://raw.githubusercontent.com/jsx-eslint/eslint-plugin-jsx-a11y/main/__tests__/src/rules",
    ext: "-test.js",
    rustDir: "jsx_a11y",
  },
  nextjs: {
    testUrl: "https://raw.githubusercontent.com/vercel/next.js/canary/test/unit/eslint-plugin-next",
    ext: ".test.ts",
    rustDir: "nextjs",
  },
  jsdoc: {
    testUrl:
      "https://raw.githubusercontent.com/gajus/eslint-plugin-jsdoc/main/test/rules/assertions",
    ext: ".js",
    useCamelCase: true,
    rustDir: "jsdoc",
  },
  node: {
    testUrl:
      "https://raw.githubusercontent.com/eslint-community/eslint-plugin-n/master/tests/lib/rules",
    ext: ".js",
    rustDir: "node",
  },
  promise: {
    testUrl:
      "https://raw.githubusercontent.com/eslint-community/eslint-plugin-promise/main/__tests__",
    ext: ".js",
    rustDir: "promise",
  },
  vitest: {
    testUrl: "https://raw.githubusercontent.com/vitest-dev/eslint-plugin-vitest/main/tests",
    ext: ".test.ts",
    rustDir: "vitest",
  },
  vue: {
    testUrl: "https://raw.githubusercontent.com/vuejs/eslint-plugin-vue/master/tests/lib/rules",
    ext: ".js",
    rustDir: "vue",
  },
};

// ── CLI parsing ─────────────────────────────────────────────────────────

const args = process.argv.slice(2);
const jsonOutput = args.includes("--json");
const positional = args.filter((a) => !a.startsWith("--"));

if (positional.length < 2) {
  console.error("Usage: node tasks/rulegen/compare-tests.mjs <rule-name> <plugin> [--json]");
  console.error("  Plugins: " + Object.keys(PLUGINS).join(", "));
  process.exit(1);
}

const [ruleName, pluginName] = positional;
const plugin = PLUGINS[pluginName];
if (!plugin) {
  console.error(`Unknown plugin: ${pluginName}`);
  console.error("  Supported: " + Object.keys(PLUGINS).join(", "));
  process.exit(1);
}

// ── Helpers ─────────────────────────────────────────────────────────────

function toKebabCase(str) {
  return str.replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase();
}

function toCamelCase(str) {
  return str.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
}

function toSnakeCase(str) {
  return str.replace(/-/g, "_");
}

/**
 * Normalize a code string for fuzzy comparison.
 * 1. Normalizes CommonJS and ESM syntax to be equivalent
 * 2. Collapses all whitespace (including newlines) to single spaces
 */
function normalize(code) {
  if (typeof code !== "string") return "";

  let normalized = code;

  // Normalize CommonJS to ESM for comparison
  // const mod = require('module') → import mod from 'module'
  normalized = normalized.replace(
    /const\s+(\w+)\s*=\s*require\s*\(\s*(['"][^'"]+['"])\s*\)\s*;?/g,
    "import $1 from $2;",
  );

  // const { a, b } = require('module') → import { a, b } from 'module'
  normalized = normalized.replace(
    /const\s*\{([^}]+)\}\s*=\s*require\s*\(\s*(['"][^'"]+['"])\s*\)\s*;?/g,
    "import {$1} from $2;",
  );

  // let mod = require('module') → import mod from 'module'
  normalized = normalized.replace(
    /let\s+(\w+)\s*=\s*require\s*\(\s*(['"][^'"]+['"])\s*\)\s*;?/g,
    "import $1 from $2;",
  );

  // var mod = require('module') → import mod from 'module'
  normalized = normalized.replace(
    /var\s+(\w+)\s*=\s*require\s*\(\s*(['"][^'"]+['"])\s*\)\s*;?/g,
    "import $1 from $2;",
  );

  // require('module') → import 'module' (side effect import)
  normalized = normalized.replace(/require\s*\(\s*(['"][^'"]+['"])\s*\)\s*;?/g, "import $1;");

  // module.exports = x → export default x
  normalized = normalized.replace(/module\.exports\s*=\s*/g, "export default ");

  // export { x }; ... → export { x };... (keep exports as-is, just normalize spacing)

  // Replace all sequences of whitespace (spaces, tabs, newlines) with a single space
  // Then trim leading/trailing whitespace
  return normalized.replace(/\s+/g, " ").trim();
}

// ── Step 1: Download and evaluate the upstream test file ────────────────

async function downloadTestFile(ruleName, plugin) {
  const kebab = toKebabCase(ruleName);
  const name = plugin.useCamelCase ? toCamelCase(kebab) : kebab;
  const url = `${plugin.testUrl}/${name}${plugin.ext}`;

  console.error(`Downloading ${url} ...`);

  const resp = await fetch(url);
  if (!resp.ok) {
    throw new Error(`Failed to download test file: ${resp.status} ${url}`);
  }
  return { body: await resp.text(), url };
}

/**
 * Evaluate the upstream test file with mocked RuleTester to capture test cases.
 *
 * Strategy:
 *   1. For .js files: convert ESM→CJS, run with --require mock
 *   2. For .ts files: convert ESM→CJS, try esbuild first, then Node's built-in
 *      type stripping (--experimental-strip-types), then naive regex stripping
 */
function evaluateTestFile(source, originalUrl) {
  const isTs = originalUrl.endsWith(".ts") || originalUrl.endsWith(".tsx");

  // Strategy 1: Try esbuild for TypeScript (produces clean CJS)
  if (isTs) {
    const esbuildResult = tryEsbuild(source);
    if (esbuildResult !== null) {
      const cjsSource = convertEsmToCjs(esbuildResult);
      const result = runWithMock(cjsSource, ".cjs");
      if (result) return result;
    }
  }

  // Strategy 2: Try Node's built-in type stripping for TypeScript
  // Save as .cts so Node uses CJS mode + type stripping
  if (isTs) {
    const cjsSource = convertEsmToCjs(source);
    const result = runWithMock(cjsSource, ".cts", [
      "--experimental-strip-types",
      "--experimental-transform-types",
    ]);
    if (result) return result;
  }

  // Strategy 3: Naive TS strip + ESM→CJS conversion
  if (isTs) {
    const stripped = naiveStripTypeScript(source);
    const cjsSource = convertEsmToCjs(stripped);
    const result = runWithMock(cjsSource, ".cjs");
    if (result) return result;
  }

  // Strategy 4: Plain JS (ESM→CJS only)
  if (!isTs) {
    const cjsSource = convertEsmToCjs(source);
    const result = runWithMock(cjsSource, ".cjs");
    if (result) return result;
  }

  console.error("Warning: All evaluation strategies failed.");
  return null;
}

/**
 * Try to transpile TypeScript using esbuild.
 * Returns the JS source or null if esbuild is not available.
 */
function tryEsbuild(source) {
  try {
    execSync("which esbuild", { encoding: "utf8", stdio: ["pipe", "pipe", "pipe"] });
  } catch {
    return null;
  }

  const tmpTs = path.join(os.tmpdir(), `oxc-compare-test-${Date.now()}.ts`);
  try {
    fs.writeFileSync(tmpTs, source);
    const result = execSync(`esbuild --bundle=false --loader=ts --format=cjs "${tmpTs}"`, {
      encoding: "utf8",
      maxBuffer: 10 * 1024 * 1024,
      stdio: ["pipe", "pipe", "pipe"],
    });
    return result;
  } catch {
    return null;
  } finally {
    try {
      fs.unlinkSync(tmpTs);
    } catch {
      /* ignore */
    }
  }
}

/**
 * Write source to a temp file and run it with the mock RuleTester preloaded.
 * Returns the captured test cases or null on failure.
 */
function runWithMock(source, ext, extraNodeArgs = []) {
  const timestamp = Date.now();
  const tmpFile = path.join(os.tmpdir(), `oxc-compare-test-${timestamp}${ext}`);
  const runnerExt = ext === ".cts" ? ".cts" : ".cjs";
  const tmpRunner = path.join(os.tmpdir(), `oxc-compare-runner-${timestamp}${runnerExt}`);

  fs.writeFileSync(tmpFile, source);
  fs.writeFileSync(
    tmpRunner,
    `require(${JSON.stringify(tmpFile)});\n` +
      `if (global.__capturedTests) {\n` +
      `  process.stdout.write(JSON.stringify(global.__capturedTests));\n` +
      `} else {\n` +
      `  process.stdout.write(JSON.stringify({valid: [], invalid: []}));\n` +
      `}\n`,
  );

  try {
    const mockPath = path.join(__dirname, "mock-rule-tester.cjs");
    const nodeArgs = [...extraNodeArgs, `--require`, mockPath].join(" ");
    const result = execSync(`node ${nodeArgs} ${JSON.stringify(tmpRunner)}`, {
      encoding: "utf8",
      maxBuffer: 10 * 1024 * 1024,
      stdio: ["pipe", "pipe", "pipe"],
    });
    return JSON.parse(result);
  } catch (err) {
    if (process.env.DEBUG) {
      console.error(`Strategy failed (${ext}):`, err.stderr?.slice(0, 500) || err.message);
    }
    return null;
  } finally {
    try {
      fs.unlinkSync(tmpFile);
    } catch {
      /* ignore */
    }
    try {
      fs.unlinkSync(tmpRunner);
    } catch {
      /* ignore */
    }
  }
}

/**
 * Naive ESM → CJS conversion for test files.
 * Handles common patterns found in eslint/typescript-eslint test files.
 */
function convertEsmToCjs(source) {
  let result = source;

  // import { X, Y } from 'module' → const { X, Y } = require('module')
  result = result.replace(
    /import\s+\{([^}]+)\}\s+from\s+['"]([^'"]+)['"]\s*;?/g,
    (_, imports, mod) => `const {${imports}} = require('${mod}');`,
  );

  // import X from 'module' → const X = require('module')
  result = result.replace(
    /import\s+(\w+)\s+from\s+['"]([^'"]+)['"]\s*;?/g,
    (_, name, mod) => `const ${name} = require('${mod}');`,
  );

  // import * as X from 'module' → const X = require('module')
  result = result.replace(
    /import\s+\*\s+as\s+(\w+)\s+from\s+['"]([^'"]+)['"]\s*;?/g,
    (_, name, mod) => `const ${name} = require('${mod}');`,
  );

  // import 'module' → require('module')
  result = result.replace(/import\s+['"]([^'"]+)['"]\s*;?/g, (_, mod) => `require('${mod}');`);

  // export default → module.exports =
  result = result.replace(/export\s+default\s+/g, "module.exports = ");

  // export { X } — remove
  result = result.replace(/export\s+\{[^}]*\}\s*;?/g, "");

  // export const/let/var → const/let/var
  result = result.replace(/export\s+(const|let|var)\s+/g, "$1 ");

  return result;
}

/**
 * Very naive regex-based TypeScript stripping. Only handles simple patterns.
 */
function naiveStripTypeScript(source) {
  let result = source;

  // Remove type imports: import type { ... } from '...'
  result = result.replace(/import\s+type\s+\{[^}]*\}\s+from\s+['"][^'"]+['"]\s*;?/g, "");

  // Remove type-only imports within braces: import { type X, Y } → import { Y }
  result = result.replace(/\btype\s+(\w+)\s*,?\s*/g, "");

  // Remove parameter/variable type annotations: `: Type`
  // Match `: <identifier>` patterns but not inside strings
  result = result.replace(
    /:\s*(?:string|number|boolean|any|void|null|undefined|never|object|unknown|bigint|symbol)\b(?:\[\])*/g,
    "",
  );

  // Remove more complex type annotations: `: Identifier` or `: Identifier[]`
  // Only after parameter names (word char or `)` or `]`)
  result = result.replace(/(?<=[\w)\]])\s*:\s*[A-Z]\w*(?:<[^>]*>)?(?:\[\])*/g, "");

  // Remove `as Type` casts (but not `import * as X`)
  result = result.replace(/(?<=[\w)\]>])\s+as\s+[A-Z]\w*(?:<[^>]*>)?(?:\[\])*/g, "");

  // Remove generic type parameters on function calls/declarations
  result = result.replace(/<[A-Z]\w*(?:\s*,\s*[A-Z]\w*)*>/g, "");

  // Remove `satisfies Type`
  result = result.replace(/\s+satisfies\s+\w+/g, "");

  // Remove `<const>` assertions
  result = result.replace(/<const>/g, "");

  // Remove interface/type declarations (entire blocks)
  result = result.replace(/(?:export\s+)?interface\s+\w+(?:<[^>]*>)?\s*\{[^}]*\}/g, "");
  result = result.replace(/(?:export\s+)?type\s+\w+(?:<[^>]*>)?\s*=[^;]+;/g, "");

  return result;
}

// ── Step 2: Extract test code strings from the Rust rule file ───────────

function findRustRuleFile(ruleName, plugin) {
  const snake = toSnakeCase(toKebabCase(ruleName));
  const rustFile = path.join(
    PROJECT_ROOT,
    "crates/oxc_linter/src/rules",
    plugin.rustDir,
    `${snake}.rs`,
  );

  if (!fs.existsSync(rustFile)) {
    return null;
  }
  return rustFile;
}

/**
 * Extract test cases (code + options) from `let pass = vec![...]` and
 * `let fail = vec![...]` blocks in a Rust test file.
 */
function extractRustTestCases(filePath) {
  const content = fs.readFileSync(filePath, "utf8");

  const passCases = extractVecBlock(content, "pass");
  const failCases = extractVecBlock(content, "fail");

  return {
    valid: passCases,
    invalid: failCases,
  };
}

/**
 * Find ALL `let <name> = vec![...]` blocks and extract all string literals from them.
 * This handles files with multiple test functions.
 */
function extractVecBlock(content, name) {
  // Match `let pass = vec![` or `let fail = vec![` - use 'g' flag for global search
  const pattern = new RegExp(`let\\s+${name}\\s*(?::\\s*Vec<[^>]*>)?\\s*=\\s*vec!\\[`, "g");
  const allCases = [];

  let match;
  while ((match = pattern.exec(content)) !== null) {
    // Find the matching closing `];` by counting brackets
    let depth = 1;
    let i = match.index + match[0].length;
    const start = i;

    while (i < content.length && depth > 0) {
      const ch = content[i];
      if (ch === "[") depth++;
      else if (ch === "]") depth--;
      else if (ch === '"') {
        // Skip string literal
        i++;
        while (i < content.length && content[i] !== '"') {
          if (content[i] === "\\") i++; // skip escaped char
          i++;
        }
      } else if (ch === "r" && content[i + 1] === "#") {
        // Raw string: count #'s
        const hashStart = i + 1;
        let hashes = 0;
        while (content[hashStart + hashes] === "#") hashes++;
        if (content[hashStart + hashes] === '"') {
          // Skip to closing "###
          const closer = '"' + "#".repeat(hashes);
          const closeIdx = content.indexOf(closer, hashStart + hashes + 1);
          if (closeIdx !== -1) {
            i = closeIdx + closer.length - 1;
          }
        }
      } else if (ch === "r" && content[i + 1] === '"') {
        // r"..." raw string with no hashes
        i += 2;
        const closeIdx = content.indexOf('"', i);
        if (closeIdx !== -1) {
          i = closeIdx;
        }
      } else if (ch === "/" && content[i + 1] === "/") {
        // Skip line comment
        while (i < content.length && content[i] !== "\n") i++;
      }
      i++;
    }

    const block = content.slice(start, i - 1); // exclude the closing ]
    const cases = extractStringLiterals(block);
    allCases.push(...cases);
  }

  return allCases;
}

/**
 * Extract test cases from a vec![...] block.
 *
 * Test cases come in two forms:
 *   1. Tuple: ("code", None) or ("code", Some(serde_json::json!([...])))
 *   2. Bare string: "code"
 *
 * Returns an array of { code, options } objects.
 */
function extractStringLiterals(block) {
  const cases = [];
  let i = 0;

  while (i < block.length) {
    skipWhitespace();
    if (i >= block.length) break;

    if (block[i] === "(") {
      // Tuple: extract first string (code), then try to extract options
      const tupleStart = i;
      i++; // skip (
      skipWhitespace();
      const codeStr = tryExtractString();
      if (codeStr !== null) {
        // Now extract options from the rest of the tuple
        const options = extractOptionsFromTuple(block, tupleStart);
        cases.push({ code: codeStr, options });
      }
      // Ensure we're past the closing )
      skipToClosingParen();
    } else if (isStringStart(block, i)) {
      // Bare string at top level
      const codeStr = tryExtractString();
      if (codeStr !== null) {
        cases.push({ code: codeStr, options: null });
      }
    } else {
      i++;
    }
  }

  return cases;

  function skipToClosingParen() {
    let depth = 1;
    while (i < block.length && depth > 0) {
      const ch = block[i];
      if (ch === "(") depth++;
      else if (ch === ")") depth--;
      else if (ch === '"') {
        // Skip string literal
        i++;
        while (i < block.length && block[i] !== '"') {
          if (block[i] === "\\") i++;
          i++;
        }
      } else if (ch === "r" && (block[i + 1] === "#" || block[i + 1] === '"')) {
        skipRawString();
        continue; // skipRawString already advances i
      } else if (ch === "/" && block[i + 1] === "/") {
        while (i < block.length && block[i] !== "\n") i++;
      }
      i++;
    }
  }

  function skipRawString() {
    if (block[i] === "r" && block[i + 1] === "#") {
      i++; // skip 'r'
      let hashes = 0;
      while (block[i] === "#") {
        hashes++;
        i++;
      }
      if (block[i] === '"') {
        i++; // skip opening "
        const closer = '"' + "#".repeat(hashes);
        const closeIdx = block.indexOf(closer, i);
        if (closeIdx !== -1) {
          i = closeIdx + closer.length;
        }
      }
    } else if (block[i] === "r" && block[i + 1] === '"') {
      i += 2;
      const closeIdx = block.indexOf('"', i);
      if (closeIdx !== -1) {
        i = closeIdx + 1;
      }
    }
  }

  function skipWhitespace() {
    while (i < block.length && /[\s,]/.test(block[i])) i++;
    // Also skip line comments
    if (i < block.length && block[i] === "/" && block[i + 1] === "/") {
      while (i < block.length && block[i] !== "\n") i++;
      skipWhitespace();
    }
  }

  function isStringStart(s, pos) {
    return s[pos] === '"' || (s[pos] === "r" && (s[pos + 1] === '"' || s[pos + 1] === "#"));
  }

  function tryExtractString() {
    skipWhitespace();
    if (i >= block.length) return null;

    // Raw string with hashes: r#"..."# or r##"..."##
    if (block[i] === "r" && block[i + 1] === "#") {
      i++; // skip 'r'
      let hashes = 0;
      while (block[i] === "#") {
        hashes++;
        i++;
      }
      if (block[i] !== '"') return null;
      i++; // skip opening "
      const closer = '"' + "#".repeat(hashes);
      const closeIdx = block.indexOf(closer, i);
      if (closeIdx === -1) return null;
      const str = block.slice(i, closeIdx);
      i = closeIdx + closer.length;
      return str;
    }

    // Raw string without hashes: r"..."
    if (block[i] === "r" && block[i + 1] === '"') {
      i += 2; // skip r"
      const closeIdx = block.indexOf('"', i);
      if (closeIdx === -1) return null;
      const str = block.slice(i, closeIdx);
      i = closeIdx + 1;
      return str;
    }

    // Regular string: "..."
    if (block[i] === '"') {
      i++; // skip opening "
      let str = "";
      while (i < block.length && block[i] !== '"') {
        if (block[i] === "\\") {
          i++;
          switch (block[i]) {
            case "n":
              str += "\n";
              break;
            case "t":
              str += "\t";
              break;
            case "r":
              str += "\r";
              break;
            case "\\":
              str += "\\";
              break;
            case '"':
              str += '"';
              break;
            default:
              str += block[i];
          }
        } else {
          str += block[i];
        }
        i++;
      }
      i++; // skip closing "
      return str;
    }

    return null;
  }
}

/**
 * Extract the options/config from a Rust tuple, given the start position of '('.
 *
 * Looks for `None` (→ null) or `Some(serde_json::json!(...))` (→ parsed JSON string).
 * Returns null if no config or if it's `None`.
 */
function extractOptionsFromTuple(block, tupleStart) {
  // Find the full tuple content between ( and matching )
  let depth = 1;
  let j = tupleStart + 1;
  while (j < block.length && depth > 0) {
    const ch = block[j];
    if (ch === "(") depth++;
    else if (ch === ")") depth--;
    else if (ch === '"') {
      j++;
      while (j < block.length && block[j] !== '"') {
        if (block[j] === "\\") j++;
        j++;
      }
    } else if (ch === "r" && (block[j + 1] === "#" || block[j + 1] === '"')) {
      // Skip raw string
      if (block[j + 1] === "#") {
        j++;
        let hashes = 0;
        while (block[j] === "#") {
          hashes++;
          j++;
        }
        if (block[j] === '"') {
          j++;
          const closer = '"' + "#".repeat(hashes);
          const closeIdx = block.indexOf(closer, j);
          if (closeIdx !== -1) j = closeIdx + closer.length - 1;
        }
      } else {
        j += 2;
        const closeIdx = block.indexOf('"', j);
        if (closeIdx !== -1) j = closeIdx;
      }
    }
    j++;
  }

  const tupleContent = block.slice(tupleStart + 1, j - 1);

  // Look for `None` or `Some(serde_json::json!(...))` after the first comma
  const commaIdx = findTopLevelComma(tupleContent);
  if (commaIdx === -1) return null;

  const afterComma = tupleContent.slice(commaIdx + 1).trim();

  if (afterComma.startsWith("None")) return null;

  // Extract the JSON content from Some(serde_json::json!(...)) or Some(json!(...))
  const jsonMatch = afterComma.match(/Some\s*\(\s*(?:serde_json::)?json!\s*\(/);
  if (!jsonMatch) return null;

  // Find the content inside json!(...)
  const jsonStart = afterComma.indexOf("json!(") + 6;
  // Find the matching closing paren for json!(...)
  let parenDepth = 1;
  let k = jsonStart;
  while (k < afterComma.length && parenDepth > 0) {
    const ch = afterComma[k];
    if (ch === "(") parenDepth++;
    else if (ch === ")") parenDepth--;
    else if (ch === '"') {
      k++;
      while (k < afterComma.length && afterComma[k] !== '"') {
        if (afterComma[k] === "\\") k++;
        k++;
      }
    } else if (ch === "[") parenDepth++;
    else if (ch === "]") parenDepth--;
    k++;
  }

  // The JSON-like content (Rust serde_json::json! macro syntax)
  let jsonContent = afterComma.slice(jsonStart, k - 1).trim();

  // Remove trailing commas (valid in Rust json! macro but not in JSON)
  // Match commas before closing brackets/braces/parens
  jsonContent = jsonContent.replace(/,(\s*[}\]\)])/g, "$1");

  // Try to parse it as JSON (serde_json::json! uses JSON-like syntax)
  try {
    return JSON.parse(jsonContent);
  } catch {
    // Return the raw string if it can't be parsed
    return jsonContent;
  }
}

/**
 * Find the first comma at the top level of nesting (not inside parens/brackets/strings).
 * This separates the code string from the config in a Rust tuple.
 */
function findTopLevelComma(s) {
  let depth = 0;
  for (let i = 0; i < s.length; i++) {
    const ch = s[i];
    if (ch === "," && depth === 0) return i;
    if (ch === "(" || ch === "[" || ch === "{") depth++;
    else if (ch === ")" || ch === "]" || ch === "}") depth--;
    else if (ch === '"') {
      i++;
      while (i < s.length && s[i] !== '"') {
        if (s[i] === "\\") i++;
        i++;
      }
    } else if (ch === "r" && (s[i + 1] === "#" || s[i + 1] === '"')) {
      if (s[i + 1] === "#") {
        i++;
        let hashes = 0;
        while (s[i] === "#") {
          hashes++;
          i++;
        }
        if (s[i] === '"') {
          i++;
          const closer = '"' + "#".repeat(hashes);
          const closeIdx = s.indexOf(closer, i);
          if (closeIdx !== -1) i = closeIdx + closer.length - 1;
        }
      } else {
        i += 2;
        const closeIdx = s.indexOf('"', i);
        if (closeIdx !== -1) i = closeIdx;
      }
    }
  }
  return -1;
}

// ── Step 3: Compare and report ──────────────────────────────────────────

/**
 * Create a comparison key from a test case.
 * Uses normalized code + sorted JSON of options for a composite key.
 */
function caseKey(t) {
  const code = normalize(t.code);
  const options = normalizeOptions(t);
  return options ? `${code}\0${options}` : code;
}

/**
 * Normalize options for comparison. Upstream uses `options` property,
 * Rust uses the second tuple element (parsed from serde_json::json!).
 * Returns a canonical JSON string or null.
 */
function normalizeOptions(t) {
  const opts = t.options ?? null;
  if (opts === null || opts === undefined) return null;
  try {
    return JSON.stringify(opts);
  } catch {
    return String(opts);
  }
}

function compareSets(upstreamCases, rustCases) {
  const upstreamKeys = new Set(upstreamCases.map((t) => caseKey(t)));
  const rustKeys = new Set(rustCases.map((t) => caseKey(t)));

  const missing = [];
  for (const t of upstreamCases) {
    const key = caseKey(t);
    if (key && !rustKeys.has(key)) {
      missing.push(t);
    }
  }

  const extra = [];
  for (const t of rustCases) {
    const key = caseKey(t);
    if (key && !upstreamKeys.has(key)) {
      extra.push(t);
    }
  }

  return { missing, extra };
}

function truncate(str, maxLen = 120) {
  if (str.length <= maxLen) return str;
  return str.slice(0, maxLen) + "...";
}

/**
 * Format a test case for JSON output. Includes options when present.
 */
function formatCaseForJson(t) {
  const opts = t.options ?? null;
  if (opts !== null && opts !== undefined) {
    return { code: t.code, options: opts };
  }
  return t.code;
}

/**
 * Format a test case for human-readable output. Appends options as a short suffix.
 */
function formatCaseForDisplay(t) {
  const code = truncate(normalize(t.code));
  const opts = t.options ?? null;
  if (opts !== null && opts !== undefined) {
    const optsStr = JSON.stringify(opts);
    if (optsStr.length <= 60) {
      return `${code}  [options: ${optsStr}]`;
    }
    return `${code}  [options: ${truncate(optsStr, 60)}]`;
  }
  return code;
}

function printReport(upstreamResult, rustResult) {
  const validComparison = compareSets(upstreamResult.valid, rustResult.valid);
  const invalidComparison = compareSets(upstreamResult.invalid, rustResult.invalid);

  if (jsonOutput) {
    console.log(
      JSON.stringify(
        {
          rule: ruleName,
          plugin: pluginName,
          upstream: {
            valid: upstreamResult.valid.length,
            invalid: upstreamResult.invalid.length,
          },
          oxlint: {
            valid: rustResult.valid.length,
            invalid: rustResult.invalid.length,
          },
          missingValid: validComparison.missing.map(formatCaseForJson),
          missingInvalid: invalidComparison.missing.map(formatCaseForJson),
          extraValid: validComparison.extra.map(formatCaseForJson),
          extraInvalid: invalidComparison.extra.map(formatCaseForJson),
        },
        null,
        2,
      ),
    );
    return;
  }

  console.log(`\nComparing test cases for ${pluginName}/${ruleName}...\n`);

  console.log(
    `Upstream (Node.js evaluated): ${upstreamResult.valid.length} valid, ${upstreamResult.invalid.length} invalid`,
  );
  console.log(
    `Oxlint (Rust file):           ${rustResult.valid.length} valid, ${rustResult.invalid.length} invalid`,
  );

  if (validComparison.missing.length === 0 && invalidComparison.missing.length === 0) {
    console.log("\nAll upstream test cases are present in the oxlint rule file.");
  }

  if (validComparison.missing.length > 0) {
    console.log(`\nMissing valid cases (${validComparison.missing.length}):`);
    for (const [i, t] of validComparison.missing.entries()) {
      console.log(`  ${i + 1}. ${formatCaseForDisplay(t)}`);
    }
  }

  if (invalidComparison.missing.length > 0) {
    console.log(`\nMissing invalid cases (${invalidComparison.missing.length}):`);
    for (const [i, t] of invalidComparison.missing.entries()) {
      console.log(`  ${i + 1}. ${formatCaseForDisplay(t)}`);
    }
  }

  if (validComparison.extra.length > 0) {
    console.log(`\nExtra valid cases in oxlint not in upstream (${validComparison.extra.length}):`);
    for (const [i, t] of validComparison.extra.entries()) {
      console.log(`  ${i + 1}. ${formatCaseForDisplay(t)}`);
    }
  }

  if (invalidComparison.extra.length > 0) {
    console.log(
      `\nExtra invalid cases in oxlint not in upstream (${invalidComparison.extra.length}):`,
    );
    for (const [i, t] of invalidComparison.extra.entries()) {
      console.log(`  ${i + 1}. ${formatCaseForDisplay(t)}`);
    }
  }

  console.log();
}

// ── Main ────────────────────────────────────────────────────────────────

async function main() {
  // Download and evaluate the upstream test file
  const { body, url } = await downloadTestFile(ruleName, plugin);
  const upstreamResult = evaluateTestFile(body, url);

  if (!upstreamResult) {
    console.error("Failed to evaluate upstream test file.");
    process.exit(1);
  }

  if (upstreamResult.valid.length === 0 && upstreamResult.invalid.length === 0) {
    console.error(
      "Warning: No test cases captured from upstream file. " +
        "The test file may use an unsupported pattern.",
    );
  }

  // Find and parse the Rust rule file
  const rustFile = findRustRuleFile(ruleName, plugin);
  let rustResult;
  if (rustFile) {
    console.error(`Reading Rust file: ${rustFile}`);
    rustResult = extractRustTestCases(rustFile);
  } else {
    console.error(
      `No Rust rule file found for ${pluginName}/${toSnakeCase(toKebabCase(ruleName))}.rs`,
    );
    rustResult = { valid: [], invalid: [] };
  }

  printReport(upstreamResult, rustResult);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
