// @compilationMode:"infer"
import { useEffect } from "react";

function createBridge(options, fallback) {
  const alias = options;
  const useBridge = () => {
    useEffect(() => {
      const sendReadyMessages = () =>
        options?.onReady?.() ?? alias?.onReady?.() ?? fallback?.onReady?.();
      sendReadyMessages();
    }, []);
  };
  return { useBridge };
}
