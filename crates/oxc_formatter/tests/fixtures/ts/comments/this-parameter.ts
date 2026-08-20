// Comments after a `this` parameter stay inside the parens (issue #25410)
interface X {
  ownLine<T>(
    this: void
    // own-line comment
  ): void;
  sameLine(
    this: void // same-line comment
  ): void;
  block(
    this: void
    /* block comment */
  ): void;
  beforeNext(
    this: void, // trailing this
    a: string
  ): void;
}
type F = (
  this: void
  // fn type
) => void;
declare function g(
  this: void,
  ...rest: number[]
  // after rest
): void;
function h(
  this: void
  // own line
) {}
