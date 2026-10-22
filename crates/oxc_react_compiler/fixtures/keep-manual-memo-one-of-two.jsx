// @compilationMode:"infer" @panicThreshold:"none"
// Only the useMemo with the incomplete dependency list is kept; the other is compiled as usual.
import { useMemo } from 'react';

function List({ s, items, title }) {
  const heading = useMemo(() => [title, items.length], [title, items]);
  const values = useMemo(() => sortBy(items, s), [items]);
  return <Rows heading={heading} values={values} />;
}
