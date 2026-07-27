/**
 * The alignment decision is per table, not per file: the narrow table below
 * gets padded while the wide one in the same comment does not.
 *
 * | Option | Type | Default |
 * | --- | --- | --- |
 * | depth | number | 2 |
 * | strict | boolean | false |
 *
 * And the wide one:
 *
 * | Option | Description | Default |
 * | --- | --- | --- |
 * | resolveExtensions | List of file extensions the resolver will try in order | [".ts", ".js"] |
 * | preserveSymlinks | Keep symlinked paths as written instead of resolving them | false |
 */
const both = 1;

/**
 * A separate comment in the same file, narrow again.
 *
 * | Left | Center | Right |
 * | :--- | :----: | ----: |
 * | a | b | c |
 */
const alsoNarrow = 2;
