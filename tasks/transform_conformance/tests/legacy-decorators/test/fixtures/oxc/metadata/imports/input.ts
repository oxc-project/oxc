import { Foo, Bar } from "mod";

declare function dec(
  target: any,
  key: string,
  descriptor: PropertyDescriptor,
): void;

class Cls {
  constructor(@dec param: Foo, param2: Foo | Bar) {
    console.log(param, param2);
  }
}
