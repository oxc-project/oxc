// `start` puts `&` at the head of continuation lines when a non-object chain breaks.
type Long = AAAAAAAAAAAAAAAAAAAAAAAA & BBBBBBBBBBBBBBBBBBBBBBBB & CCCCCCCCCCCCCCCCCCCCCCCC;

// Adjacent object types stay inline.
type State = { sharedProperty: any } & { discriminant: "FOO"; foo: any } & { discriminant: "BAR"; bar: any };

// A leading own-line comment forces the break. With `start` we keep it own-line
// above the leading `&` (idempotent), like binary-like chains do; Prettier prints
// it behind `& `, losing own-line-ness and idempotency — a deliberate divergence
// (see "Known divergences").
type WithComment = SerializedProps &
  // own line comment
  { cause: unknown };
