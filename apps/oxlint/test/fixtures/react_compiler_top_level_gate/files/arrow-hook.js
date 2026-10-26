import { useState } from "react";

let count = 0;

// A top-level hook declared with an arrow function is linted.
export const useCounter = () => {
  const [step] = useState(1);
  count = count + step;
  return count;
};
