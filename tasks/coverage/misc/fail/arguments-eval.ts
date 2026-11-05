function arguments() {}
function eval() {}

function foo({ arguments }) {}
function foo2([arguments]) {}
function foo3({ eval }) {}
function foo4([eval]) {}
declare function foo5({ arguments }: { arguments: number }): void;
declare function foo6([eval]: [number]): void;
declare function foo7([arguments = 0]: [number?]): void;
declare function foo8({ ...arguments }: object): void;
declare function foo9([...eval]: unknown[]): void;
declare class C {
  method({ eval }: { eval: number }): void;
  constructor([arguments]: [number]);
}

export {};
