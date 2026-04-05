import type { ParserLike } from "./package/parser.ts";

export interface RegisteredLanguageOptions {
  parser?: Readonly<ParserLike> | null;
  parserOptions?: Record<string, unknown> | null;
  [key: string]: unknown;
}

const languageOptionsRegistry = new Map<number, RegisteredLanguageOptions>();
const languageOptionsIds = new WeakMap<object, number>();

let nextLanguageOptionsId = 1;

function isPlainObject(value: unknown): value is Record<string, unknown> {
  if (typeof value !== "object" || value === null) return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function isParserLike(value: unknown): value is Readonly<ParserLike> {
  return isPlainObject(value) &&
    (typeof value.parse === "function" || typeof value.parseForESLint === "function");
}

function mergeParserOptionParserValue(localValue: unknown, baseValue: unknown): unknown {
  if (!isPlainObject(localValue) || !isPlainObject(baseValue)) return localValue;

  // `parserOptions.parser` is special. It can be either a parser object or a map of language names to
  // parser objects. Parser objects must be preserved by reference, not recursively merged into synthetic
  // Franken-objects which lose identity and can subtly change behavior.
  if (isParserLike(localValue) || isParserLike(baseValue)) {
    return localValue;
  }

  return { ...baseValue, ...localValue };
}

function mergePlainObjects(
  localValue: Record<string, unknown>,
  baseValue: Record<string, unknown>,
  depth: number = 0,
): Record<string, unknown> {
  const merged: Record<string, unknown> = { ...baseValue };

  for (const [key, value] of Object.entries(localValue)) {
    const baseEntry = merged[key];
    if (depth === 0 && key === "parser") {
      merged[key] = mergeParserOptionParserValue(value, baseEntry);
    } else if (isPlainObject(value) && isPlainObject(baseEntry)) {
      merged[key] = mergePlainObjects(value, baseEntry, depth + 1);
    } else {
      merged[key] = value;
    }
  }

  return merged;
}

/**
 * Store a non-serializable `languageOptions` object loaded from `oxlint.config.ts`.
 *
 * The JS config loader returns JSON to Rust, so parser objects and parser options that contain
 * functions need to be kept alive on the JS side and referenced by ID.
 */
export function registerLanguageOptions(languageOptions: unknown): number {
  if (!isPlainObject(languageOptions)) {
    throw new Error("`languageOptions` must be an object.");
  }

  const existingId = languageOptionsIds.get(languageOptions);
  if (existingId !== undefined) return existingId;

  const id = nextLanguageOptionsId++;
  languageOptionsRegistry.set(id, languageOptions as RegisteredLanguageOptions);
  languageOptionsIds.set(languageOptions, id);
  return id;
}

/**
 * Resolve and merge `languageOptions` objects previously loaded from JS config files.
 * Later IDs take precedence over earlier ones, matching Oxlint's override order.
 */
export function resolveLanguageOptionsIds(
  languageOptionsIdsInput: readonly number[],
): RegisteredLanguageOptions | null {
  if (languageOptionsIdsInput.length === 0) return null;

  let merged: RegisteredLanguageOptions | null = null;

  for (let i = 0; i < languageOptionsIdsInput.length; i++) {
    const id = languageOptionsIdsInput[i];
    const current = languageOptionsRegistry.get(id);
    if (current === undefined) {
      throw new Error(`Unknown languageOptions ID: ${id}`);
    }

    if (merged === null) {
      merged = { ...current };
      continue;
    }

    merged = {
      ...merged,
      ...current,
      parserOptions:
        isPlainObject(current.parserOptions) && isPlainObject(merged.parserOptions)
          ? mergePlainObjects(current.parserOptions, merged.parserOptions)
          : (current.parserOptions ?? merged.parserOptions ?? null),
    };
  }

  return merged;
}
