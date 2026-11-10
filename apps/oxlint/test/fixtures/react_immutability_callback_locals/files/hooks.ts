import { useCallback, useEffect, useState } from "react";

export function useTicker(isPlaying: boolean) {
  const [total, setTotal] = useState(0);
  useEffect(() => {
    if (!isPlaying) return;
    let last = Date.now();
    const id = setInterval(() => {
      const now = Date.now();
      setTotal((t) => t + (now - last));
      last = now;
    }, 1000);
    return () => clearInterval(id);
  }, [isPlaying]);
  return total;
}

export function useAfterRun(run: () => Promise<void>) {
  useEffect(() => {
    let timeoutId: ReturnType<typeof setTimeout> | undefined;
    const go = async () => {
      await run();
      timeoutId = setTimeout(() => {}, 10);
    };
    go();
    return () => clearTimeout(timeoutId);
  }, [run]);
}

export function useCancellation(load: () => Promise<() => void>) {
  useEffect(() => {
    let cancelled = false;
    let cleanup: (() => void) | undefined;
    void load().then((attach) => {
      if (cancelled) return;
      cleanup = attach();
    });
    return () => {
      cancelled = true;
      cleanup?.();
    };
  }, [load]);
}

export function useCallbackCounter() {
  return useCallback((count: number) => () => count++, []);
}

export function useCounter() {
  let count = 0;
  useEffect(() => {
    count = 1;
  });
  return count;
}
