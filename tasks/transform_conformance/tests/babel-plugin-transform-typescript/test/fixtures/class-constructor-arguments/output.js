class Foo {
  boom;
  constructor(foo, bar, zoo, bang, too) {
    this.foo = foo;
    this.bar = bar;
    this.zoo = zoo;
    this.bang = bang;
    console.log(this.foo, this.bar, this.zoo, this.bang);
  }
}
class Bar extends Foo {
  constructor(foo, bar, zoo, bang, boom, too) {
    super(foo, bar, zoo, bang, too);
    this.foo = foo;
    this.bar = bar;
    this.zoo = zoo;
    this.bang = bang;
    this.boom = boom;
  }
}
