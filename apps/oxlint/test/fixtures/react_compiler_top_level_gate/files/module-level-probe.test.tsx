import { useState } from "react";

let result: number | null = null;

// A top-level component is linted.
export function Probe() {
  result = useState(1)[0];
  return null;
}
