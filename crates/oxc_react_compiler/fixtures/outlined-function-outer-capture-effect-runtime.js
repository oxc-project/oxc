// @compilationMode:"infer"
import { useEffect } from "react";

function createBridge(options) {
  const useBridge = () => {
    useEffect(() => {
      const sendReadyMessages = () => {
        const readyMessages = options?.onReady?.() ?? [];
        readyMessages.forEach((message) => console.log(message));
      };
      sendReadyMessages();
    }, []);
  };
  return { useBridge };
}

const { useBridge } = createBridge({ onReady: () => ["CHART_READY"] });

function Component() {
  useBridge();
  return <div>bridge</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{}],
};
