// issue #25286 (sibling): the mapped type key gets its own bracket group,
// so an overlong key breaks after `[` instead of overflowing printWidth
type A = {
  [KeyNameHere in keyof SomeVeryLongGenericTypeNameHere as Uppercase<KeyNameHere & string>]: X;
};
type B = {
  +readonly [VeryLongKeyName in keyof SomeExtremelyLongSourceTypeNameGoesHere]-?: SomeValue;
};
type C = { [K in keyof T]: T[K] };
