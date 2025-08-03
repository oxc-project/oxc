import { Collection } from './collection.ts';

export class Obj {
  public readonly [Collection.identifier] = true;
  public readonly [Collection.identifier2];
  public static readonly [Collection.identifier3] = true;
  public static readonly  [Collection.identifier4];
}
