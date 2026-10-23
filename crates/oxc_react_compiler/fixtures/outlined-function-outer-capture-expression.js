// @compilationMode:"infer"
import { useEffect } from "react";

function createBridge(options) {
  const useBridge = function () {
    useEffect(() => options?.onReady?.(), []);
  };
  return useBridge;
}
