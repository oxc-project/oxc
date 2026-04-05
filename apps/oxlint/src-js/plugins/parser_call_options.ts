import { dirname as pathDirname } from "node:path";

function isTypeAwareParserOptions(options: Record<string, unknown>): boolean {
  return options.projectService === true || options.project != null;
}

function inferTsconfigRootDir(filePath: string, cwd: string | null | undefined): string {
  return typeof cwd === "string" && cwd.length > 0 ? cwd : pathDirname(filePath);
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function isParserLike(value: unknown): value is Record<string, unknown> {
  return isPlainObject(value) &&
    (typeof value.parse === "function" || typeof value.parseForESLint === "function");
}

function cloneParserOptionValue(value: unknown, seen: Map<object, unknown>): unknown {
  if (Array.isArray(value)) {
    const existingClone = seen.get(value);
    if (existingClone !== undefined) return existingClone;

    const clonedArray: unknown[] = [];
    seen.set(value, clonedArray);
    for (let i = 0; i < value.length; i++) {
      clonedArray.push(cloneParserOptionValue(value[i], seen));
    }
    return clonedArray;
  }

  if (!isPlainObject(value) || isParserLike(value)) return value;

  const existingClone = seen.get(value);
  if (existingClone !== undefined) return existingClone;

  const clonedObject: Record<string, unknown> = {};
  seen.set(value, clonedObject);
  for (const [key, entry] of Object.entries(value)) {
    clonedObject[key] = cloneParserOptionValue(entry, seen);
  }
  return clonedObject;
}

function cloneParserOptions(
  parserOptions: Record<string, unknown> | null | undefined,
): Record<string, unknown> {
  if (parserOptions == null) return {};
  return cloneParserOptionValue(parserOptions, new Map()) as Record<string, unknown>;
}

export function normalizeParserOptionsForFile(
  filePath: string,
  cwd: string | null | undefined,
  parserOptions: Record<string, unknown> | null | undefined,
): Record<string, unknown> | null {
  if (parserOptions == null) return null;
  if (parserOptions.tsconfigRootDir != null || !isTypeAwareParserOptions(parserOptions)) {
    return parserOptions;
  }

  return {
    ...parserOptions,
    tsconfigRootDir: inferTsconfigRootDir(filePath, cwd),
  };
}

export function createRequiredParserCallOptions(
  filePath: string,
  parserOptions: Record<string, unknown> | null | undefined,
  sourceType?: unknown,
  ecmaVersion?: unknown,
  cwd?: string | null,
): Record<string, unknown> {
  const options = cloneParserOptions(normalizeParserOptionsForFile(filePath, cwd, parserOptions));

  if (sourceType != null && options.sourceType == null) {
    options.sourceType = sourceType;
  }

  if (ecmaVersion != null && options.ecmaVersion == null) {
    options.ecmaVersion = ecmaVersion;
  }

  if (options.filePath == null) options.filePath = filePath;

  options.loc = true;
  options.range = true;
  options.raw = true;
  options.tokens = true;
  options.comment = true;
  options.eslintVisitorKeys = true;
  options.eslintScopeManager = true;

  return options;
}
