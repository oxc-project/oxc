class Foo {
  boom: number;
  constructor(public foo, private bar, protected zoo, readonly bang, too) {
    console.log(this.foo, this.bar, this.zoo, this.bang);
  }
}
class Bar extends Foo {
  constructor(public foo, private bar, protected zoo, readonly bang, override boom, too) {
    super(foo, bar, zoo, bang, too);
  }
}
