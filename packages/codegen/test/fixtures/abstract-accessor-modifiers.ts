abstract class C extends B {
  public abstract accessor x: number;
  protected abstract accessor y: string;
  public abstract override accessor z: number;
  protected abstract override accessor w: string;
  abstract accessor a: number;
}

class D extends B {
  public static accessor x = 1;
  protected static accessor y = 1;
  private accessor z = 1;
  public override accessor w = 1;
  accessor #a = 1;
  public accessor b!: number;
}
