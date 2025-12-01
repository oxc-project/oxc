// Shadowed
export type Foo = {};
export type Bar = {
	foo: Foo;
};
const Foo = new Map();

type Func = () => void;
function Func() {}
export type FuncType = Func;

type Module = () => void;
namespace Module {
	export const x = 1;
}
export type ModuleType = Module;

// https://github.com/oxc-project/oxc/issues/10996
// Variable emit should not be dropped when variable is shadowed as a type name
// and the type is part of a class implements interface
export interface Thing<T> {}

const Test1 = 'test1' as const;
export type Test1 = typeof Test1;
export class Class1 {
  readonly id: 'test1' = Test1;
}

const Test2 = 'test2' as const;
export type Test2 = typeof Test2;
export class Class2 implements Thing<Test2> {
  readonly id: 'test2' = Test2;
}