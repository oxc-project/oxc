import { useState } from "react";

let someGlobal = false;

// A hook is linted even without JSX.
export function useFoo() {
  someGlobal = true;
  return useState(0);
}
