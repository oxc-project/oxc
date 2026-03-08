export class ReadonlyObjectAsConst {
  writableTrue = true;
  readonly objectAsConst = ({ a: 1 } as const);
}
