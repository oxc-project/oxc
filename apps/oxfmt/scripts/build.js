// oxlint-disable no-console

import { execSync } from "node:child_process";
import { copyFileSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { quicktype, InputData, JSONSchemaInput, FetchingJSONSchemaStore } from "quicktype-core";

const oxfmtDirPath = join(import.meta.dirname, ".."),
  distDirPath = join(oxfmtDirPath, "dist"),
  jsonSchemaPath = join(oxfmtDirPath, "..", "..", "npm/oxfmt/configuration_schema.json");

// Modify `bindings.js` to use correct package names
console.log("Modifying bindings.js...");
const bindingsPath = join(oxfmtDirPath, "src-js/bindings.js");
let bindingsJs = readFileSync(bindingsPath, "utf8");
bindingsJs = bindingsJs.replace(/require\('@oxfmt\/binding-(.+?)'\)/g, (_, name) => {
  name = name.replace(/-msvc(\/|$)/g, "$1");
  return `require('@oxfmt/${name}')`;
});
writeFileSync(bindingsPath, bindingsJs);

// Build with tsdown
console.log("Building with tsdown...");
execSync("pnpm tsdown", { stdio: "inherit", cwd: oxfmtDirPath });

// Copy native `.node` files from `src-js`
console.log("Copying `.node` files...");

for (const filename of readdirSync(join(oxfmtDirPath, "src-js"))) {
  if (!filename.endsWith(".node")) continue;
  copyFile(join(oxfmtDirPath, "src-js", filename), join(distDirPath, filename));
}
try {
  const { lines } = await quicktypeJSONSchema("OxfmtConfig", readFileSync(jsonSchemaPath, "utf8"));
  writeFileSync(join(distDirPath, "config.d.ts"), lines.join("\n"));
  console.log("Translated oxfmt config JSON schema into TypeScript");
} catch (error) {
  console.error("Translating oxfmt config JSON schema into TypeScript failed:", error);
  process.exit(1);
}

console.log("Build complete!");

/**
 * Copy a file, creating parent directories if needed.
 * @param {string} srcPath - Source file path, absolute
 * @param {string} destPath - Destination file path, absolute
 * @returns {void}
 */
function copyFile(srcPath, destPath) {
  mkdirSync(join(destPath, ".."), { recursive: true });
  copyFileSync(srcPath, destPath);
  console.log(`- Copied ${srcPath.split("/").pop()}`);
}

/**
 * Quicktype a JSON schema into a target language.
 * @param {string} targetLanguage - The target language to quicktype to.
 * @param {string} typeName - The name of the type to quicktype.
 * @param {string} jsonSchemaString - The JSON schema string to quicktype.
 * @returns {Promise<import('quicktype-core').SerializedRenderResult>} The quicktyped code.
 */
async function quicktypeJSONSchema(typeName, jsonSchemaString) {
  const schemaInput = new JSONSchemaInput(new FetchingJSONSchemaStore());

  // We could add multiple schemas for multiple types,
  // but here we're just making one type from JSON schema.
  await schemaInput.addSource({ name: typeName, schema: jsonSchemaString });

  const inputData = new InputData();
  inputData.addInput(schemaInput);

  return await quicktype({
    inputData,
    lang: "typescript",
    rendererOptions: {
      "prefer-unions": true,
    },
  });
}
