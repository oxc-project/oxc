// @validateNoSetStateInRender
import { useMemo, useState } from 'react';

function Component() {
  const [value, setValue] = useState(0);
  return useMemo(() => {
    let cached;
    setValue(value + 1);
    return () => (cached ??= value);
  }, [value]);
}
