function dec(Class: any) { return Class; }

@dec
export default class {
  static value = 42;
  field = 1;
}
