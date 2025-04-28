import { Foo, Bar, type Zoo } from "mod";

declare function dec(
  target: any,
  key: string,
  descriptor: PropertyDescriptor,
): void;

class Cls {
  constructor(@dec param: Foo, param2: Foo | Bar, param3: Zoo, param4: Zoo.o.o) {
    console.log(param, param2, param3, param4);
  }
}
