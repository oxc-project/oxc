declare function Dec(target: any, key: string): void;

export enum Status {
  A = 1,
}

export enum Name {
  A = "a",
}

class U {
  @Dec()
  status!: Status;

  @Dec()
  name!: Name;
}
