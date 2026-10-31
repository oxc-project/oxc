// Computed keys are ignored when deciding whether quotes are needed
interface I {
  color: 1;
  ['meta.nonce']: 2;
  ['a-b'](): void;
}
type T = { color: 1; ['meta.nonce']: 2 };
