export class Store {
  accessor count = 0;
  static accessor label = "store";
  accessor typed: number = 1;
  accessor callback = (value: string): void => {};
  private accessor hidden = 0;
  accessor unknown = getValue();
}
