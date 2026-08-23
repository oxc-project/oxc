/**
 * @file The pure layer of the config-loader system: the per-handler quota
 *   shapes, the payer roles, the model-name helpers, and the tally that turns
 *   Database message rows into per-handler spend. The I/O that reads those rows
 *   from SQLite and the providers' own surfaces lives in
 *   `config-loader-read.mts` so this module can be imported by a SQLite-free
 *   renderer (the natively-built statusline entry) without pulling
 *   `node:sqlite` or the credential-backed readers into it. EVERY PROVIDER HAS
 *   A DIFFERENT METER. Fireworks bills dollars. Synthetic sells a REQUEST rate
 *
 *   - 500 per rolling 5 hours - so a dollar gauge over it would fill on the wrong
 *     axis and read full right up until the requests ran out.
 *   - Existing list item.
 */
export const value = 1;
