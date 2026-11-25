class Outer {
  #_0_accessor_storage = 42;
  make() {
    class Inner {
      #_0_accessor_storage2 = 1;
      get ['x']() {
        return this.#_0_accessor_storage2;
      }
      set ['x'](value) {
        this.#_0_accessor_storage2 = value;
      }
      read(outer) {
        return outer.#_0_accessor_storage;
      }
    }
    return Inner;
  }
}
const outer = new Outer();
const Inner = outer.make();
console.log(new Inner().read(outer));
