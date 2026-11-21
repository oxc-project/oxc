import { useMemo, useSyncExternalStore } from "react";

const subscribe = () => () => {};
const serverSnapshot = () => false;

export function useValue(read: () => number) {
  const getSnapshot = useMemo(() => {
    let cached: number | undefined;
    return () => (cached ??= read());
  }, [read]);
  return useSyncExternalStore(subscribe, getSnapshot, serverSnapshot);
}

export function useInitializedValue(read: () => number) {
  const getSnapshot = useMemo(() => {
    let cached: number | null = null;
    return () => cached ?? (cached = read());
  }, [read]);
  return useSyncExternalStore(subscribe, getSnapshot, serverSnapshot);
}

export function useRenderLocal(read: () => number) {
  let renderLocal: number | undefined;
  const getSnapshot = useMemo(() => {
    let cached: number | undefined;
    return () => {
      cached ??= read();
      renderLocal = cached;
      return cached;
    };
  }, [read]);
  return useSyncExternalStore(subscribe, getSnapshot, serverSnapshot);
}
