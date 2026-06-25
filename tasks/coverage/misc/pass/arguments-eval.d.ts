type K = (arguments: any[]) => void;
type K2 = (...arguments: any[]) => void;

interface Foo {
  bar(arguments: any[]): void;
  bar2(...arguments: any[]): void;
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
