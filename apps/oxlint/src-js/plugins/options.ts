/*
 * Options for rules.
 */

import type { JsonValue } from "./json.ts";

/**
 * Options for a rule on a file.
 */
export type Options = JsonValue[];

// Default rule options
const DEFAULT_OPTIONS: Readonly<Options> = Object.freeze([]);

// All rule options
export const allOptions: Readonly<Options>[] = [DEFAULT_OPTIONS];

// Index into `allOptions` for default options
export const DEFAULT_OPTIONS_ID = 0;
