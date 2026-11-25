class Outer {
  #_0_accessor_storage = 42;
  #_0_accessor_storage2 = 43;
  #_x_accessor_storage = 44;
  #_y_accessor_storage = 45;
  static #_z_accessor_storage = 46;

  make() {
    return class Inner {
      accessor ['computed'] = 1;
      accessor x = 2;
      accessor #y = 3;
      static accessor z = 4;

      read(outer: Outer) {
        return [outer.#_0_accessor_storage, outer.#_0_accessor_storage2];
      }

      readNested(outer: Outer) {
        return new class Nested {
          read() {
            outer.#_x_accessor_storage++;
            return [outer?.#_x_accessor_storage, #_x_accessor_storage in outer,
              outer.#_y_accessor_storage, Outer.#_z_accessor_storage];
          }
        }().read();
      }

      readOwn() {
        return [this.computed, this.x, this.#y];
      }
    };
  }
}

const outer = new Outer();
const Inner = outer.make();
const inner = new Inner();
expect(inner.read(outer)).toEqual([42, 43]);
expect(inner.readNested(outer)).toEqual([45, true, 45, 46]);
expect(inner.readOwn()).toEqual([1, 2, 3]);
inner.computed = 5;
inner.x = 6;
expect(inner.readOwn()).toEqual([5, 6, 3]);
expect(Inner.z).toBe(4);
Inner.z = 7;
expect(Inner.z).toBe(7);

// A nested declaration must keep shadowing the enclosing private name.
class ShadowOuter {
  #_x_accessor_storage = 8;

  make() {
    return class Inner {
      accessor x = 9;

      readNested(outer: ShadowOuter) {
        return new class Nested {
          #_x_accessor_storage = 10;

          read() {
            return [this.#_x_accessor_storage, #_x_accessor_storage in outer];
          }
        }().read();
      }
    };
  }
}

const shadowOuter = new ShadowOuter();
const ShadowInner = shadowOuter.make();
const shadowInner = new ShadowInner();
expect(shadowInner.x).toBe(9);
expect(shadowInner.readNested(shadowOuter)).toEqual([10, false]);
