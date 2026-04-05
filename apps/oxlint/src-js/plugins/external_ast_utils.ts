const EMPTY_INFERRED_EXTERNAL_CHILD_KEYS: readonly string[] = Object.freeze([]);

function isValidExternalOffset(value: unknown, maxEnd: number): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0 && value <= maxEnd;
}

export function normalizeExternalRange(
  rangeInput: unknown,
  startInput: unknown,
  endInput: unknown,
  maxEnd: number = Number.POSITIVE_INFINITY,
): [number, number] | null {
  if (
    Array.isArray(rangeInput) &&
    rangeInput.length === 2 &&
    isValidExternalOffset(rangeInput[0], maxEnd) &&
    isValidExternalOffset(rangeInput[1], maxEnd) &&
    rangeInput[1] >= rangeInput[0]
  ) {
    return [rangeInput[0], rangeInput[1]];
  }

  if (
    isValidExternalOffset(startInput, maxEnd) &&
    isValidExternalOffset(endInput, maxEnd) &&
    endInput >= startInput
  ) {
    return [startInput, endInput];
  }

  return null;
}

export function isNonVisitableExternalChildKey(key: string): boolean {
  return (
    key === "parent" ||
    key === "loc" ||
    key === "range" ||
    key === "start" ||
    key === "end" ||
    key === "comments" ||
    key === "tokens" ||
    key === "leadingComments" ||
    key === "trailingComments" ||
    key === "innerComments"
  );
}

export function isExternalNodeLike(
  value: unknown,
): value is Record<string, unknown> & { type: string } {
  return (
    typeof value === "object" &&
    value !== null &&
    !Array.isArray(value) &&
    typeof (value as { type?: unknown }).type === "string"
  );
}

export function getInferredExternalChildKeys(node: Record<string, unknown>): readonly string[] {
  const inferredKeys: string[] = [];

  for (const [key, value] of Object.entries(node)) {
    if (isNonVisitableExternalChildKey(key)) continue;

    if (
      isExternalNodeLike(value) ||
      (Array.isArray(value) && value.some((entry) => isExternalNodeLike(entry)))
    ) {
      inferredKeys.push(key);
    }
  }

  return inferredKeys.length === 0 ? EMPTY_INFERRED_EXTERNAL_CHILD_KEYS : inferredKeys;
}

export function mergeExternalChildKeys(
  inferredKeys: readonly string[],
  configuredKeys: readonly string[] | null | undefined,
): readonly string[] {
  if (configuredKeys == null) return inferredKeys;
  if (configuredKeys.length === 0 || inferredKeys.length === 0) return configuredKeys;

  let mergedKeys: string[] | null = null;

  for (let i = 0, len = inferredKeys.length; i < len; i++) {
    const key = inferredKeys[i]!;
    if (configuredKeys.includes(key)) continue;

    if (mergedKeys === null) mergedKeys = [...configuredKeys];
    mergedKeys.push(key);
  }

  return mergedKeys ?? configuredKeys;
}

function sanitizeExternalVisitorKeys(
  keys: readonly string[],
): readonly string[] {
  let sanitizedKeys: string[] | null = null;

  for (let i = 0, len = keys.length; i < len; i++) {
    const key = keys[i]!;
    if (isNonVisitableExternalChildKey(key)) {
      if (sanitizedKeys === null) sanitizedKeys = keys.slice(0, i);
      continue;
    }

    if (sanitizedKeys !== null) sanitizedKeys.push(key);
  }

  return sanitizedKeys ?? keys;
}

export function sanitizeExternalVisitorKeysRecord(
  visitorKeysInput: Readonly<Record<string, readonly string[]>> | null | undefined,
): Readonly<Record<string, readonly string[]>> | null {
  if (visitorKeysInput == null) return null;

  let sanitizedVisitorKeys: Record<string, readonly string[]> | null = null;

  for (const [nodeType, keys] of Object.entries(visitorKeysInput)) {
    const sanitizedKeys = sanitizeExternalVisitorKeys(keys);
    if (sanitizedKeys === keys) continue;

    if (sanitizedVisitorKeys === null) sanitizedVisitorKeys = { ...visitorKeysInput };
    sanitizedVisitorKeys[nodeType] = sanitizedKeys;
  }

  return sanitizedVisitorKeys ?? visitorKeysInput;
}
