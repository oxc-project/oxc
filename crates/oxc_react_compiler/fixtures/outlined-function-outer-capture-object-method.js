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


const sproutInputs = [[{
  onReady: () => {
    "use no memo";

    return console.log("first");
  }
}], [{
  onReady: () => {
    "use no memo";

    return console.log("second");
  }
}]];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return createBridge(...args)();
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
