// @compilationMode:"infer" @panicThreshold:"none"
// `s` is left out of the dependency list on purpose. The useMemo is kept as written and the
// rest of the component is still compiled.
import { useMemo } from 'react';

function List({ s, items, title }) {
  const values = useMemo(() => sortBy(items, s), [items]);
  return <Rows header={<h1>{title}</h1>} values={values} />;
}
