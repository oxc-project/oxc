import { getInferredExternalChildKeys, mergeExternalChildKeys } from "./external_ast_utils.ts";

import type { Program } from "../generated/types.d.ts";

type VisitorKeysRecord = Readonly<Record<string, readonly string[]>>;

interface ExternalSourceFlagState {
  isJsx: boolean | null;
  isTs: boolean | null;
}

export interface ExternalSourceFlags {
  isJsx: boolean;
  isTs: boolean;
}

function isObjectRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function getConfiguredExternalSourceFlagState(
  parserOptions: Record<string, unknown> | null,
): ExternalSourceFlagState {
  const flags: ExternalSourceFlagState = { isJsx: null, isTs: null };
  if (parserOptions === null) return flags;

  const ecmaFeatures = parserOptions.ecmaFeatures;
  const configuredJsx =
    isObjectRecord(ecmaFeatures) && typeof ecmaFeatures.jsx === "boolean"
      ? ecmaFeatures.jsx
      : null;

  switch (parserOptions.lang) {
    case "tsx":
      flags.isJsx = true;
      flags.isTs = true;
      break;
    case "jsx":
      flags.isJsx = true;
      flags.isTs = false;
      break;
    case "ts":
    case "dts":
      flags.isJsx = configuredJsx ?? false;
      flags.isTs = true;
      break;
    case "js":
      flags.isJsx = false;
      flags.isTs = false;
      break;
    default:
      flags.isJsx = configuredJsx;
      break;
  }

  return flags;
}

function scanExternalAstForSyntaxFlags(
  node: unknown,
  flags: ExternalSourceFlagState,
  seen: WeakSet<object>,
  visitorKeys: VisitorKeysRecord | null,
): void {
  if (!isObjectRecord(node) || (flags.isJsx !== null && flags.isTs !== null)) return;
  if (seen.has(node)) return;
  seen.add(node);

  const nodeType = typeof node.type === "string" ? node.type : null;
  if (flags.isJsx === null && nodeType?.startsWith("JSX")) {
    flags.isJsx = true;
  }
  if (flags.isTs === null && nodeType?.startsWith("TS")) {
    flags.isTs = true;
  }
  if (flags.isJsx !== null && flags.isTs !== null) return;

  const childKeys = mergeExternalChildKeys(
    getInferredExternalChildKeys(node),
    nodeType === null ? null : visitorKeys?.[nodeType],
  );
  for (let i = 0, len = childKeys.length; i < len; i++) {
    const child = node[childKeys[i]!];
    if (Array.isArray(child)) {
      for (let j = 0, childLen = child.length; j < childLen; j++) {
        scanExternalAstForSyntaxFlags(child[j], flags, seen, visitorKeys);
        if (flags.isJsx !== null && flags.isTs !== null) return;
      }
    } else {
      scanExternalAstForSyntaxFlags(child, flags, seen, visitorKeys);
      if (flags.isJsx !== null && flags.isTs !== null) return;
    }
  }
}

export function detectExternalSourceFlags(
  parserOptions: Record<string, unknown> | null,
  ast: unknown,
  visitorKeys: VisitorKeysRecord | null = null,
): ExternalSourceFlags {
  const flags = getConfiguredExternalSourceFlagState(parserOptions);
  if (flags.isJsx === null || flags.isTs === null) {
    scanExternalAstForSyntaxFlags(ast, flags, new WeakSet(), visitorKeys);
  }

  return {
    isJsx: flags.isJsx === true,
    isTs: flags.isTs === true,
  };
}

function isModuleKind(value: unknown): value is Program["sourceType"] {
  return value === "module" || value === "script" || value === "commonjs";
}

function inferSourceTypeFromProgramBody(body: unknown[] | undefined): Program["sourceType"] {
  if (!Array.isArray(body)) return "module";

  for (const statement of body) {
    if (!isObjectRecord(statement)) continue;
    switch (statement.type) {
      case "ImportDeclaration":
      case "ExportAllDeclaration":
      case "ExportDefaultDeclaration":
      case "ExportNamedDeclaration":
        return "module";
    }
  }

  return "script";
}

export function normalizeExternalProgramSourceType(
  programSourceType: unknown,
  parserSourceType: unknown,
  body?: unknown[],
): Program["sourceType"] {
  // Respect explicit languageOptions / parserOptions sourceType overrides ahead of parser defaults.
  if (isModuleKind(parserSourceType)) return parserSourceType;
  if (isModuleKind(programSourceType)) return programSourceType;
  if (parserSourceType === "unambiguous") return inferSourceTypeFromProgramBody(body);
  return "module";
}
