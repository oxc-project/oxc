// @compilationMode:"infer"
import { useEffect } from "react";
import { notify } from "./notify";

const useBridge = function () {
  useEffect(() => {
    notify(window.location.href);
  }, []);
};
