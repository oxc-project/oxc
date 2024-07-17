function decorator() {}
class Foo {
  constructor(
    @decorator readonly a,
    @decorator readonly b = 1,
  ) {}
}
