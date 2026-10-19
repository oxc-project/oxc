const receivers = [];
const obj = {
  name: "obj",
  dec(value, ctx) {
    receivers.push(this && this.name);
  },
};

@obj.dec
class Foo {
  @obj.dec method() {}
  @(obj.dec) field;
}

expect(receivers).toEqual(["obj", "obj", "obj"]);
