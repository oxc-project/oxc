// @compilationMode:"infer" @panicThreshold:"none"
import {useMemo} from 'react';

export function Component() {
  const value = useMemo(() => {
    console.log('computing');
  }, []);
  return <div>{value}</div>;
}
