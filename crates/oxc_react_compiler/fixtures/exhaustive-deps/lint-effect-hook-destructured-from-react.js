// @validateExhaustiveEffectDependencies:"all" @outputMode:"lint" @panicThreshold:"none" @lintSkipReactiveScopePasses
import * as React from 'react';

// The effect hook is only recognisable by its type: the callee is a local
// destructured from the React namespace, not a `useEffect` load or property.
function Component({value}) {
  const {useEffect} = React;
  useEffect(() => {
    console.log(value);
  }, []);
  return <div>{value}</div>;
}
