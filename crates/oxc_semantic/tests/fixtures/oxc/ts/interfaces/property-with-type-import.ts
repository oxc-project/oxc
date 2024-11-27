import type X from "mod";

type B = number;

export interface A {
  [X]: B;
}

export interface A {
  [X.X]: B;
}
