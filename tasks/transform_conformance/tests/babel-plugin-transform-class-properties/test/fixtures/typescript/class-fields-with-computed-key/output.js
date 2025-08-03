import { Collection } from "./collection.ts";
let _Collection$identifie;
_Collection$identifie = Collection.identifier;
export class Obj {
  constructor() {
    this[_Collection$identifie] = true;
  }
}