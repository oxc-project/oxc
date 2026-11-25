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


const sproutInputs = [[{
  onReady: () => {
    "use no memo";

    return console.log("first");
  }
}, null], [null, {
  onReady: () => {
    "use no memo";

    return console.log("fallback");
  }
}]];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return createBridge(...args).useBridge();
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
