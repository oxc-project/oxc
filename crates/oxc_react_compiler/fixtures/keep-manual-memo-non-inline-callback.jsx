// @compilationMode:"infer" @panicThreshold:"none"
// The callback is not an inline function, so its dependencies can't be checked. Skip the function.
import { useMemo } from 'react';

function List({ s, items }) {
  const compute = () => sortBy(items, s);
  const values = useMemo(compute, [items]);
  return <Rows values={values} />;
}
