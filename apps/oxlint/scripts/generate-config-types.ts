// oxlint-disable no-console

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { compile } from "json-schema-to-typescript";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const oxlintDir = resolve(scriptDir, "..");
const repoRoot = resolve(oxlintDir, "..", "..");

const schemaPath = resolve(repoRoot, "npm/oxlint/configuration_schema.json");
const outputPath = resolve(oxlintDir, "src-js/package/config.generated.ts");

if (!existsSync(schemaPath)) {
  throw new Error(`Missing schema at ${schemaPath}. Run just linter-schema-json first.`);
}

const schema = JSON.parse(readFileSync(schemaPath, "utf8"));

// Sanitize and format JSON code blocks so they are indented properly.
function sanitizeJsonCodeBlocks(text: string): string {
  const marker = "```json";
  let result = text;
  let searchFrom = 0;

  while (true) {
    const start = result.indexOf(marker, searchFrom);
    if (start === -1) break;

    const contentStart = start + marker.length;
    const end = result.indexOf("```", contentStart);
    if (end === -1) break;

    const jsonStr = result.slice(contentStart, end).trim();
    let json: unknown;
    try {
      json = JSON.parse(jsonStr);
    } catch {
      searchFrom = end + 3;
      continue;
    }

    const formatted = "\n" + JSON.stringify(json, null, 2) + "\n";
    result = result.slice(0, contentStart) + formatted + result.slice(end);
    searchFrom = contentStart + formatted.length;
  }

  return result;
}

function sanitizeSchema(value: any): void {
  if (typeof value !== "object" || value === null) return;
  if (Array.isArray(value)) {
    value.forEach(sanitizeSchema);
    return;
  }
  for (const key of Object.keys(value)) {
    if (
      (key === "description" || key === "markdownDescription") &&
      typeof value[key] === "string"
    ) {
      value[key] = sanitizeJsonCodeBlocks(value[key]);
    } else {
      sanitizeSchema(value[key]);
    }
  }
}

sanitizeSchema(schema);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function getEnumPrimitiveType(definition: unknown): "string" | "number" | "boolean" | null {
  if (!isRecord(definition)) {
    return null;
  }

  const definitionType = definition.type;
  if (
    (definitionType === "string" || definitionType === "number" || definitionType === "boolean") &&
    Array.isArray(definition.enum)
  ) {
    return definitionType;
  }

  if (!Array.isArray(definition.oneOf) || definition.oneOf.length === 0) {
    return null;
  }

  let primitiveType: "string" | "number" | "boolean" | null = null;
  for (const variant of definition.oneOf) {
    if (!isRecord(variant) || !Array.isArray(variant.enum) || variant.enum.length !== 1) {
      return null;
    }

    const variantType = variant.type;
    if (variantType !== "string" && variantType !== "number" && variantType !== "boolean") {
      return null;
    }

    if (primitiveType === null) {
      primitiveType = variantType;
      continue;
    }

    if (primitiveType !== variantType) {
      return null;
    }
  }

  return primitiveType;
}

function getEnumDefinitionTypes(schema: any): Map<string, "string" | "number" | "boolean"> {
  const { definitions } = schema;
  const enumDefinitionTypes = new Map<string, "string" | "number" | "boolean">();

  if (!isRecord(definitions)) {
    return enumDefinitionTypes;
  }

  for (const [name, definition] of Object.entries(definitions)) {
    const primitiveType = getEnumPrimitiveType(definition);
    if (primitiveType !== null) {
      enumDefinitionTypes.set(name, primitiveType);
    }
  }

  return enumDefinitionTypes;
}

function getDefinitionRefName(value: unknown): string | null {
  if (!isRecord(value) || typeof value.$ref !== "string") {
    return null;
  }

  const prefix = "#/definitions/";
  return value.$ref.startsWith(prefix) ? value.$ref.slice(prefix.length) : null;
}

type PrimitiveType = "string" | "number" | "boolean";

