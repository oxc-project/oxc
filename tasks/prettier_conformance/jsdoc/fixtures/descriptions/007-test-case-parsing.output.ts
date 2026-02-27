/**
 * Handles parsing of a test case file.
 *
 * A test case file consists of at least two parts, separated by a line of
 * dashes. This separation line must start at the beginning of the line and
 * consist of at least three dashes.
 *
 * The test case file can either consist of two parts:
 *
 *     const a = "";
 *     const b = { c: [] };
 *
 * Or of three parts:
 *
 *     {source code}
 *     ----
 *     {expected token stream}
 *     ----
 *     {text comment explaining the test case}
 *
 * If the file contains more than three parts, the remaining parts are just
 * ignored. If the file however does not contain at least two parts (so no
 * expected token stream), the test case will later be marked as failed.
 */
