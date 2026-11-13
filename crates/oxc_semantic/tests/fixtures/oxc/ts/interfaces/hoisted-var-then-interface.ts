export {};

switch (Number(1)) {
  case 1:
    var I = 0;
    interface I { a: number }
  case 2:
    interface I { b: number }
    const value: I = { a: 1, b: 2 };
}

type Outer = I;
