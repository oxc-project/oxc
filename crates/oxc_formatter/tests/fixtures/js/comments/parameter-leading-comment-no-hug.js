// A comment around the only parameter disables hugging, and the pattern keeps its own group:
// an own-line comment breaks the parameter list, not `{ item }`.
const resolveItemValue = useCallback(
  (
    /** @type {{ item: { id: number } }} */
    { item },
  ) => String(item?.id),
  [],
);
const line_comment = (
  // c
  { item },
) => String(item?.id);
const array_pattern = (
  // c
  [a, b],
) => a + b;
function decl(
  /** @type {{ item: { id: number } }} */
  { item },
) {}
// Same-line comments still don't hug, and the pattern stays inline.
const before = (/* c */ { item }) => String(item?.id);
const after = ({ item } /* c */) => String(item?.id);
// No comment: hugs as before.
const hugged = ({ item, other, more, evenMore, andMore, stillMore, last }) => String(item?.id);
