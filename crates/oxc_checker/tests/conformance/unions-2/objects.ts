// Unions of object types (discriminated and plain).
interface Circle {
  kind: "circle";
  radius: number;
}
interface Square {
  kind: "square";
  side: number;
}
type Shape = Circle | Square;

const ok_circle: Shape = { kind: "circle", radius: 2 };
const ok_square: Shape = { kind: "square", side: 3 };
const bad_kind: Shape = { kind: "oval", radius: 2 };
const bad_missing_side: Shape = { kind: "square" };

interface Named {
  name: string;
}
interface Aged {
  age: number;
}

declare const named_value: Named;
const ok_object_member: Named | Aged = named_value;
const ok_both_members: Named | Aged = { name: "x", age: 1 };
const bad_neither_member: Named | Aged = { label: "x" };

export {};
