// @compilationMode:"infer" @panicThreshold:"none"
// `suffix` is left out of the dependency list, so the callback keeps the `suffix` it was
// created with until `prefix` changes. The useCallback is kept as written.
import { useCallback } from 'react';

function Label({ prefix, suffix }) {
  const format = useCallback(() => prefix + suffix, [prefix]);
  return <Text format={format} suffix={suffix} />;
}
