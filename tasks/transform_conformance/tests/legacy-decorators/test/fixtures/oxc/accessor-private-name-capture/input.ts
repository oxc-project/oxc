class Outer {
  #_0_accessor_storage = 42;
  make() {
    class Inner {
      accessor ['x'] = 1;
      read(outer: Outer) {
        return outer.#_0_accessor_storage;
      }
    }
    return Inner;
  }
}
const outer = new Outer();
const Inner = outer.make();
console.log(new Inner().read(outer));
