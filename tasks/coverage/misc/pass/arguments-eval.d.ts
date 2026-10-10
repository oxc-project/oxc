type K = (arguments: any[]) => void;
type K2 = (...arguments: any[]) => void;

interface Foo {
  bar(arguments: any[]): void;
  bar2(...arguments: any[]): void;
}

declare function arguments(): void;
declare function eval(): void;
declare function f(eval: number, arguments: number): number;
declare function f2(...eval: number[]): number;
declare function f3(...arguments: number[]): number;

declare class C {
  constructor(arguments: number);
  method(eval: number, ...arguments: number[]): void;
}

declare namespace foo {
  type arguments = {};
  type eval = {};
}

declare namespace foo2 {
  interface arguments {}
  interface eval {}
}

declare global {
	function arguments(...arguments: any[]): typeof arguments;
	function eval(...eval: any[]): typeof eval;
}
