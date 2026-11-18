const events = [];
function key(name) {
  events.push(name);
  return name;
}
function decorate(name) {
  events.push(name);
  return function (_value, context) {
    events.push(`apply:${context.name}`);
  };
}
const decorators = {
  field(_value, context) {
    if (this !== decorators) throw new Error("lost receiver");
    events.push(`member:${context.name}`);
  },
};
class Instance {
  [key("instanceInitializedBefore")] = 1;
  [key("instanceUninitializedBefore")];
  @decorate("instanceDecorator")
  @decorators.field
  [key("instanceDecorated")];
  [key("instanceUninitializedAfter")];
  [key("instanceInitializedAfter")] = 2;
  @decorate("plainDecorator")
  plain;
  [key("instanceMethod")]() {}
}
class Static {
  static [key("staticInitializedBefore")] = 1;
  static [key("staticUninitializedBefore")];
  @decorate("staticDecorator")
  @(decorators["field"])
  static [key("staticDecorated")];
  static [key("staticUninitializedAfter")];
  static [key("staticInitializedAfter")] = 2;
}
