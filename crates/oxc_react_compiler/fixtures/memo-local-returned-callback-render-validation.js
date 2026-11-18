// @validateNoImpureFunctionsInRender @validateNoCapitalizedCalls
import { useMemo } from 'react';

function Component() {
  const getDate = useMemo(() => {
    // Date construction with an argument is pure during render.
    const initial = new Date(0);
    let cached;
    return () => {
      // These calls run in the returned handler, not during render.
      SomeFunc();
      return (cached ??= { initial, current: new Date() });
    };
  }, []);
  return <button onClick={getDate}>Read date</button>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [],
  sequentialRenders: [{}, {}],
};
