/**
 * Padding this table would make every row 98 chars wide, over the 60 char
 * budget, so `auto` drops the padding and emits a minimal separator row.
 *
 * | Option | Description | Default |
 * | --- | --- | --- |
 * | resolveExtensions | List of file extensions the resolver will try in order | [".ts", ".js"] |
 * | preserveSymlinks | Keep symlinked paths as written instead of resolving them | false |
 */
const wide = 1;
