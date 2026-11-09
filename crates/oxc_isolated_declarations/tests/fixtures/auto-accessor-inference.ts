declare function unavailable(): number;

export class Store {
  accessor count = 0;
  static accessor label = "store";
  accessor enabled = true;
  accessor negative = -1;
  accessor bigint = 1n;
  accessor typed: number | undefined = 1;
  protected accessor protectedValue = 1;
  private accessor hidden = unavailable();
  accessor #secret = unavailable();
}

export class Unsupported {
  accessor missing;
  accessor unknown = unavailable();
}
