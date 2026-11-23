import { useMemo } from 'react';
export function useParsed(json) {
  return useMemo(() => {
    let result = 0;
    try {
      result = json ? f(json) : null;
    } catch {}
    return result;
  }, [json]);
}
