// @compilationMode:"infer"
import { useState } from "react";

function createLabel(prefix) {
  const useLabel = (name) => {
    const [suffix] = useState("!");
    const format = () => prefix + ":";
    return format() + name + suffix;
  };
  return useLabel;
}

const useLabel = createLabel("hello");

function Component({ name }) {
  const label = useLabel(name);
  return <div>{label}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{ name: "a" }],
  sequentialRenders: [{ name: "a" }, { name: "a" }, { name: "b" }],
};
