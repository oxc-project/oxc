// @validateNoSetStateInRender @outputMode:"lint"
import { useMemo, useState } from 'react';

function Component({ flag }) {
  const [value, setValue] = useState(0);
  const read = useMemo(() => {
    let cached;
    if (flag) setValue(value + 1);
    return () => (cached ??= value);
  }, [flag, value]);
  return <button onClick={read}>{value}</button>;
}

function ReturnedHandler({ flag }) {
  const [value, setValue] = useState(0);
  const onClick = useMemo(() => {
    let cached;
    return () => {
      if (flag) setValue(cached ??= value + 1);
    };
  }, [flag, value]);
  return <button onClick={onClick}>{value}</button>;
}
