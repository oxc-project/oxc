const events: string[] = [];
function key(name: string) {
  events.push(name);
  return name;
}
function decorate(name: string) {
  events.push(name);
  return function (_value: unknown, context: any) {
    events.push(`apply:${context.name}`);
  };
}
const decorators = {
  field(_value: unknown, context: any) {
    if (this !== decorators) throw new Error("lost receiver");
    events.push(`member:${context.name}`);
  },
};

class Instance {
  [key("instanceInitializedBefore")] = 1;
  [key("instanceUninitializedBefore")]: number;
  @decorate("instanceDecorator")
  @decorators.field
  [key("instanceDecorated")]: number;
  [key("instanceUninitializedAfter")]: number;
  [key("instanceInitializedAfter")] = 2;
  @decorate("plainDecorator")
  plain: number;
  [key("instanceMethod")]() {}
}

class Static {
  static [key("staticInitializedBefore")] = 1;
  static [key("staticUninitializedBefore")]: number;
  @decorate("staticDecorator")
  @(decorators["field"])
  static [key("staticDecorated")]: number;
  static [key("staticUninitializedAfter")]: number;
  static [key("staticInitializedAfter")] = 2;
}
