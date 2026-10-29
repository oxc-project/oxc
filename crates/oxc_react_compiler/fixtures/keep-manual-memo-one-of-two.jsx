// @compilationMode:"infer" @panicThreshold:"none"
// One useMemo has incomplete dependencies; skip the whole function, preserving both calls.
import { useMemo } from 'react';

function List({ s, items, title }) {
  const heading = useMemo(() => [title, items.length], [title, items]);
  const values = useMemo(() => sortBy(items, s), [items]);
  return <Rows heading={heading} values={values} />;
}