function getDefinitionPrimitiveTypes(schema: any): Map<string, Set<PrimitiveType>> {
  const { definitions } = schema;
  const primitiveTypes = new Map<string, Set<PrimitiveType>>();

  if (!isRecord(definitions)) {
    return primitiveTypes;
  }

  const allNames = new Set(Object.keys(definitions));
  const visiting = new Set<string>();

  const inferPrimitiveTypes = (value: unknown): Set<PrimitiveType> => {
    const result = new Set<PrimitiveType>();

    if (!isRecord(value)) {
      return result;
    }

    const pushType = (candidate: unknown): void => {
      if (candidate === "string" || candidate === "number" || candidate === "boolean") {
        result.add(candidate);
      }
    };

    pushType(value.type);

    if (Array.isArray(value.type)) {
      value.type.forEach(pushType);
    }

    if (Array.isArray(value.enum) && value.enum.length > 0) {
      const enumValueType = typeof value.enum[0];
      if (
        (enumValueType === "string" || enumValueType === "number" || enumValueType === "boolean") &&
        value.enum.every((item) => typeof item === enumValueType)
      ) {
        result.add(enumValueType);
      }
    }

    for (const branchKey of ["oneOf", "anyOf", "allOf"] as const) {
      const branch = value[branchKey];
      if (!Array.isArray(branch)) continue;

      for (const item of branch) {
        inferPrimitiveTypes(item).forEach((primitiveType) => result.add(primitiveType));
      }
    }

    const refName = getDefinitionRefName(value);
    if (refName && allNames.has(refName)) {
      inferDefinitionByName(refName).forEach((primitiveType) => result.add(primitiveType));
    }

    return result;
  };

  const inferDefinitionByName = (name: string): Set<PrimitiveType> => {
    const cached = primitiveTypes.get(name);
    if (cached) {
      return cached;
    }

    if (visiting.has(name)) {
      return new Set<PrimitiveType>();
    }

    visiting.add(name);
    const inferred = inferPrimitiveTypes(definitions[name]);
    visiting.delete(name);

    primitiveTypes.set(name, inferred);
    return inferred;
  };

  for (const name of allNames) {
    inferDefinitionByName(name);
  }

  return primitiveTypes;
}

function collapseEnumPrimitiveIntersections(
  source: string,
  enumDefinitionTypes: Map<string, "string" | "number" | "boolean">,
  definitionPrimitiveTypes?: Map<string, Set<PrimitiveType>>,
): string {
  let result = source;

  for (const [name, primitiveType] of enumDefinitionTypes) {
    const pattern = new RegExp(`\\b${name}\\s*&\\s*${primitiveType}\\b`, "g");
    result = result.replace(pattern, name);
  }

  if (definitionPrimitiveTypes) {
    for (const [name, primitiveTypes] of definitionPrimitiveTypes) {
      for (const primitiveType of primitiveTypes) {
        const leftPattern = new RegExp(`\\b${name}\\s*&\\s*${primitiveType}\\b`, "g");
        const rightPattern = new RegExp(`\\b${primitiveType}\\s*&\\s*${name}\\b`, "g");
        result = result.replace(leftPattern, name);
        result = result.replace(rightPattern, name);
      }
    }
  }

  return result;
}

const dummyRuleMap = schema.definitions?.DummyRuleMap;
const dummyRuleMapAdditionalProperties = dummyRuleMap?.additionalProperties;
if (
  typeof dummyRuleMapAdditionalProperties !== "object" ||
  dummyRuleMapAdditionalProperties === null
) {
  throw new Error("Expected DummyRuleMap.additionalProperties in the oxlint config schema.");
}
// Named rule properties are optional, so the string index signature must
// accept `undefined` to keep the generated declaration internally valid.
dummyRuleMapAdditionalProperties.tsType = "DummyRule | undefined";

const bannerComment =
  "/*\n" +
  " * This file is generated from npm/oxlint/configuration_schema.json.\n" +
  " * Run `just linter-config-ts` to regenerate.\n" +
  " */";

const enumDefinitionTypes = getEnumDefinitionTypes(schema);
const definitionPrimitiveTypes = getDefinitionPrimitiveTypes(schema);

let ts = await compile(schema, "OxlintConfig", { bannerComment });
ts = collapseEnumPrimitiveIntersections(ts, enumDefinitionTypes, definitionPrimitiveTypes);

writeFileSync(outputPath, ts);
console.log(`Wrote ${outputPath}`);
