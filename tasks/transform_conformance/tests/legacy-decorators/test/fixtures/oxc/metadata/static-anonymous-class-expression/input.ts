declare function dec(): ClassDecorator;
class A {}

@dec()
export class Foo {
  static Error1 = class extends Error {};
  static Error2 = class extends Error {};

  constructor(a: A) {}
}
