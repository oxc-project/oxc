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

const args = process.argv.slice(2);
if (args.length !== 1) {
  throw new Error("Usage: pnpm --filter react_compiler compare <directory>");
}

const invocationDirectory = process.env.INIT_CWD ?? process.cwd();
const directory = resolve(invocationDirectory, args[0]);
if (!(await stat(directory)).isDirectory()) {
  throw new Error(`Not a directory: ${directory}`);
}

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
  const plugins = [[reactCompiler, {}]];
  if (extname(filename).toLowerCase() === ".tsx") {
    plugins.push([
      transformTypescript,
      {
        allExtensions: true,
        isTSX: true,
      },
    ]);
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
  const result = oxcReactCompilerSync(filename, sourceText);
  assertNoErrors(result, "oxc-transform-react failed");
  assert(result.code, "oxc-transform-react did not produce code");
  return result.code;
}

function assertNoErrors(result, message) {
  const errors = result.errors.filter(({ severity }) => severity === "Error");
  assert.equal(errors.length, 0, `${message}\n${errors.map(({ message }) => message).join("\n")}`);
}
