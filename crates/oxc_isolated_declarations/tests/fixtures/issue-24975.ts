export function foo(): void {}

export declare namespace foo {
  let bar: number;
}

foo.bar = 42;
