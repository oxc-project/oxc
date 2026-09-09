// Pins CommonMark semantics for list markers interrupting a paragraph
// (no blank line before them), matching upstream and what markdown-rendering
// consumers (VS Code hover, typedoc) already display.
// Decision record: `needs_mdast_parsing` doc in mdast_serialize/detect.rs.

/**
 * Case A star continuation becomes a list: computes min
 * * spacing between items and more text.
 */
export const a = 1;

/**
 * Case B plus continuation becomes a list: computes min
 * + spacing between items and more text.
 */
export const b = 1;

/**
 * Case C dash continuation becomes a list: result = alpha - beta
 * - gamma delta epsilon and more text.
 */
export const c = 1;

/**
 * Case D ordered continuation starting at 1 becomes a list: between 0 and
 * 1. They add up to more text.
 */
export const d = 1;

/**
 * Case E ordered continuation NOT starting at 1 stays prose (CommonMark's own
 * guard against wrapped years and versions): this was fixed in
 * 1986. What a great year that was for parsers.
 */
export const e = 1;

/**
 * Case F escaped marker stays prose (the author opt-out): result = alpha - beta
 * \- gamma delta epsilon and more text.
 */
export const f = 1;
