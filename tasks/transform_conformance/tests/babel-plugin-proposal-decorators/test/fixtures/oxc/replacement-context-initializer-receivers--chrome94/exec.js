const order = [];

let original, replaced, accessorThis, getterThis, setterThis, methodThis, propertyThis, classThis;

function dec(Klass, context) {
  original = Klass;
  replaced = class extends Klass {};

  context.addInitializer(function () {
    order.push("class");
    classThis = this;
  })

  return replaced;
}

function captureInitializerThis(callback) {
  return function (_, context) {
    context.addInitializer(function () {
      order.push(context.kind);
      callback(this);
    })
  }
}

@dec
class Foo {
  @(captureInitializerThis(v => accessorThis = v))
  static accessor accessor;
  @(captureInitializerThis(v => getterThis = v))
  static get getter() {};
  @(captureInitializerThis(v => setterThis = v))
  static set setter(_) {};
  @(captureInitializerThis(v => methodThis = v))
  static method() {}
  @(captureInitializerThis(v => propertyThis = v))
  static property;
}

expect(getterThis).toBe(replaced);
expect(setterThis).toBe(replaced);
expect(methodThis).toBe(replaced);

expect(accessorThis).toBe(replaced);
expect(propertyThis).toBe(replaced);

expect(classThis).toBe(replaced);

expect(order).toEqual(["getter", "setter", "method", "accessor", "field", "class"]);
