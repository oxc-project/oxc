// @compilationMode:"infer" @panicThreshold:"none"
// The useMemo also calls setState during render. That is reported and skips the component; it is
// not retried with the useMemo kept, which would hide the setState call from the validation.
import { useMemo, useState } from 'react';

function List({ s, items }) {
  const [count, setCount] = useState(0);
  const values = useMemo(() => {
    setCount(items.length);
    return sortBy(items, s);
  }, [items]);
  return <Rows values={values} count={count} />;
}
