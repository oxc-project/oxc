type Bar = number;

class Foo {
  [x: string]: any;
  [y: Bar]: string;
  [z: symbol]: Foo;
}
