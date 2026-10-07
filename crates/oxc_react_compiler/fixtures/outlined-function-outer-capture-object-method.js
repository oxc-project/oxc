// @compilationMode:"infer"
import { useEffect } from "react";

function createBridge(options) {
  const useBridge = () => {
    const bridge = {
      subscribe() {
        return () => options?.onReady?.();
      },
    };
    useEffect(() => bridge.subscribe()(), []);
    return bridge;
  };
  return useBridge;
}
