// Unquoting a method signature named "new" would turn it into a construct signature,
// so its quotes must be preserved.
// https://github.com/prettier/prettier/issues/19618
interface Container {
  'new'(id: string): number;
}
type Factory = {
  'new'(id: string): number;
};

// With quoteProps "consistent", the forced quotes on 'new' also force
// quoting the sibling members.
interface WithSiblings {
  'new'(id: string): number;
  foo: string;
}

// Optional, getter, and setter forms would stay a member named "new" even unquoted,
// but Prettier keeps the quotes for every method-signature form.
interface OtherSignatureForms {
  'new'?(id: string): number;
  get 'new'(): number;
  set 'new'(value: number);
}

// These are all safe to unquote: they stay a member named "new".
interface SafeToUnquote {
  'new': (id: string) => number;
}
class NewMethod {
  'new'() {}
}
const newMethod = {
  'new'() {},
};
