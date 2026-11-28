// @validateNoImpureFunctionsInRender
import { useMemo } from 'react';

function Component() {
  const getDate = useMemo(() => {
    const result = new Date();
    let cached;
    return () => (cached ??= result);
  }, []);
  return <button onClick={getDate}>Read date</button>;
}
