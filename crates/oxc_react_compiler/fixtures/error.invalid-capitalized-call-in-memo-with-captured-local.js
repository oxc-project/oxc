// @validateNoCapitalizedCalls
import { useMemo } from 'react';

function Component() {
  const getValue = useMemo(() => {
    const result = SomeFunc();
    let cached;
    return () => (cached ??= result);
  }, []);
  return <button onClick={getValue}>Read value</button>;
}
