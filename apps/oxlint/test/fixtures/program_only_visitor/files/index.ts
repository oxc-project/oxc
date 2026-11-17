// Leading comment
import { a } from "./a";

/* Block comment */
export function f(x: number): number {
  const y = { z: [x, a] };
  return y.z.length;
}
