import { join } from "node:path";
import { stat, writeFile } from "node:fs/promises";

export async function hasOxfmtrcFile(cwd: string) {
  return (await isFile(join(cwd, ".oxfmtrc.json"))) || (await isFile(join(cwd, ".oxfmtrc.jsonc")));
}

const SCHEMA_RELATIVE_PATH = "./node_modules/oxfmt/configuration_schema.json";

async function hasSchemaFile(cwd: string) {
  const schemaAbsPath = join(cwd, "node_modules/oxfmt/configuration_schema.json");
  return (await isFile(schemaAbsPath)) ? SCHEMA_RELATIVE_PATH : null;
}

export async function createBlankOxfmtrcFile(cwd: string) {
  const schemaPath = await hasSchemaFile(cwd);
  const config: Record<string, unknown> = {
    // Add `$schema` field at the top if schema file exists in `node_modules`
    $schema: schemaPath,
    // `ignorePatterns` is included to make visible and preferred over `.prettierignore`
    ignorePatterns: [],
  };

  // NOTE: To keep `$schema` field at the top, we delete it here instead of defining conditionally above
  // This happens if run with e.g. `npx`
  if (config.$schema === null) {
    delete config.$schema;
  }

  return config;
}

export async function saveOxfmtrcFile(cwd: string, jsonStr: string) {
  // Add trailing newline if missing to avoid `oxfmt` produce the diff
  await writeFile(join(cwd, ".oxfmtrc.json"), jsonStr + "\n", "utf8");
}

export function exitWithError(message: string) {
  // oxlint-disable-next-line no-console
  console.error(message);
  process.exitCode = 1;
}

// ---

async function isFile(path: string) {
  try {
    const stats = await stat(path);
    return stats.isFile();
  } catch {
    return false;
  }
}
