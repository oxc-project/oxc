declare var a: number;
declare let b: number;
declare const c: number;
declare function f(): void;
declare class C {}
declare enum E { A }
declare namespace N { const x: number; }
a++; b = c; f(); new C(); E.A; N.x;
