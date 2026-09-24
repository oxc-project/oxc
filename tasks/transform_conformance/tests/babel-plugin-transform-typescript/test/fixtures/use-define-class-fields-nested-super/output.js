class Foo {}
class Bar extends Foo {
  constructor() {
    super(), console.log();
    this.test = 'initial';
  }
}
class Baz extends Foo {
  constructor(code) {
    super(), console.log(this);
    this.code = code;
  }
}
