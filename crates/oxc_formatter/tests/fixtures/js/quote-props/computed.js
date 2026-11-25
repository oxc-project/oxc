// Computed keys are ignored when deciding whether quotes are needed
a = { color: 1, ['meta.nonce']: 2 };
a = { 'color': 1, ['meta.nonce']: 2 };
a = { [1]: 1, 'a-b': 2 };
const { color, ['meta.nonce']: nonce } = props;

class A {
  color = 1;
  ['meta.nonce'] = 2;
  ['a-b']() {}
}
