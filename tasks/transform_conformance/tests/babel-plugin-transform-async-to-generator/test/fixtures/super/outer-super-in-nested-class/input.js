class Outer extends OuterSuper {
  constructor() {
    @(super().decorate)
    class Inner extends super() {
      @(super().decorate)
      [super()] = 1;

      @(super().decorate)
      static [super()] = 2;

      @(super().decorate)
      [super()]() {}

      @(super().decorate)
      static [super()]() {}
    }

    let fn = async () => this;
  }
}
