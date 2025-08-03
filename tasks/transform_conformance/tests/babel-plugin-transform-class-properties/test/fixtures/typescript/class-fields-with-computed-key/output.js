import { Collection } from "./collection.ts";
let _Collection$identifie, _Collection$identifie2;
_Collection$identifie = Collection.identifier;
Collection.identifier2;
_Collection$identifie2 = Collection.identifier3;
Collection.identifier4;
export class Obj {
  constructor() {
    this[_Collection$identifie] = true;
  }
}
Obj[_Collection$identifie2] = true;
