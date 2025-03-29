/**
 * This file is used to copy over all legacy decorators tests from submodule TypeScript repo to OXC,
 * and re-generate output file which makes it consistent with our testing infrastructure.
 *
 * Q: Why would we need to re-generate output rather than copying over output from TypeScript as well?
 * A: Mainly our implementation is based on Babel, so the output is largely different, and we should only
 *    pay attention to the transform result that is handled by legacy decorator.
 */

import { readFileSync, rmSync, writeFileSync } from "fs";
import { transpileModule } from "typescript";
import { readdirSync, mkdirSync, statSync, copyFileSync } from "fs";
import { dirname, extname, join } from "path";

const __dirname = new URL(".", import.meta.url).pathname;

// https://github.com/microsoft/TypeScript/blob/8da951cbb629b648753454872df4e1754982aef1/tests/cases/conformance/decorators/class
const typescriptTestFolderPath = join(
  __dirname,
  "../../../../../coverage/typescript/tests/cases/conformance/decorators/class",
);
const oxcTestFolderPath = join(__dirname, "typescript");

function copyTestsFromTypeScript() {
  const items = readdirSync(typescriptTestFolderPath, { recursive: true });
  for (const item of items) {
    const originalFile = join(typescriptTestFolderPath, item);
    const stat = statSync(originalFile);
    if (stat.isFile()) {
      // Skip multi-files tests
      const hasMultiFiles =
        Array.from(
          readFileSync(originalFile)
            .toString()
            .matchAll(/\@filename\:/gi),
        ).length > 1;

      if (hasMultiFiles) {
        continue;
      }

      const targetFile = join(
        oxcTestFolderPath,
        item.replace(extname(item), "/input.ts"),
      );

      mkdirSync(dirname(targetFile), { recursive: true });
      // `abc.ts` -> `abc/input.ts`
      copyFileSync(originalFile, targetFile);
    } else if (stat.isDirectory()) {
      // Generate options.json to indicate these tests should transform by legacy decorator plugin
      const targetPath = join(oxcTestFolderPath, item);
      mkdirSync(targetPath, { recursive: true });
      writeBabelOptions(targetPath);
    }
  }
}

function writeBabelOptions(folder, emitDecoratorMetadata = false) {
  const optionsFile = join(folder, "options.json");
  const legacyDecoratorPlugin = [
    "transform-legacy-decorator",
    emitDecoratorMetadata ? { emitDecoratorMetadata: true } : null,
  ].filter(Boolean);
  const content = JSON.stringify(
    {
      plugins: [legacyDecoratorPlugin.length === 1 ? legacyDecoratorPlugin[0] : legacyDecoratorPlugin],
    },
    null,
    2,
  );
  writeFileSync(optionsFile, content);
}

async function generateOutputFiles() {
  const files = readdirSync(oxcTestFolderPath, { recursive: true });
  for (const file of files) {
    if (!file.endsWith("input.ts")) {
      continue;
    }
    const inputFile = join(oxcTestFolderPath, file);
    const source = readFileSync(inputFile, "utf8").toString();

    const emitDecoratorMetadata = /\@emitDecoratorMetadata/gi.test(source);

    if (emitDecoratorMetadata) {
      writeBabelOptions(dirname(inputFile), true);
    }

    /// Generate the output file path by using `typescript` library, and we need to set target to esnext
    const output = transpileModule(source, {
      compilerOptions: {
        target: "esnext",
        experimentalDecorators: true,
        emitDecoratorMetadata,
        noEmitHelpers: true,
      },
    });

    // Rename helper functions to our own
    const outputText = output.outputText
      .replaceAll("__decorate(", "babelHelpers.decorate(")
      .replaceAll("__param(", "babelHelpers.decorateParam(")
      .replaceAll("__metadata(", "babelHelpers.decorateMetadata(");

    const outputFile = join(dirname(inputFile), "output.js");
    writeFileSync(outputFile, outputText);
  }
}

function run() {
  try {
    rmSync(oxcTestFolderPath, { recursive: true });
  } catch { }
  copyTestsFromTypeScript();
  generateOutputFiles();
}

run();
