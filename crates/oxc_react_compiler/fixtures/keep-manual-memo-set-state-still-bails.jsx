// @compilationMode:"infer" @panicThreshold:"none"
// The useMemo also calls setState during render. Report it and skip the component.
import { useMemo, useState } from 'react';

function List({ s, items }) {
  const [count, setCount] = useState(0);
  const values = useMemo(() => {
    setCount(items.length);
    return sortBy(items, s);
  }, [items]);
  return <Rows values={values} count={count} />;
}
