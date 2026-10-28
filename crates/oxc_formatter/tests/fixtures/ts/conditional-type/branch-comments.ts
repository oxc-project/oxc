// Comments on a `?`/`:` branch hug behind the operator (like conditional
// expressions), and the branch content sits one level under the `?`,
// with no extra union indent stacked on top.

type BlockComment = any extends B
  ? /**
     * Comment
     */
    B | C
  : D;

type AlternateSide = any extends B
  ? D
  : /**
     * Comment
     */
    B | C;

// Line comment stays on the `?` line, content one level under (zen-fs/core shape)
type LineComment = T extends number
  ? // resolved as number
    { size: number } | { other: bigint }
  : { size: bigint };

// An own-line comment before the `?` in source also hugs behind it (prettier#18647)
type HugFromOwnLine = any extends B
  /**
   * Comment
   */
  ? B | C
  : D;
