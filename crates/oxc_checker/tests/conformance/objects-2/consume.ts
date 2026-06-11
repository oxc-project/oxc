// Interfaces imported across modules; argument-position structural checks.
import { Circle, Shape } from "./shapes";

const bad_extra_on_interface: Shape = { kind: "dot", area: 0, color: "red" };

function describeShape(shape: Shape): string {
  return shape.kind;
}

const inline_circle: Circle = { kind: "circle", area: 1, radius: 0.5 };
const ok_described: string = describeShape(inline_circle);

const bad_arg_missing_member: string = describeShape({ kind: "ghost" });

export {};
