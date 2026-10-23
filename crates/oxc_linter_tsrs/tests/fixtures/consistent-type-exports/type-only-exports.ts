export type TypeFoo = 1;

export interface InterfaceFoo {
  foo: 'bar';
}

class LocalClass {}

export type { LocalClass };
