// @compilationMode:"infer"
import { useEffect } from "react";
import { notify } from "./notify";

function createBridge(options) {
  function useBridge() {
    useEffect(() => options?.onReady?.(), []);
    useEffect(() => notify(window.location.href), []);
  }
  return useBridge;
}
