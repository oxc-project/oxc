import { useState } from "react";

// No top-level component or hook, so the file is skipped like in
// `eslint-plugin-react-hooks`.
export function mountProbe(): number | null {
  let result: number | null = null;
  function Probe() {
    result = useState(1)[0];
    return null;
  }
  void Probe;
  return result;
}
