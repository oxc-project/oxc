/**
 * Unpadded rows must stay long enough to be re-detected as table rows on the next
 * format pass, so a lone empty column keeps one space instead of collapsing to `||`.
 *
 * | Header |
 * | --- |
 * |  |
 * | x |
 *
 * Wider rows can hold empty cells as-is, and short rows are filled out to the
 * table's column count.
 *
 * | A | B | C |
 * | :-: | :-- | --: |
 * | 1 |
 * | 1 | 2 | 3 |
 */
const sparse = 1;
