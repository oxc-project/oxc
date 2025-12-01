// oxlint-disable no-console

import { execSync } from "node:child_process";
import { copyFileSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { quicktype, InputData, JSONSchemaInput, FetchingJSONSchemaStore } from "quicktype-core";

const oxlintDirPath = join(import.meta.dirname, ".."),
  srcDirPath = join(oxlintDirPath, "src-js"),
  distDirPath = join(oxlintDirPath, "dist"),
  jsonSchemaPath = join(oxlintDirPath, "..", "..", "npm/oxlint/configuration_schema.json");

// Modify `bindings.js` to use correct package names
console.log("Modifying bindings.js...");
const bindingsPath = join(oxlintDirPath, "src-js/bindings.js");
let bindingsJs = readFileSync(bindingsPath, "utf8");
bindingsJs = bindingsJs.replace(/require\('@oxlint\/binding-(.+?)'\)/g, (_, name) => {
  name = name.replace(/-msvc(\/|$)/g, "$1");
  return `require('@oxlint/${name}')`;
});
writeFileSync(bindingsPath, bindingsJs);

// Build with tsdown
console.log("Building with tsdown...");
execSync("pnpm tsdown", { stdio: "inherit", cwd: oxlintDirPath });

// Delete `cli.d.ts`
console.log("Deleting cli.d.ts...");
rmSync(join(distDirPath, "cli.d.ts"));

// Copy native `.node` files from `src-js`
console.log("Copying `.node` files...");
for (const filename of readdirSync(srcDirPath)) {
  if (!filename.endsWith(".node")) continue;
  const srcPath = join(srcDirPath, filename);
  copyFileSync(srcPath, join(distDirPath, filename));
}

try {
  const { lines } = await quicktypeJSONSchema(
    "OxlintConfig",
    readFileSync(jsonSchemaPath, "utf8"),
  );
  writeFileSync(join(distDirPath, "config.d.ts"), lines.join("\n"));
  console.log("Translated oxlint config JSON schema into TypeScript");
} catch (error) {
  console.error("Translating oxlint config JSON schema into TypeScript failed:", error);
  process.exit(1);
}

console.log("Build complete!");

async function quicktypeJSONSchema(typeName: string, jsonSchemaString: string) {
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
    }
  });
}
