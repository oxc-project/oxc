// Same-line comment keeps the annotation inline (Prettier's fixed point, prettier#19410)
type A1 = /* c */ { key: string };
type M1 = /* c */ {
  key: string;
};
type A = /* 1 */ /* 2 */ /* 3 */ /* 4 */ {
  key: string;
};
type B1 = /* c */ T extends U ? X : Y;
type C1 = /* c */ SomeVeryLongCheckType extends AnotherVeryLongExtendsType ? LongTrueBranch : LongFalseBranch;

// Own-line comment forces the break after `=`
type A2 =
  // c
  { key: string };
type A3 =
  /* c */
  { key: string };
type M2 =
  // c
  {
    key1: string;
    key2: string;
    key3: string;
  };
type B2 =
  // c
  T extends U ? X : Y;
type C2 =
  // c
  SomeVeryLongCheckType extends AnotherVeryLongExtendsType ? LongTrueBranch : LongFalseBranch;
