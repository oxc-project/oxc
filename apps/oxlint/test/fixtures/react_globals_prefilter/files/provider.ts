import { createElement, useState } from "react";

let someGlobal = false;

// A PascalCase component that calls hooks is linted in a non-JSX file too.
export function Provider(props) {
  const [value] = useState(0);
  someGlobal = true;
  return createElement("div", null, value, props.children);
}
