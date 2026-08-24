import type { ComponentType as Widget } from "widgets";

type Qualified = Namespace.Inner;
type Parenthesized = (string | number);
type Tuple = [first: string, second?: number, ...rest: boolean[]];
type Imported = import("widgets").Thing;
type ImportedWithOptions = import("widgets", { with: { "resolution-mode": "import" } }).Thing;
type Mapped = {
  +readonly [Key in keyof Imported as `${Key}`]?: Imported[Key];
};

declare module "widgets" {
  interface Thing {
    readonly [key: string]: unknown;
  }
}

interface Props<T extends object = object> extends Base<T> {
  readonly [key: string]: unknown;
  render?<U>(value: U): T;
}

class Registry {
  [key: string]: unknown;
}

const value = 1;
const props = { value };
const children = [<span key="one" />, <span key="two" />];

const view = (
  <>
    text
    {value}
    <Widget data={<span />} ns:label="namespaced" {...props}>
      <Namespace.Inner>{...children}</Namespace.Inner>
      <svg:path />
    </Widget>
  </>
);

const instantiated = identity<string>("value");
const template = `prefix ${value} suffix`;
const tagged = tag<number>`item ${value}`;
const loaded = import(`./${value}`);
const escaped = "line\n\r\t\\\"\u2028\u2029\u00a0";
const numeric = [-0, Infinity, -Infinity, 1n];

export { view, instantiated, template, tagged, loaded, escaped, numeric };
